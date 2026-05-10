// Kestrel - a modern instant-messaging service written in Rust
// Copyright (C) 2026 Kestrel Chat
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use kestrel_common::utils::validation::{ValidationError, email, password, username};
use kestrel_postgres::error::DatabaseError;
use rocket::serde::json::Json;
use rocket::{Request, catch, http::Status, response::Responder, response::status::Custom};
use rocket_okapi::OpenApiError;
use rocket_okapi::r#gen::OpenApiGenerator;
use rocket_okapi::okapi::openapi3::{MediaType, RefOr, Response, Responses};
use rocket_okapi::response::OpenApiResponderInner;
use serde::Serialize;
use ulid::Ulid;

#[derive(Serialize)]
pub struct ErrorObject {
    code: String,
    status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    error: ErrorObject,
    request_id: String,
}

#[derive(Debug)]
pub struct AppError {
    pub code: String,
    pub status: Status,
    pub message: Option<String>,
}

impl AppError {
    pub fn new(code: impl Into<String>, status: Status) -> Self {
        Self {
            code: code.into(),
            status,
            message: None,
        }
    }

    pub fn with_message(
        code: impl Into<String>,
        status: Status,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            status,
            message: Some(message.into()),
        }
    }

    pub fn bad_request(code: impl Into<String>) -> Self {
        Self::new(code, Status::BadRequest)
    }
    pub fn unauthorized(code: impl Into<String>) -> Self {
        Self::new(code, Status::Unauthorized)
    }
    pub fn forbidden(code: impl Into<String>) -> Self {
        Self::new(code, Status::Forbidden)
    }
    pub fn not_found(code: impl Into<String>) -> Self {
        Self::new(code, Status::NotFound)
    }
    pub fn conflict(code: impl Into<String>) -> Self {
        Self::new(code, Status::Conflict)
    }
    pub fn internal_error(code: impl Into<String>) -> Self {
        Self::new(code, Status::InternalServerError)
    }
}

impl From<DatabaseError> for AppError {
    fn from(err: DatabaseError) -> Self {
        match err {
            DatabaseError::UniqueViolation(_) => AppError::conflict("UNIQUE_VIOLATION"),
            DatabaseError::NotFound => AppError::not_found("NOT_FOUND"),
            DatabaseError::ForeignKeyViolation => AppError::bad_request("FOREIGN_KEY_VIOLATION"),
            DatabaseError::NotNullViolation => AppError::bad_request("NOT_NULL_VIOLATION"),
            DatabaseError::CheckViolation => AppError::bad_request("CHECK_VIOLATION"),
            _ => AppError::internal_error("DATABASE_ERROR"),
        }
    }
}

impl From<ValidationError> for AppError {
    fn from(err: ValidationError) -> Self {
        match err {
            ValidationError::Email(e) => match e {
                email::ValidationError::Empty => AppError::bad_request("EMAIL_EMPTY"),
                email::ValidationError::TooLong => AppError::bad_request("EMAIL_TOO_LONG"),
                email::ValidationError::MissingAt => AppError::bad_request("EMAIL_MISSING_AT"),
                email::ValidationError::InvalidStructure => {
                    AppError::bad_request("EMAIL_INVALID_STRUCTURE")
                }
                email::ValidationError::InvalidDomain => {
                    AppError::bad_request("EMAIL_INVALID_DOMAIN")
                }
            },

            ValidationError::Password(e) => match e {
                password::ValidationError::Empty => AppError::bad_request("PASSWORD_EMPTY"),
                password::ValidationError::TooShort => AppError::bad_request("PASSWORD_TOO_SHORT"),
                password::ValidationError::TooLong => AppError::bad_request("PASSWORD_TOO_LONG"),
                password::ValidationError::MissingUpper => {
                    AppError::bad_request("PASSWORD_MISSING_UPPER")
                }
                password::ValidationError::MissingLower => {
                    AppError::bad_request("PASSWORD_MISSING_LOWER")
                }
                password::ValidationError::MissingDigit => {
                    AppError::bad_request("PASSWORD_MISSING_DIGIT")
                }
                password::ValidationError::MissingSpecial => {
                    AppError::bad_request("PASSWORD_MISSING_SPECIAL")
                }
            },

            ValidationError::Username(e) => match e {
                username::ValidationError::Empty => AppError::bad_request("USERNAME_EMPTY"),
                username::ValidationError::TooShort => AppError::bad_request("USERNAME_TOO_SHORT"),
                username::ValidationError::TooLong => AppError::bad_request("USERNAME_TOO_LONG"),
                username::ValidationError::InvalidCharacters => {
                    AppError::bad_request("USERNAME_INVALID_CHARACTERS")
                }
                username::ValidationError::StartsWithInvalidChar => {
                    AppError::bad_request("USERNAME_INVALID_START")
                }
                username::ValidationError::EndsWithInvalidChar => {
                    AppError::bad_request("USERNAME_INVALID_END")
                }
                username::ValidationError::ConsecutiveSeparators => {
                    AppError::bad_request("USERNAME_CONSECUTIVE_SEPARATORS")
                }
            },
        }
    }
}

fn make_response(
    code: &str,
    status: Status,
    message: Option<&str>,
    _req: &Request<'_>,
) -> Custom<Json<ErrorResponse>> {
    let request_id = Ulid::new().into();
    let body = ErrorResponse {
        error: ErrorObject {
            code: code.into(),
            status: status.code,
            message: message.map(|s| s.into()),
        },
        request_id,
    };
    Custom(status, Json(body))
}

impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        make_response(&self.code, self.status, self.message.as_deref(), req).respond_to(req)
    }
}

impl OpenApiResponderInner for AppError {
    fn responses(_gen: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        let mut responses = Responses::default();
        let error_schema = RefOr::Object(Response {
            description: "Error".into(),
            content: std::iter::once(("application/json".into(), MediaType::default())).collect(),
            ..Default::default()
        });

        for status in &[
            Status::BadRequest,
            Status::Unauthorized,
            Status::Forbidden,
            Status::NotFound,
            Status::Conflict,
            Status::InternalServerError,
        ] {
            responses
                .responses
                .insert(status.code.to_string(), error_schema.clone());
        }

        Ok(responses)
    }
}

macro_rules! make_catcher {
    ($name:ident, $num:literal, $status:expr, $code:expr) => {
        #[catch($num)]
        pub fn $name(req: &Request<'_>) -> Custom<Json<ErrorResponse>> {
            make_response($code, $status, None, req)
        }
    };
}

make_catcher!(bad_request, 400, Status::BadRequest, "BAD_REQUEST");
make_catcher!(unauthorized, 401, Status::Unauthorized, "UNAUTHORIZED");
make_catcher!(forbidden, 403, Status::Forbidden, "FORBIDDEN");
fn is_wildcard_path(pattern: &str) -> bool {
    let seg = pattern.trim_start_matches('/');
    seg.starts_with('<') && seg.ends_with("..>")
}

fn path_matches(pattern: &str, path: &str) -> bool {
    if is_wildcard_path(pattern) {
        return true;
    }
    let pat_parts: Vec<&str> = pattern.trim_start_matches('/').split('/').collect();
    let path_parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    if pat_parts.len() != path_parts.len() {
        return false;
    }
    pat_parts
        .iter()
        .zip(path_parts.iter())
        .all(|(p, s)| (p.starts_with('<') && p.ends_with('>')) || *p == *s)
}

#[catch(404)]
pub fn not_found(req: &Request<'_>) -> Custom<Json<ErrorResponse>> {
    let req_path = req.uri().path();
    let is_method_mismatch = req.rocket().routes().any(|route| {
        route.method != req.method()
            && !is_wildcard_path(route.uri.path())
            && path_matches(route.uri.path(), req_path.as_str())
    });

    if is_method_mismatch {
        make_response("METHOD_NOT_ALLOWED", Status::MethodNotAllowed, None, req)
    } else {
        make_response("NOT_FOUND", Status::NotFound, None, req)
    }
}
make_catcher!(
    method_not_allowed,
    405,
    Status::MethodNotAllowed,
    "METHOD_NOT_ALLOWED"
);
make_catcher!(not_acceptable, 406, Status::NotAcceptable, "NOT_ACCEPTABLE");
make_catcher!(
    unprocessable_entity,
    422,
    Status::UnprocessableEntity,
    "UNPROCESSABLE_ENTITY"
);
make_catcher!(
    too_many_requests,
    429,
    Status::TooManyRequests,
    "TOO_MANY_REQUESTS"
);
make_catcher!(
    internal_error,
    500,
    Status::InternalServerError,
    "INTERNAL_ERROR"
);
make_catcher!(
    service_unavailable,
    503,
    Status::ServiceUnavailable,
    "SERVICE_UNAVAILABLE"
);

#[catch(default)]
pub fn default_catcher(status: Status, req: &Request<'_>) -> Custom<Json<ErrorResponse>> {
    make_response("UNKNOWN_ERROR", status, None, req)
}
