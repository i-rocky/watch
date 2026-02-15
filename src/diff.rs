use similar::TextDiff;

use crate::config::DifferencesMode;

pub const HIGHLIGHT_START: &str = "\x1b[7m";
pub const HIGHLIGHT_END: &str = "\x1b[0m";

#[derive(Debug, Default, Clone)]
pub struct DiffState {
    previous: Option<String>,
    baseline: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffResult {
    pub text: String,
    pub changed: bool,
}

impl DiffState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply(&mut self, current: &str, mode: Option<DifferencesMode>) -> DiffResult {
        let changed = self
            .previous
            .as_deref()
            .map(|prev| prev != current)
            .unwrap_or(false);

        let highlighted = match mode {
            None => current.to_string(),
            Some(DifferencesMode::Changes) => match self.previous.as_deref() {
                Some(prev) => highlight_diff(prev, current),
                None => current.to_string(),
            },
            Some(DifferencesMode::Permanent) => {
                if self.baseline.is_none() {
                    self.baseline = Some(current.to_string());
                    current.to_string()
                } else {
                    highlight_diff(self.baseline.as_deref().unwrap_or(""), current)
                }
            }
        };

        self.previous = Some(current.to_string());

        DiffResult {
            text: highlighted,
            changed,
        }
    }
}

pub fn highlight_diff(base: &str, current: &str) -> String {
    let diff = TextDiff::from_chars(base, current);
    let mut out = String::new();
    for change in diff.iter_all_changes() {
        match change.tag() {
            similar::ChangeTag::Equal => out.push_str(change.value()),
            similar::ChangeTag::Insert => {
                out.push_str(HIGHLIGHT_START);
                out.push_str(change.value());
                out.push_str(HIGHLIGHT_END);
            }
            similar::ChangeTag::Delete => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlight_marks_inserted_chars() {
        let out = highlight_diff("abc", "abXc");
        assert!(out.contains(HIGHLIGHT_START));
        assert!(out.contains("X"));
        assert!(out.contains(HIGHLIGHT_END));
    }

    #[test]
    fn diff_state_tracks_changes() {
        let mut state = DiffState::new();
        let first = state.apply("a", Some(DifferencesMode::Changes));
        assert!(!first.changed);
        let second = state.apply("b", Some(DifferencesMode::Changes));
        assert!(second.changed);
    }

    #[test]
    fn cumulative_mode_uses_baseline() {
        let mut state = DiffState::new();
        let _ = state.apply("abc", Some(DifferencesMode::Permanent));
        let out = state.apply("abXc", Some(DifferencesMode::Permanent));
        assert!(out.text.contains("X"));
        assert!(out.text.contains(HIGHLIGHT_START));
    }
}
