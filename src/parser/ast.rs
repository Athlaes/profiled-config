use std::fmt::Display;

use crate::parser::ExpressionParserError;

#[derive(Debug, PartialEq)]
pub struct ConfigValue {
    pub parts: Vec<ConfigValueParts>,
}

#[derive(Debug, PartialEq)]
pub enum ConfigValueParts {
    Literal(String),
    Expression(ConfigExpression),
}

#[derive(Debug, PartialEq)]
pub struct ConfigExpression {
    pub provider: String,
    pub key: String,
    pub selector: Option<SelectorExpression>,
    pub default: Option<String>,
}

impl ConfigExpression {
    pub fn get_default(&self) -> Result<String, ExpressionParserError> {
        self.default.clone().ok_or(ExpressionParserError::MissingDefaultValue {
            provider: self.provider.clone(),
            key: self.key.clone(),
            selector: self
                .selector
                .clone()
                .and_then(|s| Some(s.kind))
                .unwrap_or(String::new()),
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct SelectorExpression {
    pub kind: String,
    pub query: String,
}

impl Display for SelectorExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SelectorExpression ({}: {})", self.kind, self.query)
    }
}
