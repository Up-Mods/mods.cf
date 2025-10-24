use crate::curseforge;
use crate::web::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use std::sync::Arc;

pub(crate) async fn file_by_id(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<u64>,
) -> impl IntoResponse {
    match curseforge::mods::get_file_info(&state.curseforge.eternal_api_client, file_id).await {
        Ok(result) => {
            let Some((project, _)) = result else {
                return StatusCode::NOT_FOUND.into_response();
            };

            let url = format!(
                "{project_url}/files/{file_id}",
                project_url = project.links.website_url
            );
            Redirect::to(url.as_str()).into_response()
        }
        Err(err) => {
            log::error!("Error during file lookup for file {file_id}: {err:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
