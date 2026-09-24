use crate::curseforge::CurseforgeState;
use crate::{analytics, curseforge};
use anyhow::Context;
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::routing::get;
use axum::{Router, middleware};
use health::HealthResponse;
use std::env;
use std::sync::Arc;
use url::Url;

pub mod api;
mod files;
mod health;
pub mod projects;

pub(crate) struct AppState {
    pub http: HttpConfig,
    pub posthog_client: posthog_rs::Client,
    pub curseforge: CurseforgeState,
}

pub(crate) struct HttpConfig {
    pub frontend_url: Url,
}

#[derive(Clone)]
pub(crate) struct UserAgent {
    pub value: String,
}

impl UserAgent {
    pub(crate) fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

pub async fn init_router(
    enable_analytics: bool,
) -> anyhow::Result<(Router, impl AsyncFnOnce() -> anyhow::Result<()>)> {
    let posthog_client = analytics::init(enable_analytics).await?;
    let app_data = Arc::new(AppState {
        http: init_http()?,
        posthog_client,
        curseforge: curseforge::init()?,
    });
    let copied_state = app_data.clone();
    let shutdown = async move || {
        copied_state.posthog_client.shutdown().await;

        anyhow::Ok(())
    };

    let router: Router<()> = Router::new()
        .route(
            "/",
            get(async || Redirect::to("https://www.curseforge.com")),
        )
        .route(
            "/health",
            get(async || HealthResponse::from(StatusCode::OK)),
        )
        .nest("/api", api::create_router(app_data.clone()))
        .route("/{project_id}", get(projects::project_by_id))
        .route("/f/{file_id}", get(files::file_by_id))
        .layer(middleware::from_fn_with_state(
            app_data.clone(),
            analytics::capture_analytics,
        ))
        .with_state(app_data);

    Ok((router, shutdown))
}

fn init_http() -> anyhow::Result<HttpConfig> {
    let frontend_url = match env::var("FRONTEND_URL").ok() {
        Some(url) => Url::parse(&url).context("FRONTEND_URL not set to a valid URL")?,
        None => Url::parse("http://localhost").expect("unable to parse localhost URL"),
    };

    Ok(HttpConfig { frontend_url })
}

#[cfg(test)]
pub mod test {
    use crate::web::init_router;
    use anyhow::Context;
    use axum_test::TestServer;

    pub(crate) async fn new_test_server()
    -> anyhow::Result<(TestServer, impl AsyncFnOnce() -> anyhow::Result<()>)> {
        let (app, shutdown) = init_router(false)
            .await
            .context("Unable to create test server")?;

        let server = TestServer::builder().mock_transport().build(app);
        Ok((server, shutdown))
    }
}
