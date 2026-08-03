use std::{env, fmt::Display};

use crate::{
    parser::ast::{ConfigExpression, ConfigValueParts},
    provider::PROVIDERS,
};

mod ast;

pub enum ParseError {
    ProviderNotFound(String),
    EndOfExpressionNotFound(String),
    EmptyExpression(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::ProviderNotFound(msg) => write!(f, "{}", msg),
            ParseError::EndOfExpressionNotFound(msg) => write!(f, "{}", msg),
            ParseError::EmptyExpression(msg) => write!(f, "{}", msg),
        }
    }
}

pub fn parse_config_value(cfg_value: &str) -> Result<ast::ConfigValue, ParseError> {
    let mut parts: Vec<ConfigValueParts> = Vec::new();
    while let Some(index) = cfg_value.find('$') {
        let (left, right) = cfg_value.split_at(index);
        if !left.is_empty() {
            parts.push(ConfigValueParts::Literal(left.to_string()));
        }
        let (left, right) = right.split_at(
            right
                .find('}')
                .ok_or(ParseError::EndOfExpressionNotFound(format!(
                    "No closing '}}' found in expression: {}",
                    right
                )))?
                + 1,
        );
        if !left.is_empty() {
            parts.push(ConfigValueParts::Expression(parse_expression(left)?));
        } else {
            return Err(ParseError::EmptyExpression(
                "Empty expression not authorized".to_string(),
            ));
        }
        cfg_value = right;
    }
    if !cfg_value.is_empty() {
        parts.push(ConfigValueParts::Literal(cfg_value.to_string()));
    }
    Ok(ast::ConfigValue { parts })
}

fn parse_expression(expr: &str) -> Result<ast::ConfigExpression, ParseError> {
    let mut exp = expr;
    exp = exp.strip_prefix("${").unwrap_or(exp);
    exp = exp.strip_suffix('}').unwrap_or(exp);
    let (provider, right) = exp
        .split_once(':')
        .ok_or(ParseError::ProviderNotFound(format!(
            "Provider not found in config expression, must be one of '{}'",
            PROVIDERS.join("', '")
        )))?;
    let (value, default) = match right.split_once(':') {
        Some((value, default)) => (value, Some(default)),
        None => (right, None),
    };

    if value.is_empty() {
        return Err(ParseError::EmptyExpression(
            "Empty expression not authorized".to_string(),
        ));
    }

    Ok(ConfigExpression {
        provider: provider.to_string(),
        key: right.to_string(),
        selector: None,
        default: default.map(|d| d.to_string()),
    })
}

fn parse_selector(selector: &str) -> ast::SelectorExpression {
    env::var("key").unwrap_or("default".to_string());
    ast::SelectorExpression {
        kind: String::new(),
        query: String::new(),
    }
}
