use std::path::PathBuf;
use std::time::Duration;

use crate::cli::Cli;
use crate::interval::{default_interval, parse_interval};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DifferencesMode {
    Changes,
    Permanent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub interval: Duration,
    pub precise: bool,
    pub no_title: bool,
    pub no_wrap: bool,
    pub differences: Option<DifferencesMode>,
    pub beep: bool,
    pub errexit: bool,
    pub chgexit: bool,
    pub equexit: Option<u32>,
    pub follow: bool,
    pub no_rerun: bool,
    pub exec: bool,
    pub color: ColorMode,
    pub shotsdir: Option<PathBuf>,
    pub command: Vec<String>,
}

impl Config {
    pub fn from_cli(cli: Cli) -> Result<Self, String> {
        let interval = if let Some(raw) = cli.interval {
            parse_interval(&raw).map_err(|err| err.to_string())?
        } else if let Ok(raw) = std::env::var("WATCH_INTERVAL") {
            parse_interval(&raw).map_err(|err| err.to_string())?
        } else {
            default_interval()
        };

        let color = match (cli.color, cli.no_color) {
            (true, true) => {
                return Err("options --color and --no-color are mutually exclusive".to_string())
            }
            (true, false) => ColorMode::Always,
            (false, true) => ColorMode::Never,
            (false, false) => ColorMode::Auto,
        };

        let differences = if cli.differences.is_some() {
            cli.differences
        } else if cli.differences_flag {
            Some(DifferencesMode::Changes)
        } else {
            None
        };

        Ok(Self {
            interval,
            precise: cli.precise,
            no_title: cli.no_title,
            no_wrap: cli.no_wrap,
            differences,
            beep: cli.beep,
            errexit: cli.errexit,
            chgexit: cli.chgexit,
            equexit: cli.equexit,
            follow: cli.follow,
            no_rerun: cli.no_rerun,
            exec: cli.exec,
            color,
            shotsdir: cli.shotsdir,
            command: cli.command,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().expect("env lock")
    }

    fn with_watch_interval_env<F>(value: Option<&str>, test: F)
    where
        F: FnOnce(),
    {
        let _guard = env_lock();
        let previous = std::env::var("WATCH_INTERVAL").ok();
        match value {
            Some(val) => unsafe {
                std::env::set_var("WATCH_INTERVAL", val);
            },
            None => unsafe {
                std::env::remove_var("WATCH_INTERVAL");
            },
        }
        test();
        match previous {
            Some(val) => unsafe {
                std::env::set_var("WATCH_INTERVAL", val);
            },
            None => unsafe {
                std::env::remove_var("WATCH_INTERVAL");
            },
        }
    }

    #[test]
    fn config_uses_default_interval_without_flag_or_env() {
        with_watch_interval_env(None, || {
            let cli = Cli::parse_from_iter(["watch", "echo", "hi"]).unwrap();
            let config = Config::from_cli(cli).unwrap();
            assert_eq!(config.interval, default_interval());
        });
    }

    #[test]
    fn config_uses_watch_interval_env() {
        with_watch_interval_env(Some("0.5"), || {
            let cli = Cli::parse_from_iter(["watch", "echo", "hi"]).unwrap();
            let config = Config::from_cli(cli).unwrap();
            assert_eq!(config.interval, Duration::from_secs_f64(0.5));
        });
    }

    #[test]
    fn config_rejects_both_color_flags() {
        let cli = Cli::parse_from_iter(["watch", "-c", "-C", "echo", "hi"]).unwrap();
        let err = Config::from_cli(cli).unwrap_err();
        assert!(err.contains("mutually exclusive"));
    }
}
