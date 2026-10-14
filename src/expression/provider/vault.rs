use std::collections::BTreeMap;

use serde::Deserialize;
use serde_value::Value;
use vaultrs::{
    auth::approle,
    client::{Client, VaultClient, VaultClientSettingsBuilder},
    kv2,
};

use crate::{
    OverridesFuture, ProviderFactoryFuture,
    expression::provider::{Provider, ProviderActivation, ProviderError, ResolveFuture},
};

#[derive(Deserialize)]
struct VaultConfig {
    url: String,
    mount: String,
    auth: VaultAuth,
    #[serde(default)]
    overrides_paths: Vec<String>,
}

#[derive(Deserialize)]
enum VaultAuth {
    Token {
        token: String,
    },
    AppRole {
        mount: String,
        role_id: String,
        secret_id: String,
    },
}

pub struct VaultProvider {
    client: VaultClient,
    config: VaultConfig,
}

impl Provider for VaultProvider {
    fn key() -> String {
        "vault".to_string()
    }

    fn create<'a>(value: &Value) -> ProviderFactoryFuture<'a> {
        let config = value
            .clone()
            .deserialize_into::<VaultConfig>()
            .map_err(|err| ProviderError::Init(format!("Invalid Vault configuration: {err}")));
        let future = async {
            let config = config?;
            let settings = VaultClientSettingsBuilder::default()
                .address(&config.url)
                .build()
                .map_err(|err| ProviderError::Init(format!("Failed to build Vault client settings: {err}")))?;
            let mut client = VaultClient::new(settings)
                .map_err(|err| ProviderError::Init(format!("Failed to create Vault client: {err}")))?;
            let token = match &config.auth {
                VaultAuth::AppRole {
                    mount,
                    role_id,
                    secret_id,
                } => {
                    approle::login(&client, mount, role_id, secret_id)
                        .await
                        .map_err(|err| ProviderError::Init(format!("Vault AppRole authentication failed: {err}")))?
                        .client_token
                }
                VaultAuth::Token { token } => token.to_string(),
            };
            client.set_token(&token);

            Ok(Box::new(Self { client, config }) as Box<dyn Provider>)
        };
        Box::pin(future)
    }

    fn resolve<'a>(&'a self, key: &'a str) -> ResolveFuture<'a> {
        Box::pin(async {
            let fail = |cause_str: String| ProviderError::VariableNotFound {
                key: key.to_owned(),
                cause_str,
            };

            let (path, field) = key
                .rsplit_once('/')
                .filter(|(path, field)| !path.is_empty() && !field.is_empty())
                .ok_or_else(|| fail("Invalid Vault key format: expected path/to/secret/field".into()))?;

            let secret: serde_json::Value = vaultrs::kv2::read(&self.client, &self.config.mount, path)
                .await
                .map_err(|err| {
                    fail(format!(
                        "Failed to read Vault secret '{path}' from mount '{}': {err}",
                        self.config.mount
                    ))
                })?;

            let value = secret
                .get(field)
                .ok_or_else(|| fail(format!("Field '{field}' not found in Vault secret '{path}'")))?;

            match value {
                serde_json::Value::String(text) => Ok(text.clone()),
                other => Ok(other.to_string()),
            }
        })
    }

    fn activation() -> super::ProviderActivation
    where
        Self: Sized,
    {
        ProviderActivation::WhenConfigured
    }

    fn get_overrides<'a>(&'a self) -> OverridesFuture<'a> {
        Box::pin(async {
            let mut overrides = Vec::new();
            for path in &self.config.overrides_paths {
                let values = kv2::read::<BTreeMap<String, String>>(&self.client, &self.config.mount, path)
                    .await
                    .map_err(|err| {
                        ProviderError::Init(format!(
                            "Failed to read Vault secret '{path}' from mount '{}': {err}",
                            self.config.mount
                        ))
                    })?;
                overrides.extend(values.into_iter().map(|(key, value)| format!("{key}={value}")));
            }
            Ok(overrides)
        })
    }
}
