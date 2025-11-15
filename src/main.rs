use std::process;

use anyhow::Result;
use clap::Parser;
use tokio::fs;
use tydle::{Extract, Tydle, TydleOptions, VideoId, cookies::parse_netscape_cookies};

#[derive(Parser, Debug)]
#[clap(version)]
struct TydleArgs {
    /// Client-side IP address to bind to.
    #[arg(long)]
    source_ip: Option<String>,
    /// Netscape formatted file to read cookies from and dump cookie jar in.
    #[arg(long)]
    cookies: Option<String>,
    /// Use an unencrypted connection to retrieve information about the video.
    #[arg(long)]
    prefer_insecure: bool,
    video_id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = run().await {
        log::error!("{}", e.to_string());
        process::exit(1);
    }

    Ok(())
}

async fn run() -> Result<()> {
    let args = TydleArgs::parse();
    let auth_cookies = match args.cookies {
        Some(cookies_path) => {
            let cookie_file_content = fs::read_to_string(cookies_path).await?;
            parse_netscape_cookies(cookie_file_content)?
        }
        None => Default::default(),
    };

    tydle::logger::init_logging("info");
    let tydle = Tydle::new(TydleOptions {
        auth_cookies,
        prefer_insecure: args.prefer_insecure,
        source_address: args.source_ip.unwrap_or_default(),
    })?;

    let video_id = VideoId::new(args.video_id)?;
    let streams = tydle.get_streams(&video_id).await?;

    println!("{:#?}", streams);

    Ok(())
}
