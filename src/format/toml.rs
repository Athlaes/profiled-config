use serde_value::Value;

use crate::source::LoaderError;

pub(super) fn parse(content: &str) -> Result<Value, LoaderError> {
    toml::from_str(content).map_err(|error| LoaderError::ParseError(format!("Couldn't parse file : {error}")))
}
