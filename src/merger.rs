use std::collections::BTreeMap;

use serde_value::Value;

pub fn merge_values(files_content: &[Value]) -> Value {
    files_content
        .iter()
        .fold(Value::Map(BTreeMap::new()), |acc, content| {
            merge_two_x_two(&acc, content)
        })
}

fn merge_two_x_two(a: &Value, b: &Value) -> Value {
    match (a, b) {
        (Value::Map(a_map), Value::Map(b_map)) => {
            let mut acc = a_map.clone();
            for (key, value) in b_map.iter() {
                let merged_value = match (a_map.get(key), value) {
                    (Some(Value::Map(a_map)), Value::Map(b_map)) => {
                        merge_two_x_two(&Value::Map(a_map.clone()), &Value::Map(b_map.clone()))
                    }
                    _ => value.clone(),
                };
                acc.insert(key.clone(), merged_value);
            }
            Value::Map(acc)
        }
        _ => b.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn recursively_merges_maps_and_replaces_other_values() {
        let default = map([
            (
                "database",
                map([("host", string("localhost")), ("port", Value::U16(5432))]),
            ),
            ("servers", Value::Seq(vec![string("primary")])),
        ]);
        let profile = map([
            ("database", map([("host", string("database.internal"))])),
            ("servers", Value::Seq(vec![string("profile")])),
        ]);

        let merged = merge_values(&[default, profile]);
        let expected = map([
            (
                "database",
                map([
                    ("host", string("database.internal")),
                    ("port", Value::U16(5432)),
                ]),
            ),
            ("servers", Value::Seq(vec![string("profile")])),
        ]);

        assert_eq!(merged, expected);
    }

    #[test]
    fn later_root_value_wins() {
        assert_eq!(
            merge_values(&[string("default"), string("profile")]),
            string("profile")
        );
    }
}
