use crate::web::{AppState, UserAgent};
use crate::{curseforge, feature_flags};
use axum::Extension;
use axum::extract::{Path, Request, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect};
use chrono::Utc;
use posthog_rs::{CaptureExceptionOptions, EvaluateFlagsOptions, Event, FlagValue};
use rootcause::compat::boxed_error::IntoBoxedError;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error};

const PROJECTS_PREVIEW_HTML: &str = include_str!("previews/project.html");

pub(crate) async fn project_by_id(
    State(state): State<Arc<AppState>>,
    Extension(user_agent): Extension<Option<UserAgent>>,
    Extension(event): Extension<Arc<Mutex<Event>>>,
    Path(project_id): Path<u64>,
    req: Request,
) -> impl IntoResponse {
    let mut event = event.lock().await;

    match curseforge::mods::get_mod(&state.curseforge.eternal_api_client, project_id).await {
        Ok(result) => {
            let Some(project) = result else {
                return StatusCode::NOT_FOUND.into_response();
            };

            if user_agent.is_some_and(|ua| ua.is_discord_preview_fetch()) {
                let mut props = HashMap::new();
                props.insert(
                    "cf_project_id".to_string(),
                    Value::String(project_id.to_string()),
                );

                if let Some(current_url) = event.properties().get("$current_url") {
                    props.insert("$current_url".to_string(), current_url.clone());
                }

                match state
                    .posthog_client
                    .evaluate_flags(
                        event.distinct_id(),
                        EvaluateFlagsOptions {
                            flag_keys: Some(vec![feature_flags::DISCORD_EMBEDS.to_string()]),
                            person_properties: Some(props),
                            ..Default::default()
                        },
                    )
                    .await
                {
                    Err(err) => {
                        state
                            .posthog_client
                            .capture_exception_with(
                                &err,
                                CaptureExceptionOptions::new().distinct_id(event.distinct_id()),
                            )
                            .await
                            .ok();
                        error!("Unable to query feature flags! {err:#}");
                    }
                    Ok(snapshot) => {
                        event.with_flags(&snapshot);

                        if let Some(FlagValue::Boolean(embeds_flag)) =
                            snapshot.get_flag(feature_flags::DISCORD_EMBEDS)
                            && embeds_flag
                        {
                            debug!(
                                "Sending discord preview for Project {project_id} ({project_name})",
                                project_name = project.name
                            );

                            let mut url_partial = format!(
                                "api/discord-embed/project/{project_id}?et={time}",
                                time = Utc::now().timestamp()
                            );

                            if let Some(query) = req.uri().query() {
                                url_partial.push('&');
                                url_partial.push_str(query);
                            }

                            let component_json_url =
                                match state.http.frontend_url.join(&url_partial) {
                                    Ok(value) => value,

                                    Err(err) => {
                                        state
                                            .posthog_client
                                            .capture_exception_with(
                                                &err,
                                                CaptureExceptionOptions::new()
                                                    .distinct_id(event.distinct_id()),
                                            )
                                            .await
                                            .ok();
                                        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                                    }
                                };

                            return Html(
                                PROJECTS_PREVIEW_HTML
                                    .replace("{COMPONENT_JSON_URL}", component_json_url.as_str())
                                    .replace("{PROJECT_TITLE}", &project.name)
                                    .replace("{PROJECT_URL}", &project.links.website_url)
                                    .replace(
                                        "{PROJECT_ICON_URL}",
                                        &project.logo.thumbnail_url.unwrap_or(project.logo.url),
                                    )
                                    .replace("{PROJECT_SUMMARY}", &project.summary),
                            )
                            .into_response();
                        }
                    }
                }
            }

            Redirect::to(&project.links.website_url).into_response()
        }
        Err(err) => {
            error!("Error during project lookup for {project_id}: {err:#}");
            state
                .posthog_client
                .capture_exception_with(
                    err.into_boxed_error().as_ref(),
                    CaptureExceptionOptions::new().distinct_id(event.distinct_id()),
                )
                .await
                .ok();
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
        async fn should_redirect_to_project() -> rootcause::Result<()> {
            let (server, shutdown) = new_test_server().await?;

            let response = server.get("/911456").await;
            response.assert_status(StatusCode::SEE_OTHER);
            response.assert_header(LOCATION, "https://curseforge.com/projects/911456");

            shutdown().await
        }
    }
}
