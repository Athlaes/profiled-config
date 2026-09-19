# profiled-config

[![CI](https://github.com/Athlaes/profiled-config/actions/workflows/ci.yml/badge.svg)](https://github.com/Athlaes/profiled-config/actions/workflows/ci.yml)

Typed, layered configuration for Rust. Defaults are embedded in the binary;
profiles and overrides are selected at startup.

> [!WARNING]
> This project is experimental and not yet recommended for production.

## Quick start

```shell
cargo add profiled_config
cargo add serde --features derive
```

```toml
# config/default.toml
name = "my-service"
port = 8080

[database]
url = "${env:DATABASE_URL:postgres://localhost/my-service}"
```

```rust
use profiled_config::profiled_config;
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    name: String,
    port: u16,
    database: Database,
}

#[derive(Deserialize)]
struct Database {
    url: String,
}

#[profiled_config]
fn main(config: Config) {
    println!("{} listens on {}", config.name, config.port);
}
```

```shell
cargo run
# my-service listens on 8080
```

## Profiles and overrides

Sources are applied from top to bottom; the last value wins:

```text
config/default.*
config/<profile>.*
./overrides.*
--overrides <path>=<value>
provider expressions (env, Vault, custom)
```

```toml
# config/development.toml
port = 3000
```

```toml
# overrides.toml
port = 4000
```

```shell
cargo run -- --profiles development
# port = 4000

cargo run -- --profiles development --overrides port=9090
# port = 9090
```

Profiles and CLI overrides can be chained:

```shell
cargo run -- --profiles development,local
cargo run -- --overrides server.port=9090,features.cache=true
cargo run -- --overrides server.port=9090 --overrides features.cache=true
```

Maps merge recursively. Scalars and arrays are replaced. CLI values are parsed
as JSON scalars, so `true` and `9090` keep their types.

## Environment expressions

```toml
required = "${env:API_TOKEN}"
with_fallback = "${env:HOST:localhost}"
inside_a_string = "https://${env:HOST:localhost}:${env:PORT:8080}"
from_json = "${env:SERVICE_JSON(jsonpath:$.host):localhost}"
```

A missing value without a fallback stops loading.

## Vault secrets

Enable Vault support:

```shell
cargo add profiled_config --features vault
```

For a KV v2 secret at `apps/my-service` in the `secret` mount, containing
`{"password":"s3cr3t"}`:

```toml
# config/default.toml
[database]
password = "${vault:apps/my-service/password}"

[profiled_config.providers.vault]
url = "${env:VAULT_ADDR}"
mount = "secret"

[profiled_config.providers.vault.auth.Token]
token = "${env:VAULT_TOKEN}"
```

`${vault:<secret-path>/<field>}` reads one field; omit the mount and `/data/`
from the expression. Here, `database.password` becomes `s3cr3t`.
For a JSON object field, select a nested value with
`${vault:apps/my-service/credentials(jsonpath:$.password)}`.

For AppRole authentication, replace the `auth.Token` table with:

```toml
[profiled_config.providers.vault.auth.AppRole]
mount = "approle"
role_id = "${env:VAULT_ROLE_ID}"
secret_id = "${env:VAULT_SECRET_ID}"
```

Vault is initialized automatically when its configuration is present;
no `add_provider` call is needed.

## Formats

JSON is always available. TOML is enabled by default.

```shell
cargo add profiled_config --features yaml,ini
```

Profiles may mix `.json`, `.toml`, `.yaml`, `.yml`, and `.ini` files. INI
supports string keys and values only.

## Without the attribute macro

The default `auto-cli` mode still reads `--profiles` and `--overrides`:

```rust
fn main() {
    let config: Config = profiled_config::load_config!();
}
```

Run setup code before loading with `before_load`:

```rust
#[profiled_config(before_load = initialize_logging)]
#[tokio::main]
async fn main(config: Config) {
    // ...
}
```

## Existing Clap application

Disable the default attribute macro and `auto-cli`; keep the format you use:

```shell
cargo add profiled_config --no-default-features --features toml
cargo add clap --features derive
cargo add serde --features derive
```

Flatten the configuration arguments into your parser, then load explicitly:

```rust
use clap::Parser;
use profiled_config::{LoadOptions, ProfiledConfigArgs};

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    verbose: bool,

    #[command(flatten)]
    config: ProfiledConfigArgs,
}

fn main() -> Result<(), profiled_config::ConfigError> {
    let cli = Cli::parse();
    let options: LoadOptions = cli.config.into();
    let config: Config = profiled_config::try_load_config!(options)?;

    if cli.verbose {
        println!("{}:{}", config.name, config.port);
    }

    Ok(())
}
```

`macros` enables the `#[profiled_config]` attribute; `load_config!` and
`try_load_config!` remain available without it.

## Custom providers

Implement `Provider`, then register it in `LoadOptions` before loading.
This example exposes values from a configuration table as `${local:<key>}`:

```shell
cargo add serde-value
```

```toml
# config/default.toml
name = "${local:service_name}"

[profiled_config.providers.local]
service_name = "my-service"
```

```rust
use std::{collections::BTreeMap, future::Future, pin::Pin};
use profiled_config::{
    LoadOptions, Provider, ProviderActivation, ProviderError, ProviderFactoryFuture,
};
use serde_value::Value;

struct LocalProvider(BTreeMap<String, String>);

impl Provider for LocalProvider {
    fn key() -> String { "local".into() }

    fn activation() -> ProviderActivation { ProviderActivation::WhenConfigured }

    fn create<'a>(values: &Value) -> ProviderFactoryFuture<'a> {
        let values = values.clone().deserialize_into::<BTreeMap<String, String>>();
        Box::pin(async move {
            let values = values.map_err(|err| ProviderError::Init(err.to_string()))?;
            Ok(Box::new(Self(values)) as Box<dyn Provider>)
        })
    }

    fn resolve<'a>(&'a self, key: &'a str)
        -> Pin<Box<dyn Future<Output = Result<String, ProviderError>> + 'a>>
    {
        Box::pin(async move {
            self.0.get(key).cloned().ok_or_else(|| ProviderError::VariableNotFound {
                key: key.into(),
                cause_str: "Unknown local key".into(),
            })
        })
    }
}
```

`create` receives the provider's table after its expressions are resolved;
`resolve` receives the key after `local:`. `WhenConfigured` requires that table;
use `Always` for a provider that can initialize without one.

Register it when loading your `Config`:

```rust
let mut options = LoadOptions::new(vec![], vec![]);
options.add_provider::<LocalProvider>()?;
let config: Config = profiled_config::try_load_config!(options)?;
assert_eq!(config.name, "my-service");
```

In an async function, use `try_load_config_async!(options)` instead;
the macro awaits loading for you. With your own CLI, build `options` from
`cli.config.into()` as above to keep profiles and overrides.
