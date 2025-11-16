use std::process;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use tokio::fs;
use tydle::{Extract, Tydle, TydleOptions, VideoId, YtStream, cookies::parse_netscape_cookies};

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
    #[arg(long)]
    /// List available formats of each video.
    list_formats: bool,
    #[arg(long)]
    get_url: bool,
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
    let yt_stream_response = tydle.get_streams(&video_id).await?;

    log::info!("Got player URL: {}", yt_stream_response.player_url);

    if args.list_formats {
        list_formats(&yt_stream_response.streams);
    }

    Ok(())
}

fn list_formats(streams: &Vec<YtStream>) {
    println!(
        "{:<5} {:<8} {:<10} {:<3} | {:<12} {:<10} {:<6} | {:<14} {}",
        "ID".yellow(),
        "EXT".yellow(),
        "RESOLUTION".yellow(),
        "FPS".yellow(),
        "FILESIZE".yellow(),
        "TBR".yellow(),
        "PROTO".yellow(),
        "VCODEC".yellow(),
        "ACODEC".yellow(),
    );
    println!("{}", "-".repeat(100));

    for stream in streams {
        let resolution = get_resolution(stream.height, stream.width);
        println!(
            "{:<5} {:<8} {:<10} {:<3} | {:<12} {:<10} {:<6} | {:<14} {}",
            stream.itag.to_string().green(),
            stream.ext.as_str(),
            if resolution == "" {
                "audio only"
            } else {
                resolution.as_str()
            },
            stream.fps,
            if let Some(file_size) = stream.file_size {
                format!("~{}", human_readable_size(file_size))
            } else {
                "".into()
            }
            .bright_black(),
            compact_num(stream.tbr as u64),
            "https", // stream.proto
            stream.codec.vcodec.clone().unwrap_or_default(),
            stream.codec.acodec.clone().unwrap_or_default(),
        );
    }
}

fn get_resolution(height: Option<u64>, width: Option<u64>) -> String {
    match (height, width) {
        (Some(h), Some(w)) => format!("{}x{}", h, w),
        _ => "".to_owned(),
    }
}

fn compact_num(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.1}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn human_readable_size(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;

    let b = bytes as f64;

    if b >= GIB {
        format!("{:.2}GiB", b / GIB)
    } else if b >= MIB {
        format!("{:.2}MiB", b / MIB)
    } else if b >= KIB {
        format!("{:.2}KiB", b / KIB)
    } else {
        format!("{}B", bytes)
    }
}
