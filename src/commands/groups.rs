use anyhow::Result;
use colored::Colorize;
use regex::Regex;
use crate::aws::client::CloudWatchClient;
use indicatif::{ProgressBar, ProgressStyle};

pub async fn run(
    client: CloudWatchClient,
    filter: Option<String>,
) -> Result<()> {
    println!("{}", "Fetching log groups...".bright_blue().bold());

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    spinner.set_message("Loading log groups...");

    let groups = client.list_log_groups(None).await?;

    spinner.finish_and_clear();

    let filtered_groups = if let Some(ref pattern) = filter {
        let regex = Regex::new(pattern)?;
        groups.into_iter()
            .filter(|g| regex.is_match(g))
            .collect::<Vec<_>>()
    } else {
        groups
    };

    if filtered_groups.is_empty() {
        println!("{}", "No log groups found".yellow());
        return Ok(());
    }

    println!("{} {} log groups:\n",
        "Found".bright_green().bold(),
        filtered_groups.len().to_string().bright_yellow().bold()
    );

    for group in &filtered_groups {
        println!("  {} {}",
            "→".bright_cyan(),
            group.bright_white()
        );
    }

    println!("\n{} Use {} to tail a specific log group",
        "Tip:".bright_magenta().bold(),
        "cwl tail <log-group>".bright_white().italic()
    );

    Ok(())
}