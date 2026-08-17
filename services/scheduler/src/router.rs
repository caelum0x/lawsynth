//! Route table mapping HTTP requests to the in-process [`Scheduler`].
//!
//! Routing is a pure function of `(scheduler, now_ms, request)` so it can be
//! tested without opening a socket. Time is supplied by the caller rather than
//! read from a clock here, preserving the deterministic contract of the
//! underlying scheduler. Every domain error is translated through
//! [`crate::http_error`].
//!
//! CONTROL-PLANE ONLY: this table exposes exclusively the scheduler's
//! serializable operations. Lease acquisition, heartbeat, complete, and fail are
//! intentionally absent because they carry or fence executable `JobEnvelope`
//! values, which have no wire codec. Dispatch stays an in-process API call; the
//! network surface is limited to state a client can safely observe and mutate.

use crate::http::{HttpRequest, HttpResponse};
use crate::json::{self, Json};
use crate::{JobState, PersistedCheckpoint, Scheduler, WorkerPool};
use lawsynth_runner::ResourceRequest;
use lawsynth_store::ObjectStore;

/// Resolves a parsed request against the scheduler, returning a ready response.
pub fn route<S: ObjectStore>(
    scheduler: &mut Scheduler<S>,
    now_ms: u64,
    request: &HttpRequest,
) -> HttpResponse {
    let segments: Vec<&str> = request.path.split('/').filter(|part| !part.is_empty()).collect();
    match segments.as_slice() {
        [] => HttpResponse::json(
            200,
            &Json::Object(vec![("service".into(), Json::string("lawsynth-scheduler"))]),
        ),
        ["health"] => require(request, "GET", || health(scheduler)),
        ["pools"] => require(request, "POST", || register_pool(scheduler, request)),
        ["recover"] => require(request, "POST", || recover(scheduler, now_ms)),
        ["jobs", id] => require(request, "GET", || job_state(scheduler, id)),
        ["jobs", id, "checkpoint"] => require(request, "GET", || job_checkpoint(scheduler, id)),
        ["jobs", id, "cancel"] => {
            require(request, "POST", || cancel_job(scheduler, request, id, now_ms))
        }
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

fn health<S: ObjectStore>(scheduler: &Scheduler<S>) -> HttpResponse {
    let config = scheduler.config();
    HttpResponse::json(
        200,
        &Json::Object(vec![
            ("service".into(), Json::string("lawsynth-scheduler")),
            ("queued_count".into(), Json::Number(scheduler.queued_count() as u64)),
            (
                "config".into(),
                Json::Object(vec![
                    ("maximum_queued_jobs".into(), Json::Number(config.maximum_queued_jobs as u64)),
                    ("maximum_attempts".into(), Json::Number(config.maximum_attempts as u64)),
                    (
                        "lease_duration_ms".into(),
                        Json::Number(
                            config.lease_duration.as_millis().try_into().unwrap_or(u64::MAX),
                        ),
                    ),
                    (
                        "maximum_checkpoint_bytes".into(),
                        Json::Number(config.maximum_checkpoint_bytes as u64),
                    ),
                ]),
            ),
        ]),
    )
}

/// Registers a worker pool from a `{id, cpu_millis, memory_bytes, disk_bytes}`
/// control-plane body. This is metadata only: it never accepts executable work.
fn register_pool<S: ObjectStore>(
    scheduler: &mut Scheduler<S>,
    request: &HttpRequest,
) -> HttpResponse {
    let object = match parse_body(request) {
        Ok(object) => object,
        Err(response) => return response,
    };
    let Some(id) = object.string("id").map(str::to_owned) else {
        return HttpResponse::error_code(400, "invalid_body", "field 'id' must be a string");
    };
    let (Some(cpu_millis), Some(memory_bytes), Some(disk_bytes)) =
        (object.number("cpu_millis"), object.number("memory_bytes"), object.number("disk_bytes"))
    else {
        return HttpResponse::error_code(
            400,
            "invalid_body",
            "'cpu_millis', 'memory_bytes', and 'disk_bytes' must be unsigned numbers",
        );
    };
    let Ok(cpu_millis) = u32::try_from(cpu_millis) else {
        return HttpResponse::error_code(400, "invalid_body", "'cpu_millis' exceeds its range");
    };
    let resources = match ResourceRequest::new(cpu_millis, memory_bytes, disk_bytes) {
        Ok(resources) => resources,
        Err(error) => {
            return HttpResponse::error_code(400, "invalid_resource", &error.to_string());
        }
    };
    let pool = match WorkerPool::new(id, resources) {
        Ok(pool) => pool,
        Err(error) => return HttpResponse::error(&error),
    };
    let pool_id = pool.id.clone();
    match scheduler.register_pool(pool) {
        Ok(()) => HttpResponse::json(
            201,
            &Json::Object(vec![("id".into(), Json::string(pool_id.clone()))]),
        )
        .with_header("Location", format!("/pools/{pool_id}")),
        Err(error) => HttpResponse::error(&error),
    }
}

fn job_state<S: ObjectStore>(scheduler: &Scheduler<S>, id: &str) -> HttpResponse {
    match scheduler.state(id) {
        Ok(state) => HttpResponse::json(
            200,
            &Json::Object(vec![
                ("job_id".into(), Json::string(id)),
                ("state".into(), job_state_json(state)),
            ]),
        ),
        Err(error) => HttpResponse::error(&error),
    }
}

fn job_checkpoint<S: ObjectStore>(scheduler: &Scheduler<S>, id: &str) -> HttpResponse {
    match scheduler.checkpoint(id) {
        Ok(Some(checkpoint)) => HttpResponse::json(200, &checkpoint_json(&checkpoint)),
        Ok(None) => HttpResponse::error_code(404, "not_found", "no checkpoint exists for this job"),
        Err(error) => HttpResponse::error(&error),
    }
}

/// Cancels queued or leased work from a `{reason}` body. Cancellation is a
/// control-plane transition; interrupting an executing worker remains the
/// worker's cooperative responsibility, exactly as the in-process API documents.
fn cancel_job<S: ObjectStore>(
    scheduler: &mut Scheduler<S>,
    request: &HttpRequest,
    id: &str,
    now_ms: u64,
) -> HttpResponse {
    let object = match parse_body(request) {
        Ok(object) => object,
        Err(response) => return response,
    };
    let Some(reason) = object.string("reason") else {
        return HttpResponse::error_code(400, "invalid_body", "field 'reason' must be a string");
    };
    match scheduler.cancel(id, reason, now_ms) {
        Ok(()) => job_state(scheduler, id),
        Err(error) => HttpResponse::error(&error),
    }
}

fn recover<S: ObjectStore>(scheduler: &mut Scheduler<S>, now_ms: u64) -> HttpResponse {
    match scheduler.recover_expired(now_ms) {
        Ok(recovered) => HttpResponse::json(
            200,
            &Json::Object(vec![("recovered".into(), Json::Number(recovered as u64))]),
        ),
        Err(error) => HttpResponse::error(&error),
    }
}

/// Parses a request body as a flat JSON object, rejecting malformed input.
fn parse_body(request: &HttpRequest) -> Result<json::JsonObject, HttpResponse> {
    let text = std::str::from_utf8(&request.body)
        .map_err(|_| HttpResponse::error_code(400, "invalid_body", "body is not valid UTF-8"))?;
    json::parse_object(text).map_err(|reason| HttpResponse::error_code(400, "invalid_body", reason))
}

/// Serializes a [`JobState`] as a tagged JSON object.
fn job_state_json(state: &JobState) -> Json {
    match state {
        JobState::Queued => Json::Object(vec![("name".into(), Json::string("queued"))]),
        JobState::Leased { worker_id, generation, expires_at_ms } => Json::Object(vec![
            ("name".into(), Json::string("leased")),
            ("worker_id".into(), Json::string(worker_id.clone())),
            ("generation".into(), Json::Number(*generation)),
            ("expires_at_ms".into(), Json::Number(*expires_at_ms)),
        ]),
        JobState::Completed => Json::Object(vec![("name".into(), Json::string("completed"))]),
        JobState::Cancelled { reason } => Json::Object(vec![
            ("name".into(), Json::string("cancelled")),
            ("reason".into(), Json::string(reason.clone())),
        ]),
        JobState::DeadLetter { reason } => Json::Object(vec![
            ("name".into(), Json::string("dead_letter")),
            ("reason".into(), Json::string(reason.clone())),
        ]),
    }
}

/// Serializes a [`PersistedCheckpoint`] to JSON. The executable payload is never
/// included — it has no wire codec — so only scheduler-owned lifecycle fields
/// appear here.
fn checkpoint_json(checkpoint: &PersistedCheckpoint) -> Json {
    Json::Object(vec![
        ("job_id".into(), Json::string(checkpoint.job_id.clone())),
        ("attempt".into(), Json::Number(checkpoint.attempt as u64)),
        ("sequence".into(), Json::Number(checkpoint.sequence)),
        ("updated_at_ms".into(), Json::Number(checkpoint.updated_at_ms)),
        ("state".into(), job_state_json(&checkpoint.state)),
    ])
}
