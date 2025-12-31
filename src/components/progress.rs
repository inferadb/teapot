//! Progress bar component.
//!
//! Displays a progress bar with optional percentage and message.
//!
//! # Example
//!
//! ```rust
//! use teapot::components::Progress;
//!
//! let progress = Progress::new()
//!     .total(100)
//!     .current(45)
//!     .message("Downloading...");
//! ```

use crate::{
    runtime::{Cmd, Model},
    style::Color,
    terminal::Event,
};

/// Message type for progress bar.
#[derive(Debug, Clone)]
pub enum ProgressMsg {
    /// Set the current progress.
    SetProgress(u64),
    /// Increment progress by amount.
    Increment(u64),
    /// Set the message.
    SetMessage(String),
    /// Mark as complete.
    Complete,
}

/// A progress bar component.
#[derive(Debug, Clone)]
pub struct Progress {
    current: u64,
    total: u64,
    message: String,
    width: usize,
    show_percentage: bool,
    show_count: bool,
    filled_char: char,
    empty_char: char,
    filled_color: Color,
    empty_color: Color,
    complete: bool,
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
}

impl Progress {
    /// Create a new progress bar.
    pub fn new() -> Self {
        Self {
            current: 0,
            total: 100,
            message: String::new(),
            width: 40,
            show_percentage: true,
            show_count: false,
            filled_char: '█',
            empty_char: '░',
            filled_color: Color::Cyan,
            empty_color: Color::BrightBlack,
            complete: false,
        }
    }

    /// Set the total value.
    pub fn total(mut self, total: u64) -> Self {
        self.total = total;
        self
    }

    /// Set the current value.
    pub fn current(mut self, current: u64) -> Self {
        self.current = current.min(self.total);
        self
    }

    /// Set the message to display.
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    /// Set the bar width in characters.
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Whether to show percentage.
    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    /// Whether to show count (current/total).
    pub fn show_count(mut self, show: bool) -> Self {
        self.show_count = show;
        self
    }

    /// Set the filled character.
    pub fn filled_char(mut self, c: char) -> Self {
        self.filled_char = c;
        self
    }

    /// Set the empty character.
    pub fn empty_char(mut self, c: char) -> Self {
        self.empty_char = c;
        self
    }

    /// Set the filled color.
    pub fn filled_color(mut self, color: Color) -> Self {
        self.filled_color = color;
        self
    }

    /// Set the empty color.
    pub fn empty_color(mut self, color: Color) -> Self {
        self.empty_color = color;
        self
    }

    /// Get the current progress as a percentage.
    pub fn percentage(&self) -> f64 {
        if self.total == 0 { 100.0 } else { (self.current as f64 / self.total as f64) * 100.0 }
    }

    /// Check if progress is complete.
    pub fn is_complete(&self) -> bool {
        self.complete || self.current >= self.total
    }

    /// Set progress to current value.
    pub fn set(&mut self, current: u64) {
        self.current = current.min(self.total);
        if self.current >= self.total {
            self.complete = true;
        }
    }

    /// Increment progress.
    pub fn increment(&mut self, amount: u64) {
        self.current = (self.current + amount).min(self.total);
        if self.current >= self.total {
            self.complete = true;
        }
    }

    /// Mark as complete.
    pub fn complete(&mut self) {
        self.current = self.total;
        self.complete = true;
    }
}

impl Model for Progress {
    type Message = ProgressMsg;

    fn init(&self) -> Option<Cmd<Self::Message>> {
        None
    }

    fn update(&mut self, msg: Self::Message) -> Option<Cmd<Self::Message>> {
        match msg {
            ProgressMsg::SetProgress(current) => {
                self.set(current);
            },
            ProgressMsg::Increment(amount) => {
                self.increment(amount);
            },
            ProgressMsg::SetMessage(message) => {
                self.message = message;
            },
            ProgressMsg::Complete => {
                self.complete();
            },
        }
        None
    }

    fn view(&self) -> String {
        let pct = self.percentage();
        let filled_count = ((pct / 100.0) * self.width as f64).round() as usize;
        let empty_count = self.width.saturating_sub(filled_count);

        let filled = self.filled_char.to_string().repeat(filled_count);
        let empty = self.empty_char.to_string().repeat(empty_count);

        let bar = format!(
            "{}{}{}{}{}",
            self.filled_color.to_ansi_fg(),
            filled,
            self.empty_color.to_ansi_fg(),
            empty,
            "\x1b[0m"
        );

        let mut parts = Vec::new();

        if !self.message.is_empty() {
            parts.push(self.message.clone());
        }

        parts.push(format!("[{}]", bar));

        if self.show_percentage {
            parts.push(format!("{:.0}%", pct));
        }

        if self.show_count {
            parts.push(format!("{}/{}", self.current, self.total));
        }

        parts.join(" ")
    }

    fn handle_event(&self, _event: Event) -> Option<Self::Message> {
        None
    }
}

/// Progress bar style presets.
impl Progress {
    /// Classic ASCII style.
    pub fn ascii() -> Self {
        Self::new()
            .filled_char('=')
            .empty_char('-')
            .filled_color(Color::Green)
            .empty_color(Color::BrightBlack)
    }

    /// Block style with gradient colors.
    pub fn blocks() -> Self {
        Self::new()
            .filled_char('█')
            .empty_char('░')
            .filled_color(Color::Cyan)
            .empty_color(Color::BrightBlack)
    }

    /// Dots style.
    pub fn dots() -> Self {
        Self::new()
            .filled_char('●')
            .empty_char('○')
            .filled_color(Color::Green)
            .empty_color(Color::BrightBlack)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_creation() {
        let progress = Progress::new().total(100).current(50);
        assert_eq!(progress.percentage(), 50.0);
    }

    #[test]
    fn test_progress_increment() {
        let mut progress = Progress::new().total(100);
        progress.increment(25);
        assert_eq!(progress.current, 25);
        progress.increment(25);
        assert_eq!(progress.current, 50);
    }

    #[test]
    fn test_progress_complete() {
        let mut progress = Progress::new().total(100);
        assert!(!progress.is_complete());
        progress.complete();
        assert!(progress.is_complete());
    }

    #[test]
    fn test_progress_view() {
        let progress = Progress::new().total(100).current(50).message("Loading").width(10);
        let view = progress.view();
        assert!(view.contains("Loading"));
        assert!(view.contains("50%"));
    }
}
