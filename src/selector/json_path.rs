use serde_json::Value;
use serde_json_path::JsonPath;

use crate::selector::{Selector, SelectorError};

pub struct JsonPathSelector;

impl Selector for JsonPathSelector {
    fn select(&self, json_str: &str, query: &str) -> Result<String, SelectorError> {
        let json: Value = serde_json::from_str(json_str).map_err(|err| {
            SelectorError::ParsingError(format!("Couldn't parse json value : {}", &err))
        })?;
        let path = JsonPath::parse(query).map_err(|err| {
            SelectorError::ParsingError(format!("Couldn't parse jsonpath query : {}", &err))
        })?;
        let selected = path.query(&json).exactly_one().map_err(|err| {
            SelectorError::SelectionError(format!(
                "Error occured during json_path value selection : {}",
                &err
            ))
        })?;

        match selected {
            Value::String(str) => Ok(str.to_string()),
            value => Ok(value.to_string()),
        }
    }
}
