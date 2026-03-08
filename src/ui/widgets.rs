use ratatui::style::Color;

pub mod colors {
    use super::Color;

    pub const TYPED_CORRECT: Color = Color::White;
    pub const PENDING: Color = Color::DarkGray;
    pub const PENDING_BG: Color = Color::Rgb(40, 40, 40);
    pub const CURSOR: Color = Color::Black;
    pub const CURSOR_BG: Color = Color::White;
    pub const ERROR_FLASH: Color = Color::White;
    pub const ERROR_FLASH_BG: Color = Color::Red;
    pub const TOKEN_DESC: Color = Color::Cyan;
    pub const TREE_LINE: Color = Color::DarkGray;
    pub const HEADER: Color = Color::Yellow;
    pub const ACCENT: Color = Color::Green;
}

/// Format elapsed seconds as MM:SS
pub fn format_time(secs: f64) -> String {
    let total = secs as u64;
    let m = total / 60;
    let s = total % 60;
    format!("{:02}:{:02}", m, s)
}

pub fn format_duration_ms(ms: u64) -> String {
    format_time(ms as f64 / 1000.0)
}
