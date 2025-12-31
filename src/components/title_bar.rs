//! Title bar component for full-screen TUI views.
//!
//! Renders a decorative title bar with slash-style separators.
//!
//! # Example
//!
//! ```rust
//! use teapot::components::TitleBar;
//!
//! // Title only
//! let bar = TitleBar::new("My Application").width(80);
//! println!("{}", bar.render());
//! // Output: //  My Application  ///////////////...
//!
//! // Title with subtitle
//! let bar = TitleBar::new("My Application")
//!     .subtitle("Settings")
//!     .width(80);
//! println!("{}", bar.render());
//! // Output: //  My Application  /////...  Settings  //
//! ```

use crate::{
    runtime::{Cmd, Model},
    style::{Color, RESET},
    terminal::Event,
};

/// Message type for title bar (currently none needed).
#[derive(Debug, Clone)]
pub enum TitleBarMsg {}

/// A decorative title bar with slash-style separators.
///
/// The title bar uses a distinctive visual style with forward slashes
/// as decorative elements:
/// - Title only: `//  Title  ///////////////...`
/// - With subtitle: `//  Title  /////...  Subtitle  //`
#[derive(Debug, Clone)]
pub struct TitleBar {
    /// The main title text.
    title: String,
    /// Optional subtitle text (shown on the right).
    subtitle: Option<String>,
    /// Width of the title bar in characters.
    width: usize,
    /// Color for the title text.
    title_color: Color,
    /// Color for the subtitle text.
    subtitle_color: Color,
    /// Color for the slash separators.
    separator_color: Color,
}

impl Default for TitleBar {
    fn default() -> Self {
        Self {
            title: String::new(),
            subtitle: None,
            width: 80,
            title_color: Color::Default,
            subtitle_color: Color::Default,
            separator_color: Color::BrightBlack,
        }
    }
}

impl TitleBar {
    /// Create a new title bar with the given title.
    pub fn new(title: impl Into<String>) -> Self {
        Self { title: title.into(), ..Default::default() }
    }

    /// Set the subtitle (shown on the right side).
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Set the width in characters.
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Set the title text color.
    pub fn title_color(mut self, color: Color) -> Self {
        self.title_color = color;
        self
    }

    /// Set the subtitle text color.
    pub fn subtitle_color(mut self, color: Color) -> Self {
        self.subtitle_color = color;
        self
    }

    /// Set the separator (slash) color.
    pub fn separator_color(mut self, color: Color) -> Self {
        self.separator_color = color;
        self
    }

    /// Render the title bar as a string.
    pub fn render(&self) -> String {
        Model::view(self)
    }

    /// Get the title text.
    pub fn title_text(&self) -> &str {
        &self.title
    }

    /// Get the subtitle text, if any.
    pub fn subtitle_text(&self) -> Option<&str> {
        self.subtitle.as_deref()
    }
}

impl Model for TitleBar {
    type Message = TitleBarMsg;

    fn init(&self) -> Option<Cmd<Self::Message>> {
        None
    }

    fn update(&mut self, _msg: Self::Message) -> Option<Cmd<Self::Message>> {
        None
    }

    fn view(&self) -> String {
        if self.title.is_empty() {
            return String::new();
        }

        let reset = RESET;
        let sep_color = self.separator_color.to_ansi_fg();
        let title_color = if matches!(self.title_color, Color::Default) {
            reset.to_string()
        } else {
            self.title_color.to_ansi_fg()
        };
        let subtitle_color = if matches!(self.subtitle_color, Color::Default) {
            reset.to_string()
        } else {
            self.subtitle_color.to_ansi_fg()
        };

        match &self.subtitle {
            None => {
                // No subtitle: "//  Title  //////..."
                // Format: {sep}// {reset}{title}  {sep}///...{reset}
                let prefix_len = 2 + 2 + self.title.len() + 2; // "//" + "  " + title + "  "
                let remaining = self.width.saturating_sub(prefix_len);
                format!(
                    "{}//{}  {}{}  {}{}{}",
                    sep_color,
                    reset,
                    title_color,
                    self.title,
                    sep_color,
                    "/".repeat(remaining),
                    reset
                )
            },
            Some(subtitle) => {
                // With subtitle: "//  Title  /////...  Subtitle  //"
                let prefix_len = 2 + 2 + self.title.len() + 2; // "//" + "  " + title + "  "
                let suffix_len = 2 + subtitle.len() + 2 + 2; // "  " + subtitle + "  " + "//"
                let fill_count = self.width.saturating_sub(prefix_len + suffix_len);
                let fill = "/".repeat(fill_count);
                format!(
                    "{}//{}  {}{}  {}{}{}  {}{}  {}//{}",
                    sep_color,
                    reset,
                    title_color,
                    self.title,
                    sep_color,
                    fill,
                    reset,
                    subtitle_color,
                    subtitle,
                    sep_color,
                    reset
                )
            },
        }
    }

    fn handle_event(&self, _event: Event) -> Option<Self::Message> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_bar_creation() {
        let bar = TitleBar::new("Test");
        assert_eq!(bar.title_text(), "Test");
        assert!(bar.subtitle_text().is_none());
    }

    #[test]
    fn test_title_bar_with_subtitle() {
        let bar = TitleBar::new("App").subtitle("Settings");
        assert_eq!(bar.title_text(), "App");
        assert_eq!(bar.subtitle_text(), Some("Settings"));
    }

    #[test]
    fn test_title_bar_width() {
        let bar = TitleBar::new("Test").width(100);
        assert_eq!(bar.width, 100);
    }

    #[test]
    fn test_empty_title() {
        let bar = TitleBar::new("").width(80);
        assert_eq!(bar.render(), "");
    }

    #[test]
    fn test_render_title_only() {
        let bar = TitleBar::new("Test").width(20);
        let rendered = bar.render();
        // Should contain the title
        assert!(rendered.contains("Test"));
        // Should contain slashes
        assert!(rendered.contains("//"));
    }

    #[test]
    fn test_render_with_subtitle() {
        let bar = TitleBar::new("App").subtitle("Sub").width(40);
        let rendered = bar.render();
        // Should contain both title and subtitle
        assert!(rendered.contains("App"));
        assert!(rendered.contains("Sub"));
        // Should start and end with slashes
        assert!(rendered.contains("//"));
    }

    #[test]
    fn test_custom_colors() {
        let bar = TitleBar::new("Test")
            .title_color(Color::Cyan)
            .subtitle_color(Color::Yellow)
            .separator_color(Color::Red)
            .width(40);
        let rendered = bar.render();
        // Should contain ANSI color codes
        assert!(rendered.contains("\x1b["));
    }
}
