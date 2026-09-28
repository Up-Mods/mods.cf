use crate::web::{AppState, UserAgent};
use crate::{curseforge, feature_flags};
use axum::Extension;
use axum::extract::{Path, Request, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect};
use chrono::Utc;
use posthog_rs::{CaptureExceptionOptions, EvaluateFlagsOptions, Event, FlagValue};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

const FILES_PREVIEW_HTML: &str = include_str!("previews/file.html");

pub(crate) async fn file_by_id(
    State(state): State<Arc<AppState>>,
    Extension(user_agent): Extension<Option<UserAgent>>,
    Extension(mut event): Extension<Event>,
    Path(file_id): Path<u64>,
    req: Request,
) -> impl IntoResponse {
    match curseforge::mods::get_file_info(&state.curseforge.eternal_api_client, file_id).await {
        Ok(result) => {
            let Some((project, file)) = result else {
                return StatusCode::NOT_FOUND.into_response();
            };

            let redirect_url = format!(
                "{project_url}/files/{file_id}",
                project_url = project.links.website_url
            );

            if user_agent.is_some_and(|ua| ua.is_discord_preview_fetch()) {
                let mut props = HashMap::new();
                props.insert(
                    "cf_project_id".to_string(),
                    Value::String(project.id.to_string()),
                );
                props.insert("cf_file_id".to_string(), Value::String(file_id.to_string()));

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
                        log::error!("Unable to query feature flags! {err:#}");
                    }
                    Ok(snapshot) => {
                        event.with_flags(&snapshot);

                        if let Some(FlagValue::Boolean(embeds_flag)) =
                            snapshot.get_flag(feature_flags::DISCORD_EMBEDS)
                            && embeds_flag
                        {
                            log::debug!(
                                "Sending discord preview for Project {project_id} ({project_name}), file {file_id}",
                                project_id = project.id,
                                project_name = project.name
                            );

                            let mut url_partial = format!(
                                "api/discord-embed/file/{file_id}?et={time}",
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

                            let file_website_url = state
                                .http
                                .frontend_url
                                .join(&format!("/f/{file_id}"))
                                .ok()
                                .map(|url| url.to_string())
                                .unwrap_or(redirect_url);

                            return Html(
                                FILES_PREVIEW_HTML
                                    .replace("{COMPONENT_JSON_URL}", component_json_url.as_str())
                                    .replace(
                                        "{FILE_NAME}",
                                        &file.display_name.unwrap_or(file.file_name),
                                    )
                                    .replace("{PROJECT_NAME}", &project.name)
                                    .replace("{FILE_URL}", &file_website_url)
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

            Redirect::to(&redirect_url).into_response()
        }
        Err(err) => {
            log::error!("Error during file lookup for file {file_id}: {err:#}");
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
        async fn should_redirect_to_project_files() -> anyhow::Result<()> {
            let (server, shutdown) = new_test_server().await?;

            let response = server.get("/f/6774233").await;
            response.assert_status(StatusCode::SEE_OTHER);
            response.assert_header(
                LOCATION,
                "https://www.curseforge.com/minecraft/mc-mods/sparkweave/files/6774233",
            );

            shutdown().await
        }
    }
}
