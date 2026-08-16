use std::fmt::Display;

use crate::{
    ConfigError::ParseError,
    parser::ast::{ConfigExpression, ConfigValue, ConfigValueParts, SelectorExpression},
};

mod ast;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(&'static str),
    EndOfExpressionNotFound(&'static str),
    ProviderNotFound(&'static str),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken(msg) => write!(f, "{}", msg),
            ParseError::EndOfExpressionNotFound(msg) => write!(f, "{}", msg),
            ParseError::ProviderNotFound(msg) => write!(f, "{}", msg),
        }
    }
}

struct ConfigValueParser<'a> {
    cfg_value: &'a str,
    position: usize,
    in_literal: bool,
}

impl<'a> ConfigValueParser<'a> {
    fn new(cfg_value: &'a str) -> Self {
        Self {
            cfg_value,
            position: 0,
            in_literal: false,
        }
    }

    fn peek(&self) -> Option<char> {
        self.cfg_value.chars().nth(self.position)
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
            if stop.iter().any(|s| s.starts_with(&delimiter.as_str())) && !self.in_literal {
                if stop.iter().any(|s| s.eq(&delimiter.as_str())) {
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
        if self.cfg_value.starts_with(value) {
            self.position += value.len();
            true
        } else {
            false
        }
    }

    pub fn parse_value(&mut self) -> Result<ConfigValue, ParseError> {
        let mut parts = vec![];
        while self.peek().is_some() {
            let literal = self.consume_until(vec!["$"]);
            parts.push(ConfigValueParts::Literal(literal));
            if let Some('$') = self.peek() {
                parts.push(ConfigValueParts::Expression(self.parse_expression()?));
            }
        }
        Ok(ConfigValue { parts })
    }

    fn parse_expression(&mut self) -> Result<ConfigExpression, ParseError> {
        if !self.consume("${") {
            return Err(ParseError::UnexpectedToken(
                "Unexpected expression beginning token '$'",
            ));
        }
        let provider = self.consume_until(vec![":"]);
        if !self.consume(":") {
            return Err(ParseError::EndOfExpressionNotFound(
                "Provider not found, expected ':'",
            ));
        }
        let key = self.consume_until(vec![":", "(", "}"]);
        let selector = self.parse_selector()?;
        let default = self.parse_default();
        if !self.consume("}") {
            return Err(ParseError::EndOfExpressionNotFound(
                "Missing end of expression '}'",
            ));
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

    fn parse_selector(&mut self) -> Result<Option<SelectorExpression>, ParseError> {
        if !self.consume("(") {
            return Ok(None);
        }
        let kind = self.consume_until(vec![":"]);
        if !self.consume(":") {
            return Err(ParseError::EndOfExpressionNotFound(
                "Selector kind not found, expected ':'",
            ));
        }
        let query = self.consume_until(vec![")"]);
        if !self.consume(")") {
            return Err(ParseError::EndOfExpressionNotFound(
                "Missing end of selector expression ')'",
            ));
        }
        Ok(Some(SelectorExpression { kind, query }))
    }
}

#[cfg(test)]
mod tests {
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
    fn consume_until_succes(
        #[case] tested_value: &str,
        #[case] separator: Vec<&str>,
        #[case] expected: &str,
    ) {
        let result = ConfigValueParser::new(tested_value).consume_until(separator);
        assert_eq!(expected, result)
    }
}
