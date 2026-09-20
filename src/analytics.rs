use crate::util::{CaptureEventProperties, StatusExt};
use crate::web::AppState;
use anyhow::Context;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::http::header::USER_AGENT;
use axum::middleware::Next;
use axum::response::Response;
use posthog_rs::{Client, ClientOptionsBuilder, Event};
use std::env;
use std::sync::Arc;

pub(crate) async fn init(enable: bool) -> anyhow::Result<Client> {
    let mut builder = ClientOptionsBuilder::default();

    if enable && let Some(posthog_url) = env::var("POSTHOG_INSTANCE_URL").ok() {
        let posthog_project_api_key = env::var("POSTHOG_PROJECT_API_KEY").context(
            "PostHog analytics are enabled but no POSTHOG_PROJECT_API_KEY was provided!",
        )?;

        let posthog_personal_api_key = env::var("POSTHOG_PERSONAL_API_KEY").ok();

        builder
            .host(posthog_url)
            .api_key(posthog_project_api_key)
            .secret_key(posthog_personal_api_key.unwrap_or_default());

        log::info!("PostHog analytics enabled");
    }

    let client = posthog_rs::client(builder.build()?).await;
    Ok(client)
}

pub(crate) async fn capture_analytics(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    const IGNORED_PATHS: [&str; 1] = ["/health"];

    // headers
    let user_agent = req
        .headers()
        .get(USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .map(ToString::to_string);

    // URL
    let path = req.uri().clone();

    let full_url = state
        .http
        .frontend_url
        .join(
            path.path_and_query()
                .map(|it| it.as_str())
                .unwrap_or_default(),
        )
        .ok();

    let response = next.run(req).await;

    if !IGNORED_PATHS.contains(&path.path())
        && let Some(full_url) = full_url
    {
        let hostname = full_url.host_str().unwrap_or_default().to_string();

        let event = Event::new_anon("$pageview")
            .with("$current_url", full_url.to_string())
            .with("$host", hostname)
            .with("$pathname", path.path())
            .with("status", response.status().as_u16())
            .with("success", response.status().is_success_or_redirect())
            .with("user_agent", user_agent);

        state.posthog_client.capture(event);
    }

    Ok(response)
}
