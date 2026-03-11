// This client is a minimal scaffold. It will be replaced by auto-generated
// code from the HotData OpenAPI spec via the regenerate workflow.

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Invalid header value: {0}")]
    Header(#[from] reqwest::header::InvalidHeaderValue),
}

pub struct HotdataClient {
    client: reqwest::Client,
    base_url: String,
}

impl HotdataClient {
    pub fn new(api_token: &str, workspace_id: &str) -> Result<Self, Error> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {api_token}"))?,
        );
        headers.insert("X-Workspace-Id", HeaderValue::from_str(workspace_id)?);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self {
            client,
            base_url: "https://api.hotdata.dev".to_string(),
        })
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub async fn health(&self) -> Result<reqwest::StatusCode, Error> {
        let resp = self
            .client
            .get(format!("{}/health", self.base_url))
            .send()
            .await?;
        Ok(resp.status())
    }
}
