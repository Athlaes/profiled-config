use std::{
    ffi::OsStr,
    fs::{self},
};

use include_dir::{Dir, File};
use log::{error, warn};
use serde_value::Value;

use crate::ConfigError;

type Parser = fn(&str) -> Result<Value, ConfigError>;

pub fn load_values(config_folder: &Dir<'_>, profiles: &[String]) -> Vec<Value> {
    let mut files_values: Vec<Value> = Vec::new();
    // Load default config
    let default_value =
        load_profile(config_folder, "default").unwrap_or_else(|e| panic!("Couldn't load default config: {e}"));
    files_values.push(default_value);
    // Load profiled config
    for profile in profiles {
        match load_profile(config_folder, profile) {
            Ok(value) => {
                files_values.push(value);
            }
            Err(e) => {
                warn!("Couldn't load config for {profile}: {e}");
            }
        }
    }
    // Load config overrides
    if let Some(value) = load_override_file_value() {
        files_values.push(value);
    }

    files_values
}

fn load_override_file_value() -> Option<Value> {
    let paths = fs::read_dir("./")
        .map_err(|err| warn!("Couldn't read current folder: {err}"))
        .ok()?;

    let Some(file) = paths
        .filter_map(Result::ok)
        .find(|entry| entry.file_name().to_str().is_some_and(|n| n.starts_with("overrides.")))
    else {
        warn!("No override file found");
        return None;
    };

    let path = file.path();
    let ext = path.file_name()?.to_str().and_then(|f| f.rsplit(".").next())?;
    let parser = get_file_parser(&ext)
        .map_err(|err| error!("Couldn't get parser for {} : {err}", path.display()))
        .ok()?;
    let content = fs::read_to_string(&path)
        .map_err(|err| error!("Couldn't get file content for {} : {err}", path.display()))
        .ok()?;

    parser(&content)
        .map_err(|err| error!("Couldn't parse file {} : {err}", path.display()))
        .ok()
}

fn load_profile(config_folder: &Dir<'_>, profile: &str) -> Result<Value, ConfigError> {
    let mut files = config_folder
        .files()
        .filter(|file| file.path().file_stem() == Some(OsStr::new(profile)));
    let file = files
        .next()
        .ok_or(ConfigError::FileNotFound(format!("Profile {} not found", profile)))?;

    if let Some(other_file) = files.next() {
        return Err(ConfigError::DuplicateProfile(format!(
            "Multiple files found for profile {profile}: {} and {}",
            file.path().display(),
            other_file.path().display()
        )));
    }

    get_file_values(file)
}

fn get_file_values(file: &File) -> Result<Value, ConfigError> {
    let extension = file.path().extension().and_then(OsStr::to_str).ok_or_else(|| {
        ConfigError::ExtensionNotFound(format!("File extension not found for {}", file.path().display()))
    })?;

    get_file_parser(extension).and_then(|parser| parser(get_file_contents(file)?))
}

fn get_file_contents<'file>(file: &'file File<'_>) -> Result<&'file str, ConfigError> {
    file.contents_utf8()
        .ok_or_else(|| ConfigError::ContentUtf8Error("File is not valid UTF-8".to_string()))
}

fn get_file_parser(extension: &str) -> Result<Parser, ConfigError> {
    match extension {
        "json" => Ok(parse_json),
        #[cfg(feature = "toml")]
        "toml" => Ok(parse_toml),
        #[cfg(feature = "yaml")]
        "yaml" | "yml" => Ok(parse_yaml),
        #[cfg(feature = "ini")]
        "ini" => Ok(parse_ini),
        _ => Err(ConfigError::NotSupportedExtension(format!(
            "Extension {:?} not supported or feature is not enabled",
            extension
        ))),
    }
}

fn parse_json(content: &str) -> Result<Value, ConfigError> {
    serde_json::from_str(content).map_err(|error| ConfigError::ParseError(format!("Couldn't parse file : {error}")))
}

#[cfg(feature = "toml")]
fn parse_toml(content: &str) -> Result<Value, ConfigError> {
    toml::from_str(content).map_err(|error| ConfigError::ParseError(format!("Couldn't parse file : {error}")))
}

#[cfg(feature = "yaml")]
fn parse_yaml(content: &str) -> Result<Value, ConfigError> {
    yaml_serde::from_str(content).map_err(|error| ConfigError::ParseError(format!("Couldn't parse file : {error}")))
}

#[cfg(feature = "ini")]
fn parse_ini(content: &str) -> Result<Value, ConfigError> {
    serde_ini::from_str(content).map_err(|error| ConfigError::ParseError(format!("Couldn't parse file : {error}")))
}

#[cfg(test)]
mod tests {
    use include_dir::DirEntry;

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
            Err(ConfigError::NotSupportedExtension(_))
        ));
    }

    #[test]
    fn rejects_duplicate_profile_names() {
        let entries = [
            DirEntry::File(File::new("default.toml", b"")),
            DirEntry::File(File::new("default.json", b"")),
        ];
        let directory = Dir::new("", &entries);

        assert!(matches!(
            load_profile(&directory, "default"),
            Err(ConfigError::DuplicateProfile(_))
        ));
    }
}
