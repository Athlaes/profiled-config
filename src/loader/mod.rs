use std::path::Path;

use include_dir::Dir;
use log::{error, info};
use serde_value::Value;
use thiserror::Error;

mod cli_args;
mod embedded;
mod format;
mod runtime_files;

#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("Couldn't open current folder : '{source_str}'")]
    CurrentFolderNotReadable { source_str: String },
    #[error("File '{file_name}' not found")]
    FileNotFound { file_name: String },
    #[error("File extension not found for file '{file_name}'")]
    FileExtensionNotFound { file_name: String },
    #[error("Couldn't open current folder : '{source_str}'")]
    FileNotReadable { file_name: String, source_str: String },
    #[error("Multiple file '{file_name}' found")]
    MultipleFileFound { file_name: String },
    #[error("File '{file_name}' has no content or is not valid UTF-8")]
    NoContent { file_name: String },
    #[error("{0}")]
    ParseError(String),
    #[error("Found file with ext '{ext}' which is not supported or feature is not enabled")]
    NotSupportedExtension { ext: String },
}

pub fn load_values(
    config_folder: &Dir<'_>,
    profiles: &[String],
    overrides: &[String],
) -> Result<Vec<Value>, LoaderError> {
    let mut files_values: Vec<Value> = Vec::new();

    // Load default config
    let default_value = embedded::load_profile(config_folder, "default")?;
    files_values.push(default_value);

    // Load profiled config
    for profile in profiles {
        match embedded::load_profile(config_folder, profile) {
            Ok(value) => {
                files_values.push(value);
            }
            Err(e) => {
                error!("Couldn't load config for {profile}: {e}");
            }
        }
    }

    // Load config overrides
    match runtime_files::load(Path::new("./")) {
        Ok(Some(value)) => files_values.push(value),
        Ok(None) => {
            info!("No runtime files overrides found");
        }
        Err(err) => {
            error!("Erreur lors de la lecture du fichier overrides : {err}");
        }
    }

    match cli_args::load(overrides) {
        Ok(Some(value)) => files_values.push(value),
        Ok(None) => {
            info!("No CLI args overrides found");
        }
        Err(err) => {
            error!("Erreur lors de la lecture du fichier overrides : {err}");
        }
    }

    Ok(files_values)
}

#[cfg(test)]
mod tests {
    use include_dir::{Dir, DirEntry, File};

    use super::*;

    #[test]
    #[should_panic]
    fn rejects_duplicate_profile_files() {
        let entries = [
            DirEntry::File(File::new("default.toml", b"")),
            DirEntry::File(File::new("default.json", b"")),
        ];
        let directory = Dir::new("", &entries);

        load_values(&directory, &["default".to_string()], &[]).unwrap();
    }
}
