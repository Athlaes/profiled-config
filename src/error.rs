use serde_value::DeserializerError;
use thiserror::Error;

use crate::expression::ResolveError;

#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("Couldn't open current folder : '{source_str}'")]
    CurrentFolderNotReadable { source_str: String },
    #[error("File '{file_name}' not found")]
    FileNotFound { file_name: String },
    #[error("File extension not found for file '{file_name}'")]
    FileExtensionNotFound { file_name: String },
    #[error("Couldn't open file '{file_name}' : '{cause_str}'")]
    FileNotReadable { file_name: String, cause_str: String },
    #[error("Multiple file '{file_name}' found")]
    MultipleFileFound { file_name: String },
    #[error("File '{file_name}' has no content or is not valid UTF-8")]
    NoContent { file_name: String },
    #[error("{0}")]
    ParseError(String),
    #[error("Found file with ext '{ext}' which is not supported or feature is not enabled")]
    NotSupportedExtension { ext: String },
}

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
