# profiled-config

[![CI](https://github.com/Athlaes/profiled-config/actions/workflows/ci.yml/badge.svg)](https://github.com/Athlaes/profiled-config/actions/workflows/ci.yml)

Typed, layered configuration for Rust. Defaults are embedded in the binary;
profiles and overrides are selected at startup.

> [!WARNING]
> This project is experimental and not yet recommended for production.

## Quick start

```shell
cargo add profiled_config --features macros
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
environment expressions
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

## Formats

JSON is always available. TOML is enabled by default.

```shell
cargo add profiled_config --features macros,yaml,ini
```

Profiles may mix `.json`, `.toml`, `.yaml`, `.yml`, and `.ini` files. INI
supports string keys and values only.

## Without the attribute macro

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

## License

[MIT](LICENCE). Inspired by Spring profiles; not affiliated with Spring.
