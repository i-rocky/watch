use std::io;
use std::time::{Duration, Instant};

use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;

use crate::input::{action_from_event, Action};

pub struct TerminalGuard;

impl TerminalGuard {
    pub fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, Hide)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, Show, LeaveAlternateScreen);
    }
}

pub enum WaitOutcome {
    Quit,
    Trigger,
    Timeout,
}

pub fn wait_for_action(
    duration: Duration,
    no_rerun: bool,
    pending_screenshot: &mut bool,
) -> io::Result<WaitOutcome> {
    let deadline = Instant::now() + duration;
    loop {
        let now = Instant::now();
        if now >= deadline {
            return Ok(WaitOutcome::Timeout);
        }

        let remaining = deadline - now;
        let poll_for = remaining.min(Duration::from_millis(250));
        if event::poll(poll_for)? {
            match event::read()? {
                Event::Resize(_, _) if !no_rerun => return Ok(WaitOutcome::Trigger),
                event => {
                    if let Some(action) = action_from_event(event) {
                        match action {
                            Action::Quit => return Ok(WaitOutcome::Quit),
                            Action::Trigger => return Ok(WaitOutcome::Trigger),
                            Action::Screenshot => *pending_screenshot = true,
                        }
                    }
                }
            }
        }
    }
}

pub fn wait_for_keypress() -> io::Result<()> {
    loop {
        if let Event::Key(_) = event::read()? {
            return Ok(());
        }
    }
}
