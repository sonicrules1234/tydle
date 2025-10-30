use std::{cell::Cell, sync::Arc};

use anyhow::Result;
use reqwest::cookie;

use crate::{extractor::auth::ExtractorAuthHandle, yt_scraper::scraper::YtScraper};

pub struct YtExtractor {
    pub passed_auth_cookies: Cell<bool>,
    pub http_client: Arc<reqwest::Client>,
    pub yt_scraper: YtScraper,
    pub cookie_jar: cookie::Jar,
    pub x_forwarded_for_ip: Option<&'static str>,
}

trait InfoExtractor {
    fn initial_extract(self);
}

impl YtExtractor {
    fn new() -> Result<Self> {
        let http_client = Arc::new(reqwest::Client::new());
        let extractor = Self {
            passed_auth_cookies: Cell::new(false),
            yt_scraper: YtScraper::new(http_client.clone()),
            http_client,
            cookie_jar: cookie::Jar::default(),
            x_forwarded_for_ip: None,
        };

        extractor.initialize_cookie_auth()?;

        Ok(extractor)
    }
}

impl InfoExtractor for YtExtractor {
    fn initial_extract(self) {}
}
