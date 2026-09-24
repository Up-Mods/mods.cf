use crate::curseforge;
use crate::discord::ComponentHolder;
use crate::web::AppState;
use crate::web::api::ApiError;
use crate::web::api::discord_embed::EmbedParams;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use human_repr::HumanCount;
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
    Path(project_id): Path<u64>,
    Query(params): Query<EmbedParams>,
) -> impl IntoResponse {
    match curseforge::mods::get_mod(&state.curseforge.eternal_api_client, project_id).await {
        Ok(result) => {
            let Some(project) = result else {
                return ApiError::not_found(None).into_response();
            };

            let title_text = t!(
                "embed.discord.project",
                locale = &params.content_language.unwrap_or_default(),
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
            .description(format!(
                "Icon for {project_name}",
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
                .label("CurseForge")
                .emoji(EmojiReactionType::Custom {
                    name: Some("curseforge".to_string()),
                    animated: false,
                    id: Id::new(1552609523561271396),
                })
                .url(project_url)
                .build();

            let mut buttons = ActionRowBuilder::new().component(cf_button);

            if let Some(wiki_url) = project.links.wiki_url {
                let wiki_button = ButtonBuilder::new(ButtonStyle::Link)
                    .label("Wiki")
                    .url(wiki_url)
                    .build();

                buttons = buttons.component(wiki_button);
            }

            if let Some(issues_url) = project.links.issues_url {
                let issues_button = ButtonBuilder::new(ButtonStyle::Link)
                    .label("Issues")
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
