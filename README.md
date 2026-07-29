# profiled-config

[![CI](https://github.com/Athlaes/profiled-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/Athlaes/profiled-rust/actions/workflows/ci.yml)

`profiled-config` is a small Rust library for typed, profile-based TOML
configuration, inspired by Spring profiles.

Configuration files are embedded in the application binary at compile time. At
startup, the library loads the default configuration, applies the selected
profiles in order, resolves environment variables, and deserializes the result
into a Rust type.

> [!WARNING]
> The project is still at an early stage. The API may change, and the crate is
> not yet recommended for production use.

## Features

- TOML configuration embedded directly in the application binary
- strongly typed configuration through Serde
- one or more profiles selected from the command line
- predictable, ordered profile merging
- environment variable interpolation
- optional fallback values for environment variables
- a `#[profiled_config]` attribute for synchronous and asynchronous entry
  points

## Installation

Add the library from crates.io and enable the `macros` feature:

```shell
cargo add profile_config --features macros
```

## Quick start

Create a `config/default.toml` file at the root of your application:

```toml
app_name = "my-service"
port = 8080

[database]
url = "${DATABASE_URL:postgres://localhost/my-service}"
```

Define a matching Rust type and add `#[profiled_config]` to `main`:

```rust
use profiled_config::profiled_config;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AppConfig {
    app_name: String,
    port: u16,
    database: DatabaseConfig,
}

#[derive(Debug, Deserialize)]
struct DatabaseConfig {
    url: String,
}

#[profiled_config]
fn main(config: AppConfig) {
    println!("Starting {} on port {}", config.app_name, config.port);
}
```

The annotated `main` function must take the configuration as its only argument.
`profiled-config` loads and deserializes the configuration before calling it.

Run the application normally to use `config/default.toml`:

```shell
cargo run
```

Because the `config` directory is embedded at compile time, configuration file
changes require rebuilding the application.

## Using profiles

Add one TOML file per profile next to `default.toml`:

```text
config/
├── default.toml
├── development.toml
└── local.toml
```

For example, `config/development.toml` can override only the values needed for
development:

```toml
port = 3000

[database]
url = "postgres://localhost/my-service-dev"
```

Select profiles with `--profiles` (or `-p`):

```shell
cargo run -- --profiles development
```

Multiple profiles can be provided as a comma-separated list:

```shell
cargo run -- --profiles development,local
```

The files are applied from left to right:

1. `default.toml`
2. `development.toml`
3. `local.toml`

Later profiles override earlier ones. TOML tables are merged recursively;
scalar values and arrays are replaced. A profile that cannot be loaded is
logged and skipped.

## Environment variables

Environment expressions can be used in any TOML string, including strings
nested in tables or arrays.

Use `${VARIABLE}` when the variable is required:

```toml
api_key = "${API_KEY}"
```

Use `${VARIABLE:fallback}` to provide a value when the variable is not set:

```toml
host = "${HOST:127.0.0.1}"
```

Loading fails if a required environment variable is missing.

## Macro attributes

The `#[profiled_config]` macro currently supports one optional attribute:
`before_load`.

### `before_load`

`before_load` specifies a synchronous, zero-argument function to call
immediately before command-line arguments and configuration files are loaded.
It is useful for setting up logging or preparing environment variables needed
by the configuration.

```rust
use profiled_config::profiled_config;
use serde::Deserialize;

#[derive(Deserialize)]
struct AppConfig {
    app_name: String,
}

fn initialize() {
    env_logger::init();
}

#[profiled_config(before_load = initialize)]
fn main(config: AppConfig) {
    println!("{}", config.app_name);
}
```

A qualified function path is also accepted:

```rust
#[profiled_config(before_load = startup::initialize)]
fn main(config: AppConfig) {
    // ...
}
```

No other macro attributes are currently available. Unknown attributes and
additional tokens produce a compile-time error.

## Async runtimes

The macro supports `async fn main` and preserves other attributes placed below
it. This allows runtime macros to be used on the generated entry point:

```rust
#[profiled_config(before_load = initialize)]
#[tokio::main]
async fn main(config: AppConfig) {
    // ...
}
```

Keep `#[profiled_config]` above the runtime attribute.

## Loading without the attribute macro

The configuration can also be loaded directly. In that case, the `macros`
feature is not required:

```shell
cargo add profiled_config
```

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct AppConfig {
    app_name: String,
}

fn main() {
    let config: AppConfig = profiled_config::load_config!();
    println!("{}", config.app_name);
}
```

## How loading works

At startup, `profiled-config`:

1. reads the embedded `config/default.toml`;
2. reads each selected `<profile>.toml` file;
3. merges the files in profile order;
4. resolves environment expressions;
5. deserializes the result into the type expected by `main`.

Invalid default configuration, unresolved required environment variables, and
deserialization errors stop the application with an error.

## Development and releases

Feature branches are merged into `develop`. Every push to `develop` produces a
source snapshot artifact using the version declared once in
`[workspace.package]`, such as `0.3.0-SNAPSHOT`. Both published crates inherit
that version, which remains unchanged throughout the development cycle. The
regular CI only reads it and never rewrites a manifest. Snapshots are retained
by GitHub Actions for 14 days; they do not create a Git tag, a GitHub Release,
or a crates.io version.

Merging `develop` into `main` removes the `-SNAPSHOT` suffix only in the release
checkout and creates the corresponding stable release (`0.3.0-SNAPSHOT`
becomes `0.3.0`). The workflow also injects the crates.io version of the local
proc-macro dependency in that checkout. Once the release is complete, it
increments the minor component, resets the patch to zero, and updates
`develop` once with the next workspace snapshot version (`0.4.0-SNAPSHOT`).
Patch increments are reserved for a future hotfix workflow.

Developers do not edit package versions during day-to-day work. The release
workflow creates the `X.Y.Z` tag and GitHub Release, then publishes the
proc-macro followed by the main crate.

## Contributing and feedback

Feedback, ideas, bug reports, and real-world use cases are very welcome. I hope
to keep improving this project and let it grow with the needs of the people
using it, so please feel free to open an issue or start a discussion—even a
small suggestion can help shape what comes next.

## License

This project is available under the [MIT License](LICENCE).

## Acknowledgements

The project is inspired by Spring's profile-based configuration model. It is
not affiliated with Spring and does not aim to reproduce its complete API.
