use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;
use tracing::error;

/// Unified error type for the HTTP API.
///
/// Variants correspond broadly to HTTP status families:
/// - BadRequest      → 4xx (client input error)
/// - SimulationFailed → 422 (unprocessible input / domain failure)
/// - Internal        → 500 (unexpected server-side issue)
#[derive(Debug, Error)]
pub enum ApiError {
    /// The client sent invalid data (bad JSON values, invalid parameters, etc.).
    #[error("bad request: {0}")]
    BadRequest(String),

    /// The request was syntactically valid but the simulation could not be run
    /// (e.g., degenerate geometry or other domain-level failure).
    #[allow(dead_code)]
    #[error("simulation failed: {0}")]
    SimulationFailed(String),

    /// Catch-all for unexpected internal server errors.
    #[allow(dead_code)]
    #[error("internal server error")]
    Internal(String),
}

/// Convenience alias for handler results.
pub type ApiResult<T> = Result<T, ApiError>;

/// JSON shape for error responses.
#[derive(Debug, Serialize)]
struct ErrorBody {
    /// Short machine-readable error code.
    error: &'static str,
    /// Human-readable explanatory message.
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg),
            ApiError::SimulationFailed(msg) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "simulation_failed", msg)
            }
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", msg),
        };

        // Log the error with structured fields.
        error!(
            status = status.as_u16(),
            error_code,
            message = %message,
            "API error"
        );

        let body = ErrorBody {
            error: error_code,
            message,
        };

        (status, Json(body)).into_response()
    }
}
