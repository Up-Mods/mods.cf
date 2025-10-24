use crate::util::CaptureEventProperties;
use crate::web::AppState;
use anyhow::{Context, anyhow};
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::http::header::USER_AGENT;
use axum::middleware::Next;
use axum::response::Response;
use posthog_rs::{Client, ClientOptionsBuilder, Event};
use std::env;
use std::sync::Arc;

#[derive(Default)]
pub(crate) struct Analytics {
    client: Option<Client>,
}

impl Analytics {
    pub async fn capture(&self, event: Event) -> anyhow::Result<()> {
        if let Some(client) = &self.client {
            client.capture(event).await.map_err(|err| anyhow!(err))?;
        }

        Ok(())
    }
}

pub(crate) async fn init(enable: bool) -> anyhow::Result<Analytics> {
    if !enable {
        return Ok(Analytics::default());
    }

    let Some(posthog_url) = env::var("POSTHOG_INSTANCE_URL").ok() else {
        return Ok(Analytics::default());
    };

    let posthog_project_api_key = env::var("POSTHOG_PROJECT_API_KEY")
        .context("PostHog analytics are enabled but no POSTHOG_PROJECT_API_KEY was provided!")?;

    // TODO posthog sdk does not support using private api key or error tracking yet :/
    // let posthog_personal_api_key = env::var("POSTHOG_PERSONAL_API_KEY").ok();

    let options = ClientOptionsBuilder::default()
        .api_endpoint(posthog_url)
        .api_key(posthog_project_api_key)
        .build()?;
    let client = posthog_rs::client(options).await;

    log::info!("PostHog analytics enabled");

    Ok(Analytics {
        client: Some(client),
    })
}

pub(crate) async fn capture_analytics(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    const IGNORED_PATHS: [&str; 1] = ["/health"];

    let url = req.uri().clone();
    let user_agent = req
        .headers()
        .get(USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .map(ToString::to_string);
    let response = next.run(req).await;

    if !IGNORED_PATHS.contains(&url.path()) {
        let event = Event::new_anon("$pageview")
            .with("$current_url", url.to_string())
            .with("$host", url.host())
            .with("$pathname", url.path())
            .with("status", response.status().as_u16())
            .with("success", response.status().is_success())
            .with("user_agent", user_agent);

        if let Err(err) = state.analytics.capture(event).await {
            log::error!("Unable to capture event: {err:?}")
        }
    }

    Ok(response)
}
