use std::collections::HashMap;

use anyhow::Result;
use serde_json::Value;

use crate::extractor::{
    auth::ExtractorAuthHandle,
    client::{INNERTUBE_CLIENTS, InnerTubeClient},
    extract::YtExtractor,
    yt_interface::{DEFAULT_YT_CLIENT, YtClient},
};

pub trait ExtractorYtCfgHandle {
    fn select_api_hostname(&self, default_client: Option<&YtClient>) -> &str;
    fn select_client_version(&self, default_client: Option<&YtClient>) -> &str;
    fn select_context(&self, default_client: Option<&YtClient>) -> HashMap<&str, Value>;
    fn select_default_ytcfg(&self, default_client: Option<&YtClient>) -> Result<InnerTubeClient>;
}

impl ExtractorYtCfgHandle for YtExtractor {
    fn select_api_hostname(&self, default_client: Option<&YtClient>) -> &str {
        let client = default_client.unwrap_or(&DEFAULT_YT_CLIENT);
        let innertube_client = INNERTUBE_CLIENTS.get(client).unwrap();
        return innertube_client.innertube_host;
    }

    fn select_client_version(&self, default_client: Option<&YtClient>) -> &str {
        let client = default_client.unwrap_or(&DEFAULT_YT_CLIENT);
        let innertube_client = INNERTUBE_CLIENTS.get(client).unwrap();

        let innertube_client_context = innertube_client.innertube_context.get("client").unwrap();
        innertube_client_context
            .get("clientName")
            .unwrap()
            .as_str()
            .unwrap()
    }

    fn select_context(&self, default_client: Option<&YtClient>) -> HashMap<&str, Value> {
        let client = default_client.unwrap_or(&DEFAULT_YT_CLIENT);
        let innertube_client = INNERTUBE_CLIENTS.get(client).unwrap();
        let mut client_context = innertube_client
            .innertube_context
            .get("client")
            .unwrap()
            .clone();

        // TODO: set this correctly with pref lang
        client_context.insert("hl", "en".into());
        client_context.insert("timeZone", "UTC".into());
        client_context.insert("utcOffsetMinutes", "0".into());

        client_context
    }

    fn select_default_ytcfg(&self, default_client: Option<&YtClient>) -> Result<InnerTubeClient> {
        let client = default_client.unwrap_or(&DEFAULT_YT_CLIENT);
        let mut ytcfg = INNERTUBE_CLIENTS.get(client).cloned().unwrap();

        if let (Some(auth_ua), true) = (&ytcfg.authenticated_user_agent, self.is_authenticated()?) {
            let innertube_client_context = ytcfg
                .innertube_context
                .entry("client")
                .or_insert_with(HashMap::new);

            innertube_client_context.insert("userAgent", (*auth_ua).into());
        }

        Ok(ytcfg)
    }
}
