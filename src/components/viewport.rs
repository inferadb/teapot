//! Viewport component for scrollable content.
//!
//! A viewport displays content that can be scrolled vertically.
//!
//! # Example
//!
//! ```rust
//! use teapot::components::Viewport;
//!
//! let viewport = Viewport::new(80, 20)
//!     .content("Long content here...");
//! ```

use crate::{
    runtime::{Cmd, Model},
    terminal::{Event, KeyCode, KeyModifiers},
};

/// Message type for viewport.
#[derive(Debug, Clone)]
pub enum ViewportMsg {
    /// Scroll up by lines.
    ScrollUp(usize),
    /// Scroll down by lines.
    ScrollDown(usize),
    /// Scroll to top.
    ScrollToTop,
    /// Scroll to bottom.
    ScrollToBottom,
    /// Page up.
    PageUp,
    /// Page down.
    PageDown,
    /// Set content.
    SetContent(String),
    /// Resize viewport.
    Resize { width: usize, height: usize },
}

/// A scrollable viewport component.
#[derive(Debug, Clone)]
#[must_use = "components do nothing unless used in a view or run with Program"]
pub struct Viewport {
    content: String,
    lines: Vec<String>,
    offset: usize,
    width: usize,
    height: usize,
    focused: bool,
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new(80, 24)
    }
}

impl Viewport {
    /// Create a new viewport with dimensions.
    pub fn new(width: usize, height: usize) -> Self {
        Self { content: String::new(), lines: Vec::new(), offset: 0, width, height, focused: true }
    }

    /// Set the content.
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.set_content(content.into());
        self
    }

    /// Set content and recompute lines.
    pub fn set_content(&mut self, content: String) {
        self.content = content;
        self.lines = self.content.lines().map(String::from).collect();
        // Ensure offset is valid
        self.offset = self.offset.min(self.max_offset());
    }

    /// Get the current scroll offset.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Get total number of lines.
    pub fn total_lines(&self) -> usize {
        self.lines.len()
    }

    /// Get maximum scroll offset.
    pub fn max_offset(&self) -> usize {
        self.lines.len().saturating_sub(self.height)
    }

    /// Check if at top.
    pub fn at_top(&self) -> bool {
        self.offset == 0
    }

    /// Check if at bottom.
    pub fn at_bottom(&self) -> bool {
        self.offset >= self.max_offset()
    }

    /// Get visible line range.
    pub fn visible_range(&self) -> (usize, usize) {
        let start = self.offset;
        let end = (self.offset + self.height).min(self.lines.len());
        (start, end)
    }

    /// Scroll up.
    fn scroll_up(&mut self, lines: usize) {
        self.offset = self.offset.saturating_sub(lines);
    }

    /// Scroll down.
    fn scroll_down(&mut self, lines: usize) {
        self.offset = (self.offset + lines).min(self.max_offset());
    }

    /// Scroll to top.
    fn scroll_to_top(&mut self) {
        self.offset = 0;
    }

    /// Scroll to bottom.
    fn scroll_to_bottom(&mut self) {
        self.offset = self.max_offset();
    }

    /// Page up.
    fn page_up(&mut self) {
        self.scroll_up(self.height.saturating_sub(1));
    }

    /// Page down.
    fn page_down(&mut self) {
        self.scroll_down(self.height.saturating_sub(1));
    }

    /// Set focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
}

impl Model for Viewport {
    type Message = ViewportMsg;

    fn init(&self) -> Option<Cmd<Self::Message>> {
        None
    }

    fn update(&mut self, msg: Self::Message) -> Option<Cmd<Self::Message>> {
        match msg {
            ViewportMsg::ScrollUp(lines) => self.scroll_up(lines),
            ViewportMsg::ScrollDown(lines) => self.scroll_down(lines),
            ViewportMsg::ScrollToTop => self.scroll_to_top(),
            ViewportMsg::ScrollToBottom => self.scroll_to_bottom(),
            ViewportMsg::PageUp => self.page_up(),
            ViewportMsg::PageDown => self.page_down(),
            ViewportMsg::SetContent(content) => self.set_content(content),
            ViewportMsg::Resize { width, height } => {
                self.width = width;
                self.height = height;
                self.offset = self.offset.min(self.max_offset());
            },
        }
        None
    }

    fn view(&self) -> String {
        let (start, end) = self.visible_range();
        let mut output = String::new();

        for (i, line) in self.lines[start..end].iter().enumerate() {
            // Truncate line to width
            let display = if line.len() > self.width {
                format!("{}...", &line[..self.width.saturating_sub(3)])
            } else {
                line.clone()
            };

            output.push_str(&display);

            if start + i < end - 1 {
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
                KeyCode::Up | KeyCode::Char('k') => Some(ViewportMsg::ScrollUp(1)),
                KeyCode::Down | KeyCode::Char('j') => Some(ViewportMsg::ScrollDown(1)),
                KeyCode::PageUp => Some(ViewportMsg::PageUp),
                KeyCode::PageDown => Some(ViewportMsg::PageDown),
                KeyCode::Home => Some(ViewportMsg::ScrollToTop),
                KeyCode::End => Some(ViewportMsg::ScrollToBottom),
                KeyCode::Char('g') => Some(ViewportMsg::ScrollToTop),
                KeyCode::Char('G') => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        Some(ViewportMsg::ScrollToBottom)
                    } else {
                        None
                    }
                },
                _ => None,
            },
            Event::Resize { width, height } => {
                Some(ViewportMsg::Resize { width: width as usize, height: height as usize })
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_creation() {
        let viewport = Viewport::new(80, 10).content("line1\nline2\nline3");
        assert_eq!(viewport.total_lines(), 3);
        assert_eq!(viewport.offset(), 0);
    }

    #[test]
    fn test_scrolling() {
        let mut viewport = Viewport::new(80, 2).content("1\n2\n3\n4\n5");
        assert_eq!(viewport.offset(), 0);
        viewport.scroll_down(1);
        assert_eq!(viewport.offset(), 1);
        viewport.scroll_down(10);
        assert_eq!(viewport.offset(), 3); // Max offset
        viewport.scroll_up(1);
        assert_eq!(viewport.offset(), 2);
    }

    #[test]
    fn test_page_navigation() {
        let mut viewport = Viewport::new(80, 3).content("1\n2\n3\n4\n5\n6\n7\n8");
        viewport.page_down();
        assert_eq!(viewport.offset(), 2);
        viewport.page_up();
        assert_eq!(viewport.offset(), 0);
    }
}
