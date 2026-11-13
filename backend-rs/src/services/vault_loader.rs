#![allow(dead_code)]

use crate::config::Config;
use serde_json::Value;
use tracing::{debug, info, warn};

/// Pre-bootstrap secret resolution: authenticates against the nID Secrets
/// Vault (AppRole or a direct token) and returns Communicator's secrets so
/// they can be applied before any pool or client is built. Returns an empty
/// object when no vault credentials are configured.
pub async fn load_secrets_from_nid_vault(config: &Config) -> Value {
    let vault_addr = config.nid_vault_addr.trim_end_matches('/').to_string();
    let mut vault_token = config.nid_vault_token.clone();
    let role_id = config.nid_vault_role_id.clone();
    let secret_id = config.nid_vault_secret_id.clone();

    if vault_token.is_empty() && (role_id.is_empty() || secret_id.is_empty()) {
        debug!("nID Vault credentials not supplied; proceeding with environment configuration.");
        return Value::Object(Default::default());
    }

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        Ok(client) => client,
        Err(err) => {
            warn!("Could not build vault HTTP client: {}", err);
            return Value::Object(Default::default());
        }
    };

    if vault_token.is_empty() {
        let auth = client
            .post(format!("{vault_addr}/v1/auth/approle/login"))
            .json(&serde_json::json!({ "role_id": role_id, "secret_id": secret_id }))
            .send()
            .await;
        match auth {
            Err(err) => {
                warn!("nID Vault unreachable: {}", err);
                return Value::Object(Default::default());
            }
            Ok(response) => {
                if response.status().as_u16() != 200 {
                    warn!(
                        "nID Vault AppRole login rejected with status {}",
                        response.status().as_u16()
                    );
                    return Value::Object(Default::default());
                }
                match response.json::<Value>().await {
                    Ok(body) => {
                        vault_token = body
                            .pointer("/auth/client_token")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string();
                    }
                    Err(err) => {
                        warn!("nID Vault AppRole response was not readable: {}", err);
                        return Value::Object(Default::default());
                    }
                }
            }
        }
        if vault_token.is_empty() {
            warn!("nID Vault AppRole login returned no client token.");
            return Value::Object(Default::default());
        }
    }

    let read = client
        .get(format!("{vault_addr}/v1/secret/data/communicator"))
        .header("X-Vault-Token", &vault_token)
        .send()
        .await;
    match read {
        Err(err) => {
            warn!("Failed to fetch secrets from nID Vault: {}", err);
            Value::Object(Default::default())
        }
        Ok(response) => {
            let status = response.status().as_u16();
            if status != 200 {
                warn!("nID Vault secrets path returned status {}", status);
                return Value::Object(Default::default());
            }
            match response.json::<Value>().await {
                Ok(body) => {
                    let data = body
                        .pointer("/data/data")
                        .cloned()
                        .unwrap_or(Value::Object(Default::default()));
                    info!(
                        "Successfully loaded {} secrets from nID Vault.",
                        data.as_object().map(|map| map.len()).unwrap_or(0)
                    );
                    data
                }
                Err(err) => {
                    warn!("nID Vault secrets response was not readable: {}", err);
                    Value::Object(Default::default())
                }
            }
        }
    }
}

/// Applies resolved secrets to the live configuration and the process
/// environment, so components constructed afterwards pick them up.
pub fn apply_vault_secrets(config: &mut Config, secrets: &Value) {
    let Some(map) = secrets.as_object() else {
        return;
    };
    for (key, value) in map {
        if !config.apply_vault_secret(key, value) {
            debug!("Ignoring vault secret with no Rust consumer: {}", key);
        }
        std::env::set_var(key.to_ascii_uppercase(), value.to_string());
    }
}
