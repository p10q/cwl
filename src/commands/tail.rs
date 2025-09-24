use anyhow::Result;
use colored::Colorize;
use regex::Regex;
use chrono::{DateTime, Utc};
use crate::aws::client::CloudWatchClient;
use crate::utils::format;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub async fn run(
    client: CloudWatchClient,
    log_group: String,
    follow: bool,
    filter: Option<String>,
    highlight: bool,
) -> Result<()> {
    println!("{} {}",
        "Tailing logs from:".bright_blue().bold(),
        log_group.bright_yellow()
    );

    if let Some(ref pattern) = filter {
        println!("{} {}",
            "Filter pattern:".bright_blue().bold(),
            pattern.bright_yellow()
        );
    }

    let regex_pattern = filter.as_ref()
        .map(|f| Regex::new(&regex::escape(f)))
        .transpose()?;

    if follow {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
        );
        spinner.set_message("Waiting for logs...");

        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();

        ctrlc::set_handler(move || {
            r.store(false, Ordering::SeqCst);
        })?;

        client.tail_log_events(&log_group, filter.as_deref(), |event| {
            if !running.load(Ordering::SeqCst) {
                return Ok(());
            }

            spinner.finish_and_clear();

            if let Some(message) = event.message {
                let timestamp = event.timestamp.map(|ts| {
                    DateTime::<Utc>::from_timestamp_millis(ts)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string())
                        .unwrap_or_else(|| "Unknown time".to_string())
                }).unwrap_or_else(|| "Unknown time".to_string());

                let formatted_message = if highlight && regex_pattern.is_some() {
                    format::highlight_matches(&message, regex_pattern.as_ref().unwrap())
                } else {
                    message
                };

                println!("[{}] {}",
                    timestamp.bright_blue(),
                    formatted_message
                );
            }

            Ok(())
        }).await?;
    } else {
        let events = client.get_log_events(
            &log_group,
            Some(chrono::Utc::now().timestamp_millis() - 300000),
            None,
            filter.as_deref(),
            Some(100),
        ).await?;

        if events.is_empty() {
            println!("{}", "No log events found".yellow());
        } else {
            for event in events {
                if let Some(message) = event.message {
                    let timestamp = event.timestamp.map(|ts| {
                        DateTime::<Utc>::from_timestamp_millis(ts)
                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string())
                            .unwrap_or_else(|| "Unknown time".to_string())
                    }).unwrap_or_else(|| "Unknown time".to_string());

                    let formatted_message = if highlight && regex_pattern.is_some() {
                        format::highlight_matches(&message, regex_pattern.as_ref().unwrap())
                    } else {
                        message
                    };

                    println!("[{}] {}",
                        timestamp.bright_blue(),
                        formatted_message
                    );
                }
            }
        }
    }

    Ok(())
}