#![allow(dead_code)]

use std::time::Instant;

use chrono::Utc;

use crate::data::models::{Difficulty, Keystroke, Mode, SessionRecord};

/// Result of a single keystroke input
#[derive(Debug, Clone, PartialEq)]
pub enum InputResult {
    Correct,
    Error(char),
    AlreadyComplete,
}

pub struct TypingEngine {
    pub target: Vec<char>,
    pub cursor: usize,
    pub keystrokes: Vec<Keystroke>,
    pub current_attempts: u8,
    pub error_flash: Option<Instant>,
    pub start_time: Option<Instant>,
    pub last_correct_time: Option<Instant>,
    error_count: u32,
}

impl TypingEngine {
    pub fn new(target_str: &str) -> Self {
        Self {
            target: target_str.chars().collect(),
            cursor: 0,
            keystrokes: Vec::new(),
            current_attempts: 0,
            error_flash: None,
            start_time: None,
            last_correct_time: None,
            error_count: 0,
        }
    }

    pub fn input(&mut self, ch: char) -> InputResult {
        if self.is_complete() {
            return InputResult::AlreadyComplete;
        }

        // Start timer on first input
        if self.start_time.is_none() {
            self.start_time = Some(Instant::now());
            self.last_correct_time = Some(Instant::now());
        }

        self.current_attempts += 1;
        let expected = self.target[self.cursor];

        if ch == expected {
            let now = Instant::now();
            let latency_ms = self
                .last_correct_time
                .map(|t| now.duration_since(t).as_millis() as u64)
                .unwrap_or(0);

            let keystroke = Keystroke {
                expected,
                actual: ch,
                correct: self.current_attempts == 1,
                attempts: self.current_attempts,
                latency_ms,
                timestamp_ms: Utc::now().timestamp_millis(),
            };

            self.keystrokes.push(keystroke);
            self.cursor += 1;
            self.current_attempts = 0;
            self.last_correct_time = Some(now);
            self.error_flash = None;

            InputResult::Correct
        } else {
            self.error_count += 1;
            self.error_flash = Some(Instant::now());
            InputResult::Error(expected)
        }
    }

    pub fn is_complete(&self) -> bool {
        self.cursor >= self.target.len()
    }

    pub fn current_wpm(&self) -> f64 {
        let elapsed_secs = self
            .start_time
            .map(|t| t.elapsed().as_secs_f64())
            .unwrap_or(0.0);
        if elapsed_secs < 0.1 {
            return 0.0;
        }
        let correct_chars = self.cursor as f64;
        (correct_chars / 5.0) / (elapsed_secs / 60.0)
    }

    pub fn current_accuracy(&self) -> f64 {
        if self.keystrokes.is_empty() {
            return 1.0;
        }
        let first_try_correct = self.keystrokes.iter().filter(|k| k.correct).count() as f64;
        let total = self.keystrokes.len() as f64;
        first_try_correct / total
    }

    pub fn elapsed_secs(&self) -> f64 {
        self.start_time
            .map(|t| t.elapsed().as_secs_f64())
            .unwrap_or(0.0)
    }

    pub fn finish(&self, command_id: &str, mode: Mode, difficulty: Difficulty) -> SessionRecord {
        let now_ms = Utc::now().timestamp_millis();
        let elapsed_secs = self.elapsed_secs();
        let correct_chars = self.cursor as f64;
        let elapsed_mins = elapsed_secs / 60.0;

        let wpm = if elapsed_mins > 0.0 {
            (correct_chars / 5.0) / elapsed_mins
        } else {
            0.0
        };
        let cpm = if elapsed_mins > 0.0 {
            correct_chars / elapsed_mins
        } else {
            0.0
        };

        SessionRecord {
            id: format!("{}", now_ms),
            command_id: command_id.to_string(),
            mode,
            keystrokes: self.keystrokes.clone(),
            started_at: self
                .start_time
                .map(|_| now_ms - (elapsed_secs * 1000.0) as i64)
                .unwrap_or(now_ms),
            finished_at: now_ms,
            wpm,
            cpm,
            accuracy: self.current_accuracy(),
            error_count: self.error_count,
            difficulty,
        }
    }

    /// Check if error flash should still be active (within 150ms)
    pub fn is_error_flashing(&self) -> bool {
        self.error_flash
            .map(|t| t.elapsed().as_millis() < 150)
            .unwrap_or(false)
    }

    /// Reset to type the same target again
    pub fn reset(&mut self) {
        self.cursor = 0;
        self.keystrokes.clear();
        self.current_attempts = 0;
        self.error_flash = None;
        self.start_time = None;
        self.last_correct_time = None;
        self.error_count = 0;
    }
}
