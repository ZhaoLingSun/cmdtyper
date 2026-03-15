use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyEvent};

/// Application-level events.
#[derive(Debug)]
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    Resize(u16, u16),
}

/// Poll for the next event with a 50ms tick interval.
pub fn poll_event() -> Result<AppEvent> {
    if event::poll(Duration::from_millis(50))? {
        match event::read()? {
            Event::Key(key) => Ok(AppEvent::Key(key)),
            Event::Resize(w, h) => Ok(AppEvent::Resize(w, h)),
            _ => Ok(AppEvent::Tick),
        }
    } else {
        Ok(AppEvent::Tick)
    }
}
