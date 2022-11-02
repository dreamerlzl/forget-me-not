use anyhow::Result;
use std::time::Duration;
use task_reminder::comm::{parse_at, parse_duration};

#[test]
fn test_duration() -> Result<()> {
    let test_cases = vec![
        ("0h0m10s", 10),
        ("1h0m0s", 3600),
        ("0h1m0s", 60),
        ("0h0m0s", 0),
        ("1d", 3600 * 24),
        ("1h", 3600),
        ("1d1s", 3600 * 24 + 1),
    ];

    for (duration, expected_seconds) in test_cases {
        let expected_duration = Duration::from_secs(expected_seconds);
        assert_eq!(parse_duration(duration)?, expected_duration);
    }
    Ok(())
}

#[test]
fn test_duration_err() {
    let test_cases = vec!["1f", "abc", "@341", "1d2@3"];
    for duration in test_cases {
        //dbg!("testing {}", duration);
        assert!(parse_duration(duration).is_err());
    }
}

#[test]
fn test_parse_at() -> Result<()> {
    // no support for seconds
    let test_cases = vec![("13:24", 13, 24), ("23:01", 23, 1), ("01:59", 1, 59)];
    for (next_fire, hour, minute) in test_cases {
        let next_fire = parse_at(next_fire)?;
        let parsed_hour = next_fire.hour();
        let parsed_minute = next_fire.minute();
        assert_eq!(hour, parsed_hour);
        assert_eq!(minute, parsed_minute);
    }

    Ok(())
}

#[test]
fn test_parse_at_err() {
    let test_cases = vec!["123:24", "11:94", "098", ""];
    for next_fire in test_cases {
        assert!(parse_at(next_fire).is_err());
    }
}
