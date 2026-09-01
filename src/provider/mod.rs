use thiserror::Error;

mod env;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Variable {key} couldn't be resolved : {cause_str}")]
    VariableNotFound { key: String, cause_str: String },
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
