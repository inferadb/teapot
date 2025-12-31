//! Footer hints component for keyboard shortcut display.
//!
//! Renders a footer with keyboard shortcuts and optional scroll indicators.
//!
//! # Example
//!
//! ```rust
//! use teapot::components::FooterHints;
//!
//! // Simple footer with hints
//! let footer = FooterHints::new()
//!     .hints(vec![("↑/↓", "select"), ("q", "quit")])
//!     .width(80);
//! println!("{}", footer.render());
//!
//! // With separator and scroll indicators
//! let footer = FooterHints::new()
//!     .hints(vec![("tab", "next"), ("S", "sort"), ("q", "quit")])
//!     .width(80)
//!     .with_separator()
//!     .scroll_left(true)
//!     .scroll_right(true);
//! println!("{}", footer.render());
//! ```

use crate::{
    runtime::{Cmd, Model},
    style::{Color, RESET},
    terminal::Event,
    util::measure_text,
};

/// Message type for footer hints (currently none needed).
#[derive(Debug, Clone)]
pub enum FooterHintsMsg {}

/// A footer component displaying keyboard shortcuts.
///
/// Renders hints in the format: `key desc  key desc  key desc`
/// with optional scroll indicators and separator line.
#[derive(Debug, Clone)]
pub struct FooterHints {
    /// Keyboard shortcuts as (key, description) pairs.
    hints: Vec<(String, String)>,
    /// Width of the footer in characters.
    width: usize,
    /// Color for key text.
    key_color: Color,
    /// Color for description text.
    desc_color: Color,
    /// Whether to show a separator line above the hints.
    show_separator: bool,
    /// Separator character (default: '─').
    separator_char: char,
    /// Whether to show left scroll indicator.
    show_scroll_left: bool,
    /// Whether to show right scroll indicator.
    show_scroll_right: bool,
    /// Left scroll indicator character.
    scroll_left_char: String,
    /// Right scroll indicator character.
    scroll_right_char: String,
}

impl Default for FooterHints {
    fn default() -> Self {
        Self {
            hints: Vec::new(),
            width: 80,
            key_color: Color::Default,
            desc_color: Color::BrightBlack,
            show_separator: false,
            separator_char: '─',
            show_scroll_left: false,
            show_scroll_right: false,
            scroll_left_char: "◀ ".to_string(),
            scroll_right_char: " ▶".to_string(),
        }
    }
}

impl FooterHints {
    /// Create a new footer hints component.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the keyboard hints.
    ///
    /// Each hint is a (key, description) pair.
    pub fn hints<K, D>(mut self, hints: Vec<(K, D)>) -> Self
    where
        K: Into<String>,
        D: Into<String>,
    {
        self.hints = hints.into_iter().map(|(k, d)| (k.into(), d.into())).collect();
        self
    }

    /// Add a single hint.
    pub fn hint(mut self, key: impl Into<String>, desc: impl Into<String>) -> Self {
        self.hints.push((key.into(), desc.into()));
        self
    }

    /// Set the width in characters.
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Set the key text color.
    pub fn key_color(mut self, color: Color) -> Self {
        self.key_color = color;
        self
    }

    /// Set the description text color.
    pub fn desc_color(mut self, color: Color) -> Self {
        self.desc_color = color;
        self
    }

    /// Show a separator line above the hints.
    pub fn with_separator(mut self) -> Self {
        self.show_separator = true;
        self
    }

    /// Set whether to show the separator line.
    pub fn separator(mut self, show: bool) -> Self {
        self.show_separator = show;
        self
    }

    /// Set the separator character.
    pub fn separator_char(mut self, c: char) -> Self {
        self.separator_char = c;
        self
    }

    /// Show left scroll indicator.
    pub fn scroll_left(mut self, show: bool) -> Self {
        self.show_scroll_left = show;
        self
    }

    /// Show right scroll indicator.
    pub fn scroll_right(mut self, show: bool) -> Self {
        self.show_scroll_right = show;
        self
    }

    /// Set custom scroll indicator characters.
    pub fn scroll_chars(mut self, left: impl Into<String>, right: impl Into<String>) -> Self {
        self.scroll_left_char = left.into();
        self.scroll_right_char = right.into();
        self
    }

    /// Render the footer hints as a string.
    pub fn render(&self) -> String {
        Model::view(self)
    }

    /// Calculate the plain text length of hints (for layout).
    fn hints_plain_len(&self) -> usize {
        if self.hints.is_empty() {
            return 0;
        }
        let hint_len: usize = self.hints.iter().map(|(k, d)| measure_text(k) + 1 + d.len()).sum();
        let separator_len = (self.hints.len().saturating_sub(1)) * 2; // "  " between hints
        hint_len + separator_len
    }

    /// Render just the styled hints portion.
    fn render_hints(&self) -> String {
        let reset = RESET;
        let key_color = if matches!(self.key_color, Color::Default) {
            reset.to_string()
        } else {
            self.key_color.to_ansi_fg()
        };
        let desc_color = self.desc_color.to_ansi_fg();

        let mut output = String::new();
        for (i, (key, desc)) in self.hints.iter().enumerate() {
            if i > 0 {
                output.push_str("  ");
            }
            output.push_str(&key_color);
            output.push_str(key);
            output.push_str(&desc_color);
            output.push(' ');
            output.push_str(desc);
        }
        output.push_str(reset);
        output
    }
}

impl Model for FooterHints {
    type Message = FooterHintsMsg;

    fn init(&self) -> Option<Cmd<Self::Message>> {
        None
    }

    fn update(&mut self, _msg: Self::Message) -> Option<Cmd<Self::Message>> {
        None
    }

    fn view(&self) -> String {
        let reset = RESET;
        let dim = self.desc_color.to_ansi_fg();

        let mut output = String::new();

        // Separator line
        if self.show_separator {
            output.push_str(&dim);
            for _ in 0..self.width {
                output.push(self.separator_char);
            }
            output.push_str(reset);
            output.push_str("\r\n");
        }

        // Scroll indicators
        let left_indicator = if self.show_scroll_left { &self.scroll_left_char } else { "  " };
        let right_indicator = if self.show_scroll_right { &self.scroll_right_char } else { "  " };

        let indicators_len = measure_text(left_indicator) + measure_text(right_indicator);
        let hints_len = self.hints_plain_len();
        let padding = self.width.saturating_sub(hints_len + indicators_len);

        // Build the footer line
        output.push_str(&dim);
        output.push_str(left_indicator);
        output.push_str(&" ".repeat(padding));
        output.push_str(&self.render_hints());
        output.push_str(&dim);
        output.push_str(right_indicator);
        output.push_str(reset);

        output
    }

    fn handle_event(&self, _event: Event) -> Option<Self::Message> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_footer_hints_creation() {
        let footer = FooterHints::new();
        assert!(footer.hints.is_empty());
        assert_eq!(footer.width, 80);
    }

    #[test]
    fn test_footer_hints_with_hints() {
        let footer = FooterHints::new().hints(vec![("q", "quit"), ("↑/↓", "select")]);
        assert_eq!(footer.hints.len(), 2);
        assert_eq!(footer.hints[0], ("q".to_string(), "quit".to_string()));
    }

    #[test]
    fn test_footer_hints_add_hint() {
        let footer = FooterHints::new().hint("q", "quit").hint("tab", "next");
        assert_eq!(footer.hints.len(), 2);
    }

    #[test]
    fn test_footer_hints_width() {
        let footer = FooterHints::new().width(100);
        assert_eq!(footer.width, 100);
    }

    #[test]
    fn test_footer_hints_separator() {
        let footer = FooterHints::new().with_separator();
        assert!(footer.show_separator);
    }

    #[test]
    fn test_footer_hints_scroll_indicators() {
        let footer = FooterHints::new().scroll_left(true).scroll_right(true);
        assert!(footer.show_scroll_left);
        assert!(footer.show_scroll_right);
    }

    #[test]
    fn test_render_contains_hints() {
        let footer = FooterHints::new().hints(vec![("q", "quit")]).width(40);
        let rendered = footer.render();
        assert!(rendered.contains("quit"));
    }

    #[test]
    fn test_render_with_separator() {
        let footer = FooterHints::new().hints(vec![("q", "quit")]).width(40).with_separator();
        let rendered = footer.render();
        assert!(rendered.contains("─"));
        assert!(rendered.contains("\r\n"));
    }

    #[test]
    fn test_plain_len_calculation() {
        let footer = FooterHints::new().hints(vec![("q", "quit"), ("x", "exit")]);
        // "q quit" = 6, "x exit" = 6, separator = 2, total = 14
        assert_eq!(footer.hints_plain_len(), 14);
    }

    #[test]
    fn test_empty_hints() {
        let footer = FooterHints::new().width(40);
        let rendered = footer.render();
        // Should still render (padding and indicators)
        assert!(!rendered.is_empty());
    }
}
