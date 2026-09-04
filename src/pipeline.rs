use clap::Parser;
use include_dir::Dir;
use serde_core::de::DeserializeOwned;

use crate::{api::ConfigArgs, error::ConfigError, expression, merge, source};

pub fn load_config_from_dir<T>(config_folder: &Dir<'_>) -> Result<T, ConfigError>
where
    T: DeserializeOwned,
{
    let args = ConfigArgs::parse();
    let profiles = args.profiles;
    let overrides = args.overrides;
    let files_content = source::load_values(config_folder, &profiles, &overrides)?;
    let merged_content = merge::merge_values(&files_content);
    let processed_content = expression::process(&merged_content).map_err(|err| ConfigError::Resolve { causes: err })?;
    T::deserialize(processed_content).map_err(|err| ConfigError::Deserialize { cause: err })
}
