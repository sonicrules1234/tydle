use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
};

use anyhow::{Result, anyhow};
use fancy_regex::Regex;
use reqwest::cookie;
use serde_json::Value;

use crate::extractor::{
    auth::ExtractorAuthHandle,
    client::INNERTUBE_CLIENTS,
    download::ExtractorDownloadHandle,
    player::ExtractorPlayerHandle,
    yt_interface::{VideoId, YtClient},
    ytcfg::ExtractorYtCfgHandle,
};

pub struct YtExtractor {
    pub passed_auth_cookies: Cell<bool>,
    pub http_client: reqwest::Client,
    pub cookie_jar: cookie::Jar,
    // pub x_forwarded_for_ip: Option<&'static str>,
}

pub trait InfoExtractor {
    fn search_json(
        &self,
        start_pattern: &str,
        html: &str,
        end_pattern: Option<&str>,
        default: Option<HashMap<String, Value>>,
    ) -> Result<HashMap<String, Value>>;
    fn generate_checkok_params(&self) -> HashMap<&str, &str>;
    fn get_text(
        &self,
        data: &Value,
        path_list: Option<Vec<Vec<&str>>>,
        max_runs: Option<usize>,
    ) -> Option<String>;
    fn is_music_url(&self, url: &str) -> Result<bool>;
    fn is_premium_subscriber(&self, initial_data: &HashMap<String, Value>) -> Result<bool>;
    fn extract_ytcfg(&self, webpage_content: String) -> Result<HashMap<String, Value>>;
    fn extract_yt_initial_data(&self, webpage_content: &String) -> Result<HashMap<String, Value>>;
    fn get_clients(
        &self,
        url: &str,
        smuggled_data: &HashMap<String, String>,
        is_premium_subscriber: bool,
    ) -> Result<Vec<YtClient>>;
    async fn initial_extract(
        &self,
        url: &str,
        smuggled_data: HashMap<String, String>,
        webpage_url: &str,
        webpage_client: &YtClient,
        video_id: &VideoId,
    ) -> Result<()>;
}

impl YtExtractor {
    pub fn new() -> Result<Self> {
        let extractor = Self {
            passed_auth_cookies: Cell::new(false),
            http_client: reqwest::Client::new(),
            cookie_jar: cookie::Jar::default(),
            // x_forwarded_for_ip: None,
        };

        extractor.initialize_pref()?;
        extractor.initialize_consent()?;
        extractor.initialize_cookie_auth()?;

        Ok(extractor)
    }
}

impl InfoExtractor for YtExtractor {
    fn generate_checkok_params(&self) -> HashMap<&str, &str> {
        let mut checkout_params_map = HashMap::new();

        checkout_params_map.insert("contentCheckOk", "true");
        checkout_params_map.insert("racyCheckOk", "true");

        checkout_params_map
    }

    fn get_text(
        &self,
        data: &Value,
        path_list: Option<Vec<Vec<&str>>>,
        max_runs: Option<usize>,
    ) -> Option<String> {
        let paths = path_list.unwrap_or_else(|| vec![vec![]]);
        for path in paths {
            let mut current = data;
            for key in &path {
                if !current.is_object() {
                    current = &Value::Null;
                    break;
                }
                current = current.get(*key).unwrap_or(&Value::Null);
            }

            let objs: Vec<&Value> = if path.is_empty() {
                vec![data]
            } else if !current.is_null() {
                vec![current]
            } else {
                continue;
            };

            for item in objs {
                if let Some(text) = item.get("simpleText").and_then(|v| v.as_str()) {
                    return Some(text.to_string());
                }

                let mut runs: Vec<Value> = item
                    .get("runs")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_else(|| {
                        if let Some(arr) = item.as_array() {
                            arr.clone()
                        } else {
                            vec![]
                        }
                    });

                if runs.is_empty() {
                    continue;
                }

                if let Some(limit) = max_runs {
                    runs.truncate(limit.min(runs.len()));
                }

                let text = runs
                    .iter()
                    .filter_map(|r| r.get("text").and_then(|t| t.as_str()))
                    .collect::<String>();

                if !text.is_empty() {
                    return Some(text);
                }
            }
        }

        None
    }

    fn search_json(
        &self,
        start_pattern: &str,
        html: &str,
        end_pattern: Option<&str>,
        default: Option<HashMap<String, Value>>,
    ) -> Result<HashMap<String, Value>> {
        let default_value = default.unwrap_or_default();
        let end_pattern = end_pattern.unwrap_or("");

        let re_start =
            Regex::new(start_pattern).map_err(|e| anyhow!("Invalid start regex: {e}"))?;
        let re_end = if !end_pattern.is_empty() {
            Some(Regex::new(end_pattern).map_err(|e| anyhow!("Invalid end regex: {e}"))?)
        } else {
            None
        };

        let start_pos = if let Some(m) = re_start.find(html)? {
            m.end()
        } else {
            return Ok(default_value);
        };

        let mut json_start = None;
        let mut depth = 0usize;
        let mut in_str = false;
        let mut escape = false;

        let chars: Vec<char> = html[start_pos..].chars().collect();
        for (i, &c) in chars.iter().enumerate() {
            if json_start.is_none() {
                if c == '{' {
                    json_start = Some(i);
                    depth = 1;
                }
                continue;
            }

            if in_str {
                if escape {
                    escape = false;
                    continue;
                }
                if c == '\\' {
                    escape = true;
                    continue;
                }
                if c == '"' {
                    in_str = false;
                }
            } else {
                match c {
                    '"' => in_str = true,
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            let json_str: String = chars[json_start.unwrap()..=i].iter().collect();
                            if let Some(re_end) = &re_end {
                                if let Some(m_end) = re_end.find(&html[start_pos + i..])? {
                                    let _ = m_end;
                                }
                            }

                            return serde_json::from_str(&json_str)
                                .map_err(|e| anyhow!("Failed to parse JSON: {e}\n{json_str}"))
                                .or_else(|_| Ok(default_value.clone()));
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(default_value)
    }

    fn is_music_url(&self, url: &str) -> Result<bool> {
        let re = Regex::new(r"(https?://)?music\.youtube\.com/")?;
        Ok(re.is_match(url)?)
    }

    fn is_premium_subscriber(&self, initial_data: &HashMap<String, Value>) -> Result<bool> {
        if !self.is_authenticated()? || initial_data.is_empty() {
            return Ok(false);
        }

        let tlr = initial_data
            .get("topbar")
            .and_then(|v| v.get("desktopTopbarRenderer"))
            .and_then(|v| v.get("logo"))
            .and_then(|v| v.get("topbarLogoRenderer"));
        let logo_match = tlr
            .and_then(|v| v.get("iconImage"))
            .and_then(|v| v.get("iconType"))
            .unwrap_or(&Value::Null);
        let logo_match_str = logo_match.as_str().unwrap_or_default();

        Ok(logo_match_str == "YOUTUBE_PREMIUM_LOGO"
            || self
                .get_text(
                    tlr.unwrap_or_default(),
                    Some(vec![vec!["tooltipText"]]),
                    None,
                )
                .unwrap_or_default()
                .to_lowercase()
                .contains("premium"))
    }

    fn extract_ytcfg(&self, webpage_content: String) -> Result<HashMap<String, Value>> {
        if webpage_content.is_empty() {
            return Ok(HashMap::new());
        }

        let search_re = Regex::new(r"ytcfg\.set\s*\(\s*({.+?})\s*\)\s*;")?;
        let json_str = search_re
            .captures(&webpage_content)?
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str())
            .unwrap_or("{}");

        let ytcfg: HashMap<String, Value> = serde_json::from_str(json_str)?;

        Ok(ytcfg)
    }

    fn extract_yt_initial_data(&self, webpage_content: &String) -> Result<HashMap<String, Value>> {
        let re = Regex::new(
            r#"(?:window\s*\[\s*["']ytInitialData["']\s*\]|ytInitialData)\s*=\s*(\{.*?\})\s*(?:;|</script>)"#,
        )?;
        let json_str = re
            .captures(&webpage_content)?
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str())
            .ok_or_else(|| anyhow!("ytInitialData not found"))?;

        let json_val: HashMap<String, Value> = serde_json::from_str(json_str)?;
        Ok(json_val)
    }

    fn get_clients(
        &self,
        url: &str,
        smuggled_data: &HashMap<String, String>,
        is_premium_subscriber: bool,
    ) -> Result<Vec<YtClient>> {
        let mut clients = if is_premium_subscriber {
            // Premium does not require POT. (except for subtitles)
            vec![
                YtClient::Tv,
                YtClient::WebCreator,
                YtClient::WebSafari,
                YtClient::Web,
            ]
        } else if self.is_authenticated()? {
            vec![YtClient::Tv, YtClient::WebSafari, YtClient::Web]
        } else {
            vec![
                YtClient::AndroidSdkless,
                YtClient::Tv,
                YtClient::WebSafari,
                YtClient::Web,
            ]
        };

        if self.is_authenticated()? {
            if smuggled_data.get("is_music_url").unwrap_or(&"".to_string()) == "true"
                || self.is_music_url(url)?
            {
                clients.push(YtClient::WebMusic);
            }

            let mut unsupported_clients = Vec::new();

            for client in &clients {
                if !INNERTUBE_CLIENTS.get(&client).unwrap().supports_cookies {
                    unsupported_clients.push(*client);
                }
            }

            for client in &unsupported_clients {
                println!(
                    "[WARN] Skipping client \"{}\" since it does not support cookies.",
                    client.as_str()
                );

                clients.retain(|c| !unsupported_clients.iter().any(|u| u.as_str() == c.as_str()));
            }
        }

        let mut seen = HashSet::new();
        let unique_clients: Vec<_> = clients.into_iter().filter(|c| seen.insert(*c)).collect();

        Ok(unique_clients)
    }

    async fn initial_extract(
        &self,
        url: &str,
        smuggled_data: HashMap<String, String>,
        webpage_url: &str,
        webpage_client: &YtClient,
        video_id: &VideoId,
    ) -> Result<()> {
        let webpage = self
            .download_initial_webpage(webpage_url, webpage_client, video_id)
            .await?;

        let mut webpage_ytcfg = self.extract_ytcfg(webpage.clone())?;

        if webpage_ytcfg.is_empty() {
            webpage_ytcfg = self
                .select_default_ytcfg(Some(webpage_client))?
                .to_json_val_hashmap()?;
        }
        let initial_data = self
            .download_initial_data(video_id, &webpage, webpage_client, &webpage_ytcfg)
            .await?;

        let is_premium_subscriber = self.is_premium_subscriber(&initial_data)?;
        let clients = self.get_clients(url, &smuggled_data, is_premium_subscriber)?;
        let player_response = self.extract_player_response(
            &clients,
            video_id,
            &webpage,
            webpage_client,
            &webpage_ytcfg,
            is_premium_subscriber,
        )?;

        Ok(player_response)
    }
}
