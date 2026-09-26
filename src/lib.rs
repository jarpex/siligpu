#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::correctness,
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::indexing_slicing,
    warnings,
    missing_debug_implementations,
    unreachable_pub,
    rust_2018_idioms,
    unused_lifetimes,
    non_ascii_idents,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications
)]
#![warn(clippy::nursery, clippy::cargo)]
#![allow(
    clippy::multiple_crate_versions,
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::must_use_candidate,
    clippy::redundant_pub_crate,
    clippy::wildcard_imports,
    clippy::option_if_let_else,
    clippy::single_match_else,
    clippy::match_same_arms,
    clippy::needless_pass_by_value,
    clippy::struct_excessive_bools,
    clippy::fn_params_excessive_bools
)]

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
    /// The value is too large and would overflow internal arithmetic.
    Overflow,
}

impl fmt::Display for ParseDurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "Duration string is empty"),
            Self::InvalidNumber => write!(f, "Invalid number in duration"),
            Self::UnsupportedUnit(unit) => write!(f, "Unsupported duration unit: {unit}"),
            Self::Overflow => write!(f, "Duration value is too large"),
        }
    }
}

impl std::error::Error for ParseDurationError {}

/// Parse strings like "100", "100ms", "1s", "1m", "1h" into a `Duration`.
pub fn parse_duration(s: &str) -> Result<Duration, ParseDurationError> {
    fn strip_suffix_ignore_case<'a>(s: &'a str, suffix: &str) -> Option<&'a str> {
        let split_at = s.len().checked_sub(suffix.len())?;
        if s.is_char_boundary(split_at) && s[split_at..].eq_ignore_ascii_case(suffix) {
            Some(&s[..split_at])
        } else {
            None
        }
    }

    let parse_num = |num_str: &str| -> Result<u64, ParseDurationError> {
        num_str.parse::<u64>().map_err(|e| {
            if matches!(
                e.kind(),
                std::num::IntErrorKind::PosOverflow | std::num::IntErrorKind::NegOverflow
            ) {
                ParseDurationError::Overflow
            } else {
                ParseDurationError::InvalidNumber
            }
        })
    };

    let s = s.trim();
    if s.is_empty() {
        return Err(ParseDurationError::Empty);
    }

    let (num_str, multiplier) = if let Some(n) = strip_suffix_ignore_case(s, "ms") {
        (n, 1)
    } else if let Some(n) = strip_suffix_ignore_case(s, "s") {
        (n, 1000)
    } else if let Some(n) = strip_suffix_ignore_case(s, "m") {
        (n, 60_000)
    } else if let Some(n) = strip_suffix_ignore_case(s, "h") {
        (n, 3_600_000)
    } else {
        if s.chars().any(|c| c.is_ascii_alphabetic()) {
            return Err(ParseDurationError::UnsupportedUnit(s.to_string()));
        }
        (s, 1)
    };

    let num = parse_num(num_str)?;
    let millis = num
        .checked_mul(multiplier)
        .ok_or(ParseDurationError::Overflow)?;
    Ok(Duration::from_millis(millis))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("100ms").unwrap(), Duration::from_millis(100));
        assert_eq!(parse_duration("1s").unwrap(), Duration::from_secs(1));
        assert_eq!(parse_duration("1m").unwrap(), Duration::from_secs(60));
        assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
        assert_eq!(parse_duration("500").unwrap(), Duration::from_millis(500));
        assert!(parse_duration("invalid").is_err());
    }

    /// Fuzzing regression test for "9949999995111111111M" resulting in `attempt to multiply with overflow`.
    #[test]
    fn test_fuzz_regression_overflow_minutes() {
        let result = parse_duration("9949999995111111111M");
        assert_eq!(result, Err(ParseDurationError::Overflow));
    }

    #[test]
    fn test_overflow_all_units() {
        let huge = format!("{}m", u64::MAX);
        assert_eq!(parse_duration(&huge), Err(ParseDurationError::Overflow));

        let huge_hours = format!("{}h", u64::MAX);
        assert_eq!(
            parse_duration(&huge_hours),
            Err(ParseDurationError::Overflow)
        );

        assert_eq!(
            parse_duration("3074457345618258603m"),
            Err(ParseDurationError::Overflow)
        );

        let max_ms = format!("{}ms", u64::MAX);
        assert!(parse_duration(&max_ms).is_ok());
    }
}
