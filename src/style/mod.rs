//! Styling utilities for terminal output.
//!
//! This module provides a Lip Gloss-inspired styling system with:
//! - Colors (ANSI 16, 256, true color, and adaptive colors)
//! - Text attributes (bold, italic, underline, etc.)
//! - Borders and padding/margins with CSS-style shorthand
//! - Width, height, and alignment
//! - Layout utilities (join, place)
//! - Style inheritance and composition
//!
//! # ANSI Constants
//!
//! Common ANSI escape sequences are provided as constants for convenience:
//!
//! ```rust
//! use teapot::style::{RESET, CLEAR_LINE, Color};
//!
//! // Use Color for foreground colors
//! let green = Color::Green.to_ansi_fg();
//! println!("{}Success!{}", green, RESET);
//!
//! // Use CLEAR_LINE for terminal control
//! eprint!("\r{}Processing...", CLEAR_LINE);
//! ```

mod border;
mod color;
mod text;

pub use border::{Border, BorderStyle};
pub use color::{Color, ColorProfile, has_dark_background};
pub use text::{Position, Spacing, Style, bold, colored, dim, underline};

// ============================================================================
// ANSI Escape Sequence Constants
// ============================================================================

/// Reset all attributes (colors, bold, underline, etc.).
pub const RESET: &str = "\x1b[0m";

/// Clear from cursor to end of line.
pub const CLEAR_LINE: &str = "\x1b[K";

/// Move cursor up one line.
pub const CURSOR_UP: &str = "\x1b[1A";

/// Carriage return (move to start of line).
pub const CR: &str = "\r";

/// Bold text attribute.
pub const BOLD: &str = "\x1b[1m";

/// Dim text attribute.
pub const DIM: &str = "\x1b[2m";

/// Italic text attribute.
pub const ITALIC: &str = "\x1b[3m";

/// Underline text attribute.
pub const UNDERLINE: &str = "\x1b[4m";

/// Blink text attribute.
pub const BLINK: &str = "\x1b[5m";

/// Reverse video (swap foreground and background).
pub const REVERSE: &str = "\x1b[7m";

/// Hidden text attribute.
pub const HIDDEN: &str = "\x1b[8m";

/// Strikethrough text attribute.
pub const STRIKETHROUGH: &str = "\x1b[9m";

use unicode_width::UnicodeWidthStr;

/// Calculate the display width of a string.
pub fn width(s: &str) -> usize {
    // Strip ANSI escape codes before measuring
    let stripped = strip_ansi(s);
    UnicodeWidthStr::width(stripped.as_str())
}

/// Strip ANSI escape codes from a string.
///
/// Handles both CSI sequences (`\x1b[...m`) and OSC sequences (`\x1b]...\x07`),
/// including OSC 8 hyperlinks.
pub fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Check what type of escape sequence
            match chars.peek() {
                Some('[') => {
                    // CSI sequence: ESC [ ... m
                    chars.next(); // consume '['
                    while let Some(&next) = chars.peek() {
                        chars.next();
                        if next == 'm' {
                            break;
                        }
                    }
                },
                Some(']') => {
                    // OSC sequence: ESC ] ... (BEL or ESC \)
                    chars.next(); // consume ']'
                    while let Some(next) = chars.next() {
                        if next == '\x07' {
                            // BEL terminates OSC
                            break;
                        } else if next == '\x1b' && chars.peek() == Some(&'\\') {
                            // ESC \ (ST - String Terminator)
                            chars.next();
                            break;
                        }
                    }
                },
                _ => {
                    // Unknown escape, skip just the ESC
                },
            }
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

// ========== Layout Utilities ==========

/// Get the height (number of lines) of a string.
pub fn height(s: &str) -> usize {
    if s.is_empty() { 0 } else { s.lines().count() }
}

/// Get the size (width, height) of a string.
pub fn size(s: &str) -> (usize, usize) {
    let lines: Vec<&str> = s.lines().collect();
    let h = lines.len();
    let w = lines.iter().map(|l| width(l)).max().unwrap_or(0);
    (w, h)
}

/// Join strings horizontally with no separator.
///
/// All strings are aligned to the top and padded to match the tallest string.
pub fn join_horizontal(strs: &[&str]) -> String {
    join_horizontal_with(Position::Top, strs)
}

/// Join strings horizontally with vertical position alignment.
///
/// # Example
/// ```
/// use teapot::style::{join_horizontal_with, Position};
///
/// let left = "A\nB\nC";
/// let right = "X";
/// let result = join_horizontal_with(Position::Center, &[left, right]);
/// ```
pub fn join_horizontal_with(pos: Position, strs: &[&str]) -> String {
    if strs.is_empty() {
        return String::new();
    }

    if strs.len() == 1 {
        return strs[0].to_string();
    }

    // Get all blocks as line vectors with their widths
    let blocks: Vec<(Vec<&str>, usize)> = strs
        .iter()
        .map(|s| {
            let lines: Vec<&str> = s.lines().collect();
            let w = lines.iter().map(|l| width(l)).max().unwrap_or(0);
            (lines, w)
        })
        .collect();

    // Find the maximum height
    let max_height = blocks.iter().map(|(lines, _)| lines.len()).max().unwrap_or(0);

    if max_height == 0 {
        return String::new();
    }

    // Build result line by line
    let mut result = Vec::with_capacity(max_height);

    for row in 0..max_height {
        let mut line = String::new();

        for (lines, block_width) in &blocks {
            let block_height = lines.len();

            // Calculate which line of this block to use based on position
            let content = match pos {
                Position::Top => {
                    if row < block_height {
                        Some(lines[row])
                    } else {
                        None
                    }
                },
                Position::Center => {
                    let offset = (max_height - block_height) / 2;
                    if row >= offset && row < offset + block_height {
                        Some(lines[row - offset])
                    } else {
                        None
                    }
                },
                Position::Bottom => {
                    let offset = max_height - block_height;
                    if row >= offset { Some(lines[row - offset]) } else { None }
                },
            };

            match content {
                Some(s) => {
                    line.push_str(s);
                    let s_width = width(s);
                    if s_width < *block_width {
                        line.push_str(&" ".repeat(block_width - s_width));
                    }
                },
                None => {
                    line.push_str(&" ".repeat(*block_width));
                },
            }
        }

        result.push(line);
    }

    result.join("\n")
}

/// Join strings vertically with no separator.
pub fn join_vertical(strs: &[&str]) -> String {
    join_vertical_with(Position::Top, strs)
}

/// Join strings vertically with horizontal position alignment.
///
/// # Example
/// ```
/// use teapot::style::{join_vertical_with, Position};
///
/// let top = "Short";
/// let bottom = "A longer line";
/// let result = join_vertical_with(Position::Center, &[top, bottom]);
/// ```
pub fn join_vertical_with(pos: Position, strs: &[&str]) -> String {
    if strs.is_empty() {
        return String::new();
    }

    if strs.len() == 1 {
        return strs[0].to_string();
    }

    // Find the maximum width across all blocks
    let max_width = strs.iter().flat_map(|s| s.lines()).map(width).max().unwrap_or(0);

    // Build result with aligned lines
    let mut result = Vec::new();

    for s in strs {
        for line in s.lines() {
            let line_width = width(line);
            if line_width >= max_width {
                result.push(line.to_string());
            } else {
                let padding = max_width - line_width;
                match pos {
                    Position::Top => {
                        // Left align
                        result.push(format!("{}{}", line, " ".repeat(padding)));
                    },
                    Position::Center => {
                        let left = padding / 2;
                        let right = padding - left;
                        result.push(format!("{}{}{}", " ".repeat(left), line, " ".repeat(right)));
                    },
                    Position::Bottom => {
                        // Right align
                        result.push(format!("{}{}", " ".repeat(padding), line));
                    },
                }
            }
        }
    }

    result.join("\n")
}

/// Place a string within a given width and height, with alignment.
///
/// # Example
/// ```
/// use teapot::style::{place, Position};
///
/// let content = "Hello";
/// let placed = place(20, 5, Position::Center, Position::Center, content);
/// ```
pub fn place(
    target_width: usize,
    target_height: usize,
    h_pos: Position,
    v_pos: Position,
    content: &str,
) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let content_height = lines.len();
    let content_width = lines.iter().map(|l| width(l)).max().unwrap_or(0);

    // Handle empty content
    if content_height == 0 {
        return (0..target_height).map(|_| " ".repeat(target_width)).collect::<Vec<_>>().join("\n");
    }

    // Calculate vertical padding
    let (top_pad, _bottom_pad) = if content_height >= target_height {
        (0, 0)
    } else {
        let v_padding = target_height - content_height;
        match v_pos {
            Position::Top => (0, v_padding),
            Position::Center => {
                let top = v_padding / 2;
                (top, v_padding - top)
            },
            Position::Bottom => (v_padding, 0),
        }
    };

    // Calculate horizontal padding
    let (left_pad, right_pad) = if content_width >= target_width {
        (0, 0)
    } else {
        let h_padding = target_width - content_width;
        match h_pos {
            Position::Top => (0, h_padding), // Left align
            Position::Center => {
                let left = h_padding / 2;
                (left, h_padding - left)
            },
            Position::Bottom => (h_padding, 0), // Right align
        }
    };

    let empty_line = " ".repeat(target_width);
    let mut result = Vec::with_capacity(target_height);

    // Top padding
    for _ in 0..top_pad {
        result.push(empty_line.clone());
    }

    // Content lines with horizontal padding
    for line in &lines {
        if result.len() >= target_height {
            break;
        }
        let line_width = width(line);
        let line_right_pad = content_width - line_width + right_pad;
        result.push(format!("{}{}{}", " ".repeat(left_pad), line, " ".repeat(line_right_pad)));
    }

    // Bottom padding
    while result.len() < target_height {
        result.push(empty_line.clone());
    }

    // Truncate if needed
    if result.len() > target_height {
        result.truncate(target_height);
    }

    result.join("\n")
}

/// Place a string horizontally within a given width.
pub fn place_horizontal(target_width: usize, pos: Position, content: &str) -> String {
    let (_, content_height) = size(content);
    let target_height = if content_height == 0 { 1 } else { content_height };
    place(target_width, target_height, pos, Position::Top, content)
}

/// Place a string vertically within a given height.
pub fn place_vertical(target_height: usize, pos: Position, content: &str) -> String {
    let (content_width, _) = size(content);
    let target_width = if content_width == 0 { 1 } else { content_width };
    place(target_width, target_height, Position::Top, pos, content)
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
        // CSI sequences
        assert_eq!(strip_ansi("\x1b[31mred\x1b[0m"), "red");
        assert_eq!(strip_ansi("no escapes"), "no escapes");
        assert_eq!(strip_ansi("\x1b[1;32mbold green\x1b[0m"), "bold green");

        // OSC 8 hyperlinks (BEL terminated)
        assert_eq!(strip_ansi("\x1b]8;;https://example.com\x07link text\x1b]8;;\x07"), "link text");

        // Mixed CSI and OSC
        assert_eq!(
            strip_ansi("\x1b[36m\x1b]8;;https://example.com\x07click here\x1b]8;;\x07\x1b[0m"),
            "click here"
        );
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

    #[test]
    fn test_height() {
        assert_eq!(height(""), 0);
        assert_eq!(height("one"), 1);
        assert_eq!(height("one\ntwo\nthree"), 3);
    }

    #[test]
    fn test_size() {
        assert_eq!(size("hello"), (5, 1));
        assert_eq!(size("hi\nworld"), (5, 2));
        assert_eq!(size(""), (0, 0));
    }

    #[test]
    fn test_join_horizontal() {
        let left = "A\nB";
        let right = "X\nY";
        let result = join_horizontal(&[left, right]);
        assert_eq!(result, "AX\nBY");
    }

    #[test]
    fn test_join_horizontal_uneven() {
        let left = "A\nB\nC";
        let right = "X";
        let result = join_horizontal_with(Position::Top, &[left, right]);
        assert!(result.contains("AX"));
        assert!(result.contains("B "));

        let result_center = join_horizontal_with(Position::Center, &[left, right]);
        assert!(result_center.contains("BX")); // X should be on middle row
    }

    #[test]
    fn test_join_vertical() {
        let top = "Hi";
        let bottom = "World";
        let result = join_vertical(&[top, bottom]);
        assert_eq!(result, "Hi   \nWorld");
    }

    #[test]
    fn test_join_vertical_centered() {
        let top = "Hi";
        let bottom = "World";
        let result = join_vertical_with(Position::Center, &[top, bottom]);
        // "Hi" should be centered over "World" (5 chars)
        assert!(result.starts_with(" Hi"));
    }

    #[test]
    fn test_place() {
        let content = "X";
        let placed = place(5, 3, Position::Center, Position::Center, content);
        let lines: Vec<&str> = placed.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(width(placed.lines().next().unwrap()), 5);
        // X should be on the middle line
        assert!(lines[1].contains("X"));
    }

    #[test]
    fn test_place_horizontal() {
        let result = place_horizontal(10, Position::Center, "Hi");
        assert_eq!(width(&result), 10);
        assert!(result.starts_with("    ")); // Centered
    }

    #[test]
    fn test_place_vertical() {
        let result = place_vertical(5, Position::Center, "Hi");
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 5);
        // "Hi" should be on line 2 (0-indexed, so middle of 5)
        assert_eq!(lines[2], "Hi");
    }
}
