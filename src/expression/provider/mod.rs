use serde_value::Value;
use std::pin::Pin;
use thiserror::Error;

mod env;
pub mod registry;

#[cfg(feature = "vault")]
pub mod vault;

pub use env::EnvProvider;

use crate::{error::format_error, expression::ResolveError};

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
    #[error("Failed to resolve configuration :\n\n{}", format_error(causes))]
    Resolve { causes: Vec<ResolveError> },
}

pub type ResolveFuture<'a> = Pin<Box<dyn Future<Output = Result<String, ProviderError>> + 'a>>;

pub type OverridesFuture<'a> = Pin<Box<dyn Future<Output = Result<Vec<String>, ProviderError>> + 'a>>;

pub type ProviderFactoryFuture<'a> = Pin<Box<dyn Future<Output = Result<Box<dyn Provider>, ProviderError>> + 'a>>;

pub type ProviderFactoryClosure<'a> = Box<dyn Fn(&Value) -> ProviderFactoryFuture<'a>>;

pub enum ProviderActivation {
    Always,
    WhenConfigured,
}

pub trait Provider {
    fn key() -> String
    where
        Self: Sized;

    fn activation() -> ProviderActivation
    where
        Self: Sized;

    fn create<'a>(values: &Value) -> ProviderFactoryFuture<'a>
    where
        Self: Sized;

    fn resolve<'a>(&'a self, key: &'a str) -> ResolveFuture<'a>;

    fn get_overrides<'a>(&'a self) -> OverridesFuture<'a> {
        let future = async { Ok(vec![]) };
        Box::pin(future)
    }
}
