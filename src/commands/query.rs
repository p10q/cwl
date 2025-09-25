use anyhow::Result;
use colored::Colorize;
use regex::Regex;
use chrono::{DateTime, Utc};
use crate::aws::client::CloudWatchClient;
use crate::utils::{format, time, json_formatter};
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;

pub async fn run(
    client: CloudWatchClient,
    log_group: String,
    since: Option<String>,
    start: Option<String>,
    end: Option<String>,
    filter: Option<String>,
    limit: usize,
    formatted: bool,
) -> Result<()> {
    println!("{} {}",
        "Querying logs from:".bright_blue().bold(),
        log_group.bright_yellow()
    );

    let (start_time, end_time) = time::parse_time_range(since, start, end)?;

    if let Some(start_ts) = start_time {
        let dt = DateTime::<Utc>::from_timestamp_millis(start_ts)
            .unwrap_or_else(|| Utc::now());
        println!("{} {}",
            "Start time:".bright_blue().bold(),
            dt.format("%Y-%m-%d %H:%M:%S").to_string().bright_yellow()
        );
    }

    if let Some(end_ts) = end_time {
        let dt = DateTime::<Utc>::from_timestamp_millis(end_ts)
            .unwrap_or_else(|| Utc::now());
        println!("{} {}",
            "End time:".bright_blue().bold(),
            dt.format("%Y-%m-%d %H:%M:%S").to_string().bright_yellow()
        );
    }

    if let Some(ref pattern) = filter {
        println!("{} {}",
            "Filter pattern:".bright_blue().bold(),
            pattern.bright_yellow()
        );
    }

    println!("{} {}",
        "Max events:".bright_blue().bold(),
        if limit == usize::MAX {
            "unlimited (fetching all in time range)".bright_yellow().to_string()
        } else {
            limit.to_string().bright_yellow().to_string()
        }
    );

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg} {prefix}")
            .unwrap()
    );
    spinner.set_message("Fetching log events...");

    let events = client.get_log_events(
        &log_group,
        start_time,
        end_time,
        filter.as_deref(),
        if limit == usize::MAX { None } else { Some(limit) },
    ).await?;

    spinner.finish_and_clear();

    if events.is_empty() {
        println!("{}", "No log events found matching criteria".yellow());
        return Ok(());
    }

    println!("{} {} events\n",
        "Found".bright_green().bold(),
        events.len().to_string().bright_yellow().bold()
    );

    if formatted {
        let mut log_lines = Vec::new();

        for event in &events {
            if let Some(ref message) = event.message {
                let timestamp = event.timestamp.map(|ts| {
                    DateTime::<Utc>::from_timestamp_millis(ts)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S.%3f").to_string())
                        .unwrap_or_else(|| "Unknown time".to_string())
                }).unwrap_or_else(|| "Unknown time".to_string());

                let stream_name = event.log_stream_name
                    .as_ref()
                    .map(|s| s.clone())
                    .unwrap_or_default();

                let mut parsed_message = message.clone();

                if let Ok(json_value) = serde_json::from_str::<Value>(message) {
                    parsed_message = serde_json::to_string(&json_value).unwrap_or_else(|_| message.clone());
                }

                let log_line = format!("[{}] [{}] {}", timestamp, stream_name, parsed_message);
                log_lines.push(log_line);
            }
        }

        let output = json_formatter::analyze_json_logs(&log_lines);
        json_formatter::print_formatted_table(&output);
    } else {
        let regex_pattern = filter.as_ref()
            .map(|f| Regex::new(&regex::escape(f)))
            .transpose()?;

        for event in &events {
            if let Some(ref message) = event.message {
                let timestamp = event.timestamp.map(|ts| {
                    DateTime::<Utc>::from_timestamp_millis(ts)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string())
                        .unwrap_or_else(|| "Unknown time".to_string())
                }).unwrap_or_else(|| "Unknown time".to_string());

                let stream_name = event.log_stream_name
                    .as_ref()
                    .map(|s| format!("[{}]", s.cyan()))
                    .unwrap_or_default();

                let formatted_message = if regex_pattern.is_some() {
                    format::highlight_matches(&message, regex_pattern.as_ref().unwrap())
                } else {
                    message.clone()
                };

                println!("[{}] {} {}",
                    timestamp.bright_blue(),
                    stream_name,
                    formatted_message
                );
            }
        }
    }

    println!("\n{} {} total events displayed",
        "✓".bright_green().bold(),
        events.len().to_string().bright_yellow()
    );

    Ok(())
}