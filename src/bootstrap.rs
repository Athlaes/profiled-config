use std::collections::HashMap;

use serde::Deserialize;
use serde_value::Value;
use thiserror::Error;

use crate::{
    Provider,
    expression::provider::{EnvProvider, ProviderError, ProviderRegistry},
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

pub fn configure_providers(
    merged_values: &Value,
    additional_providers: HashMap<String, Box<dyn Provider>>,
) -> Result<ProviderRegistry, BootstrapError> {
    let mut pr = ProviderRegistry::new();
    pr.register_provider("env", Box::new(EnvProvider))?;
    #[cfg(feature = "vault")]
    pr.register_provider("env", Box::new(expression::provider::VaultProvider))?;
    for (key, provider) in additional_providers.into_iter() {
        pr.register_provider(&key, provider)?;
    }
    Ok(pr)
}
