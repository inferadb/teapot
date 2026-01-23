//! Accessible mode support for screen readers and non-visual environments.
//!
//! When accessible mode is enabled, components render as plain text prompts
//! with numbered options instead of visual TUI elements. Input is read from
//! stdin line-by-line without requiring raw terminal mode.
//!
//! # Environment Variables
//!
//! - `ACCESSIBLE=1` - Enable accessible mode
//! - `NO_COLOR=1` - Disable colors (always disabled in accessible mode)
//! - `REDUCE_MOTION=1` - Disable animations (always disabled in accessible mode)
//!
//! # Example
//!
//! ```text
//! ? What is your name?
//! > Alice
//!
//! ? Choose a color:
//! 1) Red
//! 2) Green
//! 3) Blue
//! > 2
//!
//! ? Are you sure? (yes/no)
//! > yes
//! ```

use std::io::{self, BufRead, Write};

/// Result of parsing accessible input.
#[derive(Debug, Clone)]
pub enum AccessibleInput {
    /// Text input.
    Text(String),
    /// Numeric selection (1-based).
    Selection(usize),
    /// Multiple selections (1-based).
    MultiSelection(Vec<usize>),
    /// Yes/true response.
    Yes,
    /// No/false response.
    No,
    /// Cancel/quit.
    Cancel,
    /// Empty input (use default).
    Empty,
}

impl AccessibleInput {
    /// Parse input for a text prompt.
    pub fn parse_text(input: &str) -> Self {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            AccessibleInput::Empty
        } else {
            AccessibleInput::Text(trimmed.to_string())
        }
    }

    /// Parse input for a selection prompt (1-based number).
    pub fn parse_selection(input: &str, max: usize) -> Self {
        let trimmed = input.trim().to_lowercase();

        if trimmed.is_empty() {
            return AccessibleInput::Empty;
        }

        if trimmed == "q" || trimmed == "quit" || trimmed == "cancel" {
            return AccessibleInput::Cancel;
        }

        // Try parsing as number
        if let Ok(n) = trimmed.parse::<usize>()
            && n >= 1
            && n <= max
        {
            return AccessibleInput::Selection(n);
        }

        // Invalid input - return empty to re-prompt
        AccessibleInput::Empty
    }

    /// Parse input for a multi-selection prompt (comma-separated 1-based numbers).
    pub fn parse_multi_selection(input: &str, max: usize) -> Self {
        let trimmed = input.trim().to_lowercase();

        if trimmed.is_empty() {
            return AccessibleInput::Empty;
        }

        if trimmed == "q" || trimmed == "quit" || trimmed == "cancel" {
            return AccessibleInput::Cancel;
        }

        if trimmed == "done" || trimmed == "d" {
            return AccessibleInput::MultiSelection(vec![]);
        }

        // Parse comma-separated numbers
        let mut selections = Vec::new();
        for part in trimmed.split([',', ' ']) {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            if let Ok(n) = part.parse::<usize>()
                && n >= 1
                && n <= max
                && !selections.contains(&n)
            {
                selections.push(n);
            }
        }

        if selections.is_empty() {
            AccessibleInput::Empty
        } else {
            AccessibleInput::MultiSelection(selections)
        }
    }

    /// Parse input for a yes/no prompt.
    pub fn parse_confirm(input: &str, default: Option<bool>) -> Self {
        let trimmed = input.trim().to_lowercase();

        if trimmed.is_empty() {
            return match default {
                Some(true) => AccessibleInput::Yes,
                Some(false) => AccessibleInput::No,
                None => AccessibleInput::Empty,
            };
        }

        match trimmed.as_str() {
            "y" | "yes" | "true" | "1" => AccessibleInput::Yes,
            "n" | "no" | "false" | "0" => AccessibleInput::No,
            "q" | "quit" | "cancel" => AccessibleInput::Cancel,
            _ => AccessibleInput::Empty,
        }
    }
}

/// Trait for components that support accessible mode.
///
/// Components implementing this trait can render themselves as plain text
/// prompts suitable for screen readers and text-only environments.
pub trait Accessible {
    /// The message type for accessible input.
    type Message;

    /// Render the accessible prompt.
    ///
    /// Returns a plain text prompt without ANSI codes or visual formatting.
    /// The prompt should clearly describe what input is expected.
    fn accessible_prompt(&self) -> String;

    /// Parse accessible input and return a message.
    ///
    /// Takes the raw input line from the user and converts it to a message
    /// that can be passed to `update()`.
    fn parse_accessible_input(&self, input: &str) -> Option<Self::Message>;

    /// Check if the component is complete (submitted or cancelled).
    fn is_accessible_complete(&self) -> bool;
}

/// Read a line of input from stdin.
pub fn read_line() -> io::Result<String> {
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    Ok(line)
}

/// Print a prompt and flush stdout.
pub fn print_prompt(prompt: &str) -> io::Result<()> {
    let mut stdout = io::stdout();
    write!(stdout, "{}", prompt)?;
    stdout.flush()
}

/// Print a line to stdout.
pub fn println_accessible(line: &str) -> io::Result<()> {
    println!("{}", line);
    Ok(())
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
            match chars.peek() {
                Some('[') => {
                    // CSI sequence: ESC [ ... (letter)
                    chars.next();
                    while let Some(&next) = chars.peek() {
                        chars.next();
                        if next.is_ascii_alphabetic() {
                            break;
                        }
                    }
                },
                Some(']') => {
                    // OSC sequence: ESC ] ... (BEL or ESC \)
                    chars.next();
                    while let Some(next) = chars.next() {
                        if next == '\x07' {
                            break;
                        } else if next == '\x1b' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                },
                _ => {},
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text() {
        assert!(matches!(
            AccessibleInput::parse_text("hello"),
            AccessibleInput::Text(s) if s == "hello"
        ));
        assert!(matches!(
            AccessibleInput::parse_text("  hello  "),
            AccessibleInput::Text(s) if s == "hello"
        ));
        assert!(matches!(AccessibleInput::parse_text(""), AccessibleInput::Empty));
    }

    #[test]
    fn test_parse_selection() {
        assert!(matches!(AccessibleInput::parse_selection("1", 3), AccessibleInput::Selection(1)));
        assert!(matches!(AccessibleInput::parse_selection("3", 3), AccessibleInput::Selection(3)));
        assert!(matches!(AccessibleInput::parse_selection("4", 3), AccessibleInput::Empty)); // Out of range
        assert!(matches!(AccessibleInput::parse_selection("0", 3), AccessibleInput::Empty)); // Out of range
        assert!(matches!(AccessibleInput::parse_selection("q", 3), AccessibleInput::Cancel));
    }

    #[test]
    fn test_parse_multi_selection() {
        match AccessibleInput::parse_multi_selection("1, 3", 5) {
            AccessibleInput::MultiSelection(v) => assert_eq!(v, vec![1, 3]),
            _ => panic!("Expected MultiSelection"),
        }
        match AccessibleInput::parse_multi_selection("1 2 3", 5) {
            AccessibleInput::MultiSelection(v) => assert_eq!(v, vec![1, 2, 3]),
            _ => panic!("Expected MultiSelection"),
        }
        assert!(matches!(
            AccessibleInput::parse_multi_selection("done", 5),
            AccessibleInput::MultiSelection(v) if v.is_empty()
        ));
    }

    #[test]
    fn test_parse_confirm() {
        assert!(matches!(AccessibleInput::parse_confirm("yes", None), AccessibleInput::Yes));
        assert!(matches!(AccessibleInput::parse_confirm("y", None), AccessibleInput::Yes));
        assert!(matches!(AccessibleInput::parse_confirm("no", None), AccessibleInput::No));
        assert!(matches!(AccessibleInput::parse_confirm("n", None), AccessibleInput::No));
        assert!(matches!(AccessibleInput::parse_confirm("", Some(true)), AccessibleInput::Yes));
        assert!(matches!(AccessibleInput::parse_confirm("", Some(false)), AccessibleInput::No));
        assert!(matches!(AccessibleInput::parse_confirm("", None), AccessibleInput::Empty));
    }

    #[test]
    fn test_strip_ansi() {
        // CSI sequences
        assert_eq!(strip_ansi("\x1b[31mRed\x1b[0m"), "Red");
        assert_eq!(strip_ansi("\x1b[1;32mBold Green\x1b[0m"), "Bold Green");
        assert_eq!(strip_ansi("No codes"), "No codes");
        assert_eq!(strip_ansi("\x1b[38;2;255;0;0mTruecolor\x1b[0m"), "Truecolor");

        // OSC 8 hyperlinks
        assert_eq!(strip_ansi("\x1b]8;;https://example.com\x07link\x1b]8;;\x07"), "link");
    }
}
