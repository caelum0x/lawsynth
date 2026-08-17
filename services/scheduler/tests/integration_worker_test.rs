//! Cross-service integration: the scheduler and the worker agreeing on one
//! job's lifecycle, observed over BOTH services' real HTTP transports.
//!
//! The executable `JobEnvelope` has no wire codec, so dispatch is deliberately
//! in-process (submit -> lease -> execute -> complete). What crosses the network
//! is each service's *serializable* view of the job: the scheduler's control
//! plane (`/jobs/{id}`, `/jobs/{id}/checkpoint`) and the worker's status surface
//! (`/jobs`, `/jobs/{id}`). This test drives the real lifecycle across both
//! servers and then asserts, over separate raw client sockets to each service,
//! that they independently report the SAME job (`lifecycle-1`) as completed.
//!
//! Determinism: every time input is injected (scheduler `now_ms`, the worker's
//! `execute_at` instant, and both HTTP clocks are fixed), so checkpoint sequence
//! numbers and states are reproducible with no wall-clock assertions.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_runner::{CancellationToken, ResourceRequest};
use lawsynth_scheduler::{
    Clock as SchedulerClock, JobState, Scheduler, SchedulerConfig, SchedulerServer, WorkerPool,
};
use lawsynth_sim::{SimulationConfig, SimulationRequest};
use lawsynth_store::{LocalStore, MemoryStore, StoreConfig};
use lawsynth_worker::{
    Clock as WorkerClock, Job, JobEnvelope, JobOutput, Worker, WorkerConfig, WorkerServer,
};
use lawsynth_world::{ContinuousLaw, Variable, VariableRole, World};

/// The job id shared by every service; URL-safe so the worker's status route
/// (which validates the path id) and the scheduler both accept it verbatim.
const JOB_ID: &str = "lifecycle-1";

fn identifier(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}

fn job_resources() -> ResourceRequest {
    ResourceRequest::new(250, 1024, 1024).unwrap()
}

/// Builds the same executable simulation envelope the in-process tests use.
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
        job_resources(),
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

/// A scheduler with a single registered pool able to hold the simulation job.
fn build_scheduler() -> Scheduler<MemoryStore> {
    let config = SchedulerConfig::new(8, 2, Duration::from_millis(50), 8192).unwrap();
    let mut scheduler = Scheduler::new(config, MemoryStore::default()).unwrap();
    scheduler
        .register_pool(
            WorkerPool::new("cpu-a", ResourceRequest::new(500, 4096, 4096).unwrap()).unwrap(),
        )
        .unwrap();
    scheduler
}

/// Owns a unique temp directory backing the worker's durable store.
struct WorkerRoot {
    path: PathBuf,
}

impl WorkerRoot {
    fn new() -> Self {
        let path = std::env::temp_dir()
            .join(format!("lawsynth-scheduler-worker-http-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        Self { path }
    }
}

impl Drop for WorkerRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// Serves the scheduler control plane on an ephemeral port; returns the address.
fn serve_scheduler(scheduler: Arc<Mutex<Scheduler<MemoryStore>>>) -> String {
    let clock: SchedulerClock = Arc::new(|| 1_000);
    let server = SchedulerServer::new(scheduler, clock);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap().to_string();
    thread::spawn(move || {
        let _ = server.serve(&listener);
    });
    address
}

/// Serves the worker status surface on an ephemeral port; returns the address.
fn serve_worker(worker: Arc<Worker<LocalStore>>) -> String {
    let clock: WorkerClock = Arc::new(|| 1);
    let server = WorkerServer::new(worker, clock);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap().to_string();
    thread::spawn(move || {
        let _ = server.serve(&listener);
    });
    address
}

/// Issues a `GET` over a real socket and returns `(status, body)`.
fn http_get(address: &str, path: &str) -> (u16, String) {
    let mut stream = connect_with_retry(address);
    let request = format!("GET {path} HTTP/1.1\r\nHost: local\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).unwrap();
    stream.flush().unwrap();
    let mut raw = String::new();
    stream.read_to_string(&mut raw).unwrap();
    let split = raw.find("\r\n\r\n").expect("header terminator");
    let status = raw
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .expect("status code");
    (status, raw[split + 4..].to_owned())
}

/// Connects with a small bounded retry loop; the listener is already bound before
/// its serve thread starts, so this only smooths over scheduling jitter.
fn connect_with_retry(address: &str) -> TcpStream {
    let mut last_error = None;
    for _ in 0..50 {
        match TcpStream::connect(address) {
            Ok(stream) => return stream,
            Err(error) => {
                last_error = Some(error);
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
    panic!("could not connect to {address}: {last_error:?}");
}

#[test]
fn scheduler_and_worker_agree_on_a_completed_job_over_http() {
    let scheduler = Arc::new(Mutex::new(build_scheduler()));

    // Dispatch is in-process: submit the job and lease it to the "cpu-a" pool.
    let lease = {
        let mut guard = scheduler.lock().unwrap();
        guard.submit(simulation_job(JOB_ID, 5_000), 10).unwrap();
        guard.lease_next("cpu-a", 20).unwrap().expect("a job is available to lease")
    };
    assert_eq!(lease.envelope.work.id, JOB_ID, "leased the expected envelope");

    // The worker executes the leased envelope, writing durable checkpoints
    // (Running -> Completed) to its own store.
    let worker_root = WorkerRoot::new();
    let worker = Arc::new(
        Worker::new(
            WorkerConfig::new(ResourceRequest::new(1_000, 1 << 20, 1 << 20).unwrap(), 1024)
                .unwrap(),
            LocalStore::open(&worker_root.path, StoreConfig::default()).unwrap(),
        )
        .unwrap(),
    );
    let output = worker.execute_at(&lease.envelope, &CancellationToken::default(), 21).unwrap();
    let JobOutput::Simulation(trajectory) = output else {
        panic!("scheduler dispatched the wrong kind of work");
    };
    assert_eq!(trajectory.samples(), 101, "the worker ran the real RK4 integration");

    // The scheduler records completion in-process, fenced by the lease token.
    {
        let mut guard = scheduler.lock().unwrap();
        guard.complete(&lease.token, 22).unwrap();
        assert_eq!(guard.state(JOB_ID).unwrap(), &JobState::Completed);
    }

    // Bring up both services' HTTP transports over real sockets.
    let scheduler_addr = serve_scheduler(Arc::clone(&scheduler));
    let worker_addr = serve_worker(Arc::clone(&worker));

    // The scheduler's control plane reports the job completed...
    let (status, body) = http_get(&scheduler_addr, &format!("/jobs/{JOB_ID}"));
    assert_eq!(status, 200, "scheduler job state: {body}");
    assert!(body.contains(&format!("\"job_id\":\"{JOB_ID}\"")), "scheduler job id: {body}");
    assert!(body.contains("\"name\":\"completed\""), "scheduler job not completed: {body}");

    // ...and its durable checkpoint agrees, at the third recorded transition
    // (submit -> lease -> complete).
    let (status, checkpoint) = http_get(&scheduler_addr, &format!("/jobs/{JOB_ID}/checkpoint"));
    assert_eq!(status, 200, "scheduler checkpoint: {checkpoint}");
    assert!(
        checkpoint.contains("\"name\":\"completed\""),
        "scheduler checkpoint state: {checkpoint}"
    );
    assert!(checkpoint.contains("\"sequence\":3"), "scheduler checkpoint sequence: {checkpoint}");

    // The worker's status surface independently reports the SAME job as a
    // completed, terminal checkpoint (its own Running -> Completed sequence).
    let (status, worker_job) = http_get(&worker_addr, &format!("/jobs/{JOB_ID}"));
    assert_eq!(status, 200, "worker job checkpoint: {worker_job}");
    assert!(
        worker_job.contains(&format!("\"job_id\":\"{JOB_ID}\"")),
        "worker job id: {worker_job}"
    );
    assert!(
        worker_job.contains("\"state\":\"completed\""),
        "worker state not completed: {worker_job}"
    );
    assert!(
        worker_job.contains("\"terminal\":true"),
        "worker checkpoint not terminal: {worker_job}"
    );
    assert!(worker_job.contains("\"sequence\":2"), "worker checkpoint sequence: {worker_job}");

    // The worker also advertises the job in its list of known checkpoints.
    let (status, jobs) = http_get(&worker_addr, "/jobs");
    assert_eq!(status, 200, "worker jobs list: {jobs}");
    assert!(jobs.contains(&format!("\"{JOB_ID}\"")), "worker did not list the job: {jobs}");
}
