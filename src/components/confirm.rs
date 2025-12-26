//! Confirmation component.
//!
//! A yes/no confirmation prompt.
//!
//! # Example
//!
//! ```rust
//! use ferment::components::Confirm;
//!
//! let confirm = Confirm::new("Are you sure?")
//!     .default(false);
//! ```

use crate::runtime::{Cmd, Model};
use crate::style::Color;
use crate::terminal::{Event, KeyCode};

/// Message type for confirm.
#[derive(Debug, Clone)]
pub enum ConfirmMsg {
    /// Toggle to yes.
    Yes,
    /// Toggle to no.
    No,
    /// Toggle current selection.
    Toggle,
    /// Submit selection.
    Submit,
    /// Cancel.
    Cancel,
    /// Focus the confirm.
    Focus,
    /// Blur the confirm.
    Blur,
}

/// A yes/no confirmation component.
#[derive(Debug, Clone)]
pub struct Confirm {
    title: String,
    value: bool,
    default: bool,
    focused: bool,
    submitted: bool,
    cancelled: bool,
    yes_label: String,
    no_label: String,
    selected_color: Color,
}

impl Default for Confirm {
    fn default() -> Self {
        Self {
            title: String::new(),
            value: false,
            default: false,
            focused: true,
            submitted: false,
            cancelled: false,
            yes_label: "Yes".to_string(),
            no_label: "No".to_string(),
            selected_color: Color::Cyan,
        }
    }
}

impl Confirm {
    /// Create a new confirm with a title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }

    /// Set the default value.
    pub fn default(mut self, value: bool) -> Self {
        self.default = value;
        self.value = value;
        self
    }

    /// Set the yes label.
    pub fn yes_label(mut self, label: impl Into<String>) -> Self {
        self.yes_label = label.into();
        self
    }

    /// Set the no label.
    pub fn no_label(mut self, label: impl Into<String>) -> Self {
        self.no_label = label.into();
        self
    }

    /// Set the selected color.
    pub fn selected_color(mut self, color: Color) -> Self {
        self.selected_color = color;
        self
    }

    /// Get the current value.
    pub fn value(&self) -> bool {
        self.value
    }

    /// Get the confirmed value if submitted.
    pub fn confirmed(&self) -> Option<bool> {
        if self.submitted {
            Some(self.value)
        } else {
            None
        }
    }

    /// Check if submitted.
    pub fn is_submitted(&self) -> bool {
        self.submitted
    }

    /// Check if cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    /// Set focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
}

impl Model for Confirm {
    type Message = ConfirmMsg;

    fn init(&self) -> Option<Cmd<Self::Message>> {
        None
    }

    fn update(&mut self, msg: Self::Message) -> Option<Cmd<Self::Message>> {
        match msg {
            ConfirmMsg::Yes => self.value = true,
            ConfirmMsg::No => self.value = false,
            ConfirmMsg::Toggle => self.value = !self.value,
            ConfirmMsg::Submit => self.submitted = true,
            ConfirmMsg::Cancel => self.cancelled = true,
            ConfirmMsg::Focus => self.focused = true,
            ConfirmMsg::Blur => self.focused = false,
        }
        None
    }

    fn view(&self) -> String {
        let yes_style = if self.value {
            format!(
                "{}\x1b[1m{}\x1b[0m",
                self.selected_color.to_ansi_fg(),
                self.yes_label
            )
        } else {
            format!(
                "{}{}{}",
                Color::BrightBlack.to_ansi_fg(),
                self.yes_label,
                "\x1b[0m"
            )
        };

        let no_style = if !self.value {
            format!(
                "{}\x1b[1m{}\x1b[0m",
                self.selected_color.to_ansi_fg(),
                self.no_label
            )
        } else {
            format!(
                "{}{}{}",
                Color::BrightBlack.to_ansi_fg(),
                self.no_label,
                "\x1b[0m"
            )
        };

        let hint = format!("{}(y/n){}", Color::BrightBlack.to_ansi_fg(), "\x1b[0m");

        format!("? {} {} / {} {}", self.title, yes_style, no_style, hint)
    }

    fn handle_event(&self, event: Event) -> Option<Self::Message> {
        if !self.focused {
            return None;
        }

        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => Some(ConfirmMsg::Yes),
                KeyCode::Char('n') | KeyCode::Char('N') => Some(ConfirmMsg::No),
                KeyCode::Left | KeyCode::Right | KeyCode::Tab => Some(ConfirmMsg::Toggle),
                KeyCode::Enter => Some(ConfirmMsg::Submit),
                KeyCode::Esc => Some(ConfirmMsg::Cancel),
                _ => None,
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirm_creation() {
        let confirm = Confirm::new("Are you sure?").default(true);
        assert!(confirm.value());
    }

    #[test]
    fn test_toggle() {
        let mut confirm = Confirm::new("Are you sure?");
        assert!(!confirm.value());
        confirm.value = true;
        assert!(confirm.value());
    }

    #[test]
    fn test_submit() {
        let mut confirm = Confirm::new("Are you sure?").default(true);
        assert!(confirm.confirmed().is_none());
        confirm.submitted = true;
        assert_eq!(confirm.confirmed(), Some(true));
    }
}
