use std::env;

use super::{Provider, ProviderError};

pub struct EnvProvider;

impl Provider for EnvProvider {
    fn resolve(&self, key: &str) -> Result<String, ProviderError> {
        env::var(key).map_err(|err| ProviderError::VariableNotFound {
            key: key.to_string(),
            cause_str: err.to_string(),
        })
    }
}
