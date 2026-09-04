#[derive(clap::Args)]
pub struct ProfiledConfigArgs {
    #[arg(short, long, value_delimiter = ',')]
    pub profiles: Vec<String>,
    #[arg(short, long, value_delimiter = ',')]
    pub overrides: Vec<String>,
}

pub struct LoadOptions {
    pub profiles: Vec<String>,
    pub overrides: Vec<String>,
}

impl From<ProfiledConfigArgs> for LoadOptions {
    fn from(value: ProfiledConfigArgs) -> Self {
        Self {
            profiles: value.profiles,
            overrides: value.overrides,
        }
    }
}

#[macro_export]
#[cfg(feature = "auto-cli")]
macro_rules! load_config {
    () => {{
        use $crate::include_dir;

        static CONFIG_FOLDER: include_dir::Dir<'static> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/config");

        $crate::load_config_from_dir(&CONFIG_FOLDER).unwrap_or_else(|err| panic!("{err}"))
    }};
}

#[macro_export]
#[cfg(not(feature = "auto-cli"))]
macro_rules! load_config {
    ($options:expr) => {{
        use $crate::include_dir;

        static CONFIG_FOLDER: include_dir::Dir<'static> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/config");

        $crate::load_config_from_dir_with(&CONFIG_FOLDER, &$options).unwrap_or_else(|err| panic!("{err}"))
    }};
}

#[macro_export]
#[cfg(feature = "auto-cli")]
macro_rules! try_load_config {
    () => {{
        use $crate::include_dir;

        static CONFIG_FOLDER: include_dir::Dir<'static> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/config");

        $crate::load_config_from_dir(&CONFIG_FOLDER)
    }};
}

#[macro_export]
#[cfg(not(feature = "auto-cli"))]
macro_rules! try_load_config {
    ($options:expr) => {{
        use $crate::include_dir;

        static CONFIG_FOLDER: include_dir::Dir<'static> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/config");

        $crate::load_config_from_dir_with(&CONFIG_FOLDER, &$options).unwrap_or_else(|err| panic!("{err}"))
    }};
}
