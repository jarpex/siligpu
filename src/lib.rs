pub mod ioreport;

use std::{fmt, time::Duration};

/// Errors returned by `parse_duration` when the input is malformed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseDurationError {
    /// The string was empty or whitespace.
    Empty,
    /// The numeric portion could not be parsed.
    InvalidNumber,
    /// The unit was not recognized (e.g., `1x`).
    UnsupportedUnit(String),
}

impl fmt::Display for ParseDurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseDurationError::Empty => write!(f, "Duration string is empty"),
            ParseDurationError::InvalidNumber => write!(f, "Invalid number in duration"),
            ParseDurationError::UnsupportedUnit(unit) => {
                write!(f, "Unsupported duration unit: {unit}")
            }
        }
    }
}

impl std::error::Error for ParseDurationError {}

/// Parse strings like "100", "100ms", "1s", "1m", "1h" into a `Duration`.
/// Accepts upper- or lower-case units and trims surrounding whitespace.
pub fn parse_duration(s: &str) -> Result<Duration, ParseDurationError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(ParseDurationError::Empty);
    }

    let normalized = s.to_ascii_lowercase();

    let parse_num = |num: &str| num.parse::<u64>().map_err(|_| ParseDurationError::InvalidNumber);

    if let Some(num) = normalized.strip_suffix("ms") {
        Ok(Duration::from_millis(parse_num(num)?))
    } else if let Some(num) = normalized.strip_suffix('s') {
        Ok(Duration::from_secs(parse_num(num)?))
    } else if let Some(num) = normalized.strip_suffix('m') {
        Ok(Duration::from_secs(parse_num(num)? * 60))
    } else if let Some(num) = normalized.strip_suffix('h') {
        Ok(Duration::from_secs(parse_num(num)? * 3600))
    } else {
        if normalized.chars().any(|c| c.is_ascii_alphabetic()) {
            return Err(ParseDurationError::UnsupportedUnit(normalized));
        }

        Ok(Duration::from_millis(parse_num(&normalized)?))
    }
}
