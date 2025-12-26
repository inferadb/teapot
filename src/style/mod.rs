//! Styling utilities for terminal output.
//!
//! This module provides a Lip Gloss-inspired styling system with:
//! - Colors (ANSI 16, 256, and true color)
//! - Text attributes (bold, italic, underline, etc.)
//! - Borders and padding
//! - Width and alignment

mod border;
mod color;
mod text;

pub use border::{Border, BorderStyle};
pub use color::Color;
pub use text::Style;

use unicode_width::UnicodeWidthStr;

/// Calculate the display width of a string.
pub fn width(s: &str) -> usize {
    // Strip ANSI escape codes before measuring
    let stripped = strip_ansi(s);
    UnicodeWidthStr::width(stripped.as_str())
}

/// Strip ANSI escape codes from a string.
pub fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_escape = false;

    for c in s.chars() {
        if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else if c == '\x1b' {
            in_escape = true;
        } else {
            result.push(c);
        }
    }

    result
}

/// Pad a string to a given width.
pub fn pad_right(s: &str, target_width: usize) -> String {
    let current = width(s);
    if current >= target_width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(target_width - current))
    }
}

/// Pad a string to a given width, centering the text.
pub fn pad_center(s: &str, target_width: usize) -> String {
    let current = width(s);
    if current >= target_width {
        s.to_string()
    } else {
        let padding = target_width - current;
        let left = padding / 2;
        let right = padding - left;
        format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
    }
}

/// Truncate a string to fit within a width, adding ellipsis if needed.
pub fn truncate(s: &str, max_width: usize) -> String {
    let current = width(s);
    if current <= max_width {
        s.to_string()
    } else if max_width <= 3 {
        ".".repeat(max_width)
    } else {
        let mut result = String::new();
        let mut current_width = 0;
        let target = max_width - 3; // Leave room for "..."

        for c in s.chars() {
            let char_width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
            if current_width + char_width > target {
                break;
            }
            result.push(c);
            current_width += char_width;
        }

        result.push_str("...");
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_width() {
        assert_eq!(width("hello"), 5);
        assert_eq!(width(""), 0);
        assert_eq!(width("日本語"), 6); // 3 chars * 2 width each
    }

    #[test]
    fn test_strip_ansi() {
        assert_eq!(strip_ansi("\x1b[31mred\x1b[0m"), "red");
        assert_eq!(strip_ansi("no escapes"), "no escapes");
        assert_eq!(strip_ansi("\x1b[1;32mbold green\x1b[0m"), "bold green");
    }

    #[test]
    fn test_pad_right() {
        assert_eq!(pad_right("hi", 5), "hi   ");
        assert_eq!(pad_right("hello", 3), "hello");
    }

    #[test]
    fn test_pad_center() {
        assert_eq!(pad_center("hi", 6), "  hi  ");
        assert_eq!(pad_center("a", 4), " a  ");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello world", 8), "hello...");
        assert_eq!(truncate("hi", 10), "hi");
        assert_eq!(truncate("test", 3), "...");
    }
}
