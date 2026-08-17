//! Route table mapping HTTP requests to the in-process [`ArtifactService`].
//!
//! Routing is a pure function of `(service, now, request)` so it can be tested
//! without opening a socket. Time is supplied by the caller rather than read
//! from a clock here, preserving the deterministic contract of the underlying
//! service. Every domain error is translated through [`crate::http_error`].

use crate::http::HttpResponse;
use crate::json::Json;
use crate::{
    ArtifactId, ArtifactMetadata, ArtifactService, HttpRequest, Retention, UploadId, UploadOptions,
};

const OCTET_STREAM: &str = "application/octet-stream";

/// Resolves a parsed request against the service, returning a ready response.
pub fn route(service: &ArtifactService, now: u64, request: &HttpRequest) -> HttpResponse {
    let segments: Vec<&str> = request.path.split('/').filter(|part| !part.is_empty()).collect();
    match segments.as_slice() {
        [] => HttpResponse::json(
            200,
            &Json::Object(vec![("service".into(), Json::string("lawsynth-artifact"))]),
        ),
        ["health"] => require(request, "GET", || health(service)),
        ["artifacts"] => require(request, "POST", || ingest(service, now, request)),
        ["artifacts", id] => match request.method.as_str() {
            "GET" => download(service, now, id),
            "DELETE" => delete(service, id),
            _ => method_not_allowed(&["GET", "DELETE"]),
        },
        ["artifacts", id, "metadata"] => require(request, "GET", || describe(service, now, id)),
        ["uploads"] => require(request, "POST", || begin_multipart(service, request)),
        ["uploads", id] => require(request, "DELETE", || abort_multipart(service, id)),
        ["uploads", id, "complete"] => {
            require(request, "POST", || complete_multipart(service, now, id))
        }
        ["uploads", id, "parts", number] => {
            require(request, "PUT", || add_part(service, request, id, number))
        }
        ["gc"] => require(request, "POST", || collect_garbage(service, now, request)),
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

fn health(service: &ArtifactService) -> HttpResponse {
    match service.health() {
        Ok(report) => HttpResponse::json(
            200,
            &Json::Object(vec![
                ("artifact_count".into(), Json::Number(report.artifact_count as u64)),
                ("stored_data_bytes".into(), Json::Number(report.stored_data_bytes)),
                ("capacity_bytes".into(), Json::Number(report.capacity_bytes)),
            ]),
        ),
        Err(error) => HttpResponse::error(&error),
    }
}

fn ingest(service: &ArtifactService, now: u64, request: &HttpRequest) -> HttpResponse {
    let options = match upload_options(request) {
        Ok(options) => options,
        Err(response) => return response,
    };
    match service.ingest(request.body.clone(), options, now) {
        Ok(metadata) => HttpResponse::json(201, &metadata_json(&metadata))
            .with_header("Location", format!("/artifacts/{}", metadata.id)),
        Err(error) => HttpResponse::error(&error),
    }
}

fn download(service: &ArtifactService, now: u64, id: &str) -> HttpResponse {
    let id = match ArtifactId::new(id.to_owned()) {
        Ok(id) => id,
        Err(error) => return HttpResponse::error(&error),
    };
    match service.get(&id, now) {
        Ok(artifact) => {
            let content_type =
                artifact.metadata.content_type.clone().unwrap_or_else(|| OCTET_STREAM.to_owned());
            HttpResponse::bytes(200, &content_type, artifact.bytes)
                .with_header("ETag", format!("\"{}\"", artifact.metadata.id))
        }
        Err(error) => HttpResponse::error(&error),
    }
}

fn describe(service: &ArtifactService, now: u64, id: &str) -> HttpResponse {
    let id = match ArtifactId::new(id.to_owned()) {
        Ok(id) => id,
        Err(error) => return HttpResponse::error(&error),
    };
    match service.describe(&id, now) {
        Ok(metadata) => HttpResponse::json(200, &metadata_json(&metadata)),
        Err(error) => HttpResponse::error(&error),
    }
}

fn delete(service: &ArtifactService, id: &str) -> HttpResponse {
    let id = match ArtifactId::new(id.to_owned()) {
        Ok(id) => id,
        Err(error) => return HttpResponse::error(&error),
    };
    match service.delete(&id) {
        Ok(true) => HttpResponse::empty(204),
        Ok(false) => HttpResponse::error_code(404, "not_found", "artifact does not exist"),
        Err(error) => HttpResponse::error(&error),
    }
}

fn begin_multipart(service: &ArtifactService, request: &HttpRequest) -> HttpResponse {
    let options = match upload_options(request) {
        Ok(options) => options,
        Err(response) => return response,
    };
    match service.begin_multipart(options) {
        Ok(id) => HttpResponse::json(
            201,
            &Json::Object(vec![("upload_id".into(), Json::string(id.to_string()))]),
        )
        .with_header("Location", format!("/uploads/{id}")),
        Err(error) => HttpResponse::error(&error),
    }
}

fn add_part(
    service: &ArtifactService,
    request: &HttpRequest,
    id: &str,
    number: &str,
) -> HttpResponse {
    let id = match UploadId::parse(id) {
        Ok(id) => id,
        Err(error) => return HttpResponse::error(&error),
    };
    let Ok(number) = number.parse::<u32>() else {
        return HttpResponse::error_code(400, "invalid_part", "part number must be an integer");
    };
    match service.add_multipart_part(&id, number, request.body.clone()) {
        Ok(()) => HttpResponse::empty(204),
        Err(error) => HttpResponse::error(&error),
    }
}

fn complete_multipart(service: &ArtifactService, now: u64, id: &str) -> HttpResponse {
    let id = match UploadId::parse(id) {
        Ok(id) => id,
        Err(error) => return HttpResponse::error(&error),
    };
    match service.complete_multipart(&id, now) {
        Ok(metadata) => HttpResponse::json(201, &metadata_json(&metadata))
            .with_header("Location", format!("/artifacts/{}", metadata.id)),
        Err(error) => HttpResponse::error(&error),
    }
}

fn abort_multipart(service: &ArtifactService, id: &str) -> HttpResponse {
    let id = match UploadId::parse(id) {
        Ok(id) => id,
        Err(error) => return HttpResponse::error(&error),
    };
    if service.abort_multipart(&id) {
        HttpResponse::empty(204)
    } else {
        HttpResponse::error_code(404, "not_found", "no such upload session")
    }
}

fn collect_garbage(service: &ArtifactService, now: u64, request: &HttpRequest) -> HttpResponse {
    let dry_run = request.query_param("dry_run").as_deref() == Some("true");
    match service.collect_garbage(now, dry_run) {
        Ok(report) => {
            let deleted =
                report.deleted.iter().map(|id| Json::string(id.to_string())).collect::<Vec<_>>();
            HttpResponse::json(
                200,
                &Json::Object(vec![
                    ("examined".into(), Json::Number(report.examined as u64)),
                    ("dry_run".into(), Json::Bool(dry_run)),
                    ("deleted".into(), Json::Array(deleted)),
                ]),
            )
        }
        Err(error) => HttpResponse::error(&error),
    }
}

/// Builds validated [`UploadOptions`] from the `Content-Type` and
/// `X-Retention-Expires-At` headers, rejecting a malformed retention value.
fn upload_options(request: &HttpRequest) -> Result<UploadOptions, HttpResponse> {
    let content_type = request
        .header("content-type")
        .filter(|value| !value.is_empty())
        .map(|value| value.to_owned());
    let retention = match request.header("x-retention-expires-at") {
        Some(value) => match value.parse::<u64>() {
            Ok(expires_at) => Retention::until(expires_at),
            Err(_) => {
                return Err(HttpResponse::error_code(
                    400,
                    "invalid_metadata",
                    "x-retention-expires-at must be an unsigned Unix timestamp",
                ));
            }
        },
        None => Retention::default(),
    };
    Ok(UploadOptions { content_type, retention })
}

fn metadata_json(metadata: &ArtifactMetadata) -> Json {
    Json::Object(vec![
        ("id".into(), Json::string(metadata.id.to_string())),
        ("sha256".into(), Json::string(metadata.sha256.clone())),
        ("size_bytes".into(), Json::Number(metadata.size_bytes)),
        ("created_at_unix_seconds".into(), Json::Number(metadata.created_at_unix_seconds)),
        (
            "content_type".into(),
            match &metadata.content_type {
                Some(value) => Json::string(value.clone()),
                None => Json::Null,
            },
        ),
        (
            "expires_at_unix_seconds".into(),
            match metadata.retention.expires_at_unix_seconds {
                Some(value) => Json::Number(value),
                None => Json::Null,
            },
        ),
    ])
}
