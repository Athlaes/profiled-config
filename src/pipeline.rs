use include_dir::Dir;
use serde_core::de::DeserializeOwned;

use crate::{api::LoadOptions, error::ConfigError, expression, merge, source};

#[cfg(feature = "auto-cli")]
pub fn load_config_from_dir<T>(config_folder: &Dir<'_>) -> Result<T, ConfigError>
where
    T: DeserializeOwned,
{
    use clap::Parser;

    use crate::cli::ProfiledConfigParser;

    let args = ProfiledConfigParser::parse();
    load_config_from_dir_with(
        config_folder,
        &LoadOptions {
            profiles: args.profiled_config.profiles,
            overrides: args.profiled_config.overrides,
        },
    )
}

pub fn load_config_from_dir_with<T>(config_folder: &Dir<'_>, options: &LoadOptions) -> Result<T, ConfigError>
where
    T: DeserializeOwned,
{
    let profiles = &options.profiles;
    let overrides = &options.overrides;
    let files_content = source::load_values(config_folder, profiles, overrides)?;
    let merged_content = merge::merge_values(&files_content);
    let processed_content = expression::process(&merged_content).map_err(|err| ConfigError::Resolve { causes: err })?;
    T::deserialize(processed_content).map_err(|err| ConfigError::Deserialize { cause: err })
}
