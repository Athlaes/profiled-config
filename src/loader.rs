use include_dir::Dir;
use log::{error, info};
use toml::Table;

use crate::ConfigError;

pub fn load_values(config_folder: &Dir<'_>, profiles: &[String]) -> Vec<Table> {
    let mut files_values: Vec<Table> = Vec::new();
    info!("Loading default config from default.toml");
    files_values.push(
        get_file_values(config_folder, "default.toml")
            .unwrap_or_else(|e| panic!("Unable to load default config: {}", e)),
    );
    for profile in profiles {
        let path = format!("{}.toml", profile);
        match get_file_values(config_folder, &path) {
            Ok(test) => files_values.push(test),
            Err(e) => {
                error!("Unable to load config file {} with cause : {}", path, e)
            }
        }
    }
    files_values
}

fn get_file_values(config_folder: &Dir<'_>, path: &str) -> Result<Table, ConfigError> {
    toml::from_str(
        config_folder
            .get_file(path)
            .ok_or(ConfigError::FileNotFound(format!(
                "File with path {} not found",
                path
            )))?
            .contents_utf8()
            .ok_or(ConfigError::ContentUtf8Error(
                "Error getting file content utf8".to_string(),
            ))?,
    )
    .map_err(ConfigError::ParseError)
}
