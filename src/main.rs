use anyhow::Result;

use crate::extractor::yt_interface::YtEndpoint;

mod extractor;

#[tokio::main]
async fn main() -> Result<()> {
    extractor::api::call_api(None, YtEndpoint::Browse).await?;
    Ok(())
}
