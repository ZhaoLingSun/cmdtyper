#![allow(dead_code)]

use std::time::Duration;

use crossterm::event::{self, Event, KeyEvent};

/// Application events
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    Resize(u16, u16),
}

/// Poll for events with a 50ms timeout (for tick-based animations)
pub fn poll_event() -> anyhow::Result<AppEvent> {
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
