use std::{
    ops::{Deref, DerefMut},
    sync::RwLock,
};

use anyhow::{Result, anyhow};
#[cfg(target_arch = "wasm32")]
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

#[cfg_attr(
    target_arch = "wasm32",
    derive(Serialize, Deserialize, tsify::Tsify),
    tsify(into_wasm_abi, from_wasm_abi),
    serde(rename_all = "camelCase")
)]
#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    #[cfg_attr(target_arch = "wasm32", tsify(type = "number"))]
    pub expiration: u64,
    pub http_only: bool,
}

impl Default for Cookie {
    fn default() -> Self {
        Self {
            name: String::new(),
            value: String::new(),
            domain: String::new(),
            path: "/".to_string(),
            secure: false,
            expiration: 0,
            http_only: false,
        }
    }
}

#[cfg_attr(
    target_arch = "wasm32",
    derive(Serialize, Deserialize, tsify::Tsify),
    tsify(into_wasm_abi, from_wasm_abi),
    serde(rename_all = "camelCase")
)]
#[derive(Debug, Default, Clone)]
pub struct DomainCookies(Vec<Cookie>);

impl FromIterator<Cookie> for DomainCookies {
    #[inline]
    fn from_iter<T: IntoIterator<Item = Cookie>>(iter: T) -> Self {
        DomainCookies(iter.into_iter().collect())
    }
}

impl Deref for DomainCookies {
    type Target = Vec<Cookie>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DomainCookies {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl DomainCookies {
    pub fn new(cookies: Vec<Cookie>) -> Self {
        Self(cookies)
    }

    #[inline]
    pub fn get(&self, name: &str) -> Option<&Cookie> {
        self.0.iter().find(|c| c.name == name)
    }

    #[inline]
    pub fn exists(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    /// Convert the `Vec<Cookie>` to a `String` formatted as a HTTP header.
    pub fn header_value(&self) -> String {
        let parts: Vec<String> = self
            .0
            .iter()
            .map(|c| format!("{}={}", c.name, c.value))
            .collect();

        parts.join("; ")
    }
}

#[derive(Debug)]
pub(crate) struct CookieJar {
    cookies: RwLock<DomainCookies>,
}

impl CookieJar {
    pub fn new_with_cookies(cookies: DomainCookies) -> Self {
        Self {
            cookies: RwLock::new(cookies),
        }
    }
}

pub(crate) trait CookieStore {
    fn get_all(&self, domain: &str) -> Result<DomainCookies>;
    fn set(&self, cookie: Cookie) -> Result<()>;
}

impl CookieStore for CookieJar {
    fn get_all(&self, domain: &str) -> Result<DomainCookies> {
        let cookies = self.cookies.read().map_err(|e| anyhow!(e.to_string()))?;

        Ok(cookies
            .iter()
            .filter(|c| c.domain == domain)
            .cloned()
            .collect())
    }

    fn set(&self, cookie: Cookie) -> Result<()> {
        let mut cookies = self.cookies.write().map_err(|e| anyhow!(e.to_string()))?;
        cookies.push(cookie);

        Ok(())
    }
}

/// Parse a Netscape formatted cookie file into `DomainCookies`
pub fn parse_netscape_cookies(cookie_content: String) -> Result<DomainCookies> {
    let mut cookies = DomainCookies::new(vec![]);

    for line in cookie_content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Netscape format:
        // domain \t flag \t path \t secure \t expiration \t name \t value
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() != 7 {
            continue;
        }

        let domain_raw = parts[0].trim();
        let include_subdomains = parts[1].trim().eq_ignore_ascii_case("TRUE");
        let path = parts[2].trim().to_string();
        let secure = parts[3].trim().eq_ignore_ascii_case("TRUE");
        let expiration = parts[4].trim().parse::<u64>().unwrap_or_default();
        let name = parts[5].trim().to_string();
        let value = parts[6].trim().to_string();

        if domain_raw.is_empty() {
            continue;
        }

        let domain = if include_subdomains && !domain_raw.starts_with('.') {
            format!(".{}", domain_raw)
        } else {
            domain_raw.to_string()
        };

        cookies.push(Cookie {
            http_only: name.starts_with("__Host-") || name.starts_with("__Secure-"),
            name,
            value,
            domain,
            path,
            expiration,
            secure,
        });
    }

    Ok(cookies)
}

/// Parse a Netscape formatted cookie file into a `HashMap`
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = "parseNetscapeCookies")]
pub fn parse_netscape_cookies_js(
    #[wasm_bindgen(js_name = "cookieContent")] cookie_content: String,
) -> Result<DomainCookies, JsValue> {
    let cookies =
        parse_netscape_cookies(cookie_content).map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(cookies)
}
