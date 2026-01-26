//! Simple output utilities for CLI applications.
//!
//! This module provides helper functions for common CLI output patterns:
//! status messages, progress indicators, and terminal detection.
//!
//! # Example
//!
//! ```no_run
//! use teapot::output;
//!
//! output::success("Operation completed");
//! output::warning("Deprecated feature used");
//! output::error("Failed to connect");
//! output::info("Processing 42 items...");
//! ```

use std::io::IsTerminal;

use crate::style::Color;

const RESET: &str = "\x1b[0m";

/// Print a success message with a green checkmark.
///
/// # Example
///
/// ```no_run
/// use teapot::output;
/// output::success("Build completed successfully");
/// // Output: ✓ Build completed successfully
/// ```
pub fn success(message: &str) {
    if is_tty() {
        eprintln!("{}✓{} {}", Color::Green.to_ansi_fg(), RESET, message);
    } else {
        eprintln!("+ {}", message);
    }
}

/// Print a warning message with a yellow warning icon.
///
/// # Example
///
/// ```no_run
/// use teapot::output;
/// output::warning("Configuration file not found, using defaults");
/// // Output: ⚠ Configuration file not found, using defaults
/// ```
pub fn warning(message: &str) {
    if is_tty() {
        eprintln!("{}⚠{} {}", Color::Yellow.to_ansi_fg(), RESET, message);
    } else {
        eprintln!("! {}", message);
    }
}

/// Print an error message with a red X.
///
/// # Example
///
/// ```no_run
/// use teapot::output;
/// output::error("Failed to connect to server");
/// // Output: ✗ Failed to connect to server
/// ```
pub fn error(message: &str) {
    if is_tty() {
        eprintln!("{}✗{} {}", Color::Red.to_ansi_fg(), RESET, message);
    } else {
        eprintln!("x {}", message);
    }
}

/// Print an info message with a dim bullet.
///
/// # Example
///
/// ```no_run
/// use teapot::output;
/// output::info("Checking dependencies...");
/// // Output: ○ Checking dependencies...
/// ```
pub fn info(message: &str) {
    if is_tty() {
        eprintln!("{}○{} {}", Color::BrightBlack.to_ansi_fg(), RESET, message);
    } else {
        eprintln!("- {}", message);
    }
}

/// Print a key-value pair.
///
/// # Example
///
/// ```no_run
/// use teapot::output;
/// output::kv("Version", "1.2.3");
/// // Output: Version: 1.2.3
/// ```
pub fn kv(key: &str, value: &str) {
    if is_tty() {
        eprintln!(
            "{}{}: {}{}{}",
            Color::BrightBlack.to_ansi_fg(),
            key,
            Color::Default.to_ansi_fg(),
            value,
            RESET
        );
    } else {
        eprintln!("{}: {}", key, value);
    }
}

/// Print a header/section title.
///
/// # Example
///
/// ```no_run
/// use teapot::output;
/// output::header("Configuration");
/// // Output: Configuration
/// //         ─────────────
/// ```
pub fn header(title: &str) {
    if is_tty() {
        eprintln!(
            "{}\x1b[1m{}{}\n{}{}{}",
            Color::Cyan.to_ansi_fg(),
            title,
            RESET,
            Color::BrightBlack.to_ansi_fg(),
            "─".repeat(title.chars().count()),
            RESET
        );
    } else {
        eprintln!("{}", title);
        eprintln!("{}", "-".repeat(title.len()));
    }
}

/// Print a phase/step indicator.
///
/// # Example
///
/// ```no_run
/// use teapot::output;
/// output::phase("Installing dependencies");
/// // Output: ━━━ Installing dependencies ━━━
/// ```
pub fn phase(name: &str) {
    if is_tty() {
        eprintln!("{}━━━ {} ━━━{}", Color::BrightBlack.to_ansi_fg(), name, RESET);
    } else {
        eprintln!("--- {} ---", name);
    }
}

/// Check if stdout is connected to a terminal (TTY).
///
/// Returns `false` in CI environments or when output is piped.
pub fn is_tty() -> bool {
    std::io::stdout().is_terminal() && std::io::stderr().is_terminal()
}

/// Check if running in a CI environment.
///
/// Checks for common CI environment variables like `CI`, `GITHUB_ACTIONS`, etc.
pub fn is_ci() -> bool {
    std::env::var("CI").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("GITLAB_CI").is_ok()
        || std::env::var("CIRCLECI").is_ok()
        || std::env::var("TRAVIS").is_ok()
        || std::env::var("JENKINS_URL").is_ok()
        || std::env::var("BUILDKITE").is_ok()
}

/// Check if colors should be used.
///
/// Returns `false` if:
/// - `NO_COLOR` environment variable is set
/// - Output is not a TTY
/// - Running in CI
pub fn use_color() -> bool {
    std::env::var("NO_COLOR").is_err() && is_tty() && !is_ci()
}

/// Get the terminal width, or a default of 80 columns.
pub fn terminal_width() -> usize {
    crossterm::terminal::size().map(|(w, _)| w as usize).unwrap_or(80)
}

/// Get the terminal height, or a default of 24 rows.
pub fn terminal_height() -> usize {
    crossterm::terminal::size().map(|(_, h)| h as usize).unwrap_or(24)
}

/// Strip ANSI escape codes from a string.
///
/// Useful for calculating actual display width or for non-TTY output.
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
    fn test_strip_ansi() {
        // CSI sequences
        assert_eq!(strip_ansi("\x1b[32mgreen\x1b[0m"), "green");
        assert_eq!(strip_ansi("no colors"), "no colors");
        assert_eq!(strip_ansi("\x1b[1m\x1b[31mbold red\x1b[0m"), "bold red");

        // OSC 8 hyperlinks
        assert_eq!(strip_ansi("\x1b]8;;https://example.com\x07link\x1b]8;;\x07"), "link");
    }

    #[test]
    fn test_terminal_width_default() {
        // In test environment, may not have a terminal
        let width = terminal_width();
        assert!(width > 0);
    }
}
