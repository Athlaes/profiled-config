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
                "Unexpected empty value with no default for key {}",
                self.key
            )
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct SelectorExpression {
    pub kind: String,
    pub query: String,
}
