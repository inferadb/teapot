//! Selection component.
//!
//! A single-selection list with keyboard navigation.
//!
//! # Example
//!
//! ```rust
//! use teapot::components::Select;
//!
//! let select = Select::new("Choose a color")
//!     .options(vec!["Red", "Green", "Blue"]);
//! ```

use crate::{
    runtime::{
        Cmd, Model,
        accessible::{Accessible, AccessibleInput},
    },
    style::Color,
    terminal::{Event, KeyCode},
};

/// Message type for select.
#[derive(Debug, Clone)]
pub enum SelectMsg {
    /// Move selection up.
    Up,
    /// Move selection down.
    Down,
    /// Move to first item.
    First,
    /// Move to last item.
    Last,
    /// Submit selection.
    Submit,
    /// Cancel selection.
    Cancel,
    /// Focus the select.
    Focus,
    /// Blur the select.
    Blur,
}

/// A single-selection list component.
#[derive(Debug, Clone)]
pub struct Select<T> {
    title: String,
    options: Vec<(T, String)>,
    cursor: usize,
    focused: bool,
    submitted: bool,
    cancelled: bool,
    cursor_char: &'static str,
    selected_color: Color,
    unselected_color: Color,
}

impl<T: Clone> Default for Select<T> {
    fn default() -> Self {
        Self {
            title: String::new(),
            options: Vec::new(),
            cursor: 0,
            focused: true,
            submitted: false,
            cancelled: false,
            cursor_char: "❯",
            selected_color: Color::Cyan,
            unselected_color: Color::Default,
        }
    }
}

impl<T: Clone> Select<T> {
    /// Create a new select with a title.
    pub fn new(title: impl Into<String>) -> Self {
        Self { title: title.into(), ..Default::default() }
    }

    /// Set the options with display strings.
    pub fn options_with_labels(mut self, options: Vec<(T, String)>) -> Self {
        self.options = options;
        self
    }

    /// Set the cursor character.
    pub fn cursor_char(mut self, c: &'static str) -> Self {
        self.cursor_char = c;
        self
    }

    /// Set the selected item color.
    pub fn selected_color(mut self, color: Color) -> Self {
        self.selected_color = color;
        self
    }

    /// Get the current cursor position.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Get the selected value if submitted.
    pub fn selected(&self) -> Option<&T> {
        if self.submitted && !self.options.is_empty() {
            self.options.get(self.cursor).map(|(v, _)| v)
        } else {
            None
        }
    }

    /// Get the selected value regardless of submit state.
    pub fn current(&self) -> Option<&T> {
        self.options.get(self.cursor).map(|(v, _)| v)
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

    fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    fn move_down(&mut self) {
        if self.cursor < self.options.len().saturating_sub(1) {
            self.cursor += 1;
        }
    }
}

impl Select<String> {
    /// Set options from strings.
    pub fn options<I, S>(mut self, options: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.options = options
            .into_iter()
            .map(|s| {
                let s = s.into();
                (s.clone(), s)
            })
            .collect();
        self
    }
}

impl<T: Clone + Send + 'static> Model for Select<T> {
    type Message = SelectMsg;

    fn init(&self) -> Option<Cmd<Self::Message>> {
        None
    }

    fn update(&mut self, msg: Self::Message) -> Option<Cmd<Self::Message>> {
        match msg {
            SelectMsg::Up => self.move_up(),
            SelectMsg::Down => self.move_down(),
            SelectMsg::First => self.cursor = 0,
            SelectMsg::Last => self.cursor = self.options.len().saturating_sub(1),
            SelectMsg::Submit => self.submitted = true,
            SelectMsg::Cancel => self.cancelled = true,
            SelectMsg::Focus => self.focused = true,
            SelectMsg::Blur => self.focused = false,
        }
        None
    }

    fn view(&self) -> String {
        let mut output = String::new();

        // Title
        if !self.title.is_empty() {
            output.push_str(&format!("? {}\n", self.title));
        }

        // Options
        for (i, (_, label)) in self.options.iter().enumerate() {
            let is_selected = i == self.cursor;
            let cursor = if is_selected { self.cursor_char } else { " " };

            if is_selected {
                output.push_str(&format!(
                    "{}{} {}{}",
                    self.selected_color.to_ansi_fg(),
                    cursor,
                    label,
                    "\x1b[0m"
                ));
            } else {
                output.push_str(&format!(
                    "{}{} {}{}",
                    self.unselected_color.to_ansi_fg(),
                    cursor,
                    label,
                    "\x1b[0m"
                ));
            }

            if i < self.options.len() - 1 {
                output.push('\n');
            }
        }

        output
    }

    fn handle_event(&self, event: Event) -> Option<Self::Message> {
        if !self.focused {
            return None;
        }

        match event {
            Event::Key(key) => match key.code {
                KeyCode::Up | KeyCode::Char('k') => Some(SelectMsg::Up),
                KeyCode::Down | KeyCode::Char('j') => Some(SelectMsg::Down),
                KeyCode::Home => Some(SelectMsg::First),
                KeyCode::End => Some(SelectMsg::Last),
                KeyCode::Enter | KeyCode::Char(' ') => Some(SelectMsg::Submit),
                KeyCode::Esc | KeyCode::Char('q') => Some(SelectMsg::Cancel),
                _ => None,
            },
            _ => None,
        }
    }
}

impl<T: Clone + Send + 'static> Accessible for Select<T> {
    type Message = SelectMsg;

    fn accessible_prompt(&self) -> String {
        let mut prompt = String::new();

        // Title
        if !self.title.is_empty() {
            prompt.push_str(&format!("? {}\n", self.title));
        }

        // Numbered options
        for (i, (_, label)) in self.options.iter().enumerate() {
            let marker = if i == self.cursor { "*" } else { " " };
            prompt.push_str(&format!("{} {}) {}\n", marker, i + 1, label));
        }

        prompt.push_str("Enter number (or q to cancel): ");
        prompt
    }

    fn parse_accessible_input(&self, input: &str) -> Option<Self::Message> {
        match AccessibleInput::parse_selection(input, self.options.len()) {
            AccessibleInput::Selection(_) => {
                // Selection is handled by apply_accessible_input which updates cursor
                Some(SelectMsg::Submit)
            },
            AccessibleInput::Cancel => Some(SelectMsg::Cancel),
            AccessibleInput::Empty => None,
            _ => None,
        }
    }

    fn is_accessible_complete(&self) -> bool {
        self.submitted || self.cancelled
    }
}

// Extended accessible support for Select
impl<T: Clone + Send + 'static> Select<T> {
    /// Set the cursor position directly (for accessible mode).
    pub fn set_cursor(&mut self, index: usize) {
        if index < self.options.len() {
            self.cursor = index;
        }
    }

    /// Parse accessible input and apply it.
    ///
    /// Returns true if the selection is complete.
    pub fn apply_accessible_input(&mut self, input: &str) -> bool {
        match AccessibleInput::parse_selection(input, self.options.len()) {
            AccessibleInput::Selection(n) => {
                self.cursor = n - 1; // Convert 1-based to 0-based
                self.submitted = true;
                true
            },
            AccessibleInput::Cancel => {
                self.cancelled = true;
                true
            },
            AccessibleInput::Empty => false,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_creation() {
        let select: Select<String> = Select::new("Choose").options(vec!["A", "B", "C"]);
        assert_eq!(select.cursor(), 0);
        assert_eq!(select.options.len(), 3);
    }

    #[test]
    fn test_select_navigation() {
        let mut select: Select<String> = Select::new("Choose").options(vec!["A", "B", "C"]);
        assert_eq!(select.cursor(), 0);
        select.move_down();
        assert_eq!(select.cursor(), 1);
        select.move_down();
        assert_eq!(select.cursor(), 2);
        select.move_down(); // Should not go past end
        assert_eq!(select.cursor(), 2);
        select.move_up();
        assert_eq!(select.cursor(), 1);
    }

    #[test]
    fn test_select_submit() {
        let mut select: Select<String> = Select::new("Choose").options(vec!["A", "B", "C"]);
        select.move_down();
        assert!(select.selected().is_none());
        select.submitted = true;
        assert_eq!(select.selected(), Some(&"B".to_string()));
    }

    #[test]
    fn test_accessible_prompt() {
        let select: Select<String> =
            Select::new("Choose a color").options(vec!["Red", "Green", "Blue"]);
        let prompt = select.accessible_prompt();
        assert!(prompt.contains("Choose a color"));
        assert!(prompt.contains("1) Red"));
        assert!(prompt.contains("2) Green"));
        assert!(prompt.contains("3) Blue"));
    }

    #[test]
    fn test_accessible_apply_input() {
        let mut select: Select<String> = Select::new("Choose").options(vec!["A", "B", "C"]);
        assert!(select.apply_accessible_input("2"));
        assert!(select.is_submitted());
        assert_eq!(select.selected(), Some(&"B".to_string()));
    }

    #[test]
    fn test_accessible_cancel() {
        let mut select: Select<String> = Select::new("Choose").options(vec!["A", "B", "C"]);
        assert!(select.apply_accessible_input("q"));
        assert!(select.is_cancelled());
    }
}
