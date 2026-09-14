use std::{
    collections::{
        HashMap,
        hash_map::Entry::{Occupied, Vacant},
    },
    pin::Pin,
};

use serde::de::DeserializeOwned;
use serde_value::Value;
use thiserror::Error;

mod env;
mod vault;

pub use env::EnvProvider;
#[cfg(feature = "vault")]
pub use vault::VaultProvider;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Variable {key} couldn't be resolved : {cause_str}")]
    VariableNotFound { key: String, cause_str: String },
    #[error("Provider '{key}' not supported or feature is not enabled")]
    ProviderNotFound { key: String },
    #[error("Provider key '{key}' is already defined")]
    ProviderKeyAlreadyDefined { key: String },
    #[error("Provider initialization failed: {0}")]
    Init(String),
}

type ResolveFuture<'a> = Pin<Box<dyn Future<Output = Result<String, ProviderError>> + 'a>>;

pub type ProviderFactory<C: DeserializeOwned> =
    Box<dyn Fn(C) -> Pin<Box<dyn Future<Output = Result<Box<dyn Provider>, ProviderError>>>>>;

pub trait Provider: Send + Sync {
    fn resolve<'a>(&'a self, key: &'a str) -> ResolveFuture<'a>;
}

pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn Provider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub async fn register_provider<C>(
        &mut self,
        config_values: &Value,
        key: &str,
        provider_factory: ProviderFactory<C>,
    ) -> Result<(), ProviderError>
    where
        C: DeserializeOwned,
    {
        let config: C = C::deserialize(config_values.clone()).map_err(|e| ProviderError::VariableNotFound {
            key: key.to_string(),
            cause_str: e.to_string(),
        })?;
        let provider = provider_factory(config).await?;
        match self.providers.entry(key.to_string()) {
            Occupied(_) => Err(ProviderError::ProviderKeyAlreadyDefined { key: key.to_string() }),
            Vacant(e) => {
                e.insert(provider);
                Ok(())
            }
        }
    }

    pub fn get_provider(&self, key: &str) -> Result<&dyn Provider, ProviderError> {
        self.providers
            .get(key)
            .map(|p| p.as_ref())
            .ok_or(ProviderError::ProviderNotFound { key: key.to_string() })
    }
}
