use std::{fs, path::Path};

use log::info;
use serde_value::Value;

use crate::loader::{LoaderError, format::get_file_parser};

pub fn load(directory: &Path) -> Result<Option<Value>, LoaderError> {
    let paths = fs::read_dir(directory).map_err(|err| LoaderError::CurrentFolderNotReadable {
        source_str: err.to_string(),
    })?;

    let overrrides_files = paths
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.file_type().is_ok_and(|f| f.is_file())
                && entry.file_name().to_str().is_some_and(|n| n.starts_with("overrides."))
        })
        .collect::<Vec<_>>();

    let Some(file) = overrrides_files.first() else {
        info!("No override file found");
        return Ok(None);
    };

    if overrrides_files.len() > 1 {
        return Err(LoaderError::MultipleFileFound {
            file_name: "overrides".to_string(),
        });
    }

    let path = file.path();
    let file_name = path
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or(LoaderError::FileNotFound {
            file_name: "overrides".to_string(),
        })?;
    let ext = file_name
        .rsplit(".")
        .next()
        .ok_or(LoaderError::NotSupportedExtension { ext: "".to_string() })?;
    let parser = get_file_parser(ext)?;
    let content = fs::read_to_string(&path).map_err(|err| LoaderError::FileNotReadable {
        file_name: file_name.to_string(),
        source_str: err.to_string(),
    })?;

    parser(&content).map(Some)
}
