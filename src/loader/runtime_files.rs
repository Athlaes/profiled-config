use std::{fs, path::Path};

use log::{error, warn};
use serde_value::Value;

use crate::loader::format::get_file_parser;

pub fn load(directory: &Path) -> Option<Value> {
    let paths = fs::read_dir(directory)
        .map_err(|err| warn!("Couldn't read current folder: {err}"))
        .ok()?;

    let Some(file) = paths
        .filter_map(Result::ok)
        .find(|entry| entry.file_name().to_str().is_some_and(|n| n.starts_with("overrides.")))
    else {
        warn!("No override file found");
        return None;
    };

    let path = file.path();
    let ext = path.file_name()?.to_str().and_then(|f| f.rsplit(".").next())?;
    let parser = get_file_parser(&ext)
        .map_err(|err| error!("Couldn't get parser for {} : {err}", path.display()))
        .ok()?;
    let content = fs::read_to_string(&path)
        .map_err(|err| error!("Couldn't get file content for {} : {err}", path.display()))
        .ok()?;

    parser(&content)
        .map_err(|err| error!("Couldn't parse file {} : {err}", path.display()))
        .ok()
}
