use toml::{Table, Value};

pub fn merge_values(files_content: &Vec<Table>) -> Table {
    files_content
        .iter()
        .fold(Table::new(), |acc, content| merge_two_x_two(&acc, &content))
}

fn merge_two_x_two(a: &Table, b: &Table) -> Table {
    let mut acc = a.clone();
    for (key, value) in b.iter() {
        let merged_value = match (a.get(key), value) {
            (Some(Value::Table(a_table)), Value::Table(b_table)) => {
                Value::Table(merge_two_x_two(a_table, b_table))
            }
            _ => value.clone(),
        };
        acc.insert(key.clone(), merged_value);
    }
    acc
}
