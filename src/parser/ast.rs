pub struct ConfigValue {
    pub parts: Vec<ConfigValueParts>,
}

pub enum ConfigValueParts {
    Literal(String),
    Expression(ConfigExpression),
}

pub struct ConfigExpression {
    pub provider: String,
    pub key: String,
    pub selector: Option<SelectorExpression>,
    pub default: Option<String>,
}

pub struct SelectorExpression {
    pub kind: String,
    pub query: String,
}
