use std::collections::{HashMap, BTreeMap};
use serde_json::Value;
use colored::Colorize;

pub struct ColumnInfo {
    pub name: String,
    pub frequency: usize,
    pub max_width: usize,
}

pub struct FormattedOutput {
    pub columns: Vec<ColumnInfo>,
    pub rows: Vec<Vec<String>>,
}

pub fn flatten_json_to_columns(value: &Value, prefix: &str) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();

    match value {
        Value::Object(map) => {
            for (key, val) in map {
                let new_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };

                match val {
                    Value::Object(_) | Value::Array(_) => {
                        let nested = flatten_json_to_columns(val, &new_prefix);
                        result.extend(nested);
                    }
                    Value::Null => {
                        result.insert(new_prefix, String::new());
                    }
                    _ => {
                        result.insert(new_prefix, format_json_value(val));
                    }
                }
            }
        }
        Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                let new_prefix = format!("{}[{}]", prefix, i);
                let nested = flatten_json_to_columns(val, &new_prefix);
                result.extend(nested);
            }
        }
        _ => {
            result.insert(prefix.to_string(), format_json_value(value));
        }
    }

    result
}

fn format_json_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        Value::Object(_) | Value::Array(_) => serde_json::to_string(value).unwrap_or_default(),
    }
}

pub fn analyze_json_logs(logs: &[String]) -> FormattedOutput {
    let mut column_frequency: HashMap<String, usize> = HashMap::new();
    let mut column_max_width: HashMap<String, usize> = HashMap::new();
    let mut all_rows: Vec<BTreeMap<String, String>> = Vec::new();

    for log_line in logs.iter() {
        let (timestamp, log_group, json_str) = parse_log_line(log_line);

        let mut row = BTreeMap::new();
        row.insert("timestamp".to_string(), timestamp.clone());
        row.insert("log_group".to_string(), log_group.clone());

        column_max_width.entry("timestamp".to_string())
            .and_modify(|w| *w = (*w).max(timestamp.len()))
            .or_insert(timestamp.len());
        column_max_width.entry("log_group".to_string())
            .and_modify(|w| *w = (*w).max(log_group.len()))
            .or_insert(log_group.len());

        if let Ok(json_value) = serde_json::from_str::<Value>(&json_str) {
            let flattened = flatten_json_to_columns(&json_value, "");

            for (key, value) in &flattened {
                column_frequency.entry(key.clone())
                    .and_modify(|count| *count += 1)
                    .or_insert(1);

                column_max_width.entry(key.clone())
                    .and_modify(|w| *w = (*w).max(value.len().min(100)))
                    .or_insert(value.len().min(100));

                row.insert(key.clone(), value.clone());
            }
        }

        all_rows.push(row);
    }

    column_frequency.insert("timestamp".to_string(), logs.len());
    column_frequency.insert("log_group".to_string(), logs.len());

    let mut columns: Vec<ColumnInfo> = Vec::new();

    columns.push(ColumnInfo {
        name: "timestamp".to_string(),
        frequency: logs.len(),
        max_width: column_max_width.get("timestamp").copied().unwrap_or(9).max(9),
    });

    columns.push(ColumnInfo {
        name: "log_group".to_string(),
        frequency: logs.len(),
        max_width: column_max_width.get("log_group").copied().unwrap_or(9).max(9),
    });

    let mut other_columns: Vec<ColumnInfo> = column_frequency
        .iter()
        .filter(|(k, _)| k.as_str() != "timestamp" && k.as_str() != "log_group")
        .map(|(name, frequency)| ColumnInfo {
            name: name.clone(),
            frequency: *frequency,
            max_width: column_max_width.get(name).copied().unwrap_or(0).max(name.len()),
        })
        .collect();

    other_columns.sort_by(|a, b| {
        b.frequency.cmp(&a.frequency)
            .then_with(|| a.name.cmp(&b.name))
    });

    columns.extend(other_columns);

    let mut rows = Vec::new();
    for row_map in all_rows {
        let mut row_values = Vec::new();
        for col in &columns {
            let value = row_map.get(&col.name).cloned().unwrap_or_default();
            row_values.push(truncate_string(&value, 100));
        }
        rows.push(row_values);
    }

    FormattedOutput { columns, rows }
}

fn parse_log_line(line: &str) -> (String, String, String) {
    if let Some(ts_end) = line.find(']') {
        if line.starts_with('[') {
            let timestamp = line[1..ts_end].to_string();
            let rest = &line[ts_end + 1..].trim();

            if rest.starts_with('[') {
                if let Some(group_end) = rest.find(']') {
                    let log_group = rest[1..group_end].to_string();
                    let json_str = rest[group_end + 1..].trim().to_string();
                    return (timestamp, log_group, json_str);
                }
            }

            return (timestamp, String::new(), rest.to_string());
        }
    }

    (String::new(), String::new(), line.to_string())
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

pub fn print_formatted_table(output: &FormattedOutput) {
    let mut header = Vec::new();
    for col in &output.columns {
        header.push(format!("{:width$}", col.name.bright_cyan().bold(), width = col.max_width));
    }
    println!("{}", header.join(" │ "));

    let separator: Vec<String> = output.columns.iter()
        .map(|col| "─".repeat(col.max_width))
        .collect();
    println!("{}", separator.join("─┼─").bright_black());

    for row in &output.rows {
        let mut formatted_row = Vec::new();
        for (i, value) in row.iter().enumerate() {
            let width = output.columns[i].max_width;
            let formatted_value = if i < 2 {
                format!("{:width$}", value.bright_blue(), width = width)
            } else if value.is_empty() {
                format!("{:width$}", "", width = width)
            } else {
                format!("{:width$}", value, width = width)
            };
            formatted_row.push(formatted_value);
        }
        println!("{}", formatted_row.join(" │ "));
    }

    println!("\n{} columns, {} rows",
        output.columns.len().to_string().bright_yellow(),
        output.rows.len().to_string().bright_yellow()
    );
}