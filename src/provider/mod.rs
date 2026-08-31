use thiserror::Error;

mod env;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Variable {key} couldn't be resolved : {source_str}")]
    VariableNotFound { key: String, source_str: String },
    #[error("Provider '{key}' not supported or feature is not enabled")]
    ProviderNotFound { key: String },
}

pub trait Provider {
    fn resolve(&self, key: &str) -> Result<String, ProviderError>;
}

pub fn get_provider(key: &str) -> Result<impl Provider, ProviderError> {
    match key {
        "env" => Ok(env::EnvProvider),
        _ => Err(ProviderError::ProviderNotFound { key: key.to_string() }),
    }
}
