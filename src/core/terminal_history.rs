use crate::data::models::{LineStatus, TerminalLine};

/// History of terminal lines for the "typing practice" mode.
/// Tracks completed and in-progress lines for terminal-style rendering.
pub struct TerminalHistory {
    lines: Vec<TerminalLine>,
    max_visible: usize,
}

impl TerminalHistory {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            max_visible: 100,
        }
    }

    /// Add a completed line to the history.
    pub fn push_completed(&mut self, prompt: &str, display: &str) {
        self.lines.push(TerminalLine {
            prompt: prompt.to_string(),
            command_display: display.to_string(),
            status: LineStatus::Completed,
        });
        let overflow = self.lines.len().saturating_sub(self.max_visible);
        if overflow > 0 {
            self.lines.drain(..overflow);
        }
    }

    /// Return the tail of visible lines that fit within `height` rows.
    pub fn visible_lines(&self, height: u16) -> &[TerminalLine] {
        let height = height as usize;
        if self.lines.len() <= height {
            &self.lines
        } else {
            &self.lines[self.lines.len() - height..]
        }
    }

    /// Clear all history.
    pub fn clear(&mut self) {
        self.lines.clear();
    }

    /// Total number of lines in history.
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Whether the history is empty.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

impl Default for TerminalHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_history_is_empty() {
        let history = TerminalHistory::new();
        assert!(history.is_empty());
        assert_eq!(history.len(), 0);
        assert!(history.visible_lines(10).is_empty());
    }

    #[test]
    fn push_completed_adds_line() {
        let mut history = TerminalHistory::new();
        history.push_completed("$ ", "ls -la");

        assert_eq!(history.len(), 1);
        assert!(!history.is_empty());

        let lines = history.visible_lines(10);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].prompt, "$ ");
        assert_eq!(lines[0].command_display, "ls -la");
        assert_eq!(lines[0].status, LineStatus::Completed);
    }

    #[test]
    fn visible_lines_returns_all_when_fits() {
        let mut history = TerminalHistory::new();
        history.push_completed("$ ", "cmd1");
        history.push_completed("$ ", "cmd2");
        history.push_completed("$ ", "cmd3");

        let lines = history.visible_lines(10);
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn visible_lines_truncates_to_height() {
        let mut history = TerminalHistory::new();
        for i in 0..10 {
            history.push_completed("$ ", &format!("cmd{i}"));
        }

        let lines = history.visible_lines(3);
        assert_eq!(lines.len(), 3);
        // Should show the last 3 lines
        assert_eq!(lines[0].command_display, "cmd7");
        assert_eq!(lines[1].command_display, "cmd8");
        assert_eq!(lines[2].command_display, "cmd9");
    }

    #[test]
    fn visible_lines_zero_height() {
        let mut history = TerminalHistory::new();
        history.push_completed("$ ", "cmd1");

        let lines = history.visible_lines(0);
        // With height 0, we should get an empty slice since len > 0 >= height
        assert!(lines.is_empty());
    }

    #[test]
    fn clear_removes_all_lines() {
        let mut history = TerminalHistory::new();
        history.push_completed("$ ", "cmd1");
        history.push_completed("$ ", "cmd2");
        assert_eq!(history.len(), 2);

        history.clear();
        assert!(history.is_empty());
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn multiple_push_and_visible() {
        let mut history = TerminalHistory::new();

        history.push_completed("user@host:~$ ", "ls -la /var/log");
        history.push_completed("user@host:~$ ", "grep -r 'error' /var/log");
        history.push_completed("$ ", "pwd");

        let lines = history.visible_lines(2);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].prompt, "user@host:~$ ");
        assert_eq!(lines[0].command_display, "grep -r 'error' /var/log");
        assert_eq!(lines[1].prompt, "$ ");
        assert_eq!(lines[1].command_display, "pwd");
    }

    #[test]
    fn default_creates_empty_history() {
        let history = TerminalHistory::default();
        assert!(history.is_empty());
    }
}
