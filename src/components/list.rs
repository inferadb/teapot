//! List component with filtering and pagination.
//!
//! A filterable, scrollable list with keyboard navigation.
//!
//! # Example
//!
//! ```rust
//! use teapot::components::List;
//!
//! let list = List::new("Select a file")
//!     .items(vec!["main.rs", "lib.rs", "Cargo.toml"])
//!     .height(10)
//!     .filterable(true);
//! ```

use crate::{
    runtime::{Cmd, Model},
    style::Color,
    terminal::{Event, KeyCode, KeyModifiers},
};

/// Message type for list.
#[derive(Debug, Clone)]
pub enum ListMsg {
    /// Move selection up.
    Up,
    /// Move selection down.
    Down,
    /// Move to first item.
    First,
    /// Move to last item.
    Last,
    /// Page up.
    PageUp,
    /// Page down.
    PageDown,
    /// Insert character into filter.
    InsertFilterChar(char),
    /// Delete character from filter.
    DeleteFilterChar,
    /// Clear the filter.
    ClearFilter,
    /// Submit selection.
    Submit,
    /// Cancel selection.
    Cancel,
    /// Focus the list.
    Focus,
    /// Blur the list.
    Blur,
}

/// A filterable, paginated list component.
#[derive(Debug, Clone)]
pub struct List<T> {
    title: String,
    items: Vec<(T, String)>,
    filtered_indices: Vec<usize>,
    filter: String,
    cursor: usize,
    offset: usize,
    height: usize,
    focused: bool,
    submitted: bool,
    cancelled: bool,
    filterable: bool,
    filter_placeholder: String,
    cursor_char: &'static str,
    selected_color: Color,
    unselected_color: Color,
    filter_color: Color,
    match_highlight_color: Color,
    no_match_text: String,
}

impl<T: Clone> Default for List<T> {
    fn default() -> Self {
        Self {
            title: String::new(),
            items: Vec::new(),
            filtered_indices: Vec::new(),
            filter: String::new(),
            cursor: 0,
            offset: 0,
            height: 10,
            focused: true,
            submitted: false,
            cancelled: false,
            filterable: true,
            filter_placeholder: "Type to filter...".to_string(),
            cursor_char: "❯",
            selected_color: Color::Cyan,
            unselected_color: Color::Default,
            filter_color: Color::Yellow,
            match_highlight_color: Color::Green,
            no_match_text: "No matching items".to_string(),
        }
    }
}

impl<T: Clone> List<T> {
    /// Create a new list with a title.
    pub fn new(title: impl Into<String>) -> Self {
        Self { title: title.into(), ..Default::default() }
    }

    /// Set items with display labels.
    pub fn items_with_labels(mut self, items: Vec<(T, String)>) -> Self {
        self.items = items;
        self.rebuild_filtered();
        self
    }

    /// Set the visible height.
    pub fn height(mut self, height: usize) -> Self {
        self.height = height;
        self
    }

    /// Set whether filtering is enabled.
    pub fn filterable(mut self, filterable: bool) -> Self {
        self.filterable = filterable;
        self
    }

    /// Set the filter placeholder text.
    pub fn filter_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.filter_placeholder = placeholder.into();
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

    /// Set the filter input color.
    pub fn filter_color(mut self, color: Color) -> Self {
        self.filter_color = color;
        self
    }

    /// Set the match highlight color.
    pub fn match_highlight_color(mut self, color: Color) -> Self {
        self.match_highlight_color = color;
        self
    }

    /// Set the text shown when no items match.
    pub fn no_match_text(mut self, text: impl Into<String>) -> Self {
        self.no_match_text = text.into();
        self
    }

    /// Get the current cursor position in the filtered list.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Get the current filter string.
    pub fn filter_text(&self) -> &str {
        &self.filter
    }

    /// Get the number of filtered items.
    pub fn filtered_count(&self) -> usize {
        self.filtered_indices.len()
    }

    /// Get the total number of items.
    pub fn total_count(&self) -> usize {
        self.items.len()
    }

    /// Get the selected value if submitted.
    pub fn selected(&self) -> Option<&T> {
        if self.submitted && !self.filtered_indices.is_empty() {
            let idx = self.filtered_indices.get(self.cursor)?;
            self.items.get(*idx).map(|(v, _)| v)
        } else {
            None
        }
    }

    /// Get the current value regardless of submit state.
    pub fn current(&self) -> Option<&T> {
        if self.filtered_indices.is_empty() {
            None
        } else {
            let idx = self.filtered_indices.get(self.cursor)?;
            self.items.get(*idx).map(|(v, _)| v)
        }
    }

    /// Get the current item's label.
    pub fn current_label(&self) -> Option<&str> {
        if self.filtered_indices.is_empty() {
            None
        } else {
            let idx = self.filtered_indices.get(self.cursor)?;
            self.items.get(*idx).map(|(_, l)| l.as_str())
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

    /// Set items dynamically.
    pub fn set_items(&mut self, items: Vec<(T, String)>) {
        self.items = items;
        self.rebuild_filtered();
    }

    /// Set filter and rebuild filtered list.
    pub fn set_filter(&mut self, filter: String) {
        self.filter = filter;
        self.rebuild_filtered();
    }

    /// Rebuild filtered indices based on current filter.
    fn rebuild_filtered(&mut self) {
        if self.filter.is_empty() {
            self.filtered_indices = (0..self.items.len()).collect();
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.filtered_indices = self
                .items
                .iter()
                .enumerate()
                .filter(|(_, (_, label))| label.to_lowercase().contains(&filter_lower))
                .map(|(i, _)| i)
                .collect();
        }
        // Reset cursor and offset when filter changes
        self.cursor = 0;
        self.offset = 0;
    }

    fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.ensure_visible();
        }
    }

    fn move_down(&mut self) {
        if self.cursor < self.filtered_indices.len().saturating_sub(1) {
            self.cursor += 1;
            self.ensure_visible();
        }
    }

    fn move_first(&mut self) {
        self.cursor = 0;
        self.offset = 0;
    }

    fn move_last(&mut self) {
        self.cursor = self.filtered_indices.len().saturating_sub(1);
        self.ensure_visible();
    }

    fn page_up(&mut self) {
        let page_size = self.height.saturating_sub(1);
        self.cursor = self.cursor.saturating_sub(page_size);
        self.ensure_visible();
    }

    fn page_down(&mut self) {
        let page_size = self.height.saturating_sub(1);
        self.cursor = (self.cursor + page_size).min(self.filtered_indices.len().saturating_sub(1));
        self.ensure_visible();
    }

    fn ensure_visible(&mut self) {
        // Scroll up if cursor is above visible area
        if self.cursor < self.offset {
            self.offset = self.cursor;
        }
        // Scroll down if cursor is below visible area
        if self.cursor >= self.offset + self.height {
            self.offset = self.cursor.saturating_sub(self.height - 1);
        }
    }

    fn max_offset(&self) -> usize {
        self.filtered_indices.len().saturating_sub(self.height)
    }

    fn visible_range(&self) -> (usize, usize) {
        let start = self.offset;
        let end = (self.offset + self.height).min(self.filtered_indices.len());
        (start, end)
    }

    /// Highlight matching text in label.
    fn highlight_match(&self, label: &str) -> String {
        if self.filter.is_empty() {
            return label.to_string();
        }

        let filter_lower = self.filter.to_lowercase();
        let label_lower = label.to_lowercase();

        if let Some(pos) = label_lower.find(&filter_lower) {
            let before = &label[..pos];
            let matched = &label[pos..pos + self.filter.len()];
            let after = &label[pos + self.filter.len()..];

            format!(
                "{}{}{}{}{}",
                before,
                self.match_highlight_color.to_ansi_fg(),
                matched,
                "\x1b[0m",
                after
            )
        } else {
            label.to_string()
        }
    }
}

impl List<String> {
    /// Set items from strings.
    pub fn items<I, S>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.items = items
            .into_iter()
            .map(|s| {
                let s = s.into();
                (s.clone(), s)
            })
            .collect();
        self.rebuild_filtered();
        self
    }
}

impl<T: Clone + Send + 'static> Model for List<T> {
    type Message = ListMsg;

    fn init(&self) -> Option<Cmd<Self::Message>> {
        None
    }

    fn update(&mut self, msg: Self::Message) -> Option<Cmd<Self::Message>> {
        match msg {
            ListMsg::Up => self.move_up(),
            ListMsg::Down => self.move_down(),
            ListMsg::First => self.move_first(),
            ListMsg::Last => self.move_last(),
            ListMsg::PageUp => self.page_up(),
            ListMsg::PageDown => self.page_down(),
            ListMsg::InsertFilterChar(c) => {
                if self.filterable {
                    self.filter.push(c);
                    self.rebuild_filtered();
                }
            },
            ListMsg::DeleteFilterChar => {
                if self.filterable && !self.filter.is_empty() {
                    self.filter.pop();
                    self.rebuild_filtered();
                }
            },
            ListMsg::ClearFilter => {
                if self.filterable {
                    self.filter.clear();
                    self.rebuild_filtered();
                }
            },
            ListMsg::Submit => {
                if !self.filtered_indices.is_empty() {
                    self.submitted = true;
                }
            },
            ListMsg::Cancel => self.cancelled = true,
            ListMsg::Focus => self.focused = true,
            ListMsg::Blur => self.focused = false,
        }
        None
    }

    fn view(&self) -> String {
        let mut output = String::new();

        // Title
        if !self.title.is_empty() {
            output.push_str(&format!("? {}\n", self.title));
        }

        // Filter input (if filterable)
        if self.filterable {
            output.push_str(&format!(
                "{}/ {}{}",
                self.filter_color.to_ansi_fg(),
                if self.filter.is_empty() {
                    format!("{}{}", Color::BrightBlack.to_ansi_fg(), self.filter_placeholder)
                } else {
                    self.filter.clone()
                },
                "\x1b[0m\n"
            ));
        }

        // Check if we have items
        if self.filtered_indices.is_empty() {
            if self.items.is_empty() {
                output.push_str(&format!(
                    "{}(no items){}",
                    Color::BrightBlack.to_ansi_fg(),
                    "\x1b[0m"
                ));
            } else {
                output.push_str(&format!(
                    "{}{}{}",
                    Color::BrightBlack.to_ansi_fg(),
                    self.no_match_text,
                    "\x1b[0m"
                ));
            }
            return output;
        }

        // Scroll indicator (top)
        if self.offset > 0 {
            output.push_str(&format!(
                "{}  ↑ {} more{}",
                Color::BrightBlack.to_ansi_fg(),
                self.offset,
                "\x1b[0m\n"
            ));
        }

        // Visible items
        let (start, end) = self.visible_range();
        for (view_idx, filtered_idx) in (start..end).enumerate() {
            let actual_idx = self.filtered_indices[filtered_idx];
            let (_, label) = &self.items[actual_idx];

            let is_selected = filtered_idx == self.cursor;
            let cursor = if is_selected { self.cursor_char } else { " " };

            let display_label = if self.filterable && !self.filter.is_empty() {
                self.highlight_match(label)
            } else {
                label.clone()
            };

            if is_selected {
                output.push_str(&format!(
                    "{}{} {}{}",
                    self.selected_color.to_ansi_fg(),
                    cursor,
                    display_label,
                    "\x1b[0m"
                ));
            } else {
                output.push_str(&format!(
                    "{}{} {}{}",
                    self.unselected_color.to_ansi_fg(),
                    cursor,
                    display_label,
                    "\x1b[0m"
                ));
            }

            if view_idx < (end - start - 1) {
                output.push('\n');
            }
        }

        // Scroll indicator (bottom)
        let remaining = self.filtered_indices.len().saturating_sub(end);
        if remaining > 0 {
            output.push_str(&format!(
                "\n{}  ↓ {} more{}",
                Color::BrightBlack.to_ansi_fg(),
                remaining,
                "\x1b[0m"
            ));
        }

        output
    }

    fn handle_event(&self, event: Event) -> Option<Self::Message> {
        if !self.focused {
            return None;
        }

        match event {
            Event::Key(key) => {
                // Handle control keys first
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    return match key.code {
                        KeyCode::Char('u') => Some(ListMsg::ClearFilter),
                        KeyCode::Char('n') => Some(ListMsg::Down),
                        KeyCode::Char('p') => Some(ListMsg::Up),
                        _ => None,
                    };
                }

                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => Some(ListMsg::Up),
                    KeyCode::Down | KeyCode::Char('j') => Some(ListMsg::Down),
                    KeyCode::Home => Some(ListMsg::First),
                    KeyCode::End => Some(ListMsg::Last),
                    KeyCode::PageUp => Some(ListMsg::PageUp),
                    KeyCode::PageDown => Some(ListMsg::PageDown),
                    KeyCode::Enter => Some(ListMsg::Submit),
                    KeyCode::Esc => {
                        if self.filterable && !self.filter.is_empty() {
                            Some(ListMsg::ClearFilter)
                        } else {
                            Some(ListMsg::Cancel)
                        }
                    },
                    KeyCode::Backspace => Some(ListMsg::DeleteFilterChar),
                    KeyCode::Char(c) => {
                        if self.filterable {
                            Some(ListMsg::InsertFilterChar(c))
                        } else if c == 'q' {
                            Some(ListMsg::Cancel)
                        } else {
                            None
                        }
                    },
                    _ => None,
                }
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_creation() {
        let list: List<String> = List::new("Choose").items(vec!["A", "B", "C"]);
        assert_eq!(list.cursor(), 0);
        assert_eq!(list.total_count(), 3);
        assert_eq!(list.filtered_count(), 3);
    }

    #[test]
    fn test_list_navigation() {
        let mut list: List<String> = List::new("Choose").items(vec!["A", "B", "C"]);
        assert_eq!(list.cursor(), 0);
        list.move_down();
        assert_eq!(list.cursor(), 1);
        list.move_down();
        assert_eq!(list.cursor(), 2);
        list.move_down(); // Should not go past end
        assert_eq!(list.cursor(), 2);
        list.move_up();
        assert_eq!(list.cursor(), 1);
    }

    #[test]
    fn test_list_filtering() {
        let mut list: List<String> =
            List::new("Choose").items(vec!["Apple", "Banana", "Apricot", "Orange"]);
        assert_eq!(list.filtered_count(), 4);

        list.set_filter("ap".to_string());
        assert_eq!(list.filtered_count(), 2); // Apple, Apricot

        list.set_filter("ban".to_string());
        assert_eq!(list.filtered_count(), 1); // Banana

        list.set_filter("xyz".to_string());
        assert_eq!(list.filtered_count(), 0);

        list.set_filter(String::new());
        assert_eq!(list.filtered_count(), 4);
    }

    #[test]
    fn test_list_submit() {
        let mut list: List<String> = List::new("Choose").items(vec!["A", "B", "C"]);
        list.move_down();
        assert!(list.selected().is_none());
        list.submitted = true;
        assert_eq!(list.selected(), Some(&"B".to_string()));
    }

    #[test]
    fn test_list_pagination() {
        let mut list: List<String> = List::new("Choose")
            .items(vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"])
            .height(3);

        assert_eq!(list.visible_range(), (0, 3));
        list.move_down();
        list.move_down();
        list.move_down();
        assert_eq!(list.cursor(), 3);
        assert_eq!(list.visible_range(), (1, 4)); // Scrolled down

        list.move_last();
        assert_eq!(list.cursor(), 9);
        assert_eq!(list.visible_range(), (7, 10));
    }

    #[test]
    fn test_list_with_labels() {
        let list: List<i32> = List::new("Choose")
            .items_with_labels(vec![(1, "One".to_string()), (2, "Two".to_string())]);
        assert_eq!(list.current(), Some(&1));
        assert_eq!(list.current_label(), Some("One"));
    }
}
