use thiserror::Error;

use crate::{
    parser::{ConfigValueParser, ExpressionParserError, ast::ConfigValueParts},
    provider::{self, Provider, ProviderError},
    selector::{self, Selector, SelectorError},
};

#[derive(Debug, Error)]
pub enum ResolverError {
    #[error("Selection error: {0}")]
    SelectionError(#[from] SelectorError),
    #[error("Expression parse error: {0}")]
    ExpressionParseError(#[from] ExpressionParserError),
    #[error("Provide error: {0}")]
    ProvideError(#[from] ProviderError),
}

pub fn resolve(value: &str) -> Result<String, ResolverError> {
    let mut result = String::new();
    let value = ConfigValueParser::new(value).parse_value()?;
    for part in &value.parts {
        match &part {
            ConfigValueParts::Literal(str) => {
                result.push_str(str);
            }
            ConfigValueParts::Expression(exp) => {
                let provider = provider::get_provider(&exp.provider)?;
                let mut value = provider.resolve(&exp.key)?;
                if let Some(selector) = &exp.selector {
                    let selected_selector = selector::get_selector(selector.kind.as_str())?;
                    value = selected_selector.select(&value, &selector.query)?;
                }
                if value.is_empty() {
                    result.push_str(&exp.get_default()?);
                } else {
                    result.push_str(&value);
                }
            }
        }
    }
    Ok(result)
}
