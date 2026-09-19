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
}

#[tokio::main]
async fn main() {
    let config: Config = profiled_config::load_config_async!(LoadOptions::new(
        vec!["vault_app_role".to_string()],
        vec!["test.overrides=hello".to_string()]
    ));

    assert_eq!("default", config.profile.name);
    assert_eq!("hello", config.test.overrides);
    assert_eq!("value_set_in_kv2", config.test.vault_value);
}
