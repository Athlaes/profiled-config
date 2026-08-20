# Roadmap

The following improvements are listed in priority order:

- [x] Support TOML, JSON, YAML, and INI configuration files
- [x] Replace the deprecated `serde_yaml` dependency with a maintained YAML parser
- [x] Add provider expressions: `${env:VAR:default}`, `${env:VAR(jsonpath:$.path):default}`
- [ ] Make configuration updatable after build through through CLI & files overrides
- [ ] Add provider expressions: `${vault:/secret/path:default}` and configure Vault with `[profiled_config.vault_provider]` before merge
- [ ] Load configuration files from Git, bootstrapped by `[profiled_config.git_provider]` before merge
- [ ] Load secrets and configuration Redis
- [ ] Load secrets and configuration Consul
- [ ] Improve error reporting and validation
- [ ] Add provider expressions: `${env:VAR:default}`, `${env:VAR(xpath:$.path):default}`
- [ ] Add provider expressions: `${env:VAR:default}`, `${env:VAR(yamlpath:$.path):default}`
- [ ] Provide more examples
- [ ] Add changelog tool in ci/cd
