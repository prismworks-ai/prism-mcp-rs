//! Time and date utilities

use super::{UtilError, UtilResult};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Get current Unix timestamp in seconds
pub fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

/// Get current Unix timestamp in milliseconds
pub fn unix_timestamp_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_millis() as u64
}

/// Convert Unix timestamp to ISO 8601 string
pub fn timestamp_to_iso8601(timestamp: u64) -> UtilResult<String> {
    use chrono::DateTime;

    let datetime = DateTime::from_timestamp(timestamp as i64, 0)
        .ok_or_else(|| UtilError::InvalidInput("Invalid timestamp".to_string()))?;

    Ok(datetime.to_rfc3339())
}

/// Parse ISO 8601 string to Unix timestamp
pub fn iso8601_to_timestamp(iso_string: &str) -> UtilResult<u64> {
    use chrono::DateTime;

    let datetime = DateTime::parse_from_rfc3339(iso_string)
        .map_err(|e| UtilError::InvalidInput(format!("Invalid ISO 8601 string: {}", e)))?;

    Ok(datetime.timestamp() as u64)
}

/// Format duration in human-readable form
pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();

    if total_secs < 60 {
        format!("{}.{:03}s", total_secs, duration.subsec_millis())
    } else if total_secs < 3600 {
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;
        format!("{}m {}s", minutes, seconds)
    } else if total_secs < 86400 {
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        format!("{}h {}m", hours, minutes)
    } else {
        let days = total_secs / 86400;
        let hours = (total_secs % 86400) / 3600;
        format!("{}d {}h", days, hours)
    }
}

/// Parse duration from human-readable string
pub fn parse_duration(duration_str: &str) -> UtilResult<Duration> {
    let duration_str = duration_str.trim().to_lowercase();

    if let Ok(secs) = duration_str.parse::<f64>() {
        return Ok(Duration::from_secs_f64(secs));
    }

    let (number_part, unit_part) = split_number_unit(&duration_str)?;
    let number: f64 = number_part
        .parse()
        .map_err(|_| UtilError::InvalidInput("Invalid number in duration".to_string()))?;

    let multiplier = match unit_part.as_str() {
        "ms" | "millis" | "milliseconds" => 0.001,
        "s" | "sec" | "secs" | "seconds" => 1.0,
        "m" | "min" | "mins" | "minutes" => 60.0,
        "h" | "hr" | "hrs" | "hours" => 3600.0,
        "d" | "day" | "days" => 86400.0,
        _ => {
            return Err(UtilError::InvalidInput(format!(
                "Unknown duration unit: {}",
                unit_part
            )))
        }
    };

    Ok(Duration::from_secs_f64(number * multiplier))
}

/// Split a string like "5m" into ("5", "m")
fn split_number_unit(input: &str) -> UtilResult<(String, String)> {
    let mut number_part = String::new();
    let mut unit_part = String::new();
    let mut found_unit = false;

    for c in input.chars() {
        if c.is_ascii_digit() || c == '.' {
            if found_unit {
                return Err(UtilError::InvalidInput(
                    "Number after unit not allowed".to_string(),
                ));
            }
            number_part.push(c);
        } else {
            found_unit = true;
            unit_part.push(c);
        }
    }

    if number_part.is_empty() {
        return Err(UtilError::InvalidInput(
            "No number found in duration".to_string(),
        ));
    }

    if unit_part.is_empty() {
        unit_part = "s".to_string(); // Default to seconds
    }

    Ok((number_part, unit_part))
}

/// Simple timer for measuring elapsed time
pub struct Timer {
    start: std::time::Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn elapsed_secs(&self) -> f64 {
        self.elapsed().as_secs_f64()
    }

    pub fn reset(&mut self) {
        self.start = std::time::Instant::now();
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unix_timestamp() {
        let timestamp = unix_timestamp();
        assert!(timestamp > 1_600_000_000); // After 2020
    }

    #[test]
    fn test_duration_parsing() {
        assert_eq!(parse_duration("5s").unwrap(), Duration::from_secs(5));
        assert_eq!(parse_duration("2m").unwrap(), Duration::from_secs(120));
        assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
        assert_eq!(parse_duration("500ms").unwrap(), Duration::from_millis(500));

        assert!(parse_duration("invalid").is_err());
    }

    #[test]
    fn test_duration_formatting() {
        assert_eq!(format_duration(Duration::from_secs(5)), "5.000s");
        assert_eq!(format_duration(Duration::from_secs(65)), "1m 5s");
        assert_eq!(format_duration(Duration::from_secs(3665)), "1h 1m");
    }

    #[test]
    fn test_timer() {
        let timer = Timer::new();
        std::thread::sleep(Duration::from_millis(10));
        assert!(timer.elapsed() >= Duration::from_millis(10));
    }

    #[test]
    fn test_iso8601_conversion() {
        let timestamp = 1640995200; // 2022-01-01 00:00:00 UTC
        let iso_string = timestamp_to_iso8601(timestamp).unwrap();
        assert!(iso_string.starts_with("2022-01-01"));

        let parsed_timestamp = iso8601_to_timestamp(&iso_string).unwrap();
        assert_eq!(parsed_timestamp, timestamp);
    }
}
