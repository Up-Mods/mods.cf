use axum::extract::Path;
use axum::response::{IntoResponse, Redirect};

pub(crate) async fn project_by_id(Path(project_id): Path<u64>) -> impl IntoResponse {
    let target = format!("https://curseforge.com/projects/{project_id}");
    Redirect::to(target.as_str())
}

#[cfg(test)]
mod test {
    use crate::async_tests_with_env;
    use crate::web::test::new_test_server;
    use axum::http::header::LOCATION;
    use reqwest::StatusCode;

    async_tests_with_env! {
        async fn should_redirect_to_project() -> anyhow::Result<()> {
            let server = new_test_server().await?;

            let response = server.get("/911456").await;
            response.assert_status(StatusCode::SEE_OTHER);
            response.assert_header(LOCATION, "https://curseforge.com/projects/911456");
            Ok(())
        }
    }
}
