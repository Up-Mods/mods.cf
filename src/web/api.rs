use crate::web::AppState;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use serde::Serialize;
use std::sync::Arc;
use serde_with::skip_serializing_none;

pub(crate) mod discord_embed;

#[skip_serializing_none]
#[derive(Clone, Serialize)]
struct ApiError {
    #[serde(with = "http_serde::status_code")]
    status: StatusCode,
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
