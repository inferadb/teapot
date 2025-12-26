//! Multi-selection component.
//!
//! A multi-selection list with checkboxes.
//!
//! # Example
//!
//! ```rust
//! use ferment::components::MultiSelect;
//!
//! let select = MultiSelect::new("Choose colors")
//!     .options(vec!["Red", "Green", "Blue"]);
//! ```

use crate::runtime::{Cmd, Model};
use crate::style::Color;
use crate::terminal::{Event, KeyCode};

/// Message type for multi-select.
#[derive(Debug, Clone)]
pub enum MultiSelectMsg {
    /// Move cursor up.
    Up,
    /// Move cursor down.
    Down,
    /// Toggle current item.
    Toggle,
    /// Select all items.
    SelectAll,
    /// Deselect all items.
    DeselectAll,
    /// Submit selection.
    Submit,
    /// Cancel selection.
    Cancel,
    /// Focus the select.
    Focus,
    /// Blur the select.
    Blur,
}

/// A multi-selection list component.
#[derive(Debug, Clone)]
pub struct MultiSelect<T> {
    title: String,
    options: Vec<(T, String, bool)>,
    cursor: usize,
    focused: bool,
    submitted: bool,
    cancelled: bool,
    cursor_char: &'static str,
    checked_char: &'static str,
    unchecked_char: &'static str,
    selected_color: Color,
    checked_color: Color,
    min_selections: Option<usize>,
    max_selections: Option<usize>,
}

impl<T: Clone> Default for MultiSelect<T> {
    fn default() -> Self {
        Self {
            title: String::new(),
            options: Vec::new(),
            cursor: 0,
            focused: true,
            submitted: false,
            cancelled: false,
            cursor_char: "❯",
            checked_char: "◉",
            unchecked_char: "○",
            selected_color: Color::Cyan,
            checked_color: Color::Green,
            min_selections: None,
            max_selections: None,
        }
    }
}

impl<T: Clone> MultiSelect<T> {
    /// Create a new multi-select with a title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }

    /// Set options with labels.
    pub fn options_with_labels(mut self, options: Vec<(T, String)>) -> Self {
        self.options = options.into_iter().map(|(v, l)| (v, l, false)).collect();
        self
    }

    /// Set options with pre-selected items.
    pub fn options_with_selection(mut self, options: Vec<(T, String, bool)>) -> Self {
        self.options = options;
        self
    }

    /// Set minimum required selections.
    pub fn min(mut self, min: usize) -> Self {
        self.min_selections = Some(min);
        self
    }

    /// Set maximum allowed selections.
    pub fn max(mut self, max: usize) -> Self {
        self.max_selections = Some(max);
        self
    }

    /// Get selected items.
    pub fn selected(&self) -> Vec<&T> {
        self.options
            .iter()
            .filter(|(_, _, selected)| *selected)
            .map(|(v, _, _)| v)
            .collect()
    }

    /// Get number of selected items.
    pub fn selected_count(&self) -> usize {
        self.options.iter().filter(|(_, _, s)| *s).count()
    }

    /// Check if selection meets minimum requirement.
    pub fn meets_minimum(&self) -> bool {
        match self.min_selections {
            Some(min) => self.selected_count() >= min,
            None => true,
        }
    }

    /// Check if selection meets maximum requirement.
    fn can_select_more(&self) -> bool {
        match self.max_selections {
            Some(max) => self.selected_count() < max,
            None => true,
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

    fn toggle(&mut self) {
        let can_select = self.can_select_more();
        if let Some((_, _, selected)) = self.options.get_mut(self.cursor) {
            if *selected {
                *selected = false;
            } else if can_select {
                *selected = true;
            }
        }
    }

    fn select_all(&mut self) {
        for (_, _, selected) in &mut self.options {
            *selected = true;
        }
    }

    fn deselect_all(&mut self) {
        for (_, _, selected) in &mut self.options {
            *selected = false;
        }
    }
}

impl MultiSelect<String> {
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
                (s.clone(), s, false)
            })
            .collect();
        self
    }
}

impl<T: Clone + Send + 'static> Model for MultiSelect<T> {
    type Message = MultiSelectMsg;

    fn init(&self) -> Option<Cmd<Self::Message>> {
        None
    }

    fn update(&mut self, msg: Self::Message) -> Option<Cmd<Self::Message>> {
        match msg {
            MultiSelectMsg::Up => self.move_up(),
            MultiSelectMsg::Down => self.move_down(),
            MultiSelectMsg::Toggle => self.toggle(),
            MultiSelectMsg::SelectAll => self.select_all(),
            MultiSelectMsg::DeselectAll => self.deselect_all(),
            MultiSelectMsg::Submit => {
                if self.meets_minimum() {
                    self.submitted = true;
                }
            }
            MultiSelectMsg::Cancel => self.cancelled = true,
            MultiSelectMsg::Focus => self.focused = true,
            MultiSelectMsg::Blur => self.focused = false,
        }
        None
    }

    fn view(&self) -> String {
        let mut output = String::new();

        // Title with selection count
        if !self.title.is_empty() {
            output.push_str(&format!("? {} ", self.title));
            output.push_str(&format!(
                "{}({} selected){}",
                Color::BrightBlack.to_ansi_fg(),
                self.selected_count(),
                "\x1b[0m"
            ));
            output.push('\n');
        }

        // Options
        for (i, (_, label, checked)) in self.options.iter().enumerate() {
            let is_cursor = i == self.cursor;
            let cursor = if is_cursor { self.cursor_char } else { " " };
            let check = if *checked {
                self.checked_char
            } else {
                self.unchecked_char
            };

            let check_color = if *checked {
                self.checked_color.to_ansi_fg()
            } else {
                Color::BrightBlack.to_ansi_fg()
            };

            let label_color = if is_cursor {
                self.selected_color.to_ansi_fg()
            } else {
                Color::Default.to_ansi_fg()
            };

            output.push_str(&format!(
                "{}{} {}{} {}{}{}",
                if is_cursor {
                    self.selected_color.to_ansi_fg()
                } else {
                    Color::Default.to_ansi_fg()
                },
                cursor,
                check_color,
                check,
                label_color,
                label,
                "\x1b[0m"
            ));

            if i < self.options.len() - 1 {
                output.push('\n');
            }
        }

        // Validation hint
        if let Some(min) = self.min_selections {
            if self.selected_count() < min {
                output.push_str(&format!(
                    "\n{}(Select at least {}){}",
                    Color::Yellow.to_ansi_fg(),
                    min,
                    "\x1b[0m"
                ));
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
                KeyCode::Up | KeyCode::Char('k') => Some(MultiSelectMsg::Up),
                KeyCode::Down | KeyCode::Char('j') => Some(MultiSelectMsg::Down),
                KeyCode::Char(' ') | KeyCode::Char('x') => Some(MultiSelectMsg::Toggle),
                KeyCode::Char('a') => Some(MultiSelectMsg::SelectAll),
                KeyCode::Char('n') => Some(MultiSelectMsg::DeselectAll),
                KeyCode::Enter => Some(MultiSelectMsg::Submit),
                KeyCode::Esc | KeyCode::Char('q') => Some(MultiSelectMsg::Cancel),
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
    fn test_multi_select_creation() {
        let select: MultiSelect<String> = MultiSelect::new("Choose").options(vec!["A", "B", "C"]);
        assert_eq!(select.selected_count(), 0);
    }

    #[test]
    fn test_toggle() {
        let mut select: MultiSelect<String> =
            MultiSelect::new("Choose").options(vec!["A", "B", "C"]);
        select.toggle();
        assert_eq!(select.selected_count(), 1);
        select.toggle();
        assert_eq!(select.selected_count(), 0);
    }

    #[test]
    fn test_min_max() {
        let mut select: MultiSelect<String> = MultiSelect::new("Choose")
            .options(vec!["A", "B", "C"])
            .min(1)
            .max(2);

        assert!(!select.meets_minimum());
        select.toggle();
        assert!(select.meets_minimum());
        select.move_down();
        select.toggle();
        assert_eq!(select.selected_count(), 2);
        select.move_down();
        select.toggle(); // Should not toggle, at max
        assert_eq!(select.selected_count(), 2);
    }
}
