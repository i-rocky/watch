use std::io::{self, Write};
use std::time::Instant;

use chrono::Local;
use crossterm::cursor::MoveTo;
use crossterm::terminal::{Clear, ClearType};
use crossterm::execute;

use crate::config::{ColorMode, Config};
use crate::diff::DiffState;
use crate::exec::{build_command, run_command, ExecError};
use crate::render::{format_text, header_line, strip_ansi_text, terminal_size};
use crate::screenshot::save_screenshot;
use crate::terminal::{wait_for_action, wait_for_keypress, TerminalGuard, WaitOutcome};

#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    Exec(ExecError),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Io(err) => write!(f, "io error: {err}"),
            AppError::Exec(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<ExecError> for AppError {
    fn from(err: ExecError) -> Self {
        AppError::Exec(err)
    }
}

pub fn run(config: Config) -> Result<i32, AppError> {
    let _terminal = TerminalGuard::new()?;
    let mut stdout = io::stdout();
    let mut diff_state = DiffState::new();
    let mut last_visible: Option<String> = None;
    let mut unchanged_cycles = 0u32;
    let mut pending_screenshot = false;
    let mut next_run = Instant::now();
    let mut first = true;

    loop {
        if !first {
            let wait = if config.precise {
                next_run.saturating_duration_since(Instant::now())
            } else {
                config.interval
            };
            match wait_for_action(wait, config.no_rerun, &mut pending_screenshot)? {
                WaitOutcome::Quit => return Ok(0),
                WaitOutcome::Trigger => {}
                WaitOutcome::Timeout => {}
            }
        }

        first = false;
        let run_started = Instant::now();
        let exec_output = run_command(build_command(&config.command, config.exec)?)?;
        if config.beep && !exec_output.status.success() {
            stdout.write_all(b"\x07")?;
        }

        let mut text = String::from_utf8_lossy(&exec_output.combined()).into_owned();
        if matches!(config.color, ColorMode::Auto | ColorMode::Never) {
            text = strip_ansi_text(&text);
        }

        let diff_result = diff_state.apply(&text, config.differences);
        let size = terminal_size();
        let mut output_lines = format_text(&diff_result.text, size.columns, config.no_wrap);

        let mut frame = Vec::new();
        let header_lines = if config.no_title { 0 } else { 2 };
        if !config.no_title {
            frame.push(header_line(
                &config.command,
                config.interval,
                Local::now(),
                size.columns,
            ));
            frame.push(String::new());
        }

        if !config.follow {
            let available = size.rows.saturating_sub(header_lines as u16) as usize;
            output_lines.truncate(available);
        }

        frame.extend(output_lines);

        if config.follow {
            for line in &frame {
                stdout.write_all(line.as_bytes())?;
                stdout.write_all(b"\n")?;
            }
        } else {
            execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
            for line in &frame {
                stdout.write_all(line.as_bytes())?;
                stdout.write_all(b"\n")?;
            }
        }
        stdout.flush()?;

        if pending_screenshot {
            if let Some(dir) = config.shotsdir.as_deref() {
                save_screenshot(dir, &frame)?;
            }
            pending_screenshot = false;
        }

        if config.errexit && !exec_output.status.success() {
            wait_for_keypress()?;
            return Ok(exec_output.status.code().unwrap_or(1));
        }

        if config.chgexit || config.equexit.is_some() {
            let visible = visible_output(&frame, header_lines);
            if let Some(prev) = last_visible.as_deref() {
                if prev == visible {
                    unchanged_cycles = unchanged_cycles.saturating_add(1);
                } else {
                    unchanged_cycles = 0;
                }

                if config.chgexit && prev != visible {
                    return Ok(0);
                }
                if let Some(limit) = config.equexit {
                    if unchanged_cycles >= limit {
                        return Ok(0);
                    }
                }
            }
            last_visible = Some(visible);
        }

        next_run = if config.precise {
            run_started + config.interval
        } else {
            Instant::now() + config.interval
        };
    }
}

fn visible_output(frame: &[String], header_lines: usize) -> String {
    frame
        .iter()
        .skip(header_lines)
        .cloned()
        .collect::<Vec<_>>()
        .join("\n")
}
