use crate::curseforge;
use crate::curseforge::CurseforgeState;
use axum::Router;
use axum::routing::get;
use std::sync::Arc;
use axum::response::Redirect;

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
        .route("/{project_id}", get(projects::project_by_id))
        .route("/f/{file_id}", get(files::file_by_id))
        .with_state(app_data);

    Ok(router)
}
