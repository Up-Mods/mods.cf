use anyhow::Context;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use bytes::Bytes;
use posthog_rs::Event;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub(crate) fn default_true() -> bool {
    true
}

#[derive(Serialize)]
pub(crate) struct HealthResponse {
    pub status: u16,
    pub message: Option<String>,
}

impl From<StatusCode> for HealthResponse {
    fn from(value: StatusCode) -> Self {
        HealthResponse {
            status: value.as_u16(),
            message: value.canonical_reason().map(|it| it.to_string()),
        }
    }
}

impl IntoResponse for HealthResponse {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

#[extension(pub(crate) trait BetterJsonError)]
impl reqwest::Response {
    async fn json_with_error<T>(self) -> anyhow::Result<T>
    where
        T: DeserializeOwned,
    {
        let url = self.url().clone();
        let content: Bytes = self
            .bytes()
            .await
            .with_context(|| format!("Unable to read response body for {url}"))?;
        let reader = &mut serde_json::Deserializer::from_slice(&content);
        let json = serde_path_to_error::deserialize(reader)
            .with_context(|| format!("Unable to decode response for {url}"))?;

        Ok(json)
    }
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
