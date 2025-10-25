use crate::analytics::Analytics;
use crate::curseforge::CurseforgeState;
use crate::util::HealthResponse;
use crate::{analytics, curseforge};
use anyhow::Context;
use axum::extract::Request;
use axum::http::header::HOST;
use axum::http::{HeaderName, StatusCode};
use axum::response::Redirect;
use axum::routing::get;
use axum::{Router, middleware};
use regex_macro::regex;
use std::env;
use std::str::FromStr;
use std::sync::Arc;

mod files;
pub mod projects;

pub(crate) struct AppState {
    pub http: HttpConfig,
    pub analytics: Analytics,
    pub curseforge: CurseforgeState,
}

pub(crate) struct HttpConfig {
    pub host_header_name: HeaderName,
    pub frontend_url: Option<String>,
}

pub async fn init_router(enable_analytics: bool) -> anyhow::Result<Router> {
    let app_data = Arc::new(AppState {
        http: init_http()?,
        analytics: analytics::init(enable_analytics).await?,
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

fn init_http() -> anyhow::Result<HttpConfig> {
    let host_header_name = env::var("HOST_HEADER_NAME")
        .ok()
        .map(|value| HeaderName::from_str(&value).context("Unable to parse HOST_HEADER_NAME"))
        .transpose()?
        .unwrap_or(HOST);

    let frontend_url = env::var("FRONTEND_URL").ok();

    Ok(HttpConfig {
        host_header_name,
        frontend_url,
    })
}

pub(crate) fn get_host(req: &Request, cfg: &HttpConfig) -> Option<String> {
    let mut host = req
        .headers()
        .get(&cfg.host_header_name)
        .and_then(|h| h.to_str().ok())
        .map(ToString::to_string);

    // if no host header or is localhost: try to fall back to frontend url,
    // otherwise just proceed with whatever we have.
    if host
        .as_ref()
        .is_none_or(|value| regex!(r"^https?://(?:localhost|127\.0\.0\.1)").is_match(value))
    {
        host = cfg.frontend_url.clone().or(host);
    }

    host
}

#[cfg(test)]
pub mod test {
    use crate::web::init_router;
    use anyhow::{Context, anyhow};
    use axum_test::TestServer;

    pub(crate) async fn new_test_server() -> anyhow::Result<TestServer> {
        let app = init_router(false)
            .await
            .context("Unable to create test server")?;
        TestServer::builder()
            .mock_transport()
            .build(app)
            .map_err(|err| anyhow!(err))
    }
}
