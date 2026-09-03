use thiserror::Error;

use self::json_path::JsonPathSelector;

pub mod json_path;

#[derive(Debug, Error)]
pub enum SelectorError {
    #[error("Selector '{kind}' is not supported or feature is not enabled")]
    SelectorNotFound { kind: String },
    #[error("Some value is not valid for format {format} : {source_str}")]
    FormatError { format: String, source_str: String },
    #[error("Query {query} couldn't be parsed by selector {selector} : {source_str}")]
    QueryFormatError {
        query: String,
        selector: String,
        source_str: String,
    },
    #[error("Query {query} failed with error : {source_str}")]
    QueryError { query: String, source_str: String },
}

// TODO : Authorize Result<&Value, SelectorError> ?
pub trait Selector {
    fn select(&self, format_value: &str, query: &str) -> Result<String, SelectorError>;
}

pub fn get_selector(kind: &str) -> Result<impl Selector, SelectorError> {
    match kind {
        "jsonpath" => Ok(JsonPathSelector),
        _ => Err(SelectorError::SelectorNotFound { kind: kind.to_string() }),
    }
}
