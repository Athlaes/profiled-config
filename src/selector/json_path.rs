use serde_json::Value;
use serde_json_path::JsonPath;

use crate::selector::{Selector, SelectorError};

pub struct JsonPathSelector;

impl Selector for JsonPathSelector {
    fn select(&self, json_str: &str, query: &str) -> Result<String, SelectorError> {
        let json: Value = serde_json::from_str(json_str).map_err(|err| SelectorError::FormatError {
            format: "json".to_string(),
            value: json_str.to_string(),
            source_str: err.to_string(),
        })?;
        let path = JsonPath::parse(query).map_err(|err| SelectorError::QueryFormatError {
            query: query.to_string(),
            selector: "jsonpath".to_string(),
            source_str: err.to_string(),
        })?;
        let selected = path
            .query(&json)
            .exactly_one()
            .map_err(|err| SelectorError::QueryError {
                query: query.to_string(),
                source_str: err.to_string(),
            })?;

        match selected {
            Value::String(str) => Ok(str.to_string()),
            value => Ok(value.to_string()),
        }
    }
}
