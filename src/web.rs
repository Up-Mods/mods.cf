use crate::curseforge;
use crate::curseforge::CurseforgeState;
use crate::util::HealthResponse;
use axum::Router;
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::routing::get;
use std::sync::Arc;

mod files;
pub mod projects;

pub(crate) struct AppState {
    curseforge: CurseforgeState,
}

pub fn init_router(eternal_api_token: &str) -> anyhow::Result<Router> {
    let app_data = Arc::new(AppState {
        curseforge: curseforge::create_state(eternal_api_token)?,
    });
    let router = Router::new()
        .route("/", get(async || Redirect::to("https://www.curseforge.com")))
        .route("/health", get(async || HealthResponse::from(StatusCode::OK)))
        .route("/{project_id}", get(projects::project_by_id))
        .route("/f/{file_id}", get(files::file_by_id))
        .with_state(app_data);

    Ok(router)
}
