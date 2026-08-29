use std::collections::BTreeMap;

use serde_value::Value;

use crate::loader::LoaderError;

pub fn load(overrides: &[String]) -> Result<Option<Value>, LoaderError> {
    let mut root: BTreeMap<Value, Value> = BTreeMap::new();
    if overrides.is_empty() {
        return Ok(None);
    }

    for override_str in overrides {
        let (key, value) = override_str
            .split_once('=')
            .ok_or_else(|| LoaderError::ParseError(format!("Config arg {} missing '='", override_str)))?;
        let keys = key.split('.').collect::<Vec<&str>>();
        if keys.iter().any(|key| key.is_empty()) {
            return Err(LoaderError::ParseError(format!(
                "Config arg {} missing key",
                override_str
            )));
        }
        update_map(&mut root, &keys, parse_value(value));
    }

    Ok(Some(Value::Map(root)))
}

fn parse_value(value: &str) -> Value {
    serde_json::from_str(value).unwrap_or_else(|_| Value::String(value.to_string()))
}

fn update_map(root: &mut BTreeMap<Value, Value>, keys: &[&str], value: Value) -> () {
    let key = Value::String(keys[0].to_string());
    if keys.len() == 1 {
        root.insert(key, value);
        return;
    }

    let child = root.entry(key).or_insert_with(|| Value::Map(BTreeMap::new()));
    let Value::Map(child) = child else { return };
    update_map(child, &keys[1..], value);
}

#[cfg(test)]
mod tests {
    use crate::loader::LoaderError;

    use super::*;

    fn string(value: &str) -> Value {
        Value::String(value.to_string())
    }

    fn map(entries: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
        Value::Map(entries.into_iter().map(|(key, value)| (string(key), value)).collect())
    }

    fn overrides(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn loads_no_overrides_as_an_empty_map() {
        assert_eq!(load(&[]).expect("empty overrides should be valid"), Some(map([])));
    }

    #[test]
    fn loads_a_top_level_override() {
        let result = load(&overrides(&["name=profiled-config"]))
            .expect("valid override")
            .unwrap();

        assert_eq!(result, map([("name", string("profiled-config"))]));
    }

    #[test]
    fn loads_typed_json_values() {
        let result = load(&overrides(&["enabled=true", "retries=3"]))
            .expect("valid typed overrides")
            .unwrap();

        assert_eq!(
            result,
            map([("enabled", Value::Bool(true)), ("retries", Value::U64(3))])
        );
    }

    #[test]
    fn loads_nested_overrides_with_a_shared_parent() {
        let result = load(&overrides(&["database.host=localhost", "database.port=5432"]))
            .expect("valid nested overrides")
            .unwrap();

        assert_eq!(
            result,
            map([(
                "database",
                map([("host", string("localhost")), ("port", Value::U64(5432))]),
            )])
        );
    }

    #[test]
    fn preserves_equals_signs_in_the_value() {
        let result = load(&overrides(&["database.url=postgres://localhost?sslmode=require"]))
            .expect("valid override containing an equals sign")
            .unwrap();

        assert_eq!(
            result,
            map([(
                "database",
                map([("url", string("postgres://localhost?sslmode=require"))]),
            )])
        );
    }

    #[test]
    fn later_overrides_replace_earlier_values() {
        let result = load(&overrides(&["profile.name=default", "profile.name=development"]))
            .expect("valid duplicate override")
            .unwrap();

        assert_eq!(result, map([("profile", map([("name", string("development"))]))]));
    }

    #[test]
    fn rejects_an_override_without_an_equals_sign() {
        assert!(matches!(
            load(&overrides(&["profile.name"])),
            Err(LoaderError::ParseError(_))
        ));
    }

    #[test]
    fn rejects_an_override_without_a_key() {
        assert!(matches!(
            load(&overrides(&["=development"])),
            Err(LoaderError::ParseError(_))
        ));
    }
}
