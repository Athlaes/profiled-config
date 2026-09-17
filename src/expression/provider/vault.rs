use serde::Deserialize;
use serde_value::Value;
use vaultrs::{
    auth::approle,
    client::{Client, VaultClient, VaultClientSettingsBuilder},
};

use crate::{
    ProviderFactoryFuture,
    expression::provider::{Provider, ProviderError, ResolveFuture},
};

#[derive(Deserialize)]
struct VaultConfig {
    url: String,
    mount: String,
    auth: VaultAuth,
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
    mount: String,
}

impl Provider for VaultProvider {
    fn create<'a>(value: &Value) -> ProviderFactoryFuture<'a> {
        let config = value
            .clone()
            .deserialize_into::<VaultConfig>()
            .map_err(|err| ProviderError::Init(format!("Invalid Vault configuration: {err}")));
        let future = async {
            let config = config?;
            let settings = VaultClientSettingsBuilder::default()
                .address(config.url)
                .build()
                .map_err(|err| ProviderError::Init(format!("Failed to build Vault client settings: {err}")))?;
            let mut client = VaultClient::new(settings)
                .map_err(|err| ProviderError::Init(format!("Failed to create Vault client: {err}")))?;
            let token = match config.auth {
                VaultAuth::AppRole {
                    mount,
                    role_id,
                    secret_id,
                } => {
                    approle::login(&client, &mount, &role_id, &secret_id)
                        .await
                        .map_err(|err| ProviderError::Init(format!("Vault AppRole authentication failed: {err}")))?
                        .client_token
                }
                VaultAuth::Token { token } => token,
            };
            client.set_token(&token);

            Ok(Box::new(Self {
                client,
                mount: config.mount,
            }) as Box<dyn Provider>)
        };
        Box::pin(future)
    }

    fn resolve<'a>(&'a self, key: &'a str) -> ResolveFuture<'a> {
        Box::pin(async move {
            let fail = |cause_str: String| ProviderError::VariableNotFound {
                key: key.to_owned(),
                cause_str,
            };

            let (path, field) = key
                .rsplit_once('/')
                .filter(|(path, field)| !path.is_empty() && !field.is_empty())
                .ok_or_else(|| fail("Invalid Vault key format: expected path/to/secret/field".into()))?;

            let secret: serde_json::Value =
                vaultrs::kv2::read(&self.client, &self.mount, path)
                    .await
                    .map_err(|err| {
                        fail(format!(
                            "Failed to read Vault secret '{path}' from mount '{}': {err}",
                            self.mount
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

    fn key() -> String
    where
        Self: Sized,
    {
        "vault".to_string()
    }
}
