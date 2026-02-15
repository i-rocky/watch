use std::path::PathBuf;

use clap::{ArgAction, Parser};

use crate::config::DifferencesMode;

#[derive(Parser, Debug, Clone, PartialEq, Eq)]
#[command(name = "watch", version, about = "Execute a program periodically, showing output fullscreen")]
#[command(trailing_var_arg = true, allow_hyphen_values = true)]
pub struct Cli {
    #[arg(short = 'b', long = "beep", action = ArgAction::SetTrue)]
    pub beep: bool,

    #[arg(short = 'c', long = "color", action = ArgAction::SetTrue)]
    pub color: bool,

    #[arg(short = 'C', long = "no-color", action = ArgAction::SetTrue)]
    pub no_color: bool,

    #[arg(short = 'd', action = ArgAction::SetTrue)]
    pub differences_flag: bool,

    #[arg(long = "differences", require_equals = true, value_name = "permanent", value_parser = parse_differences)]
    pub differences: Option<DifferencesMode>,

    #[arg(short = 'e', long = "errexit", action = ArgAction::SetTrue)]
    pub errexit: bool,

    #[arg(short = 'f', long = "follow", action = ArgAction::SetTrue)]
    pub follow: bool,

    #[arg(short = 'g', long = "chgexit", action = ArgAction::SetTrue)]
    pub chgexit: bool,

    #[arg(short = 'n', long = "interval", value_name = "secs")]
    pub interval: Option<String>,

    #[arg(short = 'p', long = "precise", action = ArgAction::SetTrue)]
    pub precise: bool,

    #[arg(short = 'q', long = "equexit", value_name = "cycles")]
    pub equexit: Option<u32>,

    #[arg(short = 'r', long = "no-rerun", action = ArgAction::SetTrue)]
    pub no_rerun: bool,

    #[arg(short = 's', long = "shotsdir", value_name = "dir")]
    pub shotsdir: Option<PathBuf>,

    #[arg(short = 't', long = "no-title", action = ArgAction::SetTrue)]
    pub no_title: bool,

    #[arg(short = 'w', long = "no-wrap", action = ArgAction::SetTrue)]
    pub no_wrap: bool,

    #[arg(short = 'x', long = "exec", action = ArgAction::SetTrue)]
    pub exec: bool,

    #[arg(value_name = "command", required = true, num_args(1..))]
    pub command: Vec<String>,
}

fn parse_differences(value: &str) -> Result<DifferencesMode, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("changes") {
        Ok(DifferencesMode::Changes)
    } else if trimmed.eq_ignore_ascii_case("permanent") || trimmed == "1" {
        Ok(DifferencesMode::Permanent)
    } else {
        Err("expected 'permanent' or '1'".to_string())
    }
}

impl Cli {
    pub fn parse_args() -> Result<Self, clap::Error> {
        Self::parse_from_iter(std::env::args_os())
    }

    pub fn parse_from_iter<I, T>(iter: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString>,
    {
        let normalized = normalize_args(iter);
        Self::try_parse_from(normalized)
    }
}

fn normalize_args<I, T>(iter: I) -> Vec<std::ffi::OsString>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString>,
{
    let mut out = Vec::new();
    let mut iter = iter.into_iter();
    if let Some(first) = iter.next() {
        out.push(first.into());
    }

    for arg in iter {
        let raw = arg.into();
        let raw_str = raw.to_string_lossy();

        if raw_str == "-d" {
            out.push(raw);
            continue;
        }

        if let Some(value) = raw_str.strip_prefix("-d=") {
            out.push(format!("--differences={value}").into());
            continue;
        }

        if let Some(value) = raw_str.strip_prefix("-d") {
            if !value.is_empty() {
                out.push(format!("--differences={value}").into());
                continue;
            }
        }

        out.push(raw);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_differences_default() {
        let cli = Cli::parse_from_iter(["watch", "-d", "echo", "hi"]).unwrap();
        assert!(cli.differences_flag);
        assert_eq!(cli.differences, None);
    }

    #[test]
    fn parses_differences_permanent() {
        let cli = Cli::parse_from_iter(["watch", "-d=permanent", "echo", "hi"]).unwrap();
        assert_eq!(cli.differences, Some(DifferencesMode::Permanent));
    }

    #[test]
    fn parses_differences_short_attached_value() {
        let cli = Cli::parse_from_iter(["watch", "-d1", "echo", "hi"]).unwrap();
        assert_eq!(cli.differences, Some(DifferencesMode::Permanent));
    }
}
