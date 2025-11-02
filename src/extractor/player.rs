use std::collections::HashMap;

use anyhow::Result;
use fancy_regex::Regex;
use serde_json::Value;

use crate::extractor::{
    extract::{InfoExtractor, YtExtractor},
    yt_interface::{VideoId, YtClient},
};

pub trait ExtractorPlayerHandle {
    fn extract_player_response(
        &self,
        clients: &Vec<YtClient>,
        video_id: &VideoId,
        webpage: &String,
        webpage_client: &YtClient,
        webpage_ytcfg: &HashMap<String, Value>,
        is_premium_subscriber: bool,
    ) -> Result<()>;
}

impl ExtractorPlayerHandle for YtExtractor {
    fn extract_player_response(
        &self,
        clients: &Vec<YtClient>,
        video_id: &VideoId,
        webpage: &String,
        webpage_client: &YtClient,
        webpage_ytcfg: &HashMap<String, Value>,
        is_premium_subscriber: bool,
    ) -> Result<()> {
        let initial_pr = self.search_json(r"ytInitialPlayerResponse\s*=", &webpage, None, None)?;

        println!("{:?}", webpage_ytcfg);
        println!("{}", webpage);
        println!("{:#?}", initial_pr);

        Ok(())
    }
}
