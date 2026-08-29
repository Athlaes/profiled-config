pub mod ast;

use thiserror::Error;

use crate::parser::ast::{ConfigExpression, ConfigValue, ConfigValueParts, SelectorExpression};

#[derive(Debug, Error)]
pub enum ExpressionParserError {
    #[error("Unexpected empty value with no default for provider '{provider}', key '{key}', and selector '{selector}'")]
    MissingDefaultValue {
        provider: String,
        key: String,
        selector: String,
    },
    #[error("Expression parser encountered an unexpected token: '{token}'")]
    UnexpectedToken { token: String },
    #[error("{0}")]
    EndOfExpressionNotFound(String),
}

pub struct ConfigValueParser<'a> {
    cfg_value: &'a str,
    position: usize,
    in_literal: bool,
}

impl<'a> ConfigValueParser<'a> {
    pub fn new(cfg_value: &'a str) -> Self {
        Self {
            cfg_value,
            position: 0,
            in_literal: false,
        }
    }

    fn peek(&self) -> Option<char> {
        self.cfg_value[self.position..].chars().next()
    }

    fn consume_until(&mut self, stop: Vec<&str>) -> String {
        let mut result = String::new();
        let mut delimiter = "".to_string();
        let mut escaped = false;
        while let Some(c) = self.peek() {
            if escaped {
                result.push(c);
                escaped = false;
                self.position += 1;
                continue;
            }
            if c == '\\' {
                escaped = true;
                self.position += 1;
                continue;
            }
            if c == '\'' || c == '\"' {
                self.in_literal = !self.in_literal;
            }
            delimiter.push(c);
            if stop.iter().any(|s| s.starts_with(delimiter.as_str())) && !self.in_literal {
                if stop.iter().any(|s| s.eq(&delimiter.as_str())) {
                    self.position += c.len_utf8();
                    self.position -= delimiter.len();
                    break;
                }
            } else {
                result.push_str(delimiter.as_str());
                delimiter = "".to_string();
            }
            self.position += c.len_utf8();
        }
        result
    }

    fn consume(&mut self, value: &str) -> bool {
        if self.cfg_value[self.position..].starts_with(value) {
            self.position += value.len();
            true
        } else {
            false
        }
    }

    pub fn parse_value(&mut self) -> Result<ConfigValue, ExpressionParserError> {
        let mut parts = vec![];
        while self.peek().is_some() {
            let literal = self.consume_until(vec!["${"]);
            if !literal.is_empty() {
                parts.push(ConfigValueParts::Literal(literal));
            }
            if let Some('$') = self.peek() {
                parts.push(ConfigValueParts::Expression(self.parse_expression()?));
            }
        }
        Ok(ConfigValue { parts })
    }

    fn parse_expression(&mut self) -> Result<ConfigExpression, ExpressionParserError> {
        if !self.consume("${") {
            return Err(ExpressionParserError::UnexpectedToken { token: "$".to_string() });
        }
        let provider = self.consume_until(vec![":"]);
        if !self.consume(":") {
            return Err(ExpressionParserError::EndOfExpressionNotFound(
                "Provider not found, expected ':'".to_string(),
            ));
        }
        let key = self.consume_until(vec![":", "(", "}"]);
        let selector = self.parse_selector()?;
        let default = self.parse_default();
        if !self.consume("}") {
            return Err(ExpressionParserError::EndOfExpressionNotFound(format!(
                "Missing end of expression '}}' at index {}",
                self.cfg_value[..self.position].chars().count()
            )));
        }
        Ok(ConfigExpression {
            key,
            provider,
            selector,
            default,
        })
    }

    fn parse_default(&mut self) -> Option<String> {
        if !self.consume(":") {
            return None;
        }
        let default = self.consume_until(vec!["}"]);
        Some(default)
    }

    fn parse_selector(&mut self) -> Result<Option<SelectorExpression>, ExpressionParserError> {
        if !self.consume("(") {
            return Ok(None);
        }
        let kind = self.consume_until(vec![":"]);
        if !self.consume(":") {
            return Err(ExpressionParserError::EndOfExpressionNotFound(
                "Selector kind not found, expected ':'".to_string(),
            ));
        }
        let query = self.consume_until(vec![")"]);
        if !self.consume(")") {
            return Err(ExpressionParserError::EndOfExpressionNotFound(format!(
                "Missing end of selector expression ')' at index {}",
                self.cfg_value[..self.position].chars().count()
            )));
        }
        Ok(Some(SelectorExpression { kind, query }))
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::ast::ConfigValue;

    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case("http://${env:ENV_VARIABLE}/test", vec!["${"], "http://")]
    #[case("http://${env:ENV_VARIABLE}/test", vec!["}"], "http://${env:ENV_VARIABLE")]
    #[case(
        "htt\"}\"p://${env:ENV_VARIABLE}/test",
        vec!["}"],
        "htt\"}\"p://${env:ENV_VARIABLE"
    )]
    #[case(
        "htt\\}p://${env:ENV_VARIABLE}/test",
        vec!["}"],
        "htt}p://${env:ENV_VARIABLE"
    )]
    fn consume_until_succes(#[case] tested_value: &str, #[case] separator: Vec<&str>, #[case] expected: &str) {
        let result = ConfigValueParser::new(tested_value).consume_until(separator);
        assert_eq!(expected, result)
    }

    #[rstest]
    #[case(
        "http://${env:HOSTNAME}:${env:PORT}/auth",
        ConfigValue {
            parts: vec![
                ConfigValueParts::Literal("http://".to_string()),
                ConfigValueParts::Expression(ConfigExpression { provider: "env".to_string(), key: "HOSTNAME".to_string(), selector: None, default: None }),
                ConfigValueParts::Literal(":".to_string()),
                ConfigValueParts::Expression(ConfigExpression { provider: "env".to_string(), key: "PORT".to_string(), selector: None, default: None }),
                ConfigValueParts::Literal("/auth".to_string()),
            ]
        }
    )]
    #[case(
        "${env:HOSTNAME:api.mycompany.com}",
        ConfigValue {
            parts: vec![
                ConfigValueParts::Expression(ConfigExpression { provider: "env".to_string(), key: "HOSTNAME".to_string(), selector: None, default: Some("api.mycompany.com".to_string()) }),
            ]
        }
    )]
    #[case(
        "${env:HOSTNAME(jsonpath:$.):api.mycompany.com}",
        ConfigValue {
            parts: vec![
                ConfigValueParts::Expression(ConfigExpression {
                    provider: "env".to_string(),
                    key: "HOSTNAME".to_string(),
                    selector: Some(SelectorExpression { kind: "jsonpath".to_string(), query: "$.".to_string() }),
                    default: Some("api.mycompany.com".to_string())
                }),
            ]
        }
    )]
    fn parse_value_success(#[case] tested_value: &str, #[case] expected_value: ConfigValue) {
        let result = ConfigValueParser::new(tested_value).parse_value().unwrap();
        assert_eq!(expected_value, result)
    }
}
