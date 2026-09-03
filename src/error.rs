use serde_value::DeserializerError;
use thiserror::Error;

use crate::{expression::ResolveError, source::LoaderError};

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to load configuration : {0}")]
    Loading(#[from] LoaderError),
    #[error("Failed to resolve configuration :\n\n{}", format_error(causes))]
    Resolve { causes: Vec<ResolveError> },
    #[error("Failed to deserialize configuration : {cause}")]
    Deserialize {
        #[source]
        cause: DeserializerError,
    },
}

fn format_error(causes: &[ResolveError]) -> String {
    let mut formatted = String::new();
    for error in causes {
        formatted.push_str(format!("path: {} error: {}\n", error.path, error.cause).as_str());
    }
    formatted
}
