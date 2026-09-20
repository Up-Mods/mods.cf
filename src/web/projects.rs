use crate::curseforge;
use crate::web::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use std::sync::Arc;

pub(crate) async fn project_by_id(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<u64>,
) -> impl IntoResponse {
    match curseforge::mods::get_mod(&state.curseforge.eternal_api_client, project_id).await {
        Ok(result) => {
            let Some(project) = result else {
                return StatusCode::NOT_FOUND.into_response();
            };

            Redirect::to(&project.links.website_url).into_response()
        }
        Err(err) => {
            log::error!("Error during project lookup for {project_id}: {err:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[cfg(test)]
mod test {
    use crate::async_tests_with_env;
    use crate::web::test::new_test_server;
    use axum::http::header::LOCATION;
    use reqwest::StatusCode;

    async_tests_with_env! {
        async fn should_redirect_to_project() -> anyhow::Result<()> {
            let (server, shutdown) = new_test_server().await?;

            let response = server.get("/911456").await;
            response.assert_status(StatusCode::SEE_OTHER);
            response.assert_header(LOCATION, "https://curseforge.com/projects/911456");

            shutdown().await
        }
    }
}
