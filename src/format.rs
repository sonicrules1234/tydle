use anyhow::{Result, bail};

#[derive(Debug)]
pub enum Format {
    BestAudio,
    BestVideo,
    WorstAudio,
    WorstVideo,
}

pub fn parse_format(format: &str) -> Result<Format> {
    Ok(match format {
        "bestaudio" => Format::BestAudio,
        "bestvideo" => Format::BestVideo,
        "worstaudio" => Format::WorstAudio,
        "worstvideo" => Format::WorstVideo,
        _ => bail!("Invalid format."),
    })
}

pub fn compact_num(n: u64) -> String {
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

pub fn human_readable_size(bytes: u64) -> String {
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

pub fn get_resolution(height: Option<u64>, width: Option<u64>) -> String {
    match (height, width) {
        (Some(h), Some(w)) => format!("{}x{}", h, w),
        _ => "".to_owned(),
    }
}
