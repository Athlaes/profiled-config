use std::ffi::OsStr;

use include_dir::{Dir, File};
use serde_value::Value;

use crate::loader::{LoaderError, format::get_file_parser};

pub fn load_profile(config_folder: &Dir<'_>, profile: &str) -> Result<Value, LoaderError> {
    let mut files = config_folder
        .files()
        .filter(|file| file.path().file_stem() == Some(OsStr::new(profile)));
    let file = files.next().ok_or(LoaderError::FileNotFound {
        file_name: profile.to_string(),
    })?;

    if let Some(_) = files.next() {
        return Err(LoaderError::MultipleFileFound {
            file_name: profile.to_string(),
        });
    }

    get_file_values(file)
}

fn get_file_values(file: &File) -> Result<Value, LoaderError> {
    let extension =
        file.path()
            .extension()
            .and_then(OsStr::to_str)
            .ok_or_else(|| LoaderError::FileExtensionNotFound {
                file_name: file.path().display().to_string(),
            })?;
    get_file_parser(extension).and_then(|parser| parser(get_file_contents(file)?))
}

fn get_file_contents<'file>(file: &'file File<'_>) -> Result<&'file str, LoaderError> {
    file.contents_utf8().ok_or_else(|| LoaderError::NoContent {
        file_name: file.path().display().to_string(),
    })
}
