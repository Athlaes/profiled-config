use std::env;

use toml::{Table, Value};

pub fn compute_table(table: &Table) -> Table {
    let mut computed_table = table.clone();
    for (key, value) in table.iter() {
        let computed_value = compute_any(value);
        computed_table.insert(key.clone(), computed_value);
    }
    computed_table
}

fn compute_any(value: &Value) -> Value {
    match value {
        Value::String(val) => Value::String(compute_string(val)),
        Value::Array(arr) => Value::Array(compute_array(arr)),
        Value::Table(tab) => Value::Table(compute_table(tab)),
        _ => value.clone(),
    }
}

fn compute_string(val: &str) -> String {
    let regxp = regex::Regex::new(r"\$\{([:/a-zA-Z0-9_-]*)\}")
        .unwrap_or_else(|e| panic!("Couldn't parse regex: {}", e));
    let mut value = val.to_string();
    for capture in regxp.captures_iter(val) {
        let var_name = &capture[1];
        let var_value = compute_env_var(var_name);
        value = value.replace(&capture[0], var_value.as_str());
    }
    value
}

fn compute_array(arr: &[Value]) -> Vec<Value> {
    arr.iter().map(compute_any).collect()
}

fn compute_env_var(var_name: &str) -> String {
    let splitted_values = var_name.split(':').collect::<Vec<&str>>();
    if splitted_values.len() >= 2 {
        return env::var(splitted_values[0]).unwrap_or(splitted_values[1..].join(":"));
    }
    env::var(splitted_values[0]).unwrap_or_else(|e| panic!("Couldn't find env var: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOML_VALUE: &str = r#"
        [profile]
        name = "default"

        [clients.aia]
        url = "${SERVICE_PROTOCOL}://${SERVICE_HOST}:${SERVICE_PORT}"
        "#;

    const MISSING_VAR_TOML_VALUE: &str = r#"
        [profile]
        name = "${MISSING_ENV_VAR}"

        [clients.aia]
        url = "${SERVICE_PROTOCOL}://${SERVICE_HOST}:${SERVICE_PORT}"
        "#;

    const MISSING_VAR_WITH_DEFAULT_TOML_VALUE: &str = r#"
        [profile]
        name = "${MISSING_ENV_VAR:test}"

        [clients.aia]
        url = "${SERVICE_PROTOCOL}://${SERVICE_HOST}:${SERVICE_PORT}"

        [database]
        url = "${DATABASE_URL:postres://localhost:5432}"
        "#;

    fn init_env() {
        unsafe {
            env::set_var("SERVICE_PROTOCOL", "http");
            env::set_var("SERVICE_HOST", "localhost");
            env::set_var("SERVICE_PORT", "8080");
        }
    }

    #[test]
    fn success_compute_table() {
        init_env();
        let value: Table = toml::from_str(TOML_VALUE).unwrap();
        let result = compute_table(&value);
        assert_eq!(
            result.to_string(),
            "[clients.aia]\nurl = \"http://localhost:8080\"\n\n[profile]\nname = \"default\"\n"
        );
    }

    #[test]
    fn success_missing_var_with_default() {
        init_env();
        let value: Table = toml::from_str(MISSING_VAR_WITH_DEFAULT_TOML_VALUE).unwrap();
        let result = compute_table(&value);
        assert_eq!(
            result.to_string(),
            "[clients.aia]\nurl = \"http://localhost:8080\"\n\n[database]\nurl = \"postres://localhost:5432\"\n\n[profile]\nname = \"test\"\n"
        );
    }

    #[test]
    #[should_panic]
    fn panic_compute_table_on_missing_env_var() {
        init_env();
        let value: Table = toml::from_str(MISSING_VAR_TOML_VALUE).unwrap();
        compute_table(&value);
    }
}
