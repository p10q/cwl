use anyhow::{Result, Context, bail};
use chrono::{DateTime, Duration, Utc};
use regex::Regex;

pub fn parse_time_range(
    since: Option<String>,
    start: Option<String>,
    end: Option<String>,
) -> Result<(Option<i64>, Option<i64>)> {
    let mut start_time = None;
    let mut end_time = None;

    if let Some(since_str) = since {
        let duration = parse_duration(&since_str)
            .context("Invalid duration format. Use formats like '1h', '30m', '2d'")?;
        start_time = Some(Utc::now().timestamp_millis() - duration.num_milliseconds());
        end_time = Some(Utc::now().timestamp_millis());
    } else {
        if let Some(start_str) = start {
            start_time = Some(parse_timestamp(&start_str)
                .context("Invalid start time format")?);
        }

        if let Some(end_str) = end {
            end_time = Some(parse_timestamp(&end_str)
                .context("Invalid end time format")?);
        }
    }

    if start_time.is_none() && end_time.is_none() {
        start_time = Some(Utc::now().timestamp_millis() - 3600000);
        end_time = Some(Utc::now().timestamp_millis());
    }

    Ok((start_time, end_time))
}

pub fn parse_duration(s: &str) -> Result<Duration> {
    let re = Regex::new(r"^(\d+)([smhd])$")?;

    if let Some(captures) = re.captures(s) {
        let value: i64 = captures[1].parse()?;
        let unit = &captures[2];

        let duration = match unit {
            "s" => Duration::seconds(value),
            "m" => Duration::minutes(value),
            "h" => Duration::hours(value),
            "d" => Duration::days(value),
            _ => bail!("Invalid duration unit: {}", unit),
        };

        Ok(duration)
    } else {
        bail!("Invalid duration format: {}. Use formats like '1h', '30m', '2d'", s)
    }
}

pub fn parse_timestamp(s: &str) -> Result<i64> {
    if let Ok(ts) = s.parse::<i64>() {
        if ts > 1_000_000_000_000 {
            Ok(ts)
        } else {
            Ok(ts * 1000)
        }
    } else {
        let formats = [
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%dT%H:%M:%S",
            "%Y-%m-%dT%H:%M:%SZ",
            "%Y-%m-%d %H:%M:%S%.f",
            "%Y-%m-%dT%H:%M:%S%.f",
            "%Y-%m-%dT%H:%M:%S%.fZ",
        ];

        for format in &formats {
            if let Ok(dt) = DateTime::parse_from_str(s, format) {
                return Ok(dt.timestamp_millis());
            }
        }

        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
            return Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc).timestamp_millis());
        }

        bail!("Could not parse timestamp: {}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert!(parse_duration("5s").is_ok());
        assert!(parse_duration("30m").is_ok());
        assert!(parse_duration("2h").is_ok());
        assert!(parse_duration("1d").is_ok());
        assert!(parse_duration("invalid").is_err());
    }

    #[test]
    fn test_parse_timestamp() {
        assert!(parse_timestamp("1234567890").is_ok());
        assert!(parse_timestamp("1234567890000").is_ok());
        assert!(parse_timestamp("2024-01-01 12:00:00").is_ok());
        assert!(parse_timestamp("2024-01-01T12:00:00Z").is_ok());
    }
}