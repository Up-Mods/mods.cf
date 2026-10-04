use crate::web::UserAgent;
use axum::http::StatusCode;
use bytes::Bytes;
use rootcause::prelude::*;
use serde::Serializer;
use serde::de::DeserializeOwned;

#[extension(pub(crate) trait BetterJsonError)]
impl reqwest::Response {
    async fn json_with_error<T>(self) -> rootcause::Result<T>
    where
        T: DeserializeOwned,
    {
        let url = self.url().clone();
        let content: Bytes = self
            .bytes()
            .await
            .context_with(|| format!("Unable to read response body for {url}"))?;
        let reader = &mut serde_json::Deserializer::from_slice(&content);
        let json = serde_path_to_error::deserialize(reader)
            .context_with(|| format!("Unable to decode response for {url}"))?;

        Ok(json)
    }
}

impl UserAgent {
    pub(crate) fn is_discord_preview_fetch(&self) -> bool {
        let lower = self.value.to_ascii_lowercase();
        lower.contains("discordbot")
            || lower.contains("twitterbot")
            || lower.contains("opengraphbot")
    }
}

#[extension(pub(crate) trait StatusExt)]
impl StatusCode {
    fn is_success_or_redirect(&self) -> bool {
        self.is_success() || self.is_redirection()
    }
}

pub(crate) fn serialize_status_code<S: Serializer>(
    value: &StatusCode,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_u16(value.as_u16())
}
