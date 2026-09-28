use crate::web::AppState;
use axum::Router;
use axum::routing::get;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_with::serde_as;
use std::sync::Arc;

pub(crate) mod project;

pub(crate) fn create_router<S>(state: Arc<AppState>) -> Router<S> {
    Router::new()
        .route("/project/{project_id}", get(project::project_embed_by_id)) // TODO make this .json in axum 0.9
        .with_state(state)
}

#[serde_as]
#[derive(Deserialize)]
pub(crate) struct EmbedParams {
    #[serde(default, rename = "hl")]
    content_language: Option<String>,

    #[serde(default, rename = "et")]
    embed_time: Option<i64>,
}

impl EmbedParams {
    fn content_language(&self) -> &str {
        self.content_language.as_deref().unwrap_or("en")
    }
    fn embed_timestamp(&self) -> Option<DateTime<Utc>> {
        self.embed_time
            .and_then(DateTime::from_timestamp_secs)
    }
}
