use std::time::Duration;

use chrono::{DateTime, Local};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::config::ColorMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSize {
    pub columns: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderConfig {
    pub no_title: bool,
    pub no_wrap: bool,
    pub color: ColorMode,
}

pub fn terminal_size() -> TerminalSize {
    let mut cols = env_u16("COLUMNS");
    let mut rows = env_u16("LINES");

    if cols.is_none() || rows.is_none() {
        if let Ok((term_cols, term_rows)) = crossterm::terminal::size() {
            cols = cols.or(Some(term_cols));
            rows = rows.or(Some(term_rows));
        }
    }

    TerminalSize {
        columns: cols.unwrap_or(80).max(1),
        rows: rows.unwrap_or(24).max(1),
    }
}

pub fn header_line(
    command: &[String],
    interval: Duration,
    now: DateTime<Local>,
    columns: u16,
) -> String {
    let right = now.format("%a %b %e %H:%M:%S %Y").to_string();
    header_line_with_time(command, interval, &right, columns)
}

pub fn header_line_with_time(
    command: &[String],
    interval: Duration,
    right: &str,
    columns: u16,
) -> String {
    let secs = interval.as_secs_f64();
    let command_str = command.join(" ");
    let left = format!("Every {:.1}s: {command_str}", secs);
    let mut left_truncated = truncate_to_width(&left, columns as usize);

    let left_width = UnicodeWidthStr::width(left_truncated.as_str());
    let right_width = UnicodeWidthStr::width(right);

    if columns as usize <= right_width + 1 {
        return left_truncated;
    }

    let max_left = columns as usize - right_width - 1;
    if left_width > max_left {
        left_truncated = truncate_to_width(&left_truncated, max_left);
    }

    let pad = max_left.saturating_sub(UnicodeWidthStr::width(left_truncated.as_str()));
    let mut line = left_truncated;
    line.push_str(&" ".repeat(pad + 1));
    line.push_str(right);
    line
}

pub fn format_output(
    output: &[u8],
    columns: u16,
    no_wrap: bool,
    color: ColorMode,
) -> Vec<String> {
    let input = String::from_utf8_lossy(output);
    let text = match color {
        ColorMode::Always => input.into_owned(),
        ColorMode::Auto | ColorMode::Never => strip_ansi_text(&input),
    };
    format_text(&text, columns, no_wrap)
}

fn env_u16(key: &str) -> Option<u16> {
    std::env::var(key)
        .ok()
        .and_then(|value| value.trim().parse::<u16>().ok())
        .filter(|value| *value > 0)
}

pub fn strip_ansi_text(input: &str) -> String {
    let stripped = strip_ansi_escapes::strip(input.as_bytes());
    String::from_utf8_lossy(&stripped).into_owned()
}

pub fn format_text(text: &str, columns: u16, no_wrap: bool) -> Vec<String> {
    split_lines(text, columns as usize, no_wrap)
}

fn split_lines(text: &str, width: usize, no_wrap: bool) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut col = 0usize;
    let mut truncated = false;
    let width = width.max(1);

    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\r' {
            continue;
        }

        if ch == '\n' {
            lines.push(current);
            current = String::new();
            col = 0;
            truncated = false;
            continue;
        }

        if ch == '\x1b' {
            let seq = consume_escape_sequence(ch, &mut chars);
            current.push_str(&seq);
            continue;
        }

        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if no_wrap && (truncated || col + ch_width > width) {
            truncated = true;
            continue;
        }

        if !no_wrap && col + ch_width > width {
            lines.push(current);
            current = String::new();
            col = 0;
        }

        current.push(ch);
        col += ch_width;
    }

    lines.push(current);
    lines
}

fn consume_escape_sequence(
    first: char,
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> String {
    let mut seq = String::new();
    seq.push(first);

    if let Some('[') = chars.peek().copied() {
        seq.push(chars.next().unwrap());
        while let Some(next) = chars.next() {
            seq.push(next);
            if ('@'..='~').contains(&next) {
                break;
            }
        }
    }

    seq
}

fn truncate_to_width(input: &str, width: usize) -> String {
    let mut out = String::new();
    let mut col = 0usize;
    for ch in input.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if col + ch_width > width {
            break;
        }
        out.push(ch);
        col += ch_width;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_right_aligns_time() {
        let line = header_line_with_time(&["echo".into()], Duration::from_secs(2), "TIME", 20);
        assert_eq!(line.len(), 20);
        assert!(line.ends_with("TIME"));
    }

    #[test]
    fn format_output_wraps_lines() {
        let lines = format_output(b"abcdef", 3, false, ColorMode::Never);
        assert_eq!(lines, vec!["abc", "def"]);
    }

    #[test]
    fn format_output_truncates_lines() {
        let lines = format_output(b"abcdef", 3, true, ColorMode::Never);
        assert_eq!(lines, vec!["abc"]);
    }
}
