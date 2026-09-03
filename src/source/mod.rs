use std::path::Path;

use include_dir::Dir;
use log::info;
use serde_value::Value;
use thiserror::Error;

mod embedded;
mod inline_override;
mod runtime_file;

#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("Couldn't open current folder : '{source_str}'")]
    CurrentFolderNotReadable { source_str: String },
    #[error("File '{file_name}' not found")]
    FileNotFound { file_name: String },
    #[error("File extension not found for file '{file_name}'")]
    FileExtensionNotFound { file_name: String },
    #[error("Couldn't open file '{file_name}' : '{cause_str}'")]
    FileNotReadable { file_name: String, cause_str: String },
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
        files_values.push(embedded::load_profile(config_folder, profile)?);
    }

    // Load config overrides
    match runtime_file::load(Path::new("./"))? {
        Some(value) => files_values.push(value),
        None => {
            info!("No runtime files overrides found in path './'");
        }
    }

    match inline_override::load(overrides)? {
        Some(value) => files_values.push(value),
        None => {
            info!("No CLI args overrides found");
        }
    }

    Ok(files_values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::assert_matches;

    use include_dir::{Dir, DirEntry, File};

    #[test]
    fn rejects_duplicate_profile_files() {
        let entries = [
            DirEntry::File(File::new("default.toml", b"")),
            DirEntry::File(File::new("default.json", b"")),
        ];
        let directory = Dir::new("", &entries);

        let error = load_values(&directory, &["default".to_string()], &[])
            .expect_err("duplicate profile files should be rejected");

        assert_matches!(
            error,
            LoaderError::MultipleFileFound { file_name } if file_name == "default"
        );
    }

    #[test]
    fn ignores_malformed_cli_overrides() {
        let entries = [DirEntry::File(File::new("default.json", b"{}"))];
        let directory = Dir::new("", &entries);

        let result = load_values(&directory, &[], &["invalid".to_string()]).unwrap_err();
        assert_matches!(result, LoaderError::ParseError(_));
    }
}
