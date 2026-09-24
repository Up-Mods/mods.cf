use crate::util::web::StatusExt;
use crate::web::{AppState, UserAgent};
use anyhow::Context;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::http::header::USER_AGENT;
use axum::middleware::Next;
use axum::response::Response;
use posthog_rs::{Client, ClientOptionsBuilder, Event};
use serde::Serialize;
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
    mut req: Request,
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

    req.extensions_mut()
        .insert(user_agent.clone().map(UserAgent::new));

    let method = req.method().clone();

    let response: Response;
    if !IGNORED_PATHS.contains(&path.path())
        && let Some(full_url) = full_url
    {
        let hostname = full_url.host_str().unwrap_or_default().to_string();
        let event = Event::new_anon("$pageview")
            .with("$current_url", full_url.as_str())
            .with("$host", &hostname)
            .with("$pathname", path.path())
            .with("user_agent", user_agent.as_deref().unwrap_or_default());

        req.extensions_mut().insert(event);

        response = next.run(req).await;

        if let Some(event) = response.extensions().get::<Event>().cloned() {
            state.posthog_client.capture(
                event
                    .with("status", response.status().as_u16())
                    .with("success", response.status().is_success_or_redirect()),
            );
        }
    } else {
        response = next.run(req).await;
    }

    log::debug!(
        "{method} ({status:03}) - {path}{user_agent}",
        status = response.status().as_str(),
        user_agent = user_agent.map(|s| format!(" ({s})")).unwrap_or_default()
    );

    Ok(response)
}

#[extension(pub(crate) trait CaptureEventProperties)]
impl Event {
    fn with<K: Into<String>, V: Serialize>(mut self, key: K, value: V) -> Self {
        if let Err(err) = self.insert_prop(key, value) {
            log::error!("Unable to set event error context: {err:#}");
        }

        self
    }
}
