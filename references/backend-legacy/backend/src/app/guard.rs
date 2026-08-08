use axum::Json;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Serialize)]
struct SecretErrorResponse {
    error: &'static str,
    message: &'static str,
}

pub async fn require_secret(
    State(expected_secret): State<String>,
    req: Request,
    next: Next,
) -> Result<Response, Response> {
    if expected_secret.is_empty() {
        return Err(secret_error_response());
    }

    let ok = has_valid_secret(req.headers(), expected_secret.as_str());

    if ok {
        Ok(next.run(req).await)
    } else {
        Err(secret_error_response())
    }
}

fn has_valid_secret(headers: &axum::http::HeaderMap, expected_secret: &str) -> bool {
    has_valid_secret_header(headers, expected_secret)
}

fn has_valid_secret_header(headers: &axum::http::HeaderMap, expected_secret: &str) -> bool {
    headers
        .get("x-makosh-secret")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|secret| secret == expected_secret)
}

fn secret_error_response() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(SecretErrorResponse {
            error: "invalid_api_secret",
            message: "missing or invalid x-makosh-secret header",
        }),
    )
        .into_response()
}
