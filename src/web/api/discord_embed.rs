use crate::web::AppState;
use axum::Router;
use axum::routing::get;
use std::sync::Arc;

pub(crate) mod project;

pub(crate) fn create_router<S>(state: Arc<AppState>) -> Router<S> {
    Router::new()
        .route("/project/{project_id}", get(project::project_embed_by_id)) // TODO make this .json in axum 0.9
        .with_state(state)
}
