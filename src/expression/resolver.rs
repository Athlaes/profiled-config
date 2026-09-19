use std::collections::BTreeMap;

use serde_value::Value;
use thiserror::Error;

use crate::expression::provider::registry::ProviderRegistry;

use super::{
    parser::{ConfigValueParser, ConfigValueParts, ExpressionParserError},
    provider::ProviderError,
    selector::{self, Selector, SelectorError},
};

#[derive(Debug, Error)]
#[error("Couldn't resolve variable '{path}' : {cause}")]
pub struct ResolveError {
    pub path: String,
    #[source]
    pub cause: ResolverError,
}

#[derive(Debug, Error)]
pub enum ResolverError {
    #[error("Selection error: {0}")]
    Selection(#[from] SelectorError),
    #[error("Expression parse error: {0}")]
    ExpressionParse(#[from] ExpressionParserError),
    #[error("Provide error: {0}")]
    Provide(#[from] ProviderError),
    #[error("Unexpected empty value with no default for provider '{provider}', key '{key}'")]
    MissingDefaultValue { provider: String, key: String },
}

pub struct ExpressionResolver<'a> {
    pr: &'a ProviderRegistry,
}

impl<'a> ExpressionResolver<'a> {
    pub fn new(pr: &'a ProviderRegistry) -> Self {
        Self { pr }
    }

    pub async fn process(&self, value: &Value) -> Result<Value, Vec<ResolveError>> {
        let path = vec![];
        self.process_any(value, &path).await
    }

    async fn process_any(&self, value: &Value, path: &[String]) -> Result<Value, Vec<ResolveError>> {
        match value {
            Value::String(val) => Ok(Value::String(
                self.process_string(val, path).await.map_err(|err| vec![err])?,
            )),
            Value::Seq(arr) => Ok(Value::Seq(Box::pin(self.process_array(arr, path)).await?)),
            Value::Map(tab) => Ok(Value::Map(Box::pin(self.process_table(tab, path)).await?)),
            _ => Ok(value.clone()),
        }
    }

    async fn process_table(
        &self,
        table: &BTreeMap<Value, Value>,
        path: &[String],
    ) -> Result<BTreeMap<Value, Value>, Vec<ResolveError>> {
        let mut processed_table = table.clone();
        let mut errors = Vec::new();
        for (key, value) in table.iter() {
            let mut child_path = path.to_vec();
            if let Value::String(p) = key {
                child_path.push(p.to_owned());
            }

            match self.process_any(value, &child_path).await {
                Ok(processed_value) => {
                    processed_table.insert(key.clone(), processed_value);
                }
                Err(err) => {
                    errors.extend(err);
                }
            }
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(processed_table)
    }

    async fn process_array(&self, arr: &[Value], path: &[String]) -> Result<Vec<Value>, Vec<ResolveError>> {
        let mut processed_array = Vec::new();
        let mut errors = Vec::new();
        for (index, value) in arr.iter().enumerate() {
            let mut child_path = path.to_vec();
            child_path.push(format!("{index}"));
            match self.process_any(value, &child_path).await {
                Ok(processed_value) => {
                    processed_array.push(processed_value);
                }
                Err(err) => {
                    errors.extend(err);
                }
            }
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(processed_array)
    }

    async fn process_string(&self, val: &str, path: &[String]) -> Result<String, ResolveError> {
        self.resolve(val).await.map_err(|err| ResolveError {
            path: path.join("."),
            cause: err,
        })
    }

    async fn resolve(&self, initial_value: &str) -> Result<String, ResolverError> {
        let mut result = String::new();
        let parsed_value = ConfigValueParser::new(initial_value).parse_value()?;
        for part in &parsed_value.parts {
            match &part {
                ConfigValueParts::Literal(str) => {
                    result.push_str(str);
                }
                ConfigValueParts::Expression(exp) => {
                    let provider = self.pr.get_provider(&exp.provider)?;
                    let expr_value = provider.resolve(&exp.key).await;
                    match expr_value {
                        Ok(value) => {
                            let mut tmp_val = value.clone();
                            if let Some(selector) = &exp.selector {
                                let selected_selector = selector::get_selector(selector.kind.as_str())?;
                                tmp_val = selected_selector.select(&value, &selector.query)?;
                            }
                            if tmp_val.is_empty() {
                                result.push_str(&exp.default.clone().ok_or(ResolverError::MissingDefaultValue {
                                    provider: exp.provider.clone(),
                                    key: exp.key.clone(),
                                })?);
                            } else {
                                result.push_str(&tmp_val);
                            }
                        }
                        Err(err) => {
                            result.push_str(&exp.default.clone().ok_or(err)?);
                        }
                    }
                }
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::{Mutex, MutexGuard};

    use crate::expression::provider::{EnvProvider, Provider, ProviderActivation, registry::ProviderRegistration};

    use super::*;
    use std::env;

    static ENVIRONMENT: Mutex<()> = Mutex::const_new(());
    const MISSING_ENV_VAR: &str = "PROFILED_CONFIG_TEST_MISSING_ENV_VAR";
    const DATABASE_URL: &str = "PROFILED_CONFIG_TEST_DATABASE_URL";

    fn string(value: &str) -> Value {
        Value::String(value.to_string())
    }

    fn map(entries: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
        Value::Map(entries.into_iter().map(|(key, value)| (string(key), value)).collect())
    }

    fn configuration(profile_name: &str, database_url: Option<&str>) -> Value {
        let mut entries = vec![
            ("profile", map([("name", string(profile_name))])),
            (
                "clients",
                map([(
                    "aia",
                    map([(
                        "url",
                        string("${env:SERVICE_PROTOCOL}://${env:SERVICE_HOST}:${env:SERVICE_PORT}"),
                    )]),
                )]),
            ),
        ];
        if let Some(database_url) = database_url {
            entries.push(("database", map([("url", string(database_url))])));
        }
        map(entries)
    }

    fn nested_string<'a>(value: &'a Value, path: &[&str]) -> &'a str {
        let mut current = value;
        for key in path {
            let Value::Map(map) = current else {
                panic!("expected a map while resolving {path:?}");
            };
            current = map
                .get(&string(key))
                .unwrap_or_else(|| panic!("missing key {key} while resolving {path:?}"));
        }
        let Value::String(value) = current else {
            panic!("expected a string at {path:?}");
        };
        value
    }

    async fn init_env() -> MutexGuard<'static, ()> {
        let guard = ENVIRONMENT.lock().await;
        unsafe {
            env::set_var("SERVICE_PROTOCOL", "http");
            env::set_var("SERVICE_HOST", "localhost");
            env::set_var("SERVICE_PORT", "8080");
            env::set_var(
                "JSON_DATABASE_URL",
                "{\"database\": {\"host\": \"localhost:5432\", \"credentials\": {\"username\": \"root\", \"password\": \"root\"}, \"db_name\": \"dummy_db\"}}",
            );
            env::remove_var(MISSING_ENV_VAR);
            env::remove_var(DATABASE_URL);
        }
        guard
    }

    #[tokio::test]
    async fn success_process_any() {
        let _environment = init_env().await;
        let value = configuration("default", None);
        let mut pr = ProviderRegistry::new();
        pr.register_provider(
            &value,
            "env",
            &ProviderRegistration {
                factory: Box::new(EnvProvider::create),
                activation: ProviderActivation::Always,
            },
        )
        .await
        .unwrap();
        let resolver = ExpressionResolver::new(&pr);
        let result = resolver.process(&value).await.unwrap();
        assert_eq!(
            nested_string(&result, &["clients", "aia", "url"]),
            "http://localhost:8080"
        );
    }

    #[tokio::test]
    async fn success_process_json_path() {
        let _environment = init_env().await;
        let value = configuration(
            "default",
            Some(
                "jdbc:postgresql://${env:JSON_DATABASE_URL(jsonpath:$.database.credentials.username)}:${env:JSON_DATABASE_URL(jsonpath:$.database.credentials.password)}@${env:JSON_DATABASE_URL(jsonpath:$.database.host)}/${env:JSON_DATABASE_URL(jsonpath:$.database.db_name)}",
            ),
        );

        let mut pr = ProviderRegistry::new();
        pr.register_provider(
            &value,
            "env",
            &ProviderRegistration {
                factory: Box::new(EnvProvider::create),
                activation: ProviderActivation::Always,
            },
        )
        .await
        .unwrap();
        let resolver = ExpressionResolver::new(&pr);
        let result = resolver.process(&value).await.unwrap();
        assert_eq!(nested_string(&result, &["profile", "name"]), "default");

        assert_eq!(
            nested_string(&result, &["database", "url"]),
            "jdbc:postgresql://root:root@localhost:5432/dummy_db"
        );
    }

    #[tokio::test]
    async fn success_missing_var_with_default() {
        let _environment = init_env().await;
        let value = configuration(
            "${env:PROFILED_CONFIG_TEST_MISSING_ENV_VAR:test}",
            Some("${env:PROFILED_CONFIG_TEST_DATABASE_URL:postgres://localhost:5432}"),
        );

        let mut pr = ProviderRegistry::new();
        pr.register_provider(
            &value,
            "env",
            &ProviderRegistration {
                factory: Box::new(EnvProvider::create),
                activation: ProviderActivation::Always,
            },
        )
        .await
        .unwrap();
        let resolver = ExpressionResolver::new(&pr);
        let result = resolver.process(&value).await.unwrap();

        assert_eq!(nested_string(&result, &["profile", "name"]), "test");
        assert_eq!(
            nested_string(&result, &["database", "url"]),
            "postgres://localhost:5432"
        );
    }

    #[tokio::test]
    async fn missing_variable_fallback_preserves_surrounding_literals() {
        let _environment = init_env().await;
        let value = string("prefix-${env:PROFILED_CONFIG_TEST_MISSING_ENV_VAR:fallback}-suffix");

        let mut pr = ProviderRegistry::new();
        pr.register_provider(
            &value,
            "env",
            &ProviderRegistration {
                factory: Box::new(EnvProvider::create),
                activation: ProviderActivation::Always,
            },
        )
        .await
        .unwrap();
        let resolver = ExpressionResolver::new(&pr);
        let result = resolver.process(&value).await.unwrap();

        assert_eq!(result, string("prefix-fallback-suffix"));
    }

    #[tokio::test]
    async fn preserves_a_trailing_dollar_in_a_literal() {
        let value = string("price$");
        let mut pr = ProviderRegistry::new();
        pr.register_provider(
            &value,
            "env",
            &ProviderRegistration {
                factory: Box::new(EnvProvider::create),
                activation: ProviderActivation::Always,
            },
        )
        .await
        .unwrap();
        let resolver = ExpressionResolver::new(&pr);
        let result = resolver
            .process(&value)
            .await
            .expect("a literal ending with '$' should be processed");

        assert_eq!(result, string("price$"));
    }

    #[tokio::test]
    async fn returns_the_path_and_provider_error_for_a_missing_env_var() {
        let _environment = init_env().await;
        let value = configuration("${env:PROFILED_CONFIG_TEST_MISSING_ENV_VAR}", None);

        let mut pr = ProviderRegistry::new();
        pr.register_provider(
            &value,
            "env",
            &ProviderRegistration {
                factory: Box::new(EnvProvider::create),
                activation: ProviderActivation::Always,
            },
        )
        .await
        .unwrap();
        let resolver = ExpressionResolver::new(&pr);
        let errors = resolver
            .process(&value)
            .await
            .expect_err("a missing environment variable should be rejected");

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].path, "profile.name");
        assert!(matches!(
            &errors[0].cause,
            ResolverError::Provide(crate::expression::provider::ProviderError::VariableNotFound { key, .. })
                if key == MISSING_ENV_VAR
        ));
    }
}
