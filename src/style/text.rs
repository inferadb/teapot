//! Text styling.
//!
//! Provides a fluent API for styling text with colors and attributes.

use super::Color;

/// Text style with colors and attributes.
#[derive(Debug, Clone, Default)]
pub struct Style {
    foreground: Option<Color>,
    background: Option<Color>,
    bold: bool,
    dim: bool,
    italic: bool,
    underline: bool,
    blink: bool,
    reverse: bool,
    strikethrough: bool,
}

impl Style {
    /// Create a new empty style.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the foreground color.
    pub fn foreground(mut self, color: Color) -> Self {
        self.foreground = Some(color);
        self
    }

    /// Set the background color.
    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Alias for foreground.
    pub fn fg(self, color: Color) -> Self {
        self.foreground(color)
    }

    /// Alias for background.
    pub fn bg(self, color: Color) -> Self {
        self.background(color)
    }

    /// Make the text bold.
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Make the text dim.
    pub fn dim(mut self) -> Self {
        self.dim = true;
        self
    }

    /// Make the text italic.
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// Make the text underlined.
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    /// Make the text blink.
    pub fn blink(mut self) -> Self {
        self.blink = true;
        self
    }

    /// Reverse foreground and background.
    pub fn reverse(mut self) -> Self {
        self.reverse = true;
        self
    }

    /// Make the text strikethrough.
    pub fn strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }

    /// Apply this style to a string.
    pub fn render(&self, text: &str) -> String {
        if self.is_empty() {
            return text.to_string();
        }

        let mut codes = Vec::new();

        // Attributes
        if self.bold {
            codes.push("1");
        }
        if self.dim {
            codes.push("2");
        }
        if self.italic {
            codes.push("3");
        }
        if self.underline {
            codes.push("4");
        }
        if self.blink {
            codes.push("5");
        }
        if self.reverse {
            codes.push("7");
        }
        if self.strikethrough {
            codes.push("9");
        }

        let mut result = String::new();

        // Start with attributes
        if !codes.is_empty() {
            result.push_str(&format!("\x1b[{}m", codes.join(";")));
        }

        // Apply colors separately for proper handling
        if let Some(ref fg) = self.foreground {
            result.push_str(&fg.to_ansi_fg());
        }
        if let Some(ref bg) = self.background {
            result.push_str(&bg.to_ansi_bg());
        }

        result.push_str(text);
        result.push_str("\x1b[0m"); // Reset

        result
    }

    /// Check if this style has no attributes.
    fn is_empty(&self) -> bool {
        self.foreground.is_none()
            && self.background.is_none()
            && !self.bold
            && !self.dim
            && !self.italic
            && !self.underline
            && !self.blink
            && !self.reverse
            && !self.strikethrough
    }
}

/// Create a styled string with foreground color.
pub fn colored(text: &str, color: Color) -> String {
    Style::new().fg(color).render(text)
}

/// Create a bold string.
pub fn bold(text: &str) -> String {
    Style::new().bold().render(text)
}

/// Create a dim string.
pub fn dim(text: &str) -> String {
    Style::new().dim().render(text)
}

/// Create an underlined string.
pub fn underline(text: &str) -> String {
    Style::new().underline().render(text)
}

// Convenience methods for common colors
impl Style {
    /// Red foreground.
    pub fn red() -> Self {
        Self::new().fg(Color::Red)
    }

    /// Green foreground.
    pub fn green() -> Self {
        Self::new().fg(Color::Green)
    }

    /// Yellow foreground.
    pub fn yellow() -> Self {
        Self::new().fg(Color::Yellow)
    }

    /// Blue foreground.
    pub fn blue() -> Self {
        Self::new().fg(Color::Blue)
    }

    /// Cyan foreground.
    pub fn cyan() -> Self {
        Self::new().fg(Color::Cyan)
    }

    /// Magenta foreground.
    pub fn magenta() -> Self {
        Self::new().fg(Color::Magenta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_style() {
        let style = Style::new();
        assert_eq!(style.render("text"), "text");
    }

    #[test]
    fn test_bold() {
        let result = Style::new().bold().render("text");
        assert!(result.starts_with("\x1b[1m"));
        assert!(result.ends_with("\x1b[0m"));
    }

    #[test]
    fn test_colored() {
        let result = colored("text", Color::Red);
        assert!(result.contains("\x1b[31m"));
    }

    #[test]
    fn test_combined() {
        let result = Style::new().bold().fg(Color::Green).render("text");
        assert!(result.contains("\x1b[1m"));
        assert!(result.contains("\x1b[32m"));
    }
}
