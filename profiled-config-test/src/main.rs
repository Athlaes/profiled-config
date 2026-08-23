use std::collections::HashMap;

use env_logger::Env;
use log::info;
use profiled_config::profiled_config;

#[derive(serde::Deserialize)]
struct Config {
    pub app_version: String,
    pub profile: Profile,
    pub tabs_test: Tabs,
    pub test: Test,
}

#[derive(serde::Deserialize)]
struct Test {
    pub overrided: bool,
    pub value: String,
}

#[derive(serde::Deserialize)]
struct Profile {
    pub name: String,
}

#[derive(serde::Deserialize)]
struct Tabs {
    pub tabs_1: Vec<u32>,
    pub tabs_2: Vec<String>,
    pub map_inline: HashMap<String, String>,
    pub map: HashMap<String, String>,
}

fn init_logger() {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
}

#[profiled_config(before_load = init_logger)]
#[tokio::main]
async fn main(config: Config) {
    match config.profile.name.as_str() {
        "default" => {
            info!("Default profile loaded");
            assert_eq!("1.0.0", config.app_version);
            assert!(config.test.overrided == false);
            assert_eq!(config.test.value, "default_value");
        }
        "overrided" => {
            info!("Default profile loaded");
            assert_eq!("1.0.0", config.app_version);
            assert!(config.test.overrided);
            assert_eq!(config.test.value, "overrided_value");
        }
        "dev" => {
            info!("Dev profile loaded");
            assert_eq!("1.0.0-SNAPSHOT", config.app_version);
        }
        _ => {}
    }
    assert_eq!(
        config.tabs_test.map,
        HashMap::from([
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string())
        ])
    );
    assert_eq!(
        config.tabs_test.map_inline,
        HashMap::from([
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string())
        ])
    );
    assert_eq!(config.tabs_test.tabs_1, vec![0, 1]);
    assert_eq!(config.tabs_test.tabs_2, vec!["hello".to_string(), "work".to_string()]);
    info!("Test succeeded");
}
