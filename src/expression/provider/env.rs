use serde_value::Value;
use std::env;

use crate::{
    ProviderFactoryFuture,
    expression::provider::{Provider, ProviderActivation, ResolveFuture},
};

use super::ProviderError;

pub struct EnvProvider;

impl EnvProvider {}

impl Provider for EnvProvider {
    fn key() -> String {
        "env".to_string()
    }

    fn create<'a>(_: &Value) -> ProviderFactoryFuture<'a> {
        Box::pin(async { Ok(Box::new(Self) as Box<dyn Provider>) })
    }

    fn resolve<'a>(&'a self, key: &'a str) -> ResolveFuture<'a> {
        Box::pin(async move {
            env::var(key).map_err(|err| ProviderError::VariableNotFound {
                key: key.to_string(),
                cause_str: err.to_string(),
            })
        })
    }

    fn activation() -> ProviderActivation
    where
        Self: Sized,
    {
        ProviderActivation::Always
    }
}
