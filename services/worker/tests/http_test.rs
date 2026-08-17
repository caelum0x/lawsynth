//! Transport-level tests for the worker HTTP status server.
//!
//! Most cases drive the router through `WorkerServer::handle` with a fixed
//! clock, keeping them deterministic and socket-free. One case exercises the
//! real `std::net` accept loop end to end. All routes are read-only status:
//! there is no path that accepts an executable job.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_runner::{CancellationToken, ResourceRequest};
use lawsynth_sim::{SimulationConfig, SimulationRequest};
use lawsynth_store::{LocalStore, StoreConfig};
use lawsynth_worker::{
    Clock, HttpRequest, HttpResponse, Job, JobEnvelope, Worker, WorkerConfig, WorkerServer,
};
use lawsynth_world::{ContinuousLaw, Variable, VariableRole, World};

const FIXED_NOW: u64 = 1_700_000_000;

fn id(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}

fn root(label: &str) -> std::path::PathBuf {
    let unique = format!("lawsynth-worker-http-{label}-{}", std::process::id());
    let path = std::env::temp_dir().join(unique);
    let _ = std::fs::remove_dir_all(&path);
    path
}

fn worker_at(label: &str) -> (Arc<Worker<LocalStore>>, std::path::PathBuf) {
    let path = root(label);
    let store = LocalStore::open(&path, StoreConfig::default()).unwrap();
    let config =
        WorkerConfig::new(ResourceRequest::new(1_000, 1 << 20, 1 << 20).unwrap(), 1 << 10).unwrap();
    (Arc::new(Worker::new(config, store).unwrap()), path)
}

/// A real, runnable simulation job mirroring the worker's own execution tests.
fn simulation_job(name: &str) -> JobEnvelope {
    let x = id("x");
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
        1_000,
        ResourceRequest::new(250, 1024, 1024).unwrap(),
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

fn server(worker: &Arc<Worker<LocalStore>>) -> WorkerServer<LocalStore> {
    let clock: Clock = Arc::new(|| FIXED_NOW);
    WorkerServer::new(Arc::clone(worker), clock)
}

fn get(path: &str) -> HttpRequest {
    HttpRequest::new("GET", path, Vec::new(), Vec::new())
}

fn body_text(response: &HttpResponse) -> String {
    String::from_utf8(response.body.clone()).unwrap()
}

#[test]
fn health_reports_readiness_admission_and_config() {
    let (worker, path) = worker_at("health");
    let server = server(&worker);

    let health = server.handle(&get("/health"));
    assert_eq!(health.status, 200);
    let text = body_text(&health);
    assert!(text.contains("\"service\":\"lawsynth-worker\""));
    assert!(text.contains("\"ready\":true"));
    assert!(text.contains(&format!("\"checked_at_unix_seconds\":{FIXED_NOW}")));
    // The transport is honest that it never accepts jobs.
    assert!(text.contains("\"surface\":\"http-status\""));
    assert!(text.contains("\"accepts_jobs\":false"));
    // Real admission and config numbers are surfaced.
    assert!(text.contains("\"capacity\":{\"cpu_millis\":1000,"));
    assert!(text.contains("\"reserved\":{\"cpu_millis\":0,"));
    assert!(text.contains("\"maximum_checkpoint_bytes\":1024"));

    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn job_checkpoint_is_served_after_a_real_job_runs() {
    let (worker, path) = worker_at("checkpoint");
    // Execute a genuine RK4 simulation so a durable checkpoint is recorded.
    worker.execute_at(&simulation_job("sim-http-1"), &CancellationToken::default(), 20).unwrap();

    let server = server(&worker);
    let response = server.handle(&get("/jobs/sim-http-1"));
    assert_eq!(response.status, 200);
    let text = body_text(&response);
    assert!(text.contains("\"job_id\":\"sim-http-1\""));
    assert!(text.contains("\"state\":\"completed\""));
    assert!(text.contains("\"terminal\":true"));
    assert!(text.contains("\"sequence\":2"));

    // The job also appears in the enumerated checkpoint set.
    let listing = server.handle(&get("/jobs"));
    assert_eq!(listing.status, 200);
    assert!(body_text(&listing).contains("\"sim-http-1\""));

    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn unknown_job_bad_method_and_unknown_route_are_mapped() {
    let (worker, path) = worker_at("errors");
    let server = server(&worker);

    // Well-formed id with no record -> 404.
    let missing = server.handle(&get("/jobs/does-not-exist"));
    assert_eq!(missing.status, 404);
    assert!(body_text(&missing).contains("\"code\":\"not_found\""));

    // Non URL-safe id is rejected before it can reach the store -> 400.
    let invalid = server.handle(&get("/jobs/not%20safe"));
    assert_eq!(invalid.status, 400);
    assert!(body_text(&invalid).contains("\"code\":\"invalid_job_id\""));

    // Wrong method on a known route -> 405 with Allow.
    let bad_method = server.handle(&HttpRequest::new("POST", "/health", Vec::new(), Vec::new()));
    assert_eq!(bad_method.status, 405);
    assert!(bad_method.headers.iter().any(|(name, _)| name == "Allow"));

    // Unknown route -> 404.
    assert_eq!(server.handle(&get("/nope")).status, 404);

    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn serves_health_over_a_real_socket() {
    let (worker, path) = worker_at("socket");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let clock: Clock = Arc::new(|| FIXED_NOW);
    let server = WorkerServer::new(Arc::clone(&worker), clock);
    thread::spawn(move || {
        let _ = server.serve(&listener);
    });

    let raw = read_over_socket(
        address.to_string().as_str(),
        b"GET /health HTTP/1.1\r\nHost: local\r\n\r\n",
    );
    assert!(raw.starts_with("HTTP/1.1 200 OK\r\n"), "unexpected response: {raw}");
    assert!(raw.contains("\"service\":\"lawsynth-worker\""), "unexpected body: {raw}");

    let _ = std::fs::remove_dir_all(path);
}

fn read_over_socket(address: &str, request: &[u8]) -> String {
    let mut stream = TcpStream::connect(address).unwrap();
    stream.write_all(request).unwrap();
    stream.flush().unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}
