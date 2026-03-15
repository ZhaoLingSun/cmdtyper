use std::time::{Duration, Instant};

/// A pausable timer for tracking elapsed time.
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

    /// Start (or restart) the timer, resetting accumulated time.
    pub fn start(&mut self) {
        self.start = Some(Instant::now());
        self.elapsed = Duration::ZERO;
        self.paused = false;
    }

    /// Pause the timer, accumulating elapsed time so far.
    pub fn pause(&mut self) {
        if !self.paused {
            if let Some(start) = self.start {
                self.elapsed += start.elapsed();
            }
            self.start = None;
            self.paused = true;
        }
    }

    /// Resume the timer after a pause.
    pub fn resume(&mut self) {
        if self.paused {
            self.start = Some(Instant::now());
            self.paused = false;
        }
    }

    /// Total elapsed time (accumulated + current running segment).
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

    /// Format elapsed time as MM:SS.
    pub fn format_mmss(&self) -> String {
        let total_secs = self.elapsed().as_secs();
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{:02}:{:02}", mins, secs)
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn new_timer_has_zero_elapsed() {
        let timer = Timer::new();
        assert_eq!(timer.elapsed(), Duration::ZERO);
        assert_eq!(timer.elapsed_secs(), 0.0);
        assert_eq!(timer.format_mmss(), "00:00");
    }

    #[test]
    fn start_resets_and_begins_counting() {
        let mut timer = Timer::new();
        timer.start();
        thread::sleep(Duration::from_millis(20));
        assert!(timer.elapsed() >= Duration::from_millis(15));
    }

    #[test]
    fn pause_stops_accumulation() {
        let mut timer = Timer::new();
        timer.start();
        thread::sleep(Duration::from_millis(20));
        timer.pause();

        let paused_elapsed = timer.elapsed();
        thread::sleep(Duration::from_millis(30));
        let after_wait = timer.elapsed();

        // Elapsed should not change significantly while paused
        let drift = after_wait.saturating_sub(paused_elapsed);
        assert!(
            drift < Duration::from_millis(5),
            "elapsed changed by {drift:?} while paused"
        );
    }

    #[test]
    fn resume_continues_counting() {
        let mut timer = Timer::new();
        timer.start();
        thread::sleep(Duration::from_millis(20));
        timer.pause();

        let paused_elapsed = timer.elapsed();
        timer.resume();
        thread::sleep(Duration::from_millis(20));

        assert!(timer.elapsed() > paused_elapsed);
    }

    #[test]
    fn double_pause_is_idempotent() {
        let mut timer = Timer::new();
        timer.start();
        thread::sleep(Duration::from_millis(10));
        timer.pause();
        let first_pause = timer.elapsed();
        timer.pause(); // double pause
        let second_pause = timer.elapsed();

        let drift = second_pause.saturating_sub(first_pause);
        assert!(drift < Duration::from_millis(2));
    }

    #[test]
    fn double_resume_is_idempotent() {
        let mut timer = Timer::new();
        timer.start();
        thread::sleep(Duration::from_millis(10));
        timer.pause();
        timer.resume();
        timer.resume(); // double resume — should not reset
        thread::sleep(Duration::from_millis(10));

        // Should still have accumulated time from both segments
        assert!(timer.elapsed() >= Duration::from_millis(15));
    }

    #[test]
    fn format_mmss_displays_correctly() {
        let mut timer = Timer::new();
        // Manually set elapsed to test formatting
        timer.elapsed = Duration::from_secs(75); // 1:15
        timer.paused = true;
        assert_eq!(timer.format_mmss(), "01:15");

        timer.elapsed = Duration::from_secs(0);
        assert_eq!(timer.format_mmss(), "00:00");

        timer.elapsed = Duration::from_secs(3661); // 61:01
        assert_eq!(timer.format_mmss(), "61:01");
    }

    #[test]
    fn start_resets_previous_elapsed() {
        let mut timer = Timer::new();
        timer.start();
        thread::sleep(Duration::from_millis(20));
        timer.pause();
        assert!(timer.elapsed() >= Duration::from_millis(15));

        timer.start(); // restart
        // Should be near zero again
        assert!(timer.elapsed() < Duration::from_millis(5));
    }

    #[test]
    fn default_creates_new_timer() {
        let timer = Timer::default();
        assert_eq!(timer.elapsed(), Duration::ZERO);
    }
}
