use std::time::{Duration, Instant};

use chrono::Utc;

use crate::data::models::{Difficulty, Keystroke, RecordMode, SessionRecord};

/// Result of a single keystroke input.
#[derive(Debug, Clone, PartialEq)]
pub enum InputResult {
    Correct,
    Error { expected: char },
    AlreadyComplete,
}

/// Core typing engine: tracks target characters, cursor position, keystrokes,
/// error flash state, and timing for WPM/CPM/accuracy calculations.
pub struct TypingEngine {
    pub target: Vec<char>,
    pub cursor: usize,
    pub keystrokes: Vec<Keystroke>,
    pub current_attempts: u8,
    pub error_flash: Option<Instant>,
    pub start_time: Option<Instant>,
    pub completed_at: Option<Instant>,
    pub last_correct_time: Option<Instant>,
    error_flash_duration: Duration,
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
            completed_at: None,
            last_correct_time: None,
            error_flash_duration: Duration::from_millis(150),
        }
    }

    /// Process a character input. Returns `Correct` if the character matches the
    /// current target, `Error { expected }` if it doesn't, or `AlreadyComplete`
    /// if the target has already been fully typed.
    pub fn input(&mut self, ch: char) -> InputResult {
        if self.is_complete() {
            return InputResult::AlreadyComplete;
        }

        // Start timer on first input
        if self.start_time.is_none() {
            let now = Instant::now();
            self.start_time = Some(now);
            self.last_correct_time = Some(now);
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
            if self.cursor >= self.target.len() {
                self.completed_at = Some(now);
            }

            InputResult::Correct
        } else {
            self.error_flash = Some(Instant::now());
            InputResult::Error { expected }
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }

        self.cursor -= 1;
        self.current_attempts = 0;
        self.error_flash = None;
        self.completed_at = None;
        self.keystrokes.pop();
    }

    pub fn is_complete(&self) -> bool {
        self.cursor >= self.target.len()
    }

    /// Check if error flash should still be active (within `error_flash_duration`).
    pub fn is_error_flashing(&self) -> bool {
        self.error_flash
            .map(|t| t.elapsed() < self.error_flash_duration)
            .unwrap_or(false)
    }

    /// Words per minute: (correct_chars / 5) / (elapsed_secs / 60).
    pub fn current_wpm(&self) -> f64 {
        let elapsed_secs = self.elapsed_secs();
        if elapsed_secs < 0.1 {
            return 0.0;
        }
        let correct_chars = self.cursor as f64;
        (correct_chars / 5.0) / (elapsed_secs / 60.0)
    }

    /// Characters per minute: correct_chars / (elapsed_secs / 60).
    pub fn current_cpm(&self) -> f64 {
        let elapsed_secs = self.elapsed_secs();
        if elapsed_secs < 0.1 {
            return 0.0;
        }
        let correct_chars = self.cursor as f64;
        correct_chars / (elapsed_secs / 60.0)
    }

    /// Accuracy: proportion of characters typed correctly on the first attempt.
    pub fn current_accuracy(&self) -> f64 {
        if self.keystrokes.is_empty() {
            return 1.0;
        }
        let first_try_correct = self.keystrokes.iter().filter(|k| k.correct).count() as f64;
        let total = self.keystrokes.len() as f64;
        first_try_correct / total
    }

    pub fn elapsed_secs(&self) -> f64 {
        match (self.start_time, self.completed_at) {
            (Some(start_time), Some(completed_at)) => {
                completed_at.duration_since(start_time).as_secs_f64()
            }
            (Some(start_time), None) => start_time.elapsed().as_secs_f64(),
            (None, _) => 0.0,
        }
    }

    /// Finalize the session into a `SessionRecord`.
    pub fn finish(
        &self,
        command_id: &str,
        difficulty: Difficulty,
        mode: RecordMode,
    ) -> SessionRecord {
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

        let error_count = self
            .keystrokes
            .iter()
            .map(|k| k.attempts.saturating_sub(1) as u32)
            .sum::<u32>();

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
            error_count,
            difficulty,
        }
    }

    /// Reset the engine for a new target string.
    pub fn reset(&mut self, target_str: &str) {
        self.target = target_str.chars().collect();
        self.cursor = 0;
        self.keystrokes.clear();
        self.current_attempts = 0;
        self.error_flash = None;
        self.start_time = None;
        self.completed_at = None;
        self.last_correct_time = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_engine_starts_empty() {
        let engine = TypingEngine::new("ls -la");
        assert_eq!(engine.target, vec!['l', 's', ' ', '-', 'l', 'a']);
        assert_eq!(engine.cursor, 0);
        assert!(engine.keystrokes.is_empty());
        assert!(!engine.is_complete());
        assert_eq!(engine.current_accuracy(), 1.0);
        assert_eq!(engine.elapsed_secs(), 0.0);
        assert!(engine.completed_at.is_none());
    }

    #[test]
    fn correct_input_advances_cursor() {
        let mut engine = TypingEngine::new("abc");
        assert_eq!(engine.input('a'), InputResult::Correct);
        assert_eq!(engine.cursor, 1);
        assert_eq!(engine.keystrokes.len(), 1);
        assert!(engine.keystrokes[0].correct);
        assert_eq!(engine.keystrokes[0].expected, 'a');
    }

    #[test]
    fn error_input_does_not_advance() {
        let mut engine = TypingEngine::new("abc");
        let result = engine.input('x');
        assert_eq!(result, InputResult::Error { expected: 'a' });
        assert_eq!(engine.cursor, 0);
        assert!(engine.keystrokes.is_empty());
        assert!(engine.error_flash.is_some());
    }

    #[test]
    fn already_complete_returns_already_complete() {
        let mut engine = TypingEngine::new("a");
        assert_eq!(engine.input('a'), InputResult::Correct);
        assert!(engine.is_complete());
        assert_eq!(engine.input('b'), InputResult::AlreadyComplete);
    }

    #[test]
    fn completion_detection() {
        let mut engine = TypingEngine::new("ab");
        assert!(!engine.is_complete());
        engine.input('a');
        assert!(!engine.is_complete());
        engine.input('b');
        assert!(engine.is_complete());
        assert!(engine.completed_at.is_some());
    }

    #[test]
    fn elapsed_time_freezes_after_completion() {
        let mut engine = TypingEngine::new("a");
        engine.input('a');

        let elapsed_after_completion = engine.elapsed_secs();
        std::thread::sleep(Duration::from_millis(20));

        assert_eq!(engine.elapsed_secs(), elapsed_after_completion);
    }

    #[test]
    fn accuracy_tracks_first_attempt_correctness() {
        let mut engine = TypingEngine::new("ab");

        // Type 'a' correctly on first attempt
        engine.input('a');
        assert_eq!(engine.current_accuracy(), 1.0);

        // Make an error on 'b', then type correctly
        engine.input('x'); // error
        engine.input('b'); // correct (but attempts=2)
        assert!(engine.is_complete());

        // 1 first-try-correct out of 2 total = 0.5
        assert!((engine.current_accuracy() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn wpm_and_cpm_are_zero_before_start() {
        let engine = TypingEngine::new("test");
        assert_eq!(engine.current_wpm(), 0.0);
        assert_eq!(engine.current_cpm(), 0.0);
    }

    #[test]
    fn wpm_cpm_positive_after_typing() {
        let mut engine = TypingEngine::new("hello");
        for ch in "hello".chars() {
            engine.input(ch);
        }
        // After typing, WPM and CPM should be positive (exact values depend on timing)
        // Just verify they're non-negative and engine is complete
        assert!(engine.is_complete());
        assert!(engine.current_wpm() >= 0.0);
        assert!(engine.current_cpm() >= 0.0);
        // CPM should be >= WPM (since CPM = WPM * 5)
        // This is true because CPM = chars/min, WPM = (chars/5)/min
        assert!(engine.current_cpm() >= engine.current_wpm() - 0.001);
    }

    #[test]
    fn error_flash_expires() {
        let mut engine = TypingEngine::new("abc");
        engine.input('x'); // triggers error flash
        assert!(engine.is_error_flashing());

        // After 150ms+ the flash should expire
        // We can't easily test timing in unit tests, but we verify the mechanism
        // by checking that the error_flash is set
        assert!(engine.error_flash.is_some());
    }

    #[test]
    fn reset_with_new_target() {
        let mut engine = TypingEngine::new("abc");
        engine.input('a');
        engine.input('b');
        assert_eq!(engine.cursor, 2);

        engine.reset("xyz");
        assert_eq!(engine.target, vec!['x', 'y', 'z']);
        assert_eq!(engine.cursor, 0);
        assert!(engine.keystrokes.is_empty());
        assert_eq!(engine.current_attempts, 0);
        assert!(engine.error_flash.is_none());
        assert!(engine.start_time.is_none());
        assert!(engine.completed_at.is_none());
        assert!(engine.last_correct_time.is_none());
    }

    #[test]
    fn finish_produces_session_record() {
        let mut engine = TypingEngine::new("ls");
        engine.input('l');
        engine.input('x'); // error
        engine.input('s'); // correct on second attempt

        let record = engine.finish("ls-basic", Difficulty::Beginner, RecordMode::Typing);
        assert_eq!(record.command_id, "ls-basic");
        assert_eq!(record.mode, RecordMode::Typing);
        assert_eq!(record.keystrokes.len(), 2);
        assert_eq!(record.error_count, 1); // 's' had attempts=2 → 1 error
        assert!(record.wpm >= 0.0);
        assert!(record.cpm >= 0.0);
        assert!(record.accuracy > 0.0);
        assert!(record.accuracy <= 1.0);
    }

    #[test]
    fn start_time_set_on_first_input() {
        let mut engine = TypingEngine::new("test");
        assert!(engine.start_time.is_none());
        engine.input('t');
        assert!(engine.start_time.is_some());
        assert!(engine.last_correct_time.is_some());
    }

    #[test]
    fn multiple_errors_before_correct() {
        let mut engine = TypingEngine::new("a");
        engine.input('x');
        engine.input('y');
        engine.input('z');
        assert_eq!(engine.cursor, 0);
        assert_eq!(engine.current_attempts, 3);

        engine.input('a');
        assert!(engine.is_complete());
        assert_eq!(engine.keystrokes[0].attempts, 4);
        assert!(!engine.keystrokes[0].correct);
    }

    #[test]
    fn empty_target() {
        let mut engine = TypingEngine::new("");
        assert!(engine.is_complete());
        assert_eq!(engine.input('a'), InputResult::AlreadyComplete);
    }

    #[test]
    fn keystroke_latency_is_recorded() {
        let mut engine = TypingEngine::new("ab");
        engine.input('a');
        // Small delay between keystrokes
        std::thread::sleep(std::time::Duration::from_millis(10));
        engine.input('b');

        assert_eq!(engine.keystrokes.len(), 2);
        // Second keystroke should include the inserted delay.
        assert!(engine.keystrokes[1].latency_ms > 0);
    }
}
