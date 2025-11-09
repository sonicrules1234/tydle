use std::sync::Arc;

use anyhow::{Result, bail};

use crate::{
    cache::{CacheAccess, CacheStore, PlayerCacheHandle},
    cipher::js::SignatureJsHandle,
    utils::parse_query_string,
};

pub struct SignatureDecipher {
    pub player_cache: Arc<CacheStore<(String, String)>>,
    pub code_cache: Arc<CacheStore>,
}

impl SignatureDecipher {
    pub fn new(
        player_cache: Arc<CacheStore<(String, String)>>,
        code_cache: Arc<CacheStore>,
    ) -> Self {
        Self {
            player_cache,
            code_cache,
        }
    }
}

pub trait SignatureDecipherHandle {
    async fn extract_signature_function(
        &self,
        player_url: String,
        example_sig: String,
    ) -> Result<String>;
    async fn decrypt_signature(
        &self,
        encrypted_signature: String,
        player_url: String,
    ) -> Result<String>;
    async fn decipher(&self, signature: String, player_url: String) -> Result<String>;
}

impl SignatureDecipherHandle for SignatureDecipher {
    async fn extract_signature_function(
        &self,
        player_url: String,
        example_sig: String,
    ) -> Result<String> {
        let player_js_code_key = self.player_cache.player_js_cache_key(&player_url)?;

        if let Some(code) = self.code_cache.get(&player_js_code_key)? {
            let res = self.parse_signature_js(code, example_sig).await?;
            return Ok(res);
        }

        bail!(
            "The player.js was not downloaded before, deciphering failed because the code was not found."
        )
    }

    async fn decrypt_signature(
        &self,
        encrypted_signature: String,
        player_url: String,
    ) -> Result<String> {
        let cache_id = (format!("sig-{}", player_url), encrypted_signature.clone());

        if let Some(cached_deciphered_value) = self.player_cache.get(&cache_id)? {
            return Ok(cached_deciphered_value);
        }

        let extracted_signature = self
            .extract_signature_function(player_url, encrypted_signature)
            .await?;
        Ok(extracted_signature)
    }

    async fn decipher(&self, signature: String, player_url: String) -> Result<String> {
        let sc = parse_query_string(&signature).unwrap_or_default();

        let (Some(fmt_url), Some(encrypted_sig)) = (sc.get("url").cloned(), sc.get("s").cloned())
        else {
            bail!("The provided signature cannot be deciphered because it is missing `url`.")
        };

        let decrypted_signature = self.decrypt_signature(encrypted_sig, player_url).await?;
        let final_url = format!(
            "{}&{}={}",
            fmt_url,
            sc.get("sp").map(String::as_str).unwrap_or("signature"),
            decrypted_signature,
        );

        Ok(final_url)
    }
}
