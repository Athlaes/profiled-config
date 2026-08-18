use std::collections::BTreeMap;

use serde_value::Value;

use crate::{parser::ConfigValueParser, resolver};

pub fn process_any(value: &Value) -> Value {
    match value {
        Value::String(val) => Value::String(process_string(val)),
        Value::Seq(arr) => Value::Seq(process_array(arr)),
        Value::Map(tab) => Value::Map(process_table(tab)),
        _ => value.clone(),
    }
}

fn process_table(table: &BTreeMap<Value, Value>) -> BTreeMap<Value, Value> {
    let mut processd_table = table.clone();
    for (key, value) in table.iter() {
        let processd_value = process_any(value);
        processd_table.insert(key.clone(), processd_value);
    }
    processd_table
}

fn process_string(val: &str) -> String {
    resolver::resolve(ConfigValueParser::new(val).parse_value().unwrap())
}

fn process_array(arr: &[Value]) -> Vec<Value> {
    arr.iter().map(process_any).collect()
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, MutexGuard};

    use super::*;
    use std::env;

    static ENVIRONMENT: Mutex<()> = Mutex::new(());
    const MISSING_ENV_VAR: &str = "PROFILED_CONFIG_TEST_MISSING_ENV_VAR";
    const DATABASE_URL: &str = "PROFILED_CONFIG_TEST_DATABASE_URL";

    fn string(value: &str) -> Value {
        Value::String(value.to_string())
    }

    fn map(entries: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
        Value::Map(
            entries
                .into_iter()
                .map(|(key, value)| (string(key), value))
                .collect(),
        )
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

    fn init_env() -> MutexGuard<'static, ()> {
        let guard = ENVIRONMENT
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
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

    #[test]
    fn success_process_any() {
        let _environment = init_env();
        let value = configuration("default", None);
        let result = process_any(&value);
        assert_eq!(
            nested_string(&result, &["clients", "aia", "url"]),
            "http://localhost:8080"
        );
    }

    #[test]
    fn success_process_json_path() {
        let _environment = init_env();
        let value = configuration(
            "default",
            Some(
                "jdbc:postgresql://${env:JSON_DATABASE_URL(jsonpath:$.database.credentials.username)}:${env:JSON_DATABASE_URL(jsonpath:$.database.credentials.password)}@${env:JSON_DATABASE_URL(jsonpath:$.database.host)}/${env:JSON_DATABASE_URL(jsonpath:$.database.db_name)}",
            ),
        );
        let result = process_any(&value);
        assert_eq!(nested_string(&result, &["profile", "name"]), "default");
        assert_eq!(
            nested_string(&result, &["database", "url"]),
            "jdbc:postgresql://root:root@localhost:5432/dummy_db"
        );
    }

    #[test]
    fn success_missing_var_with_default() {
        let _environment = init_env();
        let value = configuration(
            "${env:PROFILED_CONFIG_TEST_MISSING_ENV_VAR:test}",
            Some("${env:PROFILED_CONFIG_TEST_DATABASE_URL:postgres://localhost:5432}"),
        );
        let result = process_any(&value);
        assert_eq!(nested_string(&result, &["profile", "name"]), "test");
        assert_eq!(
            nested_string(&result, &["database", "url"]),
            "postgres://localhost:5432"
        );
    }

    #[test]
    #[should_panic]
    fn panic_process_any_on_missing_env_var() {
        let _environment = init_env();
        let value = configuration("${env:PROFILED_CONFIG_TEST_MISSING_ENV_VAR}", None);
        process_any(&value);
    }
}
