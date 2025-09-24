use colored::Colorize;
use regex::Regex;

pub fn highlight_matches(text: &str, pattern: &Regex) -> String {
    let mut result = String::new();
    let mut last_end = 0;

    for mat in pattern.find_iter(text) {
        result.push_str(&text[last_end..mat.start()]);
        result.push_str(&text[mat.start()..mat.end()].on_yellow().black().to_string());
        last_end = mat.end();
    }

    result.push_str(&text[last_end..]);
    result
}

pub fn colorize_log_level(text: &str) -> String {
    let error_pattern = Regex::new(r"(?i)\b(error|err|fatal|panic)\b").unwrap();
    let warn_pattern = Regex::new(r"(?i)\b(warn|warning)\b").unwrap();
    let info_pattern = Regex::new(r"(?i)\b(info|information)\b").unwrap();
    let debug_pattern = Regex::new(r"(?i)\b(debug|trace)\b").unwrap();

    let mut result = text.to_string();

    if error_pattern.is_match(&result) {
        result = error_pattern.replace_all(&result, |caps: &regex::Captures| {
            caps[0].bright_red().to_string()
        }).to_string();
    }

    if warn_pattern.is_match(&result) {
        result = warn_pattern.replace_all(&result, |caps: &regex::Captures| {
            caps[0].bright_yellow().to_string()
        }).to_string();
    }

    if info_pattern.is_match(&result) {
        result = info_pattern.replace_all(&result, |caps: &regex::Captures| {
            caps[0].bright_green().to_string()
        }).to_string();
    }

    if debug_pattern.is_match(&result) {
        result = debug_pattern.replace_all(&result, |caps: &regex::Captures| {
            caps[0].dimmed().to_string()
        }).to_string();
    }

    result
}

pub fn format_json_field(json_str: &str, field: &str) -> Option<String> {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
        let parts: Vec<&str> = field.split('.').collect();
        let mut current = &value;

        for part in parts {
            match current.get(part) {
                Some(v) => current = v,
                None => return None,
            }
        }

        Some(match current {
            serde_json::Value::String(s) => s.clone(),
            _ => current.to_string(),
        })
    } else {
        None
    }
}