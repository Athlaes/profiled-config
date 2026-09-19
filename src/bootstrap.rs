use std::collections::HashMap;

use serde_value::Value;
use thiserror::Error;

use crate::expression::provider::{
    EnvProvider, Provider, ProviderError,
    registry::{ProviderRegistration, ProviderRegistry},
};

#[derive(Debug, Error)]
pub enum BootstrapError {
    #[error("Couldn't bootstrap config : {0}")]
    ProviderConfiguration(#[from] ProviderError),
}

pub async fn configure_providers<'a>(
    merged_values: &Value,
    additional_providers: HashMap<String, ProviderRegistration<'a>>,
) -> Result<ProviderRegistry, BootstrapError> {
    let mut pr = ProviderRegistry::new();
    pr.register_provider(
        merged_values,
        "env",
        &ProviderRegistration {
            factory: Box::new(EnvProvider::create),
            activation: EnvProvider::activation(),
        },
    )
    .await?;
    #[cfg(feature = "vault")]
    pr.register_provider(
        &merged_values,
        "vault",
        &ProviderRegistration {
            factory: Box::new(crate::expression::provider::vault::VaultProvider::create),
            activation: crate::expression::provider::ProviderActivation::WhenConfigured,
        },
    )
    .await?;
    for (key, registration) in additional_providers.into_iter() {
        pr.register_provider(merged_values, &key, &registration).await?;
    }
    Ok(pr)
}
