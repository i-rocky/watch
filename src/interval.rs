use std::time::Duration;

pub const DEFAULT_INTERVAL_SECS: f64 = 2.0;
pub const MIN_INTERVAL_SECS: f64 = 0.1;
pub const MAX_INTERVAL_SECS: f64 = 2_678_400.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntervalParseError {
    Empty,
    NotANumber,
}

impl std::fmt::Display for IntervalParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntervalParseError::Empty => write!(f, "interval is empty"),
            IntervalParseError::NotANumber => write!(f, "interval is not a number"),
        }
    }
}

impl std::error::Error for IntervalParseError {}

pub fn parse_interval(input: &str) -> Result<Duration, IntervalParseError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(IntervalParseError::Empty);
    }

    let normalized = trimmed.replace(',', ".");
    let mut secs: f64 = normalized.parse().map_err(|_| IntervalParseError::NotANumber)?;
    if secs.is_nan() || secs.is_infinite() {
        return Err(IntervalParseError::NotANumber);
    }

    if secs < MIN_INTERVAL_SECS {
        secs = MIN_INTERVAL_SECS;
    } else if secs > MAX_INTERVAL_SECS {
        secs = MAX_INTERVAL_SECS;
    }

    Ok(Duration::from_secs_f64(secs))
}

pub fn default_interval() -> Duration {
    Duration::from_secs_f64(DEFAULT_INTERVAL_SECS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_interval_clamps_low() {
        let parsed = parse_interval("0.01").unwrap();
        assert_eq!(parsed, Duration::from_secs_f64(MIN_INTERVAL_SECS));
    }

    #[test]
    fn parse_interval_clamps_high() {
        let parsed = parse_interval("99999999").unwrap();
        assert_eq!(parsed, Duration::from_secs_f64(MAX_INTERVAL_SECS));
    }

    #[test]
    fn parse_interval_accepts_comma_decimal() {
        let parsed = parse_interval("0,5").unwrap();
        assert_eq!(parsed, Duration::from_secs_f64(0.5));
    }

    #[test]
    fn parse_interval_rejects_empty() {
        assert_eq!(parse_interval("  ").unwrap_err(), IntervalParseError::Empty);
    }
}
