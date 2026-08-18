use std::fmt::Display;

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
    pub fn get_default_or_panic(&self) -> String {
        self.default.clone().unwrap_or_else(|| {
            panic!(
                "Unexpected empty value with no default for key {} and selector {}",
                self.key,
                self.selector.clone().unwrap_or(SelectorExpression {
                    kind: String::new(),
                    query: String::new()
                })
            )
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
