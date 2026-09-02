# Roadmap

## Completed

- [x] **Core configuration:** embedded profiles, TOML/JSON/YAML/INI support, environment expressions, and JSONPath selection.
- [x] **Runtime overrides:** override files, typed and nested CLI values, repeated `--overrides` arguments, deterministic precedence, and detection of ambiguous `overrides.*` files.
- [x] **Getting started documentation:** concise installation, profile, override, environment expression, format, and macro-free examples.

## Next: stabilize loading

- [x] Return structured errors throughout the loading pipeline instead of logging or ignoring failures & expose a public, fallible loading API.
- [ ] Clean up the internal loader, parser, resolver, and provider module boundaries.
- [ ] Support applications that own their Clap command-line parser.

## Then: prepare remote providers

- [ ] Add a two-phase bootstrap process for provider configuration & add Vault as the first optional provider behind a Cargo feature.

## Later

- [ ] Allow the runtime override file or path to be configured.
- [ ] Add asynchronous configuration loading.
- [ ] Introduce internal abstractions for value providers and configuration sources.
- [ ] Migrate the environment provider to the new abstractions.
- [ ] Automate changelog generation.
- [ ] Add configuration validation hooks.
- [ ] Evaluate Git, Redis, and Consul integrations.
- [ ] Add focused examples and compatibility documentation.

## Planned Vault usage

Enable the built-in provider:

```toml
profiled_config = { version = "...", features = ["macros", "vault"] }
```

Configure it in the embedded configuration, using the environment for
credentials:

```toml
[profiled_config.providers.vault]
address = "${env:VAULT_ADDR:http://127.0.0.1:8200}"
token = "${env:VAULT_TOKEN}"
mount = "secret"
kv_version = 2

[database]
username = "${vault:applications/my-service/username:app}"
password = "${vault:applications/my-service/password}"
```

The loader will bootstrap the Vault client from
`[profiled_config.providers.vault]`, fetch expressions after merging the
configuration layers, remove the reserved `profiled_config` section, and then
deserialize the application configuration. Referencing Vault without enabling
the feature will produce an explicit error.

XPath and dedicated YAMLPath support are deferred until concrete use cases justify them.
