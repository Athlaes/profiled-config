use std::collections::HashMap;

use crate::Provider;

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
    pub additional_providers: HashMap<String, Box<dyn Provider>>,
}

impl From<ProfiledConfigArgs> for LoadOptions {
    fn from(value: ProfiledConfigArgs) -> Self {
        Self {
            profiles: value.profiles,
            overrides: value.overrides,
            additional_providers: HashMap::new(),
        }
    }
}

#[macro_export]
#[cfg(feature = "auto-cli")]
macro_rules! load_config {
    () => {{
        use $crate::include_dir;

        static CONFIG_FOLDER: include_dir::Dir<'static> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/config");

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap_or_else(|err| panic!("{err}"));
        runtime
            .block_on($crate::load_config_from_dir(&CONFIG_FOLDER))
            .unwrap_or_else(|err| panic!("{err}"))
    }};

    ($options:expr) => {{
        use $crate::include_dir;
        use $crate::tokio;

        static CONFIG_FOLDER: include_dir::Dir<'static> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/config");

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap_or_else(|err| panic!("{err}"));
        runtime
            .block_on($crate::load_config_from_dir_with(&CONFIG_FOLDER, $options))
            .unwrap_or_else(|err| panic!("{err}"))
    }};
}

#[macro_export]
#[cfg(not(feature = "auto-cli"))]
macro_rules! load_config {
    ($options:expr) => {{
        use $crate::include_dir;
        use $crate::tokio;

        static CONFIG_FOLDER: include_dir::Dir<'static> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/config");

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap_or_else(|err| panic!("{err}"));
        runtime
            .block_on($crate::load_config_from_dir_with(&CONFIG_FOLDER, $options))
            .unwrap_or_else(|err| panic!("{err}"))
    }};
}

#[macro_export]
#[cfg(feature = "auto-cli")]
macro_rules! load_config_async {
    () => {{
        use $crate::include_dir;

        static CONFIG_FOLDER: include_dir::Dir<'static> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/config");

        $crate::load_config_from_dir(&CONFIG_FOLDER)
            .await
            .unwrap_or_else(|err| panic!("{err}"))
    }};

    ($options:expr) => {{
        use $crate::include_dir;

        static CONFIG_FOLDER: include_dir::Dir<'static> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/config");

        $crate::load_config_from_dir_with(&CONFIG_FOLDER, $options)
            .await
            .unwrap_or_else(|err| panic!("{err}"))
    }};
}

#[macro_export]
#[cfg(not(feature = "auto-cli"))]
macro_rules! load_config_async {
    ($options:expr) => {{
        use $crate::include_dir;

        static CONFIG_FOLDER: include_dir::Dir<'static> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/config");

        $crate::load_config_from_dir_with(&CONFIG_FOLDER, $options)
            .await
            .unwrap_or_else(|err| panic!("{err}"))
    }};
}

#[macro_export]
#[cfg(feature = "auto-cli")]
macro_rules! try_load_config {
    () => {{
        use $crate::include_dir;

        static CONFIG_FOLDER: include_dir::Dir<'static> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/config");
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err($crate::ConfigError::Runtime)
            .and_then(|runtime| runtime.block_on($crate::load_config_from_dir(&CONFIG_FOLDER, $options)))
    }};

    ($options:expr) => {{
        use $crate::include_dir;

        static CONFIG_FOLDER: include_dir::Dir<'static> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/config");

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err($crate::ConfigError::Runtime)
            .and_then(|runtime| runtime.block_on($crate::load_config_from_dir_with(&CONFIG_FOLDER, $options)))
    }};
}

#[macro_export]
#[cfg(not(feature = "auto-cli"))]
macro_rules! try_load_config {
    ($options:expr) => {{
        use $crate::include_dir;

        static CONFIG_FOLDER: include_dir::Dir<'static> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/config");

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err($crate::ConfigError::Runtime)
            .and_then(|runtime| runtime.block_on($crate::load_config_from_dir_with(&CONFIG_FOLDER, $options)))
    }};
}

#[cfg(feature = "auto-cli")]
#[macro_export]
macro_rules! try_load_config_async {
    () => {{
        use $crate::include_dir;

        static CONFIG_FOLDER: include_dir::Dir<'static> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/config");

        $crate::load_config_from_dir(&CONFIG_FOLDER).await
    }};

    ($options:expr) => {{
        use $crate::include_dir;
        use $crate::tokio;

        static CONFIG_FOLDER: include_dir::Dir<'static> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/config");

        $crate::load_config_from_dir_with(&CONFIG_FOLDER, $options).await
    }};
}

#[cfg(not(feature = "auto-cli"))]
#[macro_export]
macro_rules! try_load_config_async {
    ($options:expr) => {{
        use $crate::include_dir;
        use $crate::tokio;

        static CONFIG_FOLDER: include_dir::Dir<'static> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/config");

        $crate::load_config_from_dir_with(&CONFIG_FOLDER, $options).await
    }};
}
