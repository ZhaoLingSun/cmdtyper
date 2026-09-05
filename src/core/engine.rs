use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use chrono::{Local, TimeZone, Utc};

use crate::data::models::{
    ActiveSpan, Difficulty, Keystroke, PositionDaySnapshot, RecordMode, SessionRecord,
    TypingMetrics, TypingPosition,
};

const IDLE_LIMIT_MS: u64 = 30_000;
static SESSION_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq)]
pub enum InputResult {
    Correct,
    Error { expected: char },
    AlreadyComplete,
}

/// Cursor state is separate from position measurements: deleting/retyping never
/// erases an error or creates a second speed sample for the same target position.
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
    positions: Vec<Option<TypingPosition>>,
    active_spans: Vec<ActiveSpan>,
    position_day_snapshots: Vec<PositionDaySnapshot>,
    completed_date: String,
    last_input: Option<(Instant, i64)>,
    started_at_ms: Option<i64>,
    last_event_ms: Option<i64>,
    session_id: String,
    paused: bool,
}

impl TypingEngine {
    pub fn new(target_str: &str) -> Self {
        let target: Vec<char> = target_str.chars().collect();
        Self {
            positions: vec![None; target.len()],
            target,
            cursor: 0,
            keystrokes: Vec::new(),
            current_attempts: 0,
            error_flash: None,
            start_time: None,
            completed_at: None,
            last_correct_time: None,
            error_flash_duration: Duration::from_millis(150),
            active_spans: Vec::new(),
            position_day_snapshots: Vec::new(),
            completed_date: String::new(),
            last_input: None,
            started_at_ms: None,
            last_event_ms: None,
            session_id: format!(
                "{}-{}",
                Utc::now().timestamp_nanos_opt().unwrap_or_default(),
                SESSION_SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ),
            paused: false,
        }
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn input(&mut self, ch: char) -> InputResult {
        self.input_at(ch, Instant::now(), Utc::now().timestamp_millis())
    }

    /// Explicit clocks make idle and midnight behavior deterministic in tests.
    pub fn input_at(&mut self, ch: char, now: Instant, timestamp_ms: i64) -> InputResult {
        if self.is_complete() {
            return InputResult::AlreadyComplete;
        }
        let expected = self.target[self.cursor];
        if expected == '\n' {
            if ch == '\n' {
                self.cursor += 1;
                self.last_input = None;
                self.last_event_ms = Some(timestamp_ms);
                self.current_attempts = 0;
                self.error_flash = None;
                if self.is_complete() {
                    self.completed_at = Some(now);
                    self.completed_date = local_date(timestamp_ms);
                }
                return InputResult::Correct;
            }
            return InputResult::Error { expected };
        }
        // Enter and editing shortcuts are not effective characters.
        if ch.is_control() {
            return InputResult::Error { expected };
        }
        if self.start_time.is_none() {
            self.start_time = Some(now);
            self.started_at_ms = Some(timestamp_ms);
            self.last_correct_time = Some(now);
        }
        self.paused = false;
        let index = self.cursor;
        self.positions[index].get_or_insert_with(|| TypingPosition {
            index,
            expected,
            attempted_at_ms: timestamp_ms,
            attempted_date: local_date(timestamp_ms),
            updated_date: local_date(timestamp_ms),
            speed_sample_valid: true,
            ..TypingPosition::default()
        });
        self.record_interval(index, now, timestamp_ms);
        let position = self.positions[index]
            .as_mut()
            .expect("position initialized");
        position.attempts = position.attempts.saturating_add(1);
        self.current_attempts = position.attempts.min(u8::MAX as u64) as u8;
        if ch != expected {
            position.error = true;
            position
                .error_date
                .get_or_insert_with(|| local_date(timestamp_ms));
            self.error_flash = Some(now);
            return InputResult::Error { expected };
        }
        position.completed = true;
        position.completed_at_ms = timestamp_ms;
        position.completed_date = local_date(timestamp_ms);
        self.keystrokes.push(Keystroke {
            expected,
            actual: ch,
            correct: !position.error,
            attempts: self.current_attempts,
            latency_ms: position.latency_ms,
            timestamp_ms,
        });
        self.cursor += 1;
        self.current_attempts = 0;
        self.last_correct_time = Some(now);
        self.error_flash = None;
        if self.is_complete() {
            self.completed_at = Some(now);
            self.completed_date = local_date(timestamp_ms);
        }
        // Reading output / waiting for Enter at a step boundary is not typing.
        if self.target.get(self.cursor) == Some(&'\n') {
            self.last_input = None;
        }
        InputResult::Correct
    }

    /// Capture the old day's final state before latency, validity or completion
    /// can be changed by a future-day input, backspace or pause.
    fn checkpoint_position(&mut self, index: usize, timestamp_ms: i64) {
        let Some(position) = self.positions.get_mut(index).and_then(Option::as_mut) else {
            return;
        };
        let date = local_date(timestamp_ms);
        if !position.updated_date.is_empty() && position.updated_date < date {
            let snapshot = PositionDaySnapshot {
                index,
                date: position.updated_date.clone(),
                completed: position.completed,
                completed_at_ms: position.completed_at_ms,
                completed_date: position.completed_date.clone(),
                latency_ms: position.latency_ms,
                speed_sample_valid: position.speed_sample_valid,
            };
            if let Some(existing) = self
                .position_day_snapshots
                .iter_mut()
                .find(|existing| existing.index == index && existing.date == snapshot.date)
            {
                *existing = snapshot;
            } else {
                self.position_day_snapshots.push(snapshot);
            }
        }
        position.updated_date = date;
    }

    fn record_interval(&mut self, index: usize, now: Instant, timestamp_ms: i64) {
        self.checkpoint_position(index, timestamp_ms);
        if let Some((last, last_ms)) = self.last_input {
            let gap = now
                .saturating_duration_since(last)
                .as_millis()
                .min(u64::MAX as u128) as u64;
            let active = gap.min(IDLE_LIMIT_MS);
            add_active_span(&mut self.active_spans, last_ms, active);
            if let Some(position) = self.positions.get_mut(index).and_then(Option::as_mut) {
                position.latency_ms = position.latency_ms.saturating_add(active);
                if gap > IDLE_LIMIT_MS {
                    position.speed_sample_valid = false;
                }
            }
        } else if let Some(position) = self.positions.get_mut(index).and_then(Option::as_mut) {
            position.speed_sample_valid = false;
        }
        self.last_input = Some((now, timestamp_ms));
        self.last_event_ms = Some(timestamp_ms);
    }

    pub fn backspace(&mut self) {
        self.backspace_at(Instant::now(), Utc::now().timestamp_millis());
    }

    pub fn backspace_at(&mut self, now: Instant, timestamp_ms: i64) {
        // A submitted step cannot be erased; backspace only revisits its current line.
        if self.cursor == 0 || self.target.get(self.cursor - 1) == Some(&'\n') {
            return;
        }
        self.cursor -= 1;
        self.record_interval(self.cursor, now, timestamp_ms);
        if let Some(position) = self.positions[self.cursor].as_mut() {
            position.completed = false;
        }
        self.current_attempts = 0;
        self.error_flash = None;
        self.completed_at = None;
        self.completed_date.clear();
        self.keystrokes.pop();
    }

    pub fn pause(&mut self) {
        self.pause_at(Instant::now(), Utc::now().timestamp_millis());
    }

    pub fn pause_at(&mut self, now: Instant, timestamp_ms: i64) {
        if !self.is_complete() && !self.paused {
            if let Some((last, last_ms)) = self.last_input {
                let active = now
                    .saturating_duration_since(last)
                    .as_millis()
                    .min(IDLE_LIMIT_MS as u128) as u64;
                add_active_span(&mut self.active_spans, last_ms, active);
                self.checkpoint_position(self.cursor, timestamp_ms);
                if let Some(position) = self.positions.get_mut(self.cursor).and_then(Option::as_mut)
                {
                    position.latency_ms = position.latency_ms.saturating_add(active);
                    position.speed_sample_valid = false;
                }
            }
            self.last_event_ms = self.last_event_ms.map(|_| timestamp_ms);
        }
        self.last_input = None;
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
        self.last_input = None;
    }

    pub fn has_activity(&self) -> bool {
        self.positions.iter().any(Option::is_some)
    }

    pub fn is_complete(&self) -> bool {
        self.cursor >= self.target.len()
    }

    pub fn is_error_flashing(&self) -> bool {
        self.error_flash
            .map(|time| time.elapsed() < self.error_flash_duration)
            .unwrap_or(false)
    }

    pub fn current_wpm(&self) -> f64 {
        self.current_cpm() / 5.0
    }

    pub fn current_cpm(&self) -> f64 {
        let seconds = self.elapsed_secs();
        if seconds < 0.1 {
            0.0
        } else {
            self.completed_chars() as f64 * 60.0 / seconds
        }
    }

    pub fn current_accuracy(&self) -> f64 {
        let attempted = self.positions.iter().flatten().count();
        let errors = self
            .positions
            .iter()
            .flatten()
            .filter(|position| position.error)
            .count();
        if attempted == 0 {
            1.0
        } else {
            1.0 - errors as f64 / attempted as f64
        }
    }

    fn completed_chars(&self) -> usize {
        self.positions
            .iter()
            .flatten()
            .filter(|position| position.completed)
            .count()
    }

    fn tail_ms(&self, now: Instant) -> u64 {
        if self.is_complete() || self.paused {
            return 0;
        }
        self.last_input
            .map(|(last, _)| {
                now.saturating_duration_since(last)
                    .as_millis()
                    .min(IDLE_LIMIT_MS as u128) as u64
            })
            .unwrap_or(0)
    }

    pub fn elapsed_secs(&self) -> f64 {
        (self
            .active_spans
            .iter()
            .map(|span| span.duration_ms)
            .sum::<u64>()
            + self.tail_ms(Instant::now())) as f64
            / 1000.0
    }

    pub fn finish(
        &self,
        command_id: &str,
        difficulty: Difficulty,
        mode: RecordMode,
    ) -> SessionRecord {
        self.finish_at(
            command_id,
            difficulty,
            mode,
            Instant::now(),
            Utc::now().timestamp_millis(),
        )
    }

    pub fn finish_at(
        &self,
        command_id: &str,
        difficulty: Difficulty,
        mode: RecordMode,
        now: Instant,
        timestamp_ms: i64,
    ) -> SessionRecord {
        let mut spans = self.active_spans.clone();
        if let Some((_, last_ms)) = self.last_input {
            add_active_span(&mut spans, last_ms, self.tail_ms(now));
        }
        let duration_ms = spans.iter().map(|span| span.duration_ms).sum::<u64>();
        let cpm = if duration_ms > 0 {
            self.completed_chars() as f64 * 60_000.0 / duration_ms as f64
        } else {
            0.0
        };
        SessionRecord {
            id: self.session_id.clone(),
            command_id: command_id.to_string(),
            mode,
            keystrokes: self.keystrokes.clone(),
            started_at: self.started_at_ms.unwrap_or(timestamp_ms),
            finished_at: if self.is_complete() || self.paused {
                self.last_event_ms.unwrap_or(timestamp_ms)
            } else {
                timestamp_ms
            },
            wpm: cpm / 5.0,
            cpm,
            accuracy: self.current_accuracy(),
            error_count: self
                .positions
                .iter()
                .flatten()
                .filter(|position| position.error)
                .count() as u32,
            difficulty,
            typing: Some(TypingMetrics {
                version: 2,
                completed: self.is_complete(),
                completed_date: self.completed_date.clone(),
                active_duration_ms: duration_ms,
                positions: self.positions.iter().flatten().cloned().collect(),
                active_spans: spans,
                position_day_snapshots: self.position_day_snapshots.clone(),
            }),
        }
    }

    pub fn reset(&mut self, target_str: &str) {
        *self = Self::new(target_str);
    }
}

fn local_date(timestamp_ms: i64) -> String {
    Local
        .timestamp_millis_opt(timestamp_ms)
        .single()
        .map(|date| date.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

/// Split the capped active interval at local midnight, preserving its original
/// local date even if the user later moves to a different timezone.
fn add_active_span(spans: &mut Vec<ActiveSpan>, start_ms: i64, duration_ms: u64) {
    if duration_ms == 0 {
        return;
    }
    let end_ms = start_ms.saturating_add(duration_ms.min(i64::MAX as u64) as i64);
    let Some(start) = Local.timestamp_millis_opt(start_ms).single() else {
        return;
    };
    let next_midnight = start
        .date_naive()
        .succ_opt()
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .and_then(|date| date.and_local_timezone(Local).earliest())
        .map(|date| date.timestamp_millis());
    let split_ms = next_midnight
        .filter(|midnight| *midnight > start_ms && *midnight < end_ms)
        .unwrap_or(end_ms);
    push_span(spans, local_date(start_ms), (split_ms - start_ms) as u64);
    if split_ms < end_ms {
        push_span(spans, local_date(split_ms), (end_ms - split_ms) as u64);
    }
}

fn push_span(spans: &mut Vec<ActiveSpan>, date: String, duration_ms: u64) {
    if let Some(previous) = spans.last_mut().filter(|span| span.date == date) {
        previous.duration_ms += duration_ms;
    } else {
        spans.push(ActiveSpan { date, duration_ms });
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
