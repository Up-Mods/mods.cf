use reqwest::Client;
use reqwest::header::{ACCEPT, HeaderMap, HeaderName, HeaderValue};

pub(crate) mod mods;

const API_BASE_URL: &str = "https://api.curseforge.com";

pub(crate) struct CurseforgeState {
    pub eternal_api_client: Client,
}

pub(crate) fn create_state(eternal_api_token: &str) -> anyhow::Result<CurseforgeState> {
    let mut default_headers = HeaderMap::with_capacity(4);
    default_headers.append(
        HeaderName::from_static("x-api-key"),
        HeaderValue::from_str(eternal_api_token)?,
    );
    default_headers.append(ACCEPT, HeaderValue::from_static("application/json"));
    let client = Client::builder()
        .user_agent(crate::USER_AGENT)
        .default_headers(default_headers)
        .build()?;

    Ok(CurseforgeState {
        eternal_api_client: client,
    })
}
