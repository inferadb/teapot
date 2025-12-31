//! Text input component.
//!
//! A single-line text input field with cursor support.
//!
//! # Example
//!
//! ```rust
//! use teapot::components::TextInput;
//!
//! let input = TextInput::new()
//!     .placeholder("Enter your name...")
//!     .prompt("> ");
//! ```

use crate::{
    runtime::{
        Cmd, Model,
        accessible::{Accessible, AccessibleInput},
    },
    style::Color,
    terminal::{Event, KeyCode, KeyModifiers},
};

/// Message type for text input.
#[derive(Debug, Clone)]
pub enum TextInputMsg {
    /// Insert a character at cursor.
    InsertChar(char),
    /// Delete character before cursor.
    DeleteBack,
    /// Delete character at cursor.
    DeleteForward,
    /// Move cursor left.
    CursorLeft,
    /// Move cursor right.
    CursorRight,
    /// Move cursor to start.
    CursorStart,
    /// Move cursor to end.
    CursorEnd,
    /// Delete word before cursor.
    DeleteWord,
    /// Clear all text.
    Clear,
    /// Submit the input.
    Submit,
    /// Focus the input.
    Focus,
    /// Blur the input.
    Blur,
    /// Set the value.
    SetValue(String),
    /// Paste text.
    Paste(String),
}

/// A single-line text input component.
#[derive(Debug, Clone)]
pub struct TextInput {
    value: String,
    cursor: usize,
    placeholder: String,
    prompt: String,
    focused: bool,
    hidden: bool,
    width: Option<usize>,
    cursor_color: Color,
    text_color: Color,
    placeholder_color: Color,
    submitted: bool,
    validation_error: Option<String>,
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInput {
    /// Create a new text input.
    pub fn new() -> Self {
        Self {
            value: String::new(),
            cursor: 0,
            placeholder: String::new(),
            prompt: String::new(),
            focused: true,
            hidden: false,
            width: None,
            cursor_color: Color::Cyan,
            text_color: Color::Default,
            placeholder_color: Color::BrightBlack,
            submitted: false,
            validation_error: None,
        }
    }

    /// Set the placeholder text.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set the prompt (prefix).
    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = prompt.into();
        self
    }

    /// Set whether the input is hidden (for passwords).
    pub fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    /// Set the maximum width.
    pub fn width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the initial value.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self.cursor = self.value.len();
        self
    }

    /// Set the cursor color.
    pub fn cursor_color(mut self, color: Color) -> Self {
        self.cursor_color = color;
        self
    }

    /// Set the text color.
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// Get the current value.
    pub fn get_value(&self) -> &str {
        &self.value
    }

    /// Check if the input was submitted.
    pub fn is_submitted(&self) -> bool {
        self.submitted
    }

    /// Check if the input is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Set focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Set a validation error.
    pub fn set_error(&mut self, error: impl Into<String>) {
        self.validation_error = Some(error.into());
    }

    /// Clear validation error.
    pub fn clear_error(&mut self) {
        self.validation_error = None;
    }

    /// Insert a character at the cursor position.
    fn insert_char(&mut self, c: char) {
        self.value.insert(self.cursor, c);
        self.cursor += c.len_utf8();
        self.validation_error = None;
    }

    /// Delete the character before the cursor.
    fn delete_back(&mut self) {
        if self.cursor > 0 {
            let prev_char =
                self.value[..self.cursor].chars().last().map(|c| c.len_utf8()).unwrap_or(0);
            self.cursor -= prev_char;
            self.value.remove(self.cursor);
            self.validation_error = None;
        }
    }

    /// Delete the character at the cursor.
    fn delete_forward(&mut self) {
        if self.cursor < self.value.len() {
            self.value.remove(self.cursor);
            self.validation_error = None;
        }
    }

    /// Move cursor left.
    fn cursor_left(&mut self) {
        if self.cursor > 0 {
            let prev_char =
                self.value[..self.cursor].chars().last().map(|c| c.len_utf8()).unwrap_or(0);
            self.cursor -= prev_char;
        }
    }

    /// Move cursor right.
    fn cursor_right(&mut self) {
        if self.cursor < self.value.len() {
            let next_char =
                self.value[self.cursor..].chars().next().map(|c| c.len_utf8()).unwrap_or(0);
            self.cursor += next_char;
        }
    }

    /// Move cursor to start.
    fn cursor_start(&mut self) {
        self.cursor = 0;
    }

    /// Move cursor to end.
    fn cursor_end(&mut self) {
        self.cursor = self.value.len();
    }

    /// Delete word before cursor.
    fn delete_word(&mut self) {
        // Find start of current/previous word
        let before = &self.value[..self.cursor];
        let trimmed = before.trim_end();
        let word_start = trimmed.rfind(char::is_whitespace).map(|i| i + 1).unwrap_or(0);

        // Remove characters from word_start to cursor
        self.value = format!("{}{}", &self.value[..word_start], &self.value[self.cursor..]);
        self.cursor = word_start;
        self.validation_error = None;
    }
}

impl Model for TextInput {
    type Message = TextInputMsg;

    fn init(&self) -> Option<Cmd<Self::Message>> {
        None
    }

    fn update(&mut self, msg: Self::Message) -> Option<Cmd<Self::Message>> {
        match msg {
            TextInputMsg::InsertChar(c) => self.insert_char(c),
            TextInputMsg::DeleteBack => self.delete_back(),
            TextInputMsg::DeleteForward => self.delete_forward(),
            TextInputMsg::CursorLeft => self.cursor_left(),
            TextInputMsg::CursorRight => self.cursor_right(),
            TextInputMsg::CursorStart => self.cursor_start(),
            TextInputMsg::CursorEnd => self.cursor_end(),
            TextInputMsg::DeleteWord => self.delete_word(),
            TextInputMsg::Clear => {
                self.value.clear();
                self.cursor = 0;
                self.validation_error = None;
            },
            TextInputMsg::Submit => {
                self.submitted = true;
            },
            TextInputMsg::Focus => {
                self.focused = true;
            },
            TextInputMsg::Blur => {
                self.focused = false;
            },
            TextInputMsg::SetValue(value) => {
                self.value = value;
                self.cursor = self.value.len();
                self.validation_error = None;
            },
            TextInputMsg::Paste(text) => {
                for c in text.chars() {
                    self.insert_char(c);
                }
            },
        }
        None
    }

    fn view(&self) -> String {
        let mut output = String::new();

        // Prompt
        output.push_str(&self.prompt);

        if self.value.is_empty() && !self.focused {
            // Show placeholder when empty and not focused
            output.push_str(&format!(
                "{}{}{}",
                self.placeholder_color.to_ansi_fg(),
                &self.placeholder,
                "\x1b[0m"
            ));
        } else {
            // Show value (or dots if hidden)
            let display_value = if self.hidden {
                "•".repeat(self.value.chars().count())
            } else {
                self.value.clone()
            };

            if self.focused {
                // Show cursor - need character-aware splitting for hidden mode
                let char_pos = if self.hidden {
                    self.value[..self.cursor].chars().count()
                } else {
                    // For non-hidden, cursor is already a byte position
                    // Convert to character position for consistent handling
                    self.value[..self.cursor.min(self.value.len())].chars().count()
                };

                // Split display_value at character boundary
                let mut before = String::new();
                let mut after_chars = display_value.chars().peekable();
                for _ in 0..char_pos {
                    if let Some(c) = after_chars.next() {
                        before.push(c);
                    }
                }
                let cursor_char: String =
                    after_chars.next().map(|c| c.to_string()).unwrap_or_else(|| " ".to_string());
                let after: String = after_chars.collect();

                output.push_str(&format!("{}{}", self.text_color.to_ansi_fg(), before));
                output.push_str(&format!(
                    "\x1b[7m{}{}\x1b[27m",
                    self.cursor_color.to_ansi_fg(),
                    cursor_char
                ));
                output.push_str(&format!("{}{}\x1b[0m", self.text_color.to_ansi_fg(), after));
            } else {
                output.push_str(&format!(
                    "{}{}{}",
                    self.text_color.to_ansi_fg(),
                    display_value,
                    "\x1b[0m"
                ));
            }
        }

        // Show validation error if present
        if let Some(ref error) = self.validation_error {
            output.push_str(&format!("\n{}✗ {}{}", Color::Red.to_ansi_fg(), error, "\x1b[0m"));
        }

        output
    }

    fn handle_event(&self, event: Event) -> Option<Self::Message> {
        if !self.focused {
            return None;
        }

        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char(c) => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        match c {
                            'a' => Some(TextInputMsg::CursorStart),
                            'e' => Some(TextInputMsg::CursorEnd),
                            'w' => Some(TextInputMsg::DeleteWord),
                            'u' => Some(TextInputMsg::Clear),
                            _ => None,
                        }
                    } else {
                        Some(TextInputMsg::InsertChar(c))
                    }
                },
                KeyCode::Backspace => Some(TextInputMsg::DeleteBack),
                KeyCode::Delete => Some(TextInputMsg::DeleteForward),
                KeyCode::Left => Some(TextInputMsg::CursorLeft),
                KeyCode::Right => Some(TextInputMsg::CursorRight),
                KeyCode::Home => Some(TextInputMsg::CursorStart),
                KeyCode::End => Some(TextInputMsg::CursorEnd),
                KeyCode::Enter => Some(TextInputMsg::Submit),
                _ => None,
            },
            Event::Paste(text) => Some(TextInputMsg::Paste(text)),
            _ => None,
        }
    }
}

impl Accessible for TextInput {
    type Message = TextInputMsg;

    fn accessible_prompt(&self) -> String {
        let mut prompt = String::new();

        // Show placeholder as hint if value is empty
        if self.value.is_empty() && !self.placeholder.is_empty() {
            prompt.push_str(&format!("? {} ({})\n", self.prompt.trim(), self.placeholder));
        } else if !self.prompt.is_empty() {
            prompt.push_str(&format!("? {}\n", self.prompt.trim()));
        }

        // Show current value if any
        if !self.value.is_empty() {
            if self.hidden {
                prompt.push_str(&format!("Current: {}\n", "*".repeat(self.value.len())));
            } else {
                prompt.push_str(&format!("Current: {}\n", self.value));
            }
        }

        prompt.push_str("> ");
        prompt
    }

    fn parse_accessible_input(&self, input: &str) -> Option<Self::Message> {
        match AccessibleInput::parse_text(input) {
            AccessibleInput::Text(text) => Some(TextInputMsg::SetValue(text)),
            AccessibleInput::Empty => {
                // Keep current value and submit
                Some(TextInputMsg::Submit)
            },
            _ => None,
        }
    }

    fn is_accessible_complete(&self) -> bool {
        self.submitted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_input_creation() {
        let input = TextInput::new().placeholder("Enter text...");
        assert_eq!(input.get_value(), "");
        assert_eq!(input.placeholder, "Enter text...");
    }

    #[test]
    fn test_insert_char() {
        let mut input = TextInput::new();
        input.insert_char('a');
        input.insert_char('b');
        input.insert_char('c');
        assert_eq!(input.get_value(), "abc");
    }

    #[test]
    fn test_cursor_movement() {
        let mut input = TextInput::new().value("hello");
        assert_eq!(input.cursor, 5);
        input.cursor_left();
        assert_eq!(input.cursor, 4);
        input.cursor_start();
        assert_eq!(input.cursor, 0);
        input.cursor_end();
        assert_eq!(input.cursor, 5);
    }

    #[test]
    fn test_delete() {
        let mut input = TextInput::new().value("hello");
        input.delete_back();
        assert_eq!(input.get_value(), "hell");
        input.cursor_start();
        input.delete_forward();
        assert_eq!(input.get_value(), "ell");
    }

    #[test]
    fn test_accessible_prompt() {
        let input = TextInput::new().prompt("Name").placeholder("Enter name");
        let prompt = input.accessible_prompt();
        assert!(prompt.contains("Name"));
        assert!(prompt.contains("Enter name"));
        assert!(prompt.contains("> "));
    }

    #[test]
    fn test_accessible_parse_input() {
        let input = TextInput::new();
        let msg = input.parse_accessible_input("hello");
        assert!(matches!(msg, Some(TextInputMsg::SetValue(s)) if s == "hello"));

        let msg = input.parse_accessible_input("");
        assert!(matches!(msg, Some(TextInputMsg::Submit)));
    }

    #[test]
    fn test_hidden_mode_paste() {
        // This test ensures that pasting into a hidden input doesn't panic
        // due to multi-byte character handling (• is 3 bytes in UTF-8).
        // The bug occurred because split_at was called with a character count
        // but it expects a byte position - when • (3 bytes) is used for display,
        // this would cause a panic at non-ASCII-aligned positions.
        let mut input = TextInput::new().hidden(true);
        input.set_focused(true);

        // Use a long mock secret to trigger the byte/char position mismatch
        let secret = "sk-test-abc123def456ghi789jkl012mno345pqr678stu901vwx234yz";
        input.update(TextInputMsg::Paste(secret.to_string()));

        // Should not panic and value should be set correctly
        assert_eq!(input.get_value(), secret);

        // View should render without panicking
        let view = input.view();
        assert!(view.contains("•")); // Should show bullets, not the actual text
    }
}
