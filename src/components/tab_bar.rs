//! Tab bar component for tabbed interfaces.
//!
//! A horizontal tab bar with keyboard hints and optional status indicator.
//!
//! # Example
//!
//! ```rust
//! use ferment::components::{TabBar, Tab};
//!
//! let tab_bar = TabBar::new()
//!     .tabs(vec![
//!         Tab::new("urls", "URLs").key('u'),
//!         Tab::new("services", "Services").key('s'),
//!         Tab::new("nodes", "Nodes").key('n'),
//!         Tab::new("pods", "Pods").key('p'),
//!     ])
//!     .selected("urls");
//! ```

use crate::{
    runtime::{Cmd, Model},
    style::Color,
    terminal::{Event, KeyCode},
};

/// A single tab in the tab bar.
#[derive(Debug, Clone)]
pub struct Tab {
    /// Unique identifier for the tab.
    pub id: String,
    /// Display label for the tab.
    pub label: String,
    /// Keyboard shortcut (optional).
    pub key: Option<char>,
}

impl Tab {
    /// Create a new tab with an ID and label.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self { id: id.into(), label: label.into(), key: None }
    }

    /// Set the keyboard shortcut for this tab.
    pub fn key(mut self, key: char) -> Self {
        self.key = Some(key);
        self
    }

    /// Set the keyboard shortcut to the first character of the label.
    pub fn auto_key(mut self) -> Self {
        self.key = self.label.chars().next().map(|c| c.to_ascii_lowercase());
        self
    }
}

/// Message type for tab bar.
#[derive(Debug, Clone)]
pub enum TabBarMsg {
    /// Select a tab by ID.
    Select(String),
    /// Select next tab.
    Next,
    /// Select previous tab.
    Previous,
}

/// A horizontal tab bar component.
#[derive(Debug, Clone)]
pub struct TabBar {
    tabs: Vec<Tab>,
    selected: String,
    active_color: Color,
    active_bg_color: Option<Color>,
    inactive_color: Color,
    key_color: Color,
    separator: String,
    width: Option<usize>,
}

impl Default for TabBar {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            selected: String::new(),
            active_color: Color::Cyan,
            active_bg_color: None,
            inactive_color: Color::BrightBlack,
            key_color: Color::Cyan,
            separator: " ".to_string(),
            width: None,
        }
    }
}

impl TabBar {
    /// Create a new empty tab bar.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the tabs.
    pub fn tabs(mut self, tabs: Vec<Tab>) -> Self {
        if !tabs.is_empty() && self.selected.is_empty() {
            self.selected = tabs[0].id.clone();
        }
        self.tabs = tabs;
        self
    }

    /// Set the selected tab by ID.
    pub fn selected(mut self, id: impl Into<String>) -> Self {
        self.selected = id.into();
        self
    }

    /// Set the active tab color.
    pub fn active_color(mut self, color: Color) -> Self {
        self.active_color = color;
        self
    }

    /// Set the active tab background color.
    pub fn active_bg_color(mut self, color: Color) -> Self {
        self.active_bg_color = Some(color);
        self
    }

    /// Set the inactive tab color.
    pub fn inactive_color(mut self, color: Color) -> Self {
        self.inactive_color = color;
        self
    }

    /// Set the keyboard hint color.
    pub fn key_color(mut self, color: Color) -> Self {
        self.key_color = color;
        self
    }

    /// Set the separator between tabs.
    pub fn separator(mut self, separator: impl Into<String>) -> Self {
        self.separator = separator.into();
        self
    }

    /// Set the width for right-padding.
    pub fn width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Get the currently selected tab ID.
    pub fn selected_id(&self) -> &str {
        &self.selected
    }

    /// Get the currently selected tab.
    pub fn selected_tab(&self) -> Option<&Tab> {
        self.tabs.iter().find(|t| t.id == self.selected)
    }

    /// Get the index of the selected tab.
    pub fn selected_index(&self) -> Option<usize> {
        self.tabs.iter().position(|t| t.id == self.selected)
    }

    /// Get all tabs.
    pub fn get_tabs(&self) -> &[Tab] {
        &self.tabs
    }

    /// Set the selected tab by ID (mutable).
    pub fn set_selected(&mut self, id: impl Into<String>) {
        self.selected = id.into();
    }

    /// Select the next tab.
    fn select_next(&mut self) {
        if let Some(idx) = self.selected_index() {
            let next_idx = (idx + 1) % self.tabs.len();
            self.selected = self.tabs[next_idx].id.clone();
        }
    }

    /// Select the previous tab.
    fn select_previous(&mut self) {
        if let Some(idx) = self.selected_index() {
            let prev_idx = if idx == 0 { self.tabs.len() - 1 } else { idx - 1 };
            self.selected = self.tabs[prev_idx].id.clone();
        }
    }

    /// Get the tab ID for a keyboard shortcut.
    pub fn tab_for_key(&self, key: char) -> Option<&str> {
        self.tabs.iter().find(|t| t.key == Some(key)).map(|t| t.id.as_str())
    }

    /// Render the tab bar as a string.
    pub fn render(&self) -> String {
        use crate::Model;
        Model::view(self)
    }
}

impl Model for TabBar {
    type Message = TabBarMsg;

    fn init(&self) -> Option<Cmd<Self::Message>> {
        None
    }

    fn update(&mut self, msg: Self::Message) -> Option<Cmd<Self::Message>> {
        match msg {
            TabBarMsg::Select(id) => {
                if self.tabs.iter().any(|t| t.id == id) {
                    self.selected = id;
                }
            },
            TabBarMsg::Next => self.select_next(),
            TabBarMsg::Previous => self.select_previous(),
        }
        None
    }

    fn view(&self) -> String {
        if self.tabs.is_empty() {
            return String::new();
        }

        let mut output = String::new();
        let mut content_len = 0;

        for (i, tab) in self.tabs.iter().enumerate() {
            let is_active = tab.id == self.selected;

            if is_active {
                // Active tab: entire label in active color (with optional background)
                let bg = self.active_bg_color.as_ref().map(|c| c.to_ansi_bg()).unwrap_or_default();
                output.push_str(&format!(
                    "{}{}{}{}",
                    bg,
                    self.active_color.to_ansi_fg(),
                    tab.label,
                    "\x1b[0m",
                ));
            } else if tab.key.is_some() {
                // Inactive tab with key hint: first char highlighted, rest dimmed
                let first_char = tab.label.chars().next().unwrap_or_default();
                let rest: String = tab.label.chars().skip(1).collect();

                output.push_str(&format!(
                    "{}{}{}{}{}",
                    self.key_color.to_ansi_fg(),
                    first_char,
                    "\x1b[0m",
                    self.inactive_color.to_ansi_fg(),
                    rest,
                ));
                output.push_str("\x1b[0m");
            } else {
                // Inactive tab without key hint: all dimmed
                output.push_str(&format!(
                    "{}{}{}",
                    self.inactive_color.to_ansi_fg(),
                    tab.label,
                    "\x1b[0m",
                ));
            }

            content_len += tab.label.chars().count();

            if i < self.tabs.len() - 1 {
                output.push_str(&self.separator);
                content_len += self.separator.chars().count();
            }
        }

        // Add padding if width is specified
        if let Some(width) = self.width {
            let padding = width.saturating_sub(content_len);
            output.push_str(&" ".repeat(padding));
        }

        output
    }

    fn handle_event(&self, event: Event) -> Option<Self::Message> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Tab => Some(TabBarMsg::Next),
                KeyCode::BackTab => Some(TabBarMsg::Previous),
                KeyCode::Char(c) => {
                    // Check if character matches any tab's key
                    self.tab_for_key(c).map(|id| TabBarMsg::Select(id.to_string()))
                },
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
    fn test_tab_creation() {
        let tab = Tab::new("test", "Test Tab").key('t');
        assert_eq!(tab.id, "test");
        assert_eq!(tab.label, "Test Tab");
        assert_eq!(tab.key, Some('t'));
    }

    #[test]
    fn test_tab_auto_key() {
        let tab = Tab::new("urls", "URLs").auto_key();
        assert_eq!(tab.key, Some('u'));
    }

    #[test]
    fn test_tab_bar_creation() {
        let bar = TabBar::new()
            .tabs(vec![Tab::new("a", "Alpha").key('a'), Tab::new("b", "Beta").key('b')]);

        assert_eq!(bar.selected_id(), "a");
        assert_eq!(bar.get_tabs().len(), 2);
    }

    #[test]
    fn test_tab_bar_selection() {
        let mut bar = TabBar::new()
            .tabs(vec![Tab::new("a", "Alpha"), Tab::new("b", "Beta"), Tab::new("c", "Charlie")])
            .selected("b");

        assert_eq!(bar.selected_id(), "b");
        assert_eq!(bar.selected_index(), Some(1));

        bar.select_next();
        assert_eq!(bar.selected_id(), "c");

        bar.select_next();
        assert_eq!(bar.selected_id(), "a"); // Wraps around

        bar.select_previous();
        assert_eq!(bar.selected_id(), "c"); // Wraps around
    }

    #[test]
    fn test_tab_for_key() {
        let bar = TabBar::new().tabs(vec![
            Tab::new("urls", "URLs").key('u'),
            Tab::new("services", "Services").key('s'),
        ]);

        assert_eq!(bar.tab_for_key('u'), Some("urls"));
        assert_eq!(bar.tab_for_key('s'), Some("services"));
        assert_eq!(bar.tab_for_key('x'), None);
    }
}
