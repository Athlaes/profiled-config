use std::collections::HashMap;

use serde_value::Value;
use thiserror::Error;

use crate::expression::provider::{
    EnvProvider, Provider, ProviderError, ProviderFactoryClosure, registry::ProviderRegistry,
};

#[derive(Debug, Error)]
pub enum BootstrapError {
    #[error("Couldn't bootstrap config : {0}")]
    ProviderConfiguration(#[from] ProviderError),
}

pub async fn configure_providers<'a>(
    merged_values: &Value,
    additional_providers: HashMap<String, ProviderFactoryClosure<'a>>,
) -> Result<ProviderRegistry, BootstrapError> {
    let mut pr = ProviderRegistry::new();
    pr.register_provider(merged_values, "env", Box::new(EnvProvider::create))
        .await?;
    #[cfg(feature = "vault")]
    pr.register_provider(
        merged_values,
        "vault",
        Box::new(crate::expression::provider::vault::VaultProvider::create),
    )
    .await?;
    for (key, provider) in additional_providers.into_iter() {
        pr.register_provider(merged_values, &key, provider).await?;
    }
    Ok(pr)
}
