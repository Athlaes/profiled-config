# profiled-config

[![CI](https://github.com/Athlaes/profiled-config/actions/workflows/ci.yml/badge.svg)](https://github.com/Athlaes/profiled-config/actions/workflows/ci.yml)

Typed, profile-based configuration for Rust: embed defaults in the binary,
select profiles at startup, apply runtime overrides, and receive the result as a
Serde type in `main`.

> [!WARNING]
> This project is at an early stage. Its API may change, and it is not yet
> recommended for production use.

## Why this crate?

Layered, typed configuration is already well covered by crates such as
[`config`](https://crates.io/crates/config),
[`Figment`](https://crates.io/crates/figment), and
[`confique`](https://crates.io/crates/confique). `profiled-config` does not try
to replace them. It packages a narrower workflow into one convention:

- `config/default.*` and named profiles are embedded automatically;
- built-in CLI flags select profiles and override individual values;
- sources follow a fixed, predictable priority order;
- the merged result is deserialized and passed directly to `main`.

Choose it when this exact workflow matches your application and you would
rather avoid configuring providers or builders. Choose a general-purpose crate
when you need custom sources, hot reload, provenance tracking, remote
configuration, or a different merging strategy.

## Usage

TOML is enabled by default. Install the crate and its optional entry-point
macro with:

```shell
cargo add profiled_config --features macros
```

The crate expects `config/default.<extension>` and applies sources in this
order, with later values taking priority:

1. embedded default;
2. embedded profiles selected by `--profiles` / `-p`;
3. one optional `overrides.<extension>` file from the working directory;
4. values passed through `--overrides` / `-o`;
5. environment expression resolution;
6. Serde deserialization.

Maps merge recursively; later scalars and arrays replace earlier ones. Embedded
files require a rebuild, while profiles and overrides are selected at runtime.

Profiles can be chained from left to right:

```shell
cargo run -- --profiles development,local
```

CLI overrides use `<dotted.path>=<value>` and can be repeated or separated by
commas:

```shell
cargo run -- --overrides server.port=9090,features.cache=true
```

Values are parsed as JSON scalars first, preserving booleans and numbers;
otherwise they remain strings. Only the first `=` is a separator, and the last
value for the same path wins. The resulting type must match the Rust field.

Environment variables use `${env:NAME}` or `${env:NAME:fallback}` inside any
string. A missing variable without a fallback stops loading.

JSON is always supported. TOML uses the default `toml` feature; YAML and INI
use the `yaml` and `ini` features. Formats can be mixed between profiles. INI
supports only string keys and values, not sequences.

### Secondary API

`#[profiled_config]` supports synchronous and asynchronous `main` functions.
Its only option is a synchronous hook called before loading:

```rust
#[profiled_config(before_load = initialize_logging)]
fn main(config: AppConfig) {
    // ...
}
```

Keep it above runtime attributes such as `#[tokio::main]`. The macro is only a
convenience; without the `macros` feature, use
`let config: AppConfig = profiled_config::load_config!();` directly.

## Complete example

```toml
# config/default.toml
app_name = "my-service"
port = 8080

[database]
url = "${env:DATABASE_URL:postgres://localhost/my-service}"
```

```rust
use profiled_config::profiled_config;
use serde::Deserialize;

#[derive(Deserialize)]
struct AppConfig {
    app_name: String,
    port: u16,
    database: DatabaseConfig,
}

#[derive(Deserialize)]
struct DatabaseConfig {
    url: String,
}

#[profiled_config]
fn main(config: AppConfig) {
    println!("Starting {} on port {}", config.app_name, config.port);
}
```

An optional `config/development.json` can contain only what changes:

```json
{
  "port": 3000
}
```

```shell
cargo run -- --profiles development --overrides port=9090
```

## License

[MIT](LICENCE). Inspired by Spring profiles; not affiliated with Spring.
