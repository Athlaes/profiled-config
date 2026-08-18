use std::fmt::Display;

use crate::provider::ProviderError::VariableNotFound;

mod env;

#[derive(Debug)]
pub enum ProviderError {
    VariableNotFound(String),
}

impl Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableNotFound(str) => write!(f, "{}", str),
        }
    }
}

pub trait Provider {
    fn resolve(&self, key: &str) -> Result<String, ProviderError>;
}

pub fn get_provider(key: &str) -> impl Provider {
    match key {
        "env" => env::EnvProvider,
        _ => panic!("Provider '{key}' not supported or feature is not enabled"),
    }
}
