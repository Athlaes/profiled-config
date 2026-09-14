use std::collections::HashMap;

use serde::Deserialize;
use serde_value::Value;
use thiserror::Error;

use crate::{
    Provider,
    expression::provider::{EnvProvider, ProviderError, ProviderFactory, ProviderRegistry},
};

#[derive(Debug, Deserialize)]
struct BootstrapConfig {
    #[cfg(feature = "vault")]
    vault: Option<VaultConfig>,
}

#[cfg(feature = "vault")]
#[derive(Debug, Deserialize)]
struct VaultConfig {}

#[derive(Debug, Error)]
pub enum BootstrapError {
    #[error("Couldn't bootstrap config : {0}")]
    ProviderConfiguration(#[from] ProviderError),
}

pub async fn configure_providers(
    merged_values: &Value,
    additional_providers: HashMap<String, ProviderFactory<C>>,
) -> Result<ProviderRegistry, BootstrapError> {
    let mut pr = ProviderRegistry::new();
    pr.register_provider(merged_values, "env", Box::new(EnvProvider::create))
        .await?;
    #[cfg(feature = "vault")]
    pr.register_provider("vault", Box::new(VaultProvider::create)).await?;
    for (key, provider) in additional_providers.into_iter() {
        pr.register_provider(merged_values, &key, provider).await?;
    }
    Ok(pr)
}
