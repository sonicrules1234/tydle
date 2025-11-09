mod cache;
mod cipher;
mod cookies;
mod extractor;
mod ty;
mod utils;
mod yt_interface;

use crate::ty::Extract;
use anyhow::Result;
use ty::Ty;

use crate::yt_interface::{VideoId, YtStreamSource};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let ty = Ty::new()?;
    let video_id = VideoId::new("UWn9RdueB7E")?;
    let stream_response = ty.get_streams(&video_id).await?;

    for s in stream_response.streams {
        if let YtStreamSource::Signature(sig) = s.source {
            let url = ty
                .decipher_stream_signature(sig, stream_response.player_url.clone())
                .await?;

            println!("{url}");
        }
    }

    Ok(())
}
