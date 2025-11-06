use anyhow::Result;

use extractor::yt_interface::VideoId;

use crate::extractor::extract::{InfoExtractor, YtExtractor};

mod extractor;
mod ty;

#[tokio::main]
async fn main() -> Result<()> {
    let mut extractor = YtExtractor::new()?;
    let video_id = VideoId::new("UWn9RdueB7E")?;

    extractor.extract_streams(&video_id).await?;

    Ok(())
}
