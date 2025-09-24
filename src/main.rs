mod aws;
mod commands;
mod config;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cwl")]
#[command(about = "CloudWatch Logs CLI - A powerful tool for interacting with AWS CloudWatch Logs")]
#[command(version)]
struct Cli {
    #[arg(short, long, global = true, help = "AWS profile to use")]
    profile: Option<String>,

    #[arg(short, long, global = true, help = "AWS region")]
    region: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Stream logs in real-time")]
    Tail {
        #[arg(help = "Log group name")]
        log_group: String,

        #[arg(short, long, help = "Follow log stream")]
        follow: bool,

        #[arg(short = 'f', long, help = "Filter pattern")]
        filter: Option<String>,

        #[arg(long, help = "Highlight matches")]
        highlight: bool,
    },

    #[command(about = "Query historical logs")]
    Query {
        #[arg(help = "Log group name")]
        log_group: String,

        #[arg(long, help = "Time since (e.g., 1h, 30m, 1d)")]
        since: Option<String>,

        #[arg(long, help = "Start time (ISO 8601 or Unix timestamp)")]
        start: Option<String>,

        #[arg(long, help = "End time (ISO 8601 or Unix timestamp)")]
        end: Option<String>,

        #[arg(short = 'f', long, help = "Filter pattern")]
        filter: Option<String>,

        #[arg(long, default_value = "1000", help = "Maximum number of events")]
        limit: usize,
    },

    #[command(about = "List available log groups")]
    Groups {
        #[arg(short = 'f', long, help = "Filter log groups by pattern")]
        filter: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let aws_client = aws::client::CloudWatchClient::new(
        cli.profile.as_deref(),
        cli.region.as_deref(),
    ).await?;

    match cli.command {
        Commands::Tail { log_group, follow, filter, highlight } => {
            commands::tail::run(aws_client, log_group, follow, filter, highlight).await?;
        },
        Commands::Query { log_group, since, start, end, filter, limit } => {
            commands::query::run(aws_client, log_group, since, start, end, filter, limit).await?;
        },
        Commands::Groups { filter } => {
            commands::groups::run(aws_client, filter).await?;
        },
    }

    Ok(())
}
