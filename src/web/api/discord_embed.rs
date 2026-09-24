use crate::web::AppState;
use axum::Router;
use axum::routing::get;
use std::sync::Arc;
use serde::Deserialize;

pub(crate) mod project;

pub(crate) fn create_router<S>(state: Arc<AppState>) -> Router<S> {
    Router::new()
        .route("/project/{project_id}", get(project::project_embed_by_id)) // TODO make this .json in axum 0.9
        .with_state(state)
}

#[derive(Deserialize)]
pub(crate) struct EmbedParams {
    #[serde(default, rename = "hl")]
    content_language: Option<String>,
}
