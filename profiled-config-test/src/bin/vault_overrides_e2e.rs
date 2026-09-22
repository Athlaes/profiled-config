use profiled_config::LoadOptions;

#[derive(serde::Deserialize)]
struct Config {
    profile: Profile,
    test: Test,
}

#[derive(serde::Deserialize)]
struct Profile {
    name: String,
}

#[derive(serde::Deserialize)]
struct Test {
    vault_value: String,
    overrides: String,
    overrided: bool,
    value: String,
    retries: u32,
}

#[tokio::main]
async fn main() {
    let config: Config = profiled_config::load_config_async!(LoadOptions::new(
        vec!["vault_overrides".to_string()],
        vec!["test.overrides=hello".to_string()]
    ));

    assert_eq!("default", config.profile.name);
    assert_eq!("value_set_in_kv2", config.test.vault_value);
    assert_eq!("value_from_last_path", config.test.overrides);
    assert!(config.test.overrided);
    assert_eq!("value_set_in_kv2", config.test.value);
    assert_eq!(3, config.test.retries);
}
