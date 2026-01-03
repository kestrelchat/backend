use rocket::{Request, catch, http::Status, response::status::Custom};
use rocket::response::Responder;
use rocket::serde::json::Json;
use serde::Serialize;
use ulid::Ulid;

use kestrel_config::{capture_message, Level};

// Error shapes returned by the API in JSON. These mirror the shape
// consumed by clients and include a machine-readable `code` and the
// numeric HTTP `status`.
#[derive(Serialize)]
pub struct ErrorObject {
    code: String,
    status: u16,
}

// The top-level error response includes the `ErrorObject` and a
// `request_id` that can be used to correlate logs and Sentry events
// with the response returned to the client.
#[derive(Serialize)]
pub struct ErrorResponse {
    error: ErrorObject,
    request_id: String,
}

// Application-level error type that can be returned by handlers.
// It implements `Responder` so handlers can return `AppError` directly
// and have it converted into the JSON error shape and proper status.
#[derive(Debug)]
pub struct AppError {
    pub code: String,
    pub status: Status,
    pub level: Level,
}

impl AppError {
    // Construct a new `AppError` with a short `code`, an HTTP `status`
    // and a Sentry `level` to control how the error is reported.
    pub fn new(code: impl Into<String>, status: Status, level: Level) -> Self {
        Self {
            code: code.into(),
            status,
            level,
        }
    }
}

// Helper to build the JSON error response, attach Sentry metadata and
// send the event. This centralizes request-id generation, Sentry
// fingerprinting and avoids duplicating the logic in each catcher.
fn make_response(code: &str, status: Status, req: &Request<'_>, level: Level) -> Custom<Json<ErrorResponse>> {
    // Generate a globally unique request id to return to clients and
    // attach to Sentry events for correlation.
    let request_id = Ulid::new().to_string();

    let body = ErrorResponse {
        error: ErrorObject {
            code: code.to_string(),
            status: status.code,
        },
        request_id: request_id.clone(),
    };

    // Ignore common noise like the browser requesting `/favicon.ico`.
    let req_path = req.uri().path();
    if req_path.ends_with("/favicon.ico") {
        return Custom(status, Json(body));
    }

    // Build a fingerprint and route tag for Sentry so similar errors
    // from the same route are grouped together.
    let (fingerprint, route_tag) = {
        let route_label = if let Some(route) = req.route() {
            route.uri.to_string()
        } else {
            req.uri().path().to_string()
        };

        let route_label = route_label.to_uppercase();
        (format!("{} - {}", code, route_label), Some(route_label))
    };

    // Attach useful tags/metadata to the Sentry scope and capture the
    // message at the provided level. This keeps telemetry consistent
    // across the application.
    sentry::with_scope(
        |scope| {
            scope.set_fingerprint(Some(&[fingerprint.as_str()]));
            if let Some(rt) = &route_tag {
                scope.set_tag("route", rt);
            }
            scope.set_tag("request_id", request_id.as_str());
            scope.set_tag("method", req.method().as_str());
            scope.set_tag("path", &req.uri().to_string());
        },
        || {
            capture_message(&fingerprint, level);
        },
    );

    Custom(status, Json(body))
}

// `Responder` implementation for `AppError` so handlers can `return
// Err(AppError::new(...))` and get a consistent JSON error payload.
impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let request_id = Ulid::new().to_string();
        let body = ErrorResponse {
            error: ErrorObject {
                code: self.code,
                status: self.status.code,
            },
            request_id: request_id.clone(),
        };

        // Send an aggregated message to Sentry including the HTTP
        // method, path and generated request id for troubleshooting.
        capture_message(&format!("{} {} -> {} (request_id={})", req.method(), req.uri(), self.status.code, request_id), self.level);

        let custom = Custom(self.status, Json(body));
        custom.respond_to(req)
    }
}

// Default catcher, any unmapped status code falls back to UNKNOWN_ERROR.
#[catch(default)]
pub fn default_catcher(status: Status, req: &Request<'_>) -> Custom<Json<ErrorResponse>> {
    make_response("UNKNOWN_ERROR", status, req, Level::Error)
}

/// 400 Bad Request
#[catch(400)]
pub fn bad_request(req: &Request<'_>) -> Custom<Json<ErrorResponse>> {
    make_response("BAD_REQUEST", Status::BadRequest, req, Level::Warning)
}

/// 401 Unauthorized
#[catch(401)]
pub fn unauthorized(req: &Request<'_>) -> Custom<Json<ErrorResponse>> {
    make_response("UNAUTHORIZED", Status::Unauthorized, req, Level::Warning)
}

/// 403 Forbidden
#[catch(403)]
pub fn forbidden(req: &Request<'_>) -> Custom<Json<ErrorResponse>> {
    make_response("FORBIDDEN", Status::Forbidden, req, Level::Warning)
}

/// 404 Not Found
#[catch(404)]
pub fn not_found(req: &Request<'_>) -> Custom<Json<ErrorResponse>> {
    make_response("NOT_FOUND", Status::NotFound, req, Level::Error)
}

/// 405 Method Not Allowed
#[catch(405)]
pub fn method_not_allowed(req: &Request<'_>) -> Custom<Json<ErrorResponse>> {
    make_response("METHOD_NOT_ALLOWED", Status::MethodNotAllowed, req, Level::Warning)
}

/// 406 Not Acceptable
#[catch(406)]
pub fn not_acceptable(req: &Request<'_>) -> Custom<Json<ErrorResponse>> {
    make_response("NOT_ACCEPTABLE", Status::NotAcceptable, req, Level::Warning)
}

/// 422 Unprocessable Entity
#[catch(422)]
pub fn unprocessable_entity(req: &Request<'_>) -> Custom<Json<ErrorResponse>> {
    make_response("UNPROCESSABLE_ENTITY", Status::UnprocessableEntity, req, Level::Warning)
}


/// 429 Too Many Requests
#[catch(429)]
pub fn too_many_requests(req: &Request<'_>) -> Custom<Json<ErrorResponse>> {
    make_response("TOO_MANY_REQUESTS", Status::TooManyRequests, req, Level::Warning)
}

/// 500 Internal Server Error
#[catch(500)]
pub fn internal_error(req: &Request<'_>) -> Custom<Json<ErrorResponse>> {
    make_response("INTERNAL_ERROR", Status::InternalServerError, req, Level::Fatal)
}

/// 503 Service Unavailable
#[catch(503)]
pub fn service_unavailable(req: &Request<'_>) -> Custom<Json<ErrorResponse>> {
    make_response("SERVICE_UNAVAILABLE", Status::ServiceUnavailable, req, Level::Error)
}
