use anyhow::{Result, Context};
use aws_config::BehaviorVersion;
use aws_sdk_cloudwatchlogs::{
    Client,
    types::FilteredLogEvent,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct CloudWatchClient {
    pub client: Arc<Client>,
}

impl CloudWatchClient {
    pub async fn new(profile: Option<&str>, region: Option<&str>) -> Result<Self> {
        let mut config_loader = aws_config::defaults(BehaviorVersion::latest());

        if let Some(profile) = profile {
            config_loader = config_loader.profile_name(profile);
        }

        // Use provided region, or default to us-east-1
        let region_to_use = region.unwrap_or("us-east-1");
        config_loader = config_loader.region(aws_config::Region::new(region_to_use.to_string()));

        let config = config_loader.load().await;
        let client = Client::new(&config);

        Ok(Self {
            client: Arc::new(client),
        })
    }

    pub async fn list_log_groups(&self, prefix: Option<&str>) -> Result<Vec<String>> {
        let mut groups = Vec::new();
        let mut next_token = None;

        loop {
            let mut request = self.client.describe_log_groups();

            if let Some(prefix) = prefix {
                request = request.log_group_name_prefix(prefix);
            }

            if let Some(token) = next_token {
                request = request.next_token(token);
            }

            let response = request.send().await
                .context("Failed to list log groups")?;

            if let Some(log_groups) = response.log_groups {
                for group in log_groups {
                    if let Some(name) = group.log_group_name {
                        groups.push(name);
                    }
                }
            }

            next_token = response.next_token;
            if next_token.is_none() {
                break;
            }
        }

        Ok(groups)
    }

    pub async fn get_log_events(
        &self,
        log_group: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        filter_pattern: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<FilteredLogEvent>> {
        let mut events = Vec::new();
        let mut next_token = None;

        // CloudWatch API has a max of 10,000 events per request
        const MAX_EVENTS_PER_REQUEST: i32 = 10000;

        loop {
            let mut request = self.client.filter_log_events()
                .log_group_name(log_group);

            if let Some(start) = start_time {
                request = request.start_time(start);
            }

            if let Some(end) = end_time {
                request = request.end_time(end);
            }

            if let Some(pattern) = filter_pattern {
                request = request.filter_pattern(pattern);
            }

            // Calculate how many events to request in this batch
            let batch_limit = if let Some(user_limit) = limit {
                let remaining = user_limit.saturating_sub(events.len());
                if remaining == 0 {
                    break;
                }
                std::cmp::min(remaining as i32, MAX_EVENTS_PER_REQUEST)
            } else {
                MAX_EVENTS_PER_REQUEST
            };

            request = request.limit(batch_limit);

            if let Some(token) = next_token {
                request = request.next_token(token);
            }

            let response = request.send().await
                .context(format!("Failed to get log events for group: {}", log_group))?;

            if let Some(log_events) = response.events {
                events.extend(log_events);
            }

            // Check if we've reached the user-specified limit
            if let Some(user_limit) = limit {
                if events.len() >= user_limit {
                    // Truncate to exactly the limit requested
                    events.truncate(user_limit);
                    break;
                }
            }

            next_token = response.next_token;
            if next_token.is_none() {
                // No more pages available
                break;
            }
        }

        Ok(events)
    }

    pub async fn tail_log_events(
        &self,
        log_group: &str,
        filter_pattern: Option<&str>,
        mut callback: impl FnMut(FilteredLogEvent) -> Result<()>,
    ) -> Result<()> {
        let mut next_forward_token: Option<String> = None;
        let mut last_event_time = None;

        loop {
            let mut request = self.client.filter_log_events()
                .log_group_name(log_group);

            if let Some(pattern) = filter_pattern {
                request = request.filter_pattern(pattern);
            }

            if let Some(last_time) = last_event_time {
                request = request.start_time(last_time);
            } else {
                request = request.start_time(
                    chrono::Utc::now().timestamp_millis() - 60000
                );
            }

            if let Some(token) = &next_forward_token {
                request = request.next_token(token.clone());
            }

            let response = request.send().await?;

            if let Some(events) = response.events {
                for event in events {
                    if let Some(timestamp) = event.timestamp {
                        last_event_time = Some(timestamp + 1);
                    }
                    callback(event)?;
                }
            }

            next_forward_token = response.next_token;

            if next_forward_token.is_none() {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }
}