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

use crate::{
    runtime::{
        Cmd, Model,
        accessible::{Accessible, AccessibleInput},
    },
    style::Color,
    terminal::{Event, KeyCode},
};

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
        Self { title: title.into(), ..Default::default() }
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
        if self.submitted { Some(self.value) } else { None }
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
            format!("{}\x1b[1m{}\x1b[0m", self.selected_color.to_ansi_fg(), self.yes_label)
        } else {
            format!("{}{}{}", Color::BrightBlack.to_ansi_fg(), self.yes_label, "\x1b[0m")
        };

        let no_style = if !self.value {
            format!("{}\x1b[1m{}\x1b[0m", self.selected_color.to_ansi_fg(), self.no_label)
        } else {
            format!("{}{}{}", Color::BrightBlack.to_ansi_fg(), self.no_label, "\x1b[0m")
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

impl Accessible for Confirm {
    type Message = ConfirmMsg;

    fn accessible_prompt(&self) -> String {
        let default_hint = if self.default { "Y/n" } else { "y/N" };
        format!("? {} ({}) ", self.title, default_hint)
    }

    fn parse_accessible_input(&self, input: &str) -> Option<Self::Message> {
        match AccessibleInput::parse_confirm(input, Some(self.default)) {
            AccessibleInput::Yes => Some(ConfirmMsg::Submit),
            AccessibleInput::No => Some(ConfirmMsg::Submit),
            AccessibleInput::Cancel => Some(ConfirmMsg::Cancel),
            AccessibleInput::Empty => Some(ConfirmMsg::Submit), // Use default
            _ => None,
        }
    }

    fn is_accessible_complete(&self) -> bool {
        self.submitted || self.cancelled
    }
}

impl Confirm {
    /// Parse accessible input and apply it.
    ///
    /// Returns true if the confirmation is complete.
    pub fn apply_accessible_input(&mut self, input: &str) -> bool {
        match AccessibleInput::parse_confirm(input, Some(self.default)) {
            AccessibleInput::Yes => {
                self.value = true;
                self.submitted = true;
                true
            },
            AccessibleInput::No => {
                self.value = false;
                self.submitted = true;
                true
            },
            AccessibleInput::Cancel => {
                self.cancelled = true;
                true
            },
            AccessibleInput::Empty => {
                // Use default value
                self.value = self.default;
                self.submitted = true;
                true
            },
            _ => false,
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

    #[test]
    fn test_accessible_prompt() {
        let confirm = Confirm::new("Continue?").default(true);
        let prompt = confirm.accessible_prompt();
        assert!(prompt.contains("Continue?"));
        assert!(prompt.contains("Y/n")); // Default yes, so Y is capitalized

        let confirm = Confirm::new("Continue?").default(false);
        let prompt = confirm.accessible_prompt();
        assert!(prompt.contains("y/N")); // Default no, so N is capitalized
    }

    #[test]
    fn test_accessible_apply_input() {
        let mut confirm = Confirm::new("Continue?").default(false);
        assert!(confirm.apply_accessible_input("y"));
        assert!(confirm.is_submitted());
        assert!(confirm.value());

        let mut confirm = Confirm::new("Continue?").default(true);
        assert!(confirm.apply_accessible_input(""));
        assert!(confirm.is_submitted());
        assert!(confirm.value()); // Default true

        let mut confirm = Confirm::new("Continue?").default(true);
        assert!(confirm.apply_accessible_input("n"));
        assert!(confirm.is_submitted());
        assert!(!confirm.value());
    }
}
