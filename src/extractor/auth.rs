use std::collections::HashMap;

use anyhow::{Result, anyhow};
use reqwest::Url;

use crate::extractor::{
    cookies::ExtractorCookieHandle,
    extract::YtExtractor,
    utils::{convert_to_query_string, parse_query_string},
    yt_interface::{PREFERRED_LOCALE, YT_URL},
};

pub trait ExtractorAuthHandle {
    fn initialize_cookie_auth(&self) -> Result<()>;
    fn initialize_consent(&self) -> Result<()>;
    fn initialize_pref(&self) -> Result<()>;
    fn is_authenticated(&self) -> Result<bool>;
    fn has_auth_cookies(&self) -> Result<bool>;
}

/// Handles Auth with cookies and user-set preferences.
impl ExtractorAuthHandle for YtExtractor {
    fn initialize_cookie_auth(&self) -> Result<()> {
        self.passed_auth_cookies.set(false);
        if self.has_auth_cookies()? {
            self.passed_auth_cookies.set(true);
        }

        Ok(())
    }

    fn initialize_consent(&self) -> Result<()> {
        if self.has_auth_cookies()? {
            return Ok(());
        }

        let yt_cookies = self.get_youtube_cookies()?;

        if let Some(socs) = yt_cookies.get("SOCS") {
            if !socs.starts_with("CAA") {
                return Ok(());
            }
        }

        self.cookie_jar
            .add_cookie_str("SOCS=CAI", &Url::parse(YT_URL)?);
        Ok(())
    }

    fn initialize_pref(&self) -> Result<()> {
        let youtube_cookies = self.get_youtube_cookies()?;
        let mut pref: HashMap<String, String> = HashMap::new();

        if let Some(raw_pref) = youtube_cookies.get("PREF") {
            match parse_query_string(&raw_pref) {
                Some(parsed_qs) => pref = parsed_qs,
                None => return Err(anyhow!("Failed to parse user PREF cookie.")),
            }
        }

        pref.insert("hl".into(), PREFERRED_LOCALE.into());
        pref.insert("tz".into(), "UTC".into());

        let pref_qs = convert_to_query_string(&pref);
        self.cookie_jar
            .add_cookie_str(format!("PREF={}", pref_qs).as_str(), &Url::parse(YT_URL)?);

        Ok(())
    }

    fn is_authenticated(&self) -> Result<bool> {
        return self.has_auth_cookies();
    }

    fn has_auth_cookies(&self) -> Result<bool> {
        let sid_cookies = self.get_sid_cookies()?;
        let youtube_cookies = self.get_youtube_cookies()?;

        Ok(youtube_cookies.contains_key("LOGIN_INFO")
            && (sid_cookies.yt_sapisid.is_some()
                || sid_cookies.yt_1psapisid.is_some()
                || sid_cookies.yt_3psapisid.is_some()))
    }
}
