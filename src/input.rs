use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    Trigger,
}

pub fn action_from_event(event: Event) -> Option<Action> {
    match event {
        Event::Key(key) => action_from_key(key),
        _ => None,
    }
}

fn action_from_key(key: KeyEvent) -> Option<Action> {
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), _) | (KeyCode::Char('Q'), _) => Some(Action::Quit),
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(Action::Quit),
        (KeyCode::Char(' '), _) => Some(Action::Trigger),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_quit_keys() {
        assert_eq!(
            action_from_event(Event::Key(KeyEvent::from(KeyCode::Char('q')))),
            Some(Action::Quit)
        );
        assert_eq!(
            action_from_event(Event::Key(KeyEvent::from(KeyCode::Char('Q')))),
            Some(Action::Quit)
        );
    }

    #[test]
    fn maps_trigger_key() {
        assert_eq!(
            action_from_event(Event::Key(KeyEvent::from(KeyCode::Char(' ')))),
            Some(Action::Trigger)
        );
    }
}
