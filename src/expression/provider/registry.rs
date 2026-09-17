use std::{
    collections::{
        HashMap,
        hash_map::Entry::{Occupied, Vacant},
    },
    pin::Pin,
};

use serde_value::Value;

use crate::expression::provider::{Provider, ProviderError, ProviderFactoryClosure};

pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn Provider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub async fn register_provider(
        &mut self,
        config_values: &Value,
        key: &str,
        provider_factory: ProviderFactoryClosure<'_>,
    ) -> Result<(), ProviderError> {
        let mut config = config_values;
        if let Value::Map(value) = config_values
            && let Some(Value::Map(value)) = value.get(&Value::String("profiled_config".to_string()))
            && let Some(Value::Map(value)) = value.get(&Value::String("providers".to_string()))
            && let Some(value) = value.get(&Value::String(key.to_string()))
        {
            config = value;
        }
        match self.providers.entry(key.to_string()) {
            Occupied(_) => Err(ProviderError::ProviderKeyAlreadyDefined { key: key.to_string() }),
            Vacant(e) => {
                e.insert(provider_factory(config).await?);
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
