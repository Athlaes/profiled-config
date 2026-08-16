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

#[derive(Debug, PartialEq)]
pub struct SelectorExpression {
    pub kind: String,
    pub query: String,
}
