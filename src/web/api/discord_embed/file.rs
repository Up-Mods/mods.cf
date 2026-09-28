use crate::analytics::CaptureEventProperties;
use crate::curseforge;
use crate::curseforge::mods::SocialLinkType;
use crate::discord::ComponentHolder;
use crate::web::AppState;
use crate::web::api::ApiError;
use crate::web::api::discord_embed::EmbedParams;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::{Extension, Json};
use axum_test::expect_json::__private::serde_trampoline::de::StdError;
use human_repr::HumanCount;
use posthog_rs::{CaptureExceptionOptions, Event};
use std::sync::Arc;
use twilight_model::channel::message::EmojiReactionType;
use twilight_model::channel::message::component::{ButtonStyle, UnfurledMediaItem};
use twilight_model::id::Id;
use twilight_util::builder::message::{
    ActionRowBuilder, ButtonBuilder, ContainerBuilder, SectionBuilder, TextDisplayBuilder,
    ThumbnailBuilder,
};

pub(crate) async fn file_embed_by_id(
    State(state): State<Arc<AppState>>,
    Extension(mut event): Extension<Event>,
    Path(file_id): Path<u64>,
    Query(params): Query<EmbedParams>,
) -> impl IntoResponse {
    match curseforge::mods::get_file_info(&state.curseforge.eternal_api_client, file_id).await {
        Ok(result) => {
            let Some((project, file)) = result else {
                return ApiError::not_found(None).into_response();
            };

            let hl = params.content_language();
            event.with("content_language", hl);

            if let Some(embed_time) = params.embed_timestamp() {
                event.with("embed_timestamp", embed_time);
            }

            let changelog = match file
                .get_truncated_changelog(&state.curseforge.eternal_api_client, 2048)
                .await
            {
                Err(err) => {
                    log::error!("Unable to get file changelog for {file_id}! {err:#}");

                    let real_error = err.into_boxed_dyn_error();
                    state
                        .posthog_client
                        .capture_exception_with::<dyn StdError + Send + Sync>(
                            real_error.as_ref(),
                            CaptureExceptionOptions::new().distinct_id(event.distinct_id()),
                        )
                        .await
                        .ok();

                    Some(t!("embed.discord.file_changelog.error").to_string())
                }
                Ok(changelog) => changelog,
            };

            let title_text = t!(
                "embed.discord.file",
                locale = hl,
                project_title = project.name,
                project_summary = project.summary,
                project_downloads = project.download_count.human_count_bare(),
                file_name = file.display_name.clone().unwrap_or(file.file_name.clone()),
                file_release_type = file.release_type.translated(hl),
                file_release_type_emoji = file.release_type.as_emoji(),
                file_downloads = file.download_count.human_count_bare(),
                file_changelog = changelog
                    .clone()
                    .unwrap_or_else(|| t!("embed.discord.file_changelog.empty").to_string())
            );

            let thumbnail = ThumbnailBuilder::new(UnfurledMediaItem {
                url: project.logo.thumbnail_url.unwrap_or(project.logo.url),
                proxy_url: None,
                height: None,
                width: None,
                content_type: None,
            })
            .description(t!(
                "embed.discord.project_icon_alt",
                locale = hl,
                project_name = project.name
            ))
            .build();
            let desc = TextDisplayBuilder::new(title_text).build();
            let main_section = SectionBuilder::new(thumbnail).component(desc).build();

            let file_url = state
                .http
                .frontend_url
                .join(&format!("/f/{file_id}"))
                .map(|url| url.to_string())
                .ok()
                .unwrap_or_else(|| {
                    format!(
                        "{project_url}/files/{file_id}",
                        project_url = project.links.website_url
                    )
                });

            let cf_button = ButtonBuilder::new(ButtonStyle::Link)
                .label(t!("embed.discord.label.curseforge", locale = hl))
                .emoji(EmojiReactionType::Custom {
                    id: Id::new(1552609523561271396),
                    name: Some("curseforge".to_string()),
                    animated: false,
                })
                .url(file_url)
                .build();

            let mut buttons = ActionRowBuilder::new().component(cf_button);

            if let Some(wiki_url) = project.links.wiki_url {
                let wiki_button = ButtonBuilder::new(ButtonStyle::Link)
                    .label(t!("embed.discord.label.wiki", locale = hl))
                    .url(wiki_url)
                    .build();

                buttons = buttons.component(wiki_button);
            }

            if let Some(discord_url) = project
                .social_links
                .and_then(|map| map.0.get(&SocialLinkType::Discord).cloned())
            {
                let discord_button = ButtonBuilder::new(ButtonStyle::Link)
                    .label(t!("embed.discord.label.discord", locale = hl))
                    .emoji(EmojiReactionType::Custom {
                        id: Id::new(1552678152931770458),
                        name: Some("discord".to_string()),
                        animated: false,
                    })
                    .url(discord_url)
                    .build();

                buttons = buttons.component(discord_button);
            }

            if let Some(issues_url) = project.links.issues_url {
                let issues_button = ButtonBuilder::new(ButtonStyle::Link)
                    .label(t!("embed.discord.label.issues", locale = hl))
                    .url(issues_url)
                    .build();

                buttons = buttons.component(issues_button);
            }

            let root = ContainerBuilder::new()
                .accent_color(Some(0xFF784D))
                .component(main_section)
                .component(buttons.build())
                .build();

            Json(ComponentHolder::new(root)).into_response()
        }
        Err(err) => {
            log::error!("Error during project lookup for {file_id}: {err:#}");
            ApiError::server_error(Some(format!("Error during project lookup for {file_id}")))
                .into_response()
        }
    }
}
