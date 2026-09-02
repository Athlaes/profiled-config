use thiserror::Error;

use crate::{
    parser::{ConfigValueParser, ExpressionParserError, ast::ConfigValueParts},
    provider::{self, Provider, ProviderError},
    selector::{self, Selector, SelectorError},
};

#[derive(Debug, Error)]
pub enum ResolverError {
    #[error("Selection error: {0}")]
    Selection(#[from] SelectorError),
    #[error("Expression parse error: {0}")]
    ExpressionParse(#[from] ExpressionParserError),
    #[error("Provide error: {0}")]
    Provide(#[from] ProviderError),
    #[error("Unexpected empty value with no default for provider '{provider}', key '{key}'")]
    MissingDefaultValue { provider: String, key: String },
}

pub fn resolve(initial_value: &str) -> Result<String, ResolverError> {
    let mut result = String::new();
    let parsed_value = ConfigValueParser::new(initial_value).parse_value()?;
    for part in &parsed_value.parts {
        match &part {
            ConfigValueParts::Literal(str) => {
                result.push_str(str);
            }
            ConfigValueParts::Expression(exp) => {
                let provider = provider::get_provider(&exp.provider)?;
                let expr_value = provider.resolve(&exp.key);
                match expr_value {
                    Ok(value) => {
                        let mut tmp_val = value.clone();
                        if let Some(selector) = &exp.selector {
                            let selected_selector = selector::get_selector(selector.kind.as_str())?;
                            tmp_val = selected_selector.select(&value, &selector.query)?;
                        }
                        if tmp_val.is_empty() {
                            result.push_str(&exp.default.clone().ok_or(ResolverError::MissingDefaultValue {
                                provider: exp.provider.clone(),
                                key: exp.key.clone(),
                            })?);
                        } else {
                            result.push_str(&tmp_val);
                        }
                    }
                    Err(err) => {
                        result.push_str(&exp.default.clone().ok_or(err)?);
                    }
                }
            }
        }
    }
    Ok(result)
}
