use std::ffi::OsStr;

use testcontainers::{
    GenericImage, ImageExt,
    core::{ContainerPort, WaitFor},
    runners::AsyncRunner,
};
use tokio::process::Command;
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
use vaultrs::{api::auth::approle::requests::SetAppRoleRequest, auth::approle, sys};

#[tokio::test]
async fn vault_config_should_work() {
    let port = ContainerPort::Tcp(8200);
    let container = GenericImage::new("hashicorp/vault", "1.21")
        .with_exposed_port(port)
        .with_wait_for(WaitFor::message_on_stdout(
            "Development mode should NOT be used in production",
        ))
        .with_env_var("VAULT_DEV_ROOT_TOKEN_ID", "dev-only-token")
        .with_cmd(["server", "-dev", "-dev-listen-address=0.0.0.0:8200"])
        .start()
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    let host = container.get_host().await.unwrap_or_else(|err| panic!("{err}"));
    let port = container
        .get_host_port_ipv4(port)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let address = format!("http://{host}:{port}");

    // Préparer le secret de test.
    let client = VaultClient::new(
        VaultClientSettingsBuilder::default()
            .address(&address)
            .token("dev-only-token")
            .build()
            .unwrap(),
    )
    .unwrap_or_else(|err| panic!("{err}"));

    vaultrs::kv2::set(
        &client,
        "secret",
        "profiled_config/vault",
        &serde_json::json!({
            "vault_value": "value_set_in_kv2"
        }),
    )
    .await
    .unwrap_or_else(|err| panic!("{err}"));

    let output = Command::new(env!("CARGO_BIN_EXE_vault_e2e"))
        .env("VAULT_PORT", OsStr::new(&port.to_string()))
        .output()
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    assert!(
        output.status.success(),
        "Execution failed with error : {}",
        String::from_utf8_lossy(&output.stderr)
    )
}

#[tokio::test]
async fn vault_app_role_config_should_work() {
    let port = ContainerPort::Tcp(8200);
    let container = GenericImage::new("hashicorp/vault", "1.21")
        .with_exposed_port(port)
        .with_wait_for(WaitFor::message_on_stdout(
            "Development mode should NOT be used in production",
        ))
        .with_env_var("VAULT_DEV_ROOT_TOKEN_ID", "dev-only-token")
        .with_cmd(["server", "-dev", "-dev-listen-address=0.0.0.0:8200"])
        .start()
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    let host = container.get_host().await.unwrap_or_else(|err| panic!("{err}"));
    let port = container
        .get_host_port_ipv4(port)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let address = format!("http://{host}:{port}");

    let client = VaultClient::new(
        VaultClientSettingsBuilder::default()
            .address(&address)
            .token("dev-only-token")
            .build()
            .unwrap(),
    )
    .unwrap_or_else(|err| panic!("{err}"));

    vaultrs::kv2::set(
        &client,
        "secret",
        "profiled_config/vault",
        &serde_json::json!({
            "vault_value": "value_set_in_kv2"
        }),
    )
    .await
    .unwrap_or_else(|err| panic!("{err}"));

    sys::auth::enable(&client, "approle", "approle", None)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    sys::policy::set(
        &client,
        "profiled-config-read",
        r#"path "secret/data/profiled_config/vault" { capabilities = ["read"] }"#,
    )
    .await
    .unwrap_or_else(|err| panic!("{err}"));
    approle::role::set(
        &client,
        "approle",
        "profiled-config",
        Some(
            SetAppRoleRequest::builder()
                .token_policies(vec!["profiled-config-read".to_string()])
                .token_no_default_policy(true),
        ),
    )
    .await
    .unwrap_or_else(|err| panic!("{err}"));

    let role_id = approle::role::read_id(&client, "approle", "profiled-config")
        .await
        .unwrap_or_else(|err| panic!("{err}"))
        .role_id;
    let secret_id = approle::role::secret::generate(&client, "approle", "profiled-config", None)
        .await
        .unwrap_or_else(|err| panic!("{err}"))
        .secret_id;

    let output = Command::new(env!("CARGO_BIN_EXE_vault_app_role_e2e"))
        .env("VAULT_PORT", OsStr::new(&port.to_string()))
        .env("VAULT_ROLE_ID", role_id)
        .env("VAULT_SECRET_ID", secret_id)
        .env_remove("VAULT_TOKEN")
        .output()
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    assert!(
        output.status.success(),
        "Execution failed with error : {}",
        String::from_utf8_lossy(&output.stderr)
    )
}
