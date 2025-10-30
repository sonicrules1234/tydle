use anyhow::Result;

use crate::extractor::{cookies::ExtractorCookieHandle, extract::YtExtractor};

pub trait ExtractorAuthHandle {
    fn initialize_cookie_auth(&self) -> Result<()>;
    fn is_authenticated(&self) -> Result<bool>;
    fn has_auth_cookies(&self) -> Result<bool>;
}

impl ExtractorAuthHandle for YtExtractor {
    fn initialize_cookie_auth(&self) -> Result<()> {
        self.passed_auth_cookies.set(false);
        if self.has_auth_cookies()? {
            self.passed_auth_cookies.set(true);
        }

        Ok(())
    }

    fn is_authenticated(&self) -> Result<bool> {
        return self.has_auth_cookies();
    }

    fn has_auth_cookies(&self) -> Result<bool> {
        const LOGIN_INFO_COOKIE_KEY: &str = "LOGIN_INFO";

        let sid_cookies = self.get_sid_cookies()?;
        let youtube_cookies = self.get_youtube_cookies()?;

        Ok(youtube_cookies.contains_key(LOGIN_INFO_COOKIE_KEY)
            && (sid_cookies.yt_sapisid.is_some()
                || sid_cookies.yt_1psapisid.is_some()
                || sid_cookies.yt_3psapisid.is_some()))
    }
}
