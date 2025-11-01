use std::{cell::Cell, collections::HashMap};

use anyhow::{Result, anyhow};
use fancy_regex::Regex;
use reqwest::cookie;
use serde_json::Value;

use crate::extractor::{
    auth::ExtractorAuthHandle,
    download::ExtractorDownloadHandle,
    yt_interface::{VideoId, YtClient},
    ytcfg::ExtractorYtCfgHandle,
};

pub struct YtExtractor {
    pub passed_auth_cookies: Cell<bool>,
    pub http_client: reqwest::Client,
    pub cookie_jar: cookie::Jar,
    pub x_forwarded_for_ip: Option<&'static str>,
}

pub trait InfoExtractor {
    /// Index of current account in account list.
    fn extract_session_index(&self, data: Vec<HashMap<String, String>>) -> Option<i32>;
    fn extract_ytcfg(&self, webpage_content: String) -> Result<HashMap<String, Value>>;
    fn extract_yt_initial_data(&self, webpage_content: String) -> Result<HashMap<String, Value>>;
    async fn initial_extract(
        &self,
        url: &str,
        smuggled_data: &str,
        webpage_url: &str,
        webpage_client: &YtClient,
        video_id: &VideoId,
    ) -> Result<()>;
}

impl YtExtractor {
    pub fn new() -> Result<Self> {
        let extractor = Self {
            passed_auth_cookies: Cell::new(false),
            http_client: reqwest::Client::new(),
            cookie_jar: cookie::Jar::default(),
            x_forwarded_for_ip: None,
        };

        extractor.initialize_pref()?;
        extractor.initialize_consent()?;
        extractor.initialize_cookie_auth()?;

        Ok(extractor)
    }
}

impl InfoExtractor for YtExtractor {
    fn extract_session_index(&self, data: Vec<HashMap<String, String>>) -> Option<i32> {
        for yt_cfg in data {
            if let Some(session_index) = yt_cfg.get("SESSION_INDEX") {
                return Some(session_index.parse().unwrap_or_default());
            }
        }

        None
    }

    fn extract_ytcfg(&self, webpage_content: String) -> Result<HashMap<String, Value>> {
        if webpage_content.is_empty() {
            return Ok(HashMap::new());
        }

        let search_re = Regex::new(r"ytcfg\.set\s*\(\s*({.+?})\s*\)\s*;")?;
        let json_str = search_re
            .captures(&webpage_content)?
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str())
            .unwrap_or("{}");

        let ytcfg: HashMap<String, Value> = serde_json::from_str(json_str)?;

        Ok(ytcfg)
    }

    fn extract_yt_initial_data(&self, webpage_content: String) -> Result<HashMap<String, Value>> {
        let re = Regex::new(
            r#"(?:window\s*\[\s*["']ytInitialData["']\s*\]|ytInitialData)\s*=\s*(\{.*?\})\s*(?:;|</script>)"#,
        )?;
        let json_str = re
            .captures(&webpage_content)?
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str())
            .ok_or_else(|| anyhow!("ytInitialData not found"))?;

        let json_val: HashMap<String, Value> = serde_json::from_str(json_str)?;
        Ok(json_val)
    }

    async fn initial_extract(
        &self,
        url: &str,
        smuggled_data: &str,
        webpage_url: &str,
        webpage_client: &YtClient,
        video_id: &VideoId,
    ) -> Result<()> {
        let webpage = self
            .download_initial_webpage(webpage_url, webpage_client, video_id)
            .await?;

        // ! SKIPPED DEFAULT HERE
        let mut webpage_ytcfg = self.extract_ytcfg(webpage.clone())?;

        println!("{:?}", webpage_ytcfg);

        println!("{:?}", self.extract_yt_initial_data(webpage)?);

        Ok(())
    }
}
