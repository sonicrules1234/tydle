use anyhow::Result;

use crate::yt_scraper::scraper::YtScraper;

pub trait Decoder {
    async fn download_initial_webpage(
        self,
        // webpage_url: &str,
        // webpage_client: &YtClient,
        // video_id: &VideoId,
    ) -> Result<String>;
}

impl Decoder for YtScraper {
    async fn download_initial_webpage(
        self,
        // webpage_url: &str,
        // webpage_client: &YtClient,
        // video_id: &VideoId,
    ) -> Result<String> {
        Ok("".into())
    }
}
