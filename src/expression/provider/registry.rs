use std::collections::HashMap;

use serde_value::Value;

use crate::expression::{
    ExpressionResolver,
    provider::{Provider, ProviderActivation, ProviderError, ProviderFactoryClosure},
};

pub struct ProviderRegistration<'a> {
    pub factory: ProviderFactoryClosure<'a>,
    pub activation: ProviderActivation,
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

    pub async fn register_provider(
        &mut self,
        config_values: &Value,
        key: &str,
        registration: &ProviderRegistration<'_>,
    ) -> Result<(), ProviderError> {
        if self.providers.contains_key(key) {
            return Err(ProviderError::ProviderKeyAlreadyDefined { key: key.to_owned() });
        }
        if let Value::Map(value) = config_values
            && let Some(Value::Map(value)) = value.get(&Value::String("profiled_config".to_string()))
            && let Some(Value::Map(value)) = value.get(&Value::String("providers".to_string()))
            && let Some(value) = value.get(&Value::String(key.to_string()))
        {
            let resolver = ExpressionResolver::new(self);
            let resolved_value = resolver
                .process(value)
                .await
                .map_err(|err| ProviderError::Resolve { causes: err })?;
            self.providers
                .insert(key.to_owned(), (registration.factory)(&resolved_value).await?);
            Ok(())
        } else {
            match registration.activation {
                ProviderActivation::Always => {
                    self.providers
                        .insert(key.to_owned(), (registration.factory)(&Value::Option(None)).await?);
                    Ok(())
                }
                ProviderActivation::WhenConfigured => Ok(()),
            }
        }
    }

    pub fn get_provider(&self, key: &str) -> Result<&dyn Provider, ProviderError> {
        self.providers
            .get(key)
            .map(|p| p.as_ref())
            .ok_or(ProviderError::ProviderNotFound { key: key.to_string() })
    }

    pub async fn get_overrides(&self) -> Result<Vec<String>, ProviderError> {
        let mut overrides = vec![];
        for (_, provider) in &self.providers {
            overrides.extend(provider.get_overrides().await?);
        }
        Ok(overrides)
    }
}
