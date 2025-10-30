use std::sync::Arc;

pub struct YtScraper {
    pub http_client: Arc<reqwest::Client>,
}

impl YtScraper {
    pub fn new(http_client: Arc<reqwest::Client>) -> Self {
        Self { http_client }
    }
}
