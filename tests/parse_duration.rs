// Integration tests for the CLI duration parser.
// Makes use of the crate's `parse_duration` helper.

use std::time::Duration;
use siligpu::{parse_duration, ParseDurationError};

#[test]
fn integration_parse_duration_various() {
    assert_eq!(parse_duration("100ms").unwrap(), Duration::from_millis(100));
    assert_eq!(parse_duration("1s").unwrap(), Duration::from_secs(1));
    assert_eq!(parse_duration("1m").unwrap(), Duration::from_secs(60));
    assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
    assert_eq!(parse_duration("500").unwrap(), Duration::from_millis(500));
    assert_eq!(
        parse_duration("2S").unwrap(),
        Duration::from_secs(2),
        "upper-case unit should parse"
    );
    assert!(matches!(
        parse_duration("invalid"),
        Err(ParseDurationError::InvalidNumber | ParseDurationError::UnsupportedUnit(_))
    ));
    assert!(matches!(parse_duration(""), Err(ParseDurationError::Empty)));
}
