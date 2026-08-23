# Roadmap

- [x] **Core configuration:** embedded profiles, TOML/JSON/YAML/INI support, environment expressions, JSONPath selection, and runtime overrides through files and CLI arguments.
- [ ] **Stabilize loading:** structured errors, validation, application-owned Clap compatibility, public loading API, configurable override file, and internal module cleanup.
- [ ] **Prepare remote providers:** two-phase bootstrap, asynchronous loading, internal abstractions for value providers and configuration sources, then migrate the environment provider to this model.
- [ ] **Add optional providers:** start with Vault, validate the design, then evaluate Git, Redis, and Consul integrations behind Cargo features.
- [ ] **Finish the project experience:** focused examples, compatibility documentation, and automated changelog generation.

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
