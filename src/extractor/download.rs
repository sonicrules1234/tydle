use std::collections::HashMap;

use anyhow::Result;
use serde_json::Value;
use url::Url;

use crate::extractor::{
    client::INNERTUBE_CLIENTS,
    extract::{InfoExtractor, YtExtractor},
    yt_interface::{VideoId, YtClient},
};

pub trait ExtractorDownloadHandle {
    async fn download_initial_data(
        &self,
        webpage_content: String,
        webpage_client: &YtClient,
        webpage_ytcfg: &HashMap<String, Value>,
    ) -> Result<()>;
    async fn download_initial_webpage(
        &self,
        webpage_url: &str,
        webpage_client: &YtClient,
        video_id: &VideoId,
    ) -> Result<String>;
}

impl ExtractorDownloadHandle for YtExtractor {
    async fn download_initial_data(
        &self,
        webpage_content: String,
        webpage_client: &YtClient,
        webpage_ytcfg: &HashMap<String, Value>,
    ) -> Result<()> {
        let initial_data: Option<HashMap<String, Value>> = if !webpage_content.is_empty() {
            Some(self.extract_yt_initial_data(webpage_content)?)
        } else {
            None
        };

        Ok(())
    }

    // ! DOES NOT YET IMPLEMENT PLAYER PARAMS
    async fn download_initial_webpage(
        &self,
        webpage_url: &str,
        webpage_client: &YtClient,
        video_id: &VideoId,
    ) -> Result<String> {
        let formatted_url = Url::parse(webpage_url)?;
        let watch_page_url = formatted_url.join("watch")?;
        let mut webpage_request = self.http_client.get(watch_page_url).query(&[
            ("bpctr", "9999999999"),
            ("has_verified", "1"),
            ("v", video_id.as_str()),
        ]);
        let innertube_client = INNERTUBE_CLIENTS.get(webpage_client).unwrap();

        let client = innertube_client.innertube_context.get("client").unwrap();
        if let Some(user_agent) = client.get("userAgent") {
            webpage_request =
                webpage_request.header("User-Agent", user_agent.as_str().unwrap_or_default());
        }

        let response = webpage_request.send().await?;

        for (key, value) in response.headers() {
            println!("{}: {}", key.as_str(), value.to_str()?);
        }
        let webpage = response.text().await.map_err(|e| anyhow::Error::new(e))?;

        Ok(webpage)
    }
}
