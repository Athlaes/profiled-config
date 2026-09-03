use serde_value::Value;

use crate::source::LoaderError;

#[cfg(feature = "ini")]
mod ini;
mod json;
#[cfg(feature = "toml")]
mod toml;
#[cfg(feature = "yaml")]
mod yaml;

#[cfg(feature = "ini")]
use ini::parse as parse_ini;
use json::parse as parse_json;
#[cfg(feature = "toml")]
use toml::parse as parse_toml;
#[cfg(feature = "yaml")]
use yaml::parse as parse_yaml;

type Parser = fn(&str) -> Result<Value, LoaderError>;

pub fn get_file_parser(extension: &str) -> Result<Parser, LoaderError> {
    match extension {
        "json" => Ok(parse_json),
        #[cfg(feature = "toml")]
        "toml" => Ok(parse_toml),
        #[cfg(feature = "yaml")]
        "yaml" | "yml" => Ok(parse_yaml),
        #[cfg(feature = "ini")]
        "ini" => Ok(parse_ini),
        _ => Err(LoaderError::NotSupportedExtension {
            ext: extension.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use crate::source::LoaderError;

    use super::*;

    fn nested_string<'a>(value: &'a Value, section: &str, key: &str) -> &'a str {
        let Value::Map(root) = value else {
            panic!("expected a map at the document root");
        };
        let Value::Map(section) = root.get(&Value::String(section.to_string())).expect("missing section") else {
            panic!("expected the section to be a map");
        };
        let Value::String(value) = section.get(&Value::String(key.to_string())).expect("missing key") else {
            panic!("expected a string value");
        };
        value
    }

    #[cfg(feature = "toml")]
    #[test]
    fn parses_toml() {
        let value = parse_toml("[profile]\nname = \"toml\"").expect("valid TOML");
        assert_eq!(nested_string(&value, "profile", "name"), "toml");
    }

    #[test]
    fn parses_json() {
        let value = parse_json(r#"{"profile":{"name":"json"}}"#).expect("valid JSON");
        assert_eq!(nested_string(&value, "profile", "name"), "json");
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn parses_yaml_and_accepts_both_extensions() {
        let value = parse_yaml("profile:\n  name: yaml").expect("valid YAML");
        assert_eq!(nested_string(&value, "profile", "name"), "yaml");
        assert!(get_file_parser("yaml").is_ok());
        assert!(get_file_parser("yml").is_ok());
    }

    #[cfg(feature = "ini")]
    #[test]
    fn parses_ini() {
        let value = parse_ini("[profile]\nname=ini").expect("valid INI");
        assert_eq!(nested_string(&value, "profile", "name"), "ini");
    }

    #[test]
    fn rejects_unknown_extensions() {
        assert!(matches!(
            get_file_parser("xml"),
            Err(LoaderError::NotSupportedExtension { ext: _ })
        ));
    }
}
