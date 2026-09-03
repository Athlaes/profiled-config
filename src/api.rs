use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = "")]
pub(crate) struct ConfigArgs {
    #[arg(short, long, value_delimiter = ',')]
    pub(crate) profiles: Vec<String>,
    #[arg(short, long, value_delimiter = ',')]
    pub(crate) overrides: Vec<String>,
}

#[macro_export]
macro_rules! load_config {
    () => {{
        use $crate::include_dir;

        static CONFIG_FOLDER: include_dir::Dir<'static> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/config");

        $crate::load_config_from_dir(&CONFIG_FOLDER).unwrap_or_else(|err| panic!("{err}"))
    }};
}

#[macro_export]
macro_rules! try_load_config {
    () => {{
        use $crate::include_dir;

        static CONFIG_FOLDER: include_dir::Dir<'static> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/config");

        $crate::load_config_from_dir(&CONFIG_FOLDER)
    }};
}
