use std::env;

use crate::provider::{Provider, ProviderError};

pub struct EnvProvider;

impl Provider for EnvProvider {
    fn resolve(&self, key: &str) -> Result<String, ProviderError> {
        Ok(env::var(key).map_err(|err| ProviderError::VariableNotFound {
            key: key.to_string(),
            source_str: err.to_string(),
        })?)
    }
}
