use std::fmt::Display;

mod env;

#[derive(Debug)]
pub enum ProviderError {}

impl Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            _ => write!(f, "{}", ""),
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
