use env_logger::Env;
use log::info;
use profiled_config::profiled_config;

#[derive(serde::Deserialize)]
struct Config {
    pub app_version: String,
    pub profile: Profile,
}

#[derive(serde::Deserialize)]
struct Profile {
    pub name: String,
}

fn init_logger() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
}

#[profiled_config(before_load = init_logger)]
fn main(config: Config) {
    info!("Folder loaded");
    assert_eq!("1.0.0", config.app_version);
    assert_eq!("TestApp", config.profile.name);
    info!("Test succeeded");
}
