use std::{collections::HashMap, sync::RwLock};

use anyhow::{Result, anyhow};
use url::Url;

pub(crate) type Cookies = HashMap<String, String>;
pub(crate) type DomainMap = HashMap<Url, Cookies>;

#[derive(Debug)]
pub(crate) struct CookieJar {
    cookies: RwLock<DomainMap>,
}

impl CookieJar {
    pub fn new_from_domain(domain: &str, cookies: Cookies) -> Result<Self> {
        let mut domain_map = HashMap::new();
        domain_map.insert(Url::parse(domain)?, cookies);

        Ok(Self {
            cookies: RwLock::new(domain_map),
        })
    }
}

pub(crate) trait CookieStore {
    fn get_all(&self, domain: &str) -> Result<Option<Cookies>>;
    fn set(&self, domain: &str, name: &str, value: &str) -> Result<()>;
}

impl CookieStore for CookieJar {
    fn get_all(&self, domain: &str) -> Result<Option<Cookies>> {
        let domain_url = Url::parse(domain)?;
        Ok(self
            .cookies
            .write()
            .map_err(|e| anyhow!(e.to_string()))?
            .get(&domain_url)
            .cloned())
    }

    fn set(&self, domain: &str, name: &str, value: &str) -> Result<()> {
        let domain_url = Url::parse(domain)?;
        let mut cookies = self.cookies.write().map_err(|e| anyhow!(e.to_string()))?;

        if let Some(cookies) = cookies.get_mut(&domain_url) {
            cookies.insert(name.into(), value.into());
        }

        Ok(())
    }
}

/// Parse a Netscape formatted cookie file into a `HashMap`
pub fn read_from_cookie_file(path: &str) -> Result<HashMap<String, String>> {
    let cookie_file_contents = std::fs::read_to_string(path)?;
    let mut cookies = HashMap::new();

    for line in cookie_file_contents.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split('\t').collect();

        if parts.len() < 7 {
            continue;
        }

        let name = parts[5].trim();
        let value = parts[6].trim();

        cookies.insert(name.to_string(), value.to_string());
    }

    Ok(cookies)
}
