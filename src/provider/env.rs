use std::env;

use crate::provider::{Provider, ProviderError};

pub struct EnvProvider;

impl Provider for EnvProvider {
    fn resolve(&self, key: &str) -> Result<String, ProviderError> {
        return env::var(key).map_err(|err| {
            ProviderError::VariableNotFound(format!("Variable {key} not found : {err}"))
        });
    }
}
