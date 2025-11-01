use std::collections::HashMap;

use anyhow::Result;
use reqwest::Url;

use crate::extractor::{
    client::INNERTUBE_CLIENTS,
    extract::YtExtractor,
    yt_interface::{DEFAULT_YT_CLIENT, YtClient, YtEndpoint},
    ytcfg::ExtractorYtCfgHandle,
};

pub trait ExtractorApiHandle {
    fn generate_api_headers(&self, default_client: Option<&YtClient>) -> HashMap<&str, String>;
    async fn call_api(&self, default_client: Option<&YtClient>, endpoint: YtEndpoint)
    -> Result<()>;
}

impl ExtractorApiHandle for YtExtractor {
    fn generate_api_headers(&self, default_client: Option<&YtClient>) -> HashMap<&str, String> {
        let client = default_client.unwrap_or(&DEFAULT_YT_CLIENT);
        let innertube_client = INNERTUBE_CLIENTS.get(client).unwrap();
        let host_name = self.select_api_hostname(Some(client));

        let origin = format!("https://{}", host_name);
        let mut headers = HashMap::new();

        headers.insert(
            "X-YouTube-Client-Name",
            innertube_client.innertube_context_client_name.to_string(),
        );

        headers.insert(
            "X-YouTube-Client-Version",
            self.select_client_version(Some(client)).to_string(),
        );

        headers.insert("Origin", origin);

        // TODO:
        // headers.insert("X-Goog-Visitor-Id", origin);

        let innertube_client_context = innertube_client.innertube_context.get("client").unwrap();

        if let Some(user_agent) = innertube_client_context.get("userAgent") {
            headers.insert("User-Agent", user_agent.to_string());
        }

        headers
    }

    async fn call_api(
        &self,
        default_client: Option<&YtClient>,
        endpoint: YtEndpoint,
    ) -> Result<()> {
        let client = default_client.unwrap_or(&DEFAULT_YT_CLIENT);

        let host_name = self.select_api_hostname(Some(client));
        let ep = endpoint.as_str();
        let api_url = format!("https://{}/youtubei/v1/{}", host_name, ep);
        let yt_url = Url::parse(api_url.as_str())?;

        let http_client = reqwest::Client::new();
        let real_headers: HashMap<&str, String> = self.generate_api_headers(Some(client));
        let context = self.select_context(Some(client));
        let mut request_builder = http_client
            .post(yt_url)
            .json(&context)
            .query(&[("prettyPrint", false)]);

        for (key, value) in real_headers {
            request_builder = request_builder.header(key, value);
        }

        request_builder = request_builder.header("Content-Type", "application/json");

        let response = request_builder.send().await?;
        let data = response.text().await?;

        println!("{}", data);

        Ok(())
    }
}
