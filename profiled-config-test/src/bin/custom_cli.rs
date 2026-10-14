use clap::Parser;
use profiled_config::{LoadOptions, ProfiledConfigArgs};

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    application_name: String,

    #[command(flatten)]
    profiled_config: ProfiledConfigArgs,
}

#[derive(serde::Deserialize)]
struct Config {
    app_version: String,
    profile: Profile,
    test: Test,
}

#[derive(serde::Deserialize)]
struct Profile {
    name: String,
}

#[derive(serde::Deserialize)]
struct Test {
    value: String,
}

fn main() {
    let cli = Cli::parse();
    let options: LoadOptions = cli.profiled_config.into();
    let config: Config = profiled_config::try_load_config!(options)
        .unwrap_or_else(|error| panic!("failed to load configuration: {error}"));

    println!(
        "{}|{}|{}|{}",
        cli.application_name, config.profile.name, config.app_version, config.test.value
    );
}
