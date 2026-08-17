//! Route table mapping HTTP requests to the in-process [`Worker`] status surface.
//!
//! Routing is a pure function of `(worker, now, request)` so it can be tested
//! without opening a socket. Time is supplied by the caller rather than read
//! from a clock here, preserving the deterministic contract of the worker.
//!
//! Only serializable observability is exposed: readiness plus the config and
//! admission summary (`GET /health`), the set of durable checkpoints the worker
//! has recorded (`GET /jobs`), and one job's lifecycle checkpoint
//! (`GET /jobs/{id}`). There is deliberately no route that accepts an executable
//! job, because [`crate::JobEnvelope`] carries typed, in-memory payloads with no
//! wire codec. Every domain error is translated through [`crate::http_error`].

use lawsynth_runner::ResourceRequest;
use lawsynth_store::ObjectStore;

use crate::http::HttpResponse;
use crate::json::Json;
use crate::{HttpRequest, JobCheckpoint, TransportSurface, Worker};

/// Resolves a parsed request against the worker, returning a ready response.
pub fn route<S: ObjectStore>(worker: &Worker<S>, now: u64, request: &HttpRequest) -> HttpResponse {
    let segments: Vec<&str> = request.path.split('/').filter(|part| !part.is_empty()).collect();
    match segments.as_slice() {
        [] => HttpResponse::json(
            200,
            &Json::Object(vec![("service".into(), Json::string("lawsynth-worker"))]),
        ),
        ["health"] => require(request, "GET", || health(worker, now)),
        ["jobs"] => require(request, "GET", || list_jobs(worker)),
        ["jobs", id] => require(request, "GET", || job_checkpoint(worker, id)),
        _ => HttpResponse::error_code(404, "not_found", "no route matches the request path"),
    }
}

/// Enforces the single method a route accepts, otherwise answers `405`.
fn require(
    request: &HttpRequest,
    method: &'static str,
    handler: impl FnOnce() -> HttpResponse,
) -> HttpResponse {
    if request.method == method { handler() } else { method_not_allowed(&[method]) }
}

fn method_not_allowed(allowed: &[&str]) -> HttpResponse {
    HttpResponse::error_code(405, "method_not_allowed", "the method is not supported here")
        .with_header("Allow", allowed.join(", "))
}

/// Readiness plus the config and live admission summary. The worker is ready as
/// long as it is running; capacity pressure is reported through the admission
/// snapshot rather than by flipping readiness.
fn health<S: ObjectStore>(worker: &Worker<S>, now: u64) -> HttpResponse {
    let admission = worker.admission();
    let config = worker.config();
    HttpResponse::json(
        200,
        &Json::Object(vec![
            ("service".into(), Json::string("lawsynth-worker")),
            ("ready".into(), Json::Bool(true)),
            ("checked_at_unix_seconds".into(), Json::Number(now)),
            (
                "transport".into(),
                Json::Object(vec![
                    ("surface".into(), Json::string("http-status")),
                    ("accepts_jobs".into(), Json::Bool(false)),
                    ("reason".into(), Json::string(TransportSurface::HttpStatus.reason())),
                ]),
            ),
            ("capacity".into(), resources_json(admission.capacity)),
            ("reserved".into(), resources_json(admission.reserved)),
            ("available".into(), resources_json(admission.available)),
            (
                "maximum_checkpoint_bytes".into(),
                Json::Number(config.maximum_checkpoint_bytes as u64),
            ),
        ]),
    )
}

/// Lists the ids of every job for which the worker holds a durable checkpoint.
fn list_jobs<S: ObjectStore>(worker: &Worker<S>) -> HttpResponse {
    match worker.known_checkpoints() {
        Ok(mut ids) => {
            ids.sort();
            let jobs = ids.into_iter().map(Json::string).collect::<Vec<_>>();
            HttpResponse::json(200, &Json::Object(vec![("jobs".into(), Json::Array(jobs))]))
        }
        Err(error) => HttpResponse::error(&error),
    }
}

/// Returns one job's durable lifecycle checkpoint, or `404` if none is recorded.
fn job_checkpoint<S: ObjectStore>(worker: &Worker<S>, id: &str) -> HttpResponse {
    if !is_valid_job_id(id) {
        return HttpResponse::error_code(
            400,
            "invalid_job_id",
            "job id must be URL-safe and no longer than 128 bytes",
        );
    }
    match worker.checkpoint(id) {
        Ok(Some(record)) => HttpResponse::json(200, &checkpoint_json(&record)),
        Ok(None) => HttpResponse::error_code(
            404,
            "not_found",
            "the worker has no lifecycle record for this job",
        ),
        Err(error) => HttpResponse::error(&error),
    }
}

/// Validates a path-supplied job id against the same URL-safe rule the executable
/// [`crate::JobEnvelope`] enforces, so a hostile path can never reach the object
/// store's key builder.
fn is_valid_job_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn resources_json(resources: ResourceRequest) -> Json {
    Json::Object(vec![
        ("cpu_millis".into(), Json::Number(u64::from(resources.cpu_millis))),
        ("memory_bytes".into(), Json::Number(resources.memory_bytes)),
        ("disk_bytes".into(), Json::Number(resources.disk_bytes)),
    ])
}

fn checkpoint_json(record: &JobCheckpoint) -> Json {
    Json::Object(vec![
        ("job_id".into(), Json::string(record.job_id.clone())),
        ("sequence".into(), Json::Number(record.sequence)),
        ("recorded_at_ms".into(), Json::Number(record.recorded_at_ms)),
        ("state".into(), Json::string(record.state.as_str())),
        ("terminal".into(), Json::Bool(record.state.is_terminal())),
        ("detail".into(), Json::string(record.detail.clone())),
    ])
}
