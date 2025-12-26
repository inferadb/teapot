//! Terminal abstraction layer.
//!
//! This module provides a unified interface over crossterm for:
//! - Event handling (keyboard, mouse, resize)
//! - Terminal capabilities detection
//! - Raw mode management

mod backend;
mod input;
mod output;

pub use backend::Backend;
pub use input::{Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
pub use output::TerminalOutput;

/// Get the current terminal size.
pub fn size() -> std::io::Result<(u16, u16)> {
    crossterm::terminal::size()
}

/// Check if stdout is a terminal.
pub fn is_tty() -> bool {
    use std::io::IsTerminal;
    std::io::stdout().is_terminal()
}

/// Check if we're running in a CI environment.
pub fn is_ci() -> bool {
    std::env::var("CI").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("GITLAB_CI").is_ok()
        || std::env::var("JENKINS_HOME").is_ok()
        || std::env::var("BUILDKITE").is_ok()
}

/// Check if NO_COLOR is set.
pub fn no_color() -> bool {
    std::env::var("NO_COLOR").is_ok()
}

/// Check if the terminal supports colors.
pub fn supports_color() -> bool {
    if no_color() {
        return false;
    }

    // Check TERM variable
    if let Ok(term) = std::env::var("TERM") {
        if term == "dumb" {
            return false;
        }
    }

    // Check COLORTERM for true color support
    if std::env::var("COLORTERM").is_ok() {
        return true;
    }

    is_tty()
}
