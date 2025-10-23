use axum::extract::Path;
use axum::response::{IntoResponse, Redirect};

pub(crate) async fn project_by_id(Path(project_id): Path<u64>) -> impl IntoResponse {
    let target = format!("https://curseforge.com/projects/{project_id}");
    Redirect::temporary(target.as_str())
}
