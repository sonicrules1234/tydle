use std::sync::Arc;

use anyhow::Result;

use extractor::yt_interface::{VideoId, YtClient};
use yt_scraper::{downloader::Downloader, scraper::YtScraper};

mod extractor;
mod yt_scraper;

#[tokio::main]
async fn main() -> Result<()> {
    // extractor::api::call_api(None, YtEndpoint::Browse).await?;
    let scraper = YtScraper::new(Arc::new(reqwest::Client::new()));
    let video_id = VideoId::new("UWn9RdueB7E")?;
    let webpage = scraper
        .download_initial_webpage("https://www.youtube.com", &YtClient::Web, &video_id)
        .await?;

    println!("{}", webpage);

    Ok(())
}
