use clap::Parser;
use include_dir::Dir;

#[doc(hidden)]
pub use include_dir;
#[cfg(feature = "macros")]
pub use profiled_config_macros::profiled_config;

use serde_core::de::DeserializeOwned;
use serde_value::DeserializerError;
use thiserror::Error;

use crate::{loader::LoaderError, processor::ResolveError};

mod formatter;
mod loader;
mod merger;
mod parser;
mod processor;
mod provider;
mod resolver;
mod selector;

#[derive(Parser)]
#[command(version, about, long_about = "")]
struct ConfigArgs {
    #[arg(short, long, value_delimiter = ',')]
    profiles: Vec<String>,
    #[arg(short, long, value_delimiter = ',')]
    overrides: Vec<String>,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to load configuration : {0}")]
    Loading(#[from] LoaderError),
    #[error("Failed to resolve configuration :\n\n{}", formatter::format_error(causes))]
    Resolve { causes: Vec<ResolveError> },
    #[error("Failed to deserialize configuration : {cause}")]
    Deserialize {
        #[source]
        cause: DeserializerError,
    },
}

#[macro_export]
macro_rules! load_config {
    () => {{
        use $crate::include_dir;

        static CONFIG_FOLDER: include_dir::Dir<'static> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/config");

        $crate::load_config_from_dir(&CONFIG_FOLDER).unwrap_or_else(|err| panic!("{err}"))
    }};
}

#[macro_export]
macro_rules! try_load_config {
    () => {{
        use $crate::include_dir;

        static CONFIG_FOLDER: include_dir::Dir<'static> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/config");

        $crate::load_config_from_dir(&CONFIG_FOLDER)
    }};
}

#[doc(hidden)]
pub fn load_config_from_dir<T>(config_folder: &Dir<'_>) -> Result<T, ConfigError>
where
    T: DeserializeOwned,
{
    let profiles = ConfigArgs::parse().profiles;
    let overrides = ConfigArgs::parse().overrides;
    let files_content = loader::load_values(config_folder, &profiles, &overrides)
        .unwrap_or_else(|err| panic!("Erreur lors du chargement dans la configuration : {err}"));
    let merged_content = merger::merge_values(&files_content);
    let processed_content = processor::process(&merged_content).unwrap_or_else(|errors| {
        panic!(
            "Failed to resolve configuration :\n\n{}",
            formatter::format_error(&errors)
        )
    });
    T::deserialize(processed_content).map_err(|err| ConfigError::Deserialize { cause: err })
}
