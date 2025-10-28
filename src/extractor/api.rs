use std::collections::HashMap;

use anyhow::Result;
use reqwest::Url;

use crate::extractor::{
    client::INNERTUBE_CLIENTS,
    yt_interface::{DEFAULT_YT_CLIENT, YtClient, YtEndpoint},
};

fn select_api_hostname(default_client: Option<&YtClient>) -> &str {
    let client = default_client.unwrap_or(&DEFAULT_YT_CLIENT);
    let innertube_client = INNERTUBE_CLIENTS.get(client).unwrap();
    return innertube_client.innertube_host;
}

fn extract_client_version(default_client: Option<&YtClient>) -> &str {
    let client = default_client.unwrap_or(&DEFAULT_YT_CLIENT);
    let innertube_client = INNERTUBE_CLIENTS.get(client).unwrap();

    let innertube_client_context = innertube_client.innertube_context.get("client").unwrap();
    innertube_client_context.get("clientName").unwrap()
}

fn extract_context(default_client: Option<&YtClient>) -> HashMap<&str, &str> {
    let client = default_client.unwrap_or(&DEFAULT_YT_CLIENT);
    let innertube_client = INNERTUBE_CLIENTS.get(client).unwrap();
    let mut client_context = innertube_client
        .innertube_context
        .get("client")
        .unwrap()
        .clone();

    // TODO: set this correctly with pref lang
    client_context.insert("hl", "en");
    client_context.insert("timeZone", "UTC");
    client_context.insert("utcOffsetMinutes", "0");

    client_context
}

fn generate_api_headers(default_client: Option<&YtClient>) -> HashMap<&str, String> {
    let client = default_client.unwrap_or(&DEFAULT_YT_CLIENT);
    let innertube_client = INNERTUBE_CLIENTS.get(client).unwrap();
    let host_name = select_api_hostname(Some(client));

    let origin = format!("https://{}", host_name);
    let mut headers = HashMap::new();

    headers.insert(
        "X-YouTube-Client-Name",
        innertube_client.innertube_context_client_name.to_string(),
    );

    headers.insert(
        "X-YouTube-Client-Version",
        extract_client_version(Some(client)).to_string(),
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

pub async fn call_api(default_client: Option<&YtClient>, endpoint: YtEndpoint) -> Result<()> {
    let client = default_client.unwrap_or(&DEFAULT_YT_CLIENT);

    let host_name = select_api_hostname(Some(client));
    let ep = endpoint.as_str();
    let api_url = format!("https://{}/youtubei/v1/{}", host_name, ep);
    let yt_url = Url::parse(api_url.as_str())?;

    let http_client = reqwest::Client::new();
    let real_headers: HashMap<&str, String> = generate_api_headers(Some(client));
    let context = extract_context(Some(client));
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
