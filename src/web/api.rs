use crate::util::web::serialize_status_code;
use crate::web::AppState;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use serde::Serialize;
use std::sync::Arc;

pub(crate) mod discord_embed;

#[derive(Clone, Serialize)]
struct ApiError {
    #[serde(serialize_with = "serialize_status_code")]
    status: StatusCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

impl ApiError {
    fn not_found(message: Option<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message,
        }
    }

    fn server_error(message: Option<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message,
        }
    }

    fn new(status: StatusCode, message: Option<String>) -> Self {
        Self { status, message }
    }
}

impl Default for ApiError {
    fn default() -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, None)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self)).into_response()
    }
}

pub(crate) fn create_router<S>(state: Arc<AppState>) -> Router<S> {
    Router::new()
        .nest("/discord-embed", discord_embed::create_router(state))
        .with_state(())
}
