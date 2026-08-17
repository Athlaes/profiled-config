use std::fmt::Display;

use crate::selector::{
    SelectorError::{ParsingError, SelectionError},
    json_path::JsonPathSelector,
};

pub mod json_path;

pub enum SelectorError {
    ParsingError(String),
    SelectionError(String),
}

impl Display for SelectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingError(str) => write!(f, "{}", &str),
            SelectionError(str) => write!(f, "{}", &str),
        }
    }
}

pub trait Selector {
    fn select(&self, json_str: &str, query: &str) -> Result<String, SelectorError>;
}

pub fn get_selector(kind: &str) -> impl Selector {
    match kind {
        "jsonpath" => JsonPathSelector,
        _ => {
            panic!("Provider '{kind}' not supported or feature is not enabled")
        }
    }
}
