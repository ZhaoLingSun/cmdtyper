#![allow(dead_code)]

use std::time::{Duration, Instant};

pub struct Timer {
    start: Option<Instant>,
    elapsed: Duration,
    paused: bool,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start: None,
            elapsed: Duration::ZERO,
            paused: false,
        }
    }

    pub fn start(&mut self) {
        self.start = Some(Instant::now());
        self.elapsed = Duration::ZERO;
        self.paused = false;
    }

    pub fn pause(&mut self) {
        if !self.paused {
            if let Some(start) = self.start {
                self.elapsed += start.elapsed();
            }
            self.start = None;
            self.paused = true;
        }
    }

    pub fn resume(&mut self) {
        if self.paused {
            self.start = Some(Instant::now());
            self.paused = false;
        }
    }

    pub fn elapsed(&self) -> Duration {
        let running = match self.start {
            Some(start) if !self.paused => start.elapsed(),
            _ => Duration::ZERO,
        };
        self.elapsed + running
    }

    pub fn elapsed_secs(&self) -> f64 {
        self.elapsed().as_secs_f64()
    }

    pub fn format_mmss(&self) -> String {
        let total_secs = self.elapsed().as_secs();
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{:02}:{:02}", mins, secs)
    }
}
