use std::fmt::Display;

use clap::Parser;
use include_dir::Dir;

#[doc(hidden)]
pub use include_dir;
#[cfg(feature = "macros")]
pub use profiled_config_macros::profiled_config;

use serde_core::de::DeserializeOwned;

mod loader;
mod merger;
mod parser;
mod processor;
mod provider;
mod resolver;
mod selector;

#[derive(Debug)]
enum ConfigError {
    FileNotFound(String),
    ContentUtf8Error(String),
    ParseError(String),
    DuplicateProfile(String),
    NotSupportedExtension(String),
    ExtensionNotFound(String),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileNotFound(error) => write!(f, "{error}"),
            ConfigError::ContentUtf8Error(error) => write!(f, "{error}"),
            ConfigError::ParseError(error) => write!(f, "{error}"),
            ConfigError::DuplicateProfile(error) => write!(f, "{error}"),
            ConfigError::NotSupportedExtension(error) => write!(f, "{error}"),
            ConfigError::ExtensionNotFound(error) => write!(f, "{error}"),
        }
    }
}

#[derive(Parser)]
#[command(version, about, long_about = "")]
struct ConfigArgs {
    #[arg(short, long, value_delimiter = ',')]
    profiles: Vec<String>,
    #[arg(short, long, value_delimiter = ',')]
    overrides: Vec<String>,
}

#[macro_export]
macro_rules! load_config {
    () => {{
        use $crate::include_dir;

        static CONFIG_FOLDER: include_dir::Dir<'static> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/config");

        $crate::load_config_from_dir(&CONFIG_FOLDER)
    }};
}

#[doc(hidden)]
pub fn load_config_from_dir<T>(config_folder: &Dir<'_>) -> T
where
    T: DeserializeOwned,
{
    let profiles = ConfigArgs::parse().profiles;
    let overrides = ConfigArgs::parse().overrides;
    let files_content = loader::load_values(&config_folder, &profiles, &overrides);
    let merged_content = merger::merge_values(&files_content);
    let processed_content = processor::process_any(&merged_content);
    T::deserialize(processed_content).unwrap_or_else(|err| panic!("Couldn't deserialize configuration : {err}"))
}
