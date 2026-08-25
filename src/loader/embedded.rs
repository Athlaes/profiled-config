use std::ffi::OsStr;

use include_dir::{Dir, File};
use serde_value::Value;

use crate::{ConfigError, loader::format::get_file_parser};

pub fn load_profile(config_folder: &Dir<'_>, profile: &str) -> Result<Value, ConfigError> {
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
