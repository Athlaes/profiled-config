use std::ffi::OsStr;

use testcontainers::{
    GenericImage, ImageExt,
    core::{ContainerPort, WaitFor},
    runners::AsyncRunner,
};
use tokio::process::Command;
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};

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
