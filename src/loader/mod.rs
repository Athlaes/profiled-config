use std::path::Path;

use include_dir::Dir;
use log::warn;
use serde_value::Value;

mod cli_args;
mod embedded;
mod format;
mod runtime_files;

pub fn load_values(config_folder: &Dir<'_>, profiles: &[String], overrides: &[String]) -> Vec<Value> {
    let mut files_values: Vec<Value> = Vec::new();
    // Load default config
    let default_value = embedded::load_profile(config_folder, "default")
        .unwrap_or_else(|e| panic!("Couldn't load default config: {e}"));
    files_values.push(default_value);
    // Load profiled config
    for profile in profiles {
        match embedded::load_profile(config_folder, profile) {
            Ok(value) => {
                files_values.push(value);
            }
            Err(e) => {
                warn!("Couldn't load config for {profile}: {e}");
            }
        }
    }
    // Load config overrides
    if let Some(value) = runtime_files::load(Path::new("./")) {
        files_values.push(value);
    }

    if let Ok(value) = cli_args::load(overrides) {
        files_values.push(value);
    }

    files_values
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

        load_values(&directory, &["default".to_string()], &[]);
    }
}
