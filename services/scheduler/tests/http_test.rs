//! Transport tests for the scheduler's HTTP control plane.
//!
//! The pure-router tests drive [`SchedulerServer::handle`] with a fixed clock and
//! never open a socket. Executable work is submitted through the in-process API
//! (there is no wire codec for `JobEnvelope`); only the serializable control
//! plane is exercised over HTTP. One test binds a real socket to prove the
//! blocking server answers a live client.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_runner::ResourceRequest;
use lawsynth_scheduler::{
    Clock, HttpRequest, HttpResponse, Scheduler, SchedulerConfig, SchedulerServer,
};
use lawsynth_sim::{SimulationConfig, SimulationRequest};
use lawsynth_store::MemoryStore;
use lawsynth_worker::{Job, JobEnvelope};
use lawsynth_world::{ContinuousLaw, Variable, VariableRole, World};

const FIXED_NOW_MS: u64 = 1_000;

fn identifier(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}

fn resources() -> ResourceRequest {
    ResourceRequest::new(250, 1024, 1024).unwrap()
}

/// Builds the same kind of executable envelope used by the in-process tests.
fn simulation_job(name: &str, deadline_at_ms: u64) -> JobEnvelope {
    let x = identifier("x");
    let world = World::new(
        [Variable::new(x.clone(), VariableRole::State)],
        [],
        [ContinuousLaw::new(x.clone(), Expr::symbol(x.clone()))],
    )
    .unwrap();
    JobEnvelope::new(
        name,
        1,
        10,
        deadline_at_ms,
        resources(),
        Job::Simulate {
            world,
            config: SimulationConfig::new(0.0, 1.0, 0.01).unwrap(),
            request: SimulationRequest {
                initial_state: BTreeMap::from([(x, 1.0)]),
                ..Default::default()
            },
        },
    )
    .unwrap()
}

fn make_scheduler() -> Scheduler<MemoryStore> {
    let config = SchedulerConfig::new(8, 2, Duration::from_millis(50), 8192).unwrap();
    Scheduler::new(config, MemoryStore::default()).unwrap()
}

fn fixed_clock_server(
    scheduler: &Arc<Mutex<Scheduler<MemoryStore>>>,
) -> SchedulerServer<MemoryStore> {
    let clock: Clock = Arc::new(|| FIXED_NOW_MS);
    SchedulerServer::new(Arc::clone(scheduler), clock)
}

fn body_text(response: &HttpResponse) -> String {
    String::from_utf8(response.body.clone()).unwrap()
}

fn get(path: &str) -> HttpRequest {
    HttpRequest::new("GET", path, Vec::new(), Vec::new())
}

fn post(path: &str, body: &str) -> HttpRequest {
    HttpRequest::new(
        "POST",
        path,
        vec![("Content-Type".into(), "application/json".into())],
        body.as_bytes().to_vec(),
    )
}

#[test]
fn control_plane_registers_observes_cancels_recovers_and_reports_health() {
    let scheduler = Arc::new(Mutex::new(make_scheduler()));
    let server = fixed_clock_server(&scheduler);

    // Register a worker pool over the control plane.
    let registered = server.handle(&post(
        "/pools",
        "{\"id\":\"cpu-a\",\"cpu_millis\":500,\"memory_bytes\":4096,\"disk_bytes\":4096}",
    ));
    assert_eq!(registered.status, 201);
    assert!(body_text(&registered).contains("cpu-a"));

    // Submit executable work in-process: the HTTP surface never dispatches jobs.
    scheduler.lock().unwrap().submit(simulation_job("job-1", 5_000), 100).unwrap();

    // The queued job is observable over HTTP.
    let queued = server.handle(&get("/jobs/job-1"));
    assert_eq!(queued.status, 200);
    let queued_body = body_text(&queued);
    assert!(queued_body.contains("\"job_id\":\"job-1\""));
    assert!(queued_body.contains("\"name\":\"queued\""));

    // Cancelling transitions the job and echoes its new state.
    let cancelled = server.handle(&post("/jobs/job-1/cancel", "{\"reason\":\"operator stop\"}"));
    assert_eq!(cancelled.status, 200);
    let cancelled_body = body_text(&cancelled);
    assert!(cancelled_body.contains("\"name\":\"cancelled\""));
    assert!(cancelled_body.contains("operator stop"));

    // The durable checkpoint reflects the cancellation.
    let checkpoint = server.handle(&get("/jobs/job-1/checkpoint"));
    assert_eq!(checkpoint.status, 200);
    let checkpoint_body = body_text(&checkpoint);
    assert!(checkpoint_body.contains("\"job_id\":\"job-1\""));
    assert!(checkpoint_body.contains("\"name\":\"cancelled\""));
    assert!(checkpoint_body.contains("\"sequence\":2"));

    // Cancelling a terminal job is an invalid transition, surfaced as a 409.
    let again = server.handle(&post("/jobs/job-1/cancel", "{\"reason\":\"redundant\"}"));
    assert_eq!(again.status, 409);
    assert!(body_text(&again).contains("\"code\":\"invalid_transition\""));

    // No leased jobs are outstanding, so recovery reports zero.
    let recovered = server.handle(&post("/recover", ""));
    assert_eq!(recovered.status, 200);
    assert_eq!(body_text(&recovered), "{\"recovered\":0}");

    // Health summarizes the queue depth and the configured bounds.
    let health = server.handle(&get("/health"));
    assert_eq!(health.status, 200);
    let health_body = body_text(&health);
    assert!(health_body.contains("\"queued_count\":0"));
    assert!(health_body.contains("\"maximum_queued_jobs\":8"));
    assert!(health_body.contains("\"lease_duration_ms\":50"));
}

#[test]
fn control_plane_rejects_bad_requests_with_documented_statuses() {
    let scheduler = Arc::new(Mutex::new(make_scheduler()));
    let server = fixed_clock_server(&scheduler);

    // Unknown job -> 404.
    let unknown = server.handle(&get("/jobs/missing"));
    assert_eq!(unknown.status, 404);
    assert!(body_text(&unknown).contains("\"code\":\"unknown_job\""));

    // Wrong method on a known route -> 405 with an Allow header.
    let bad_method = server.handle(&HttpRequest::new("DELETE", "/health", Vec::new(), Vec::new()));
    assert_eq!(bad_method.status, 405);
    assert!(bad_method.headers.iter().any(|(name, value)| name == "Allow" && value == "GET"));

    // Unknown route -> 404 (transport-level, not a domain error).
    let no_route = server.handle(&get("/does-not-exist"));
    assert_eq!(no_route.status, 404);
    assert!(body_text(&no_route).contains("\"code\":\"not_found\""));

    // Duplicate pool registration is rejected as an invalid worker (400): the
    // scheduler treats a re-registration as a bad pool, not a job conflict.
    let pool = "{\"id\":\"cpu-a\",\"cpu_millis\":500,\"memory_bytes\":4096,\"disk_bytes\":4096}";
    assert_eq!(server.handle(&post("/pools", pool)).status, 201);
    let duplicate = server.handle(&post("/pools", pool));
    assert_eq!(duplicate.status, 400);
    assert!(body_text(&duplicate).contains("\"code\":\"invalid_worker\""));

    // Malformed JSON body -> 400.
    let malformed = server.handle(&post("/pools", "not json"));
    assert_eq!(malformed.status, 400);
    assert!(body_text(&malformed).contains("\"code\":\"invalid_body\""));
}

#[test]
fn serves_health_over_a_real_socket() {
    let scheduler = Arc::new(Mutex::new(make_scheduler()));
    let clock: Clock = Arc::new(|| FIXED_NOW_MS);
    let server = SchedulerServer::new(Arc::clone(&scheduler), clock);

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let serving = thread::spawn(move || {
        // `serve` loops until the listener errors; the client's `Connection:
        // close` and process exit tear it down.
        let _ = server.serve(&listener);
    });

    let mut stream = TcpStream::connect(address).unwrap();
    stream
        .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .unwrap();
    stream.flush().unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();

    assert!(response.starts_with("HTTP/1.1 200 OK\r\n"), "unexpected status line: {response}");
    assert!(response.contains("application/json"));
    assert!(response.contains("\"queued_count\":0"));

    drop(stream);
    // The accept loop lives on its own thread; detaching it is intentional.
    drop(serving);
}
