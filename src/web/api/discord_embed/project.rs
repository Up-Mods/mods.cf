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
use human_repr::HumanCount;
use posthog_rs::Event;
use std::sync::Arc;
use twilight_model::channel::message::EmojiReactionType;
use twilight_model::channel::message::component::{ButtonStyle, UnfurledMediaItem};
use twilight_model::id::Id;
use twilight_util::builder::message::{
    ActionRowBuilder, ButtonBuilder, ContainerBuilder, SectionBuilder, TextDisplayBuilder,
    ThumbnailBuilder,
};

pub(crate) async fn project_embed_by_id(
    State(state): State<Arc<AppState>>,
    Extension(mut event): Extension<Event>,
    Path(project_id): Path<u64>,
    Query(params): Query<EmbedParams>,
) -> impl IntoResponse {
    match curseforge::mods::get_mod(&state.curseforge.eternal_api_client, project_id).await {
        Ok(result) => {
            let Some(project) = result else {
                return ApiError::not_found(None).into_response();
            };

            let hl = params.content_language();
            event.with("content_language", hl);

            if let Some(embed_time) = params.embed_timestamp() {
                event.with("embed_timestamp", embed_time);
            }

            let title_text = t!(
                "embed.discord.project",
                locale = hl,
                title = project.name,
                summary = project.summary,
                downloads = project.download_count.human_count_bare()
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

            let project_url = state
                .http
                .frontend_url
                .join(&format!("/{project_id}"))
                .map(|url| url.to_string())
                .unwrap_or(project.links.website_url);

            let cf_button = ButtonBuilder::new(ButtonStyle::Link)
                .label(t!("embed.discord.label.curseforge", locale = hl))
                .emoji(EmojiReactionType::Custom {
                    id: Id::new(1552609523561271396),
                    name: Some("curseforge".to_string()),
                    animated: false,
                })
                .url(project_url)
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
            log::error!("Error during project lookup for {project_id}: {err:#}");
            ApiError::server_error(Some(format!(
                "Error during project lookup for {project_id}"
            )))
            .into_response()
        }
    }
}
