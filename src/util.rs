use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

pub(crate) fn default_true() -> bool {
    true
}

#[derive(Serialize)]
pub(crate) struct HealthResponse {
    pub status: u16,
    pub message: Option<String>,
}

impl From<StatusCode> for HealthResponse {
    fn from(value: StatusCode) -> Self {
        HealthResponse {
            status: value.as_u16(),
            message: value.canonical_reason().map(|it| it.to_string()),
        }
    }
}

impl IntoResponse for HealthResponse {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}
