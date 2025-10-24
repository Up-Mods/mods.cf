use crate::analytics::Analytics;
use crate::curseforge::CurseforgeState;
use crate::util::HealthResponse;
use crate::{analytics, curseforge};
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::routing::get;
use axum::{Router, middleware};
use std::sync::Arc;

mod files;
pub mod projects;

pub(crate) struct AppState {
    pub analytics: Analytics,
    pub curseforge: CurseforgeState,
}

pub async fn init_router() -> anyhow::Result<Router> {
    let app_data = Arc::new(AppState {
        analytics: analytics::init().await?,
        curseforge: curseforge::init()?,
    });
    let router = Router::new()
        .route(
            "/",
            get(async || Redirect::to("https://www.curseforge.com")),
        )
        .route(
            "/health",
            get(async || HealthResponse::from(StatusCode::OK)),
        )
        .route("/{project_id}", get(projects::project_by_id))
        .route("/f/{file_id}", get(files::file_by_id))
        .layer(middleware::from_fn_with_state(
            app_data.clone(),
            analytics::capture_analytics,
        ))
        .with_state(app_data);

    Ok(router)
}
