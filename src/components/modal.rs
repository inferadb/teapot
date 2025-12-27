//! Modal overlay component.
//!
//! A centered modal dialog that overlays on top of existing content.
//! Useful for error messages, confirmations, and other focused interactions.
//!
//! # Example
//!
//! ```rust,ignore
//! use ferment::components::Modal;
//!
//! let modal = Modal::new(60, 10)
//!     .title("Error")
//!     .content("Something went wrong.\nPlease try again.")
//!     .footer_hint("esc", "close");
//!
//! // Render the modal centered on a terminal of given size
//! let output = modal.render_overlay(80, 24, &background_content);
//! ```

use crate::style::Color;
use crate::util::measure_text;

/// Border style for the modal.
#[derive(Debug, Clone, Copy, Default)]
pub enum ModalBorder {
    /// Single line border (─│┌┐└┘).
    #[default]
    Single,
    /// Double line border (═║╔╗╚╝).
    Double,
    /// Rounded corners (─│╭╮╰╯).
    Rounded,
    /// ASCII border (-|+).
    Ascii,
    /// No border.
    None,
}

impl ModalBorder {
    /// Get the border characters (top, bottom, left, right, tl, tr, bl, br).
    fn chars(&self) -> (&str, &str, &str, &str, &str, &str, &str, &str) {
        match self {
            ModalBorder::Single => ("─", "─", "│", "│", "┌", "┐", "└", "┘"),
            ModalBorder::Double => ("═", "═", "║", "║", "╔", "╗", "╚", "╝"),
            ModalBorder::Rounded => ("─", "─", "│", "│", "╭", "╮", "╰", "╯"),
            ModalBorder::Ascii => ("-", "-", "|", "|", "+", "+", "+", "+"),
            ModalBorder::None => (" ", " ", " ", " ", " ", " ", " ", " "),
        }
    }
}

/// A footer hint for the modal.
#[derive(Debug, Clone)]
pub struct ModalHint {
    /// The keyboard shortcut.
    pub shortcut: String,
    /// Description of what it does.
    pub description: String,
}

impl ModalHint {
    /// Create a new hint.
    pub fn new(shortcut: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            shortcut: shortcut.into(),
            description: description.into(),
        }
    }
}

/// A modal dialog overlay.
#[derive(Debug, Clone)]
pub struct Modal {
    /// Modal width.
    width: usize,
    /// Modal height (including border).
    height: usize,
    /// Title text.
    title: String,
    /// Title fill character.
    title_fill: char,
    /// Content lines.
    content: Vec<String>,
    /// Footer hints.
    hints: Vec<ModalHint>,
    /// Border style.
    border: ModalBorder,
    /// Border color.
    border_color: Color,
    /// Title color.
    title_color: Color,
    /// Content color.
    content_color: Color,
    /// Hint key color.
    hint_key_color: Color,
    /// Hint description color.
    hint_desc_color: Color,
    /// Horizontal padding (applied to left and right of content).
    padding: usize,
}

impl Modal {
    /// Create a new modal with given dimensions.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            title: String::new(),
            title_fill: '/',
            content: Vec::new(),
            hints: Vec::new(),
            border: ModalBorder::default(),
            border_color: Color::BrightBlack,
            title_color: Color::Default,
            content_color: Color::Default,
            hint_key_color: Color::Default,
            hint_desc_color: Color::BrightBlack,
            padding: 1,
        }
    }

    /// Set horizontal padding (applied to left and right of content).
    pub fn padding(mut self, padding: usize) -> Self {
        self.padding = padding;
        self
    }

    /// Set the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the title fill character (default: '/').
    pub fn title_fill(mut self, c: char) -> Self {
        self.title_fill = c;
        self
    }

    /// Set the content as a single string (will be split by newlines).
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into().lines().map(|s| s.to_string()).collect();
        self
    }

    /// Set the content as multiple lines.
    pub fn content_lines(mut self, lines: Vec<String>) -> Self {
        self.content = lines;
        self
    }

    /// Add a footer hint.
    pub fn footer_hint(
        mut self,
        shortcut: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        self.hints.push(ModalHint::new(shortcut, description));
        self
    }

    /// Set multiple footer hints.
    pub fn footer_hints(mut self, hints: Vec<(&str, &str)>) -> Self {
        self.hints = hints
            .into_iter()
            .map(|(s, d)| ModalHint::new(s, d))
            .collect();
        self
    }

    /// Set the border style.
    pub fn border(mut self, border: ModalBorder) -> Self {
        self.border = border;
        self
    }

    /// Set the border color.
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    /// Set the title color.
    pub fn title_color(mut self, color: Color) -> Self {
        self.title_color = color;
        self
    }

    /// Set the content color.
    pub fn content_color(mut self, color: Color) -> Self {
        self.content_color = color;
        self
    }

    /// Get the inner width (excluding borders).
    fn inner_width(&self) -> usize {
        self.width.saturating_sub(2) // left + right border
    }

    /// Get the inner height (excluding borders and title/footer).
    fn inner_height(&self) -> usize {
        // total - top border - title - bottom border - footer (if hints)
        let footer_lines = if self.hints.is_empty() { 0 } else { 2 }; // blank + hints
        self.height.saturating_sub(3 + footer_lines)
    }

    /// Render the modal as a vector of lines.
    pub fn render_lines(&self) -> Vec<String> {
        let (h, _, left, right, tl, tr, bl, br) = self.border.chars();
        let inner_w = self.inner_width();
        let reset = "\x1b[0m";
        let border_fg = self.border_color.to_ansi_fg();
        let title_fg = self.title_color.to_ansi_fg();
        let content_fg = self.content_color.to_ansi_fg();

        let mut lines = Vec::new();

        // Top border
        lines.push(format!(
            "{}{}{}{}{}",
            border_fg,
            tl,
            h.repeat(inner_w),
            tr,
            reset
        ));

        // Title line (if title is set)
        if !self.title.is_empty() {
            // Format: (pad) // Title //////// (pad)
            let fill_char = self.title_fill.to_string();
            let title_w = inner_w.saturating_sub(self.padding * 2);
            let title_content = format!(
                "{}{} {} {}{}",
                fill_char,
                fill_char,
                self.title,
                fill_char.repeat(title_w.saturating_sub(self.title.len() + 5)),
                fill_char
            );
            let truncated = if title_content.len() > title_w {
                title_content[..title_w].to_string()
            } else {
                title_content
            };
            let left_pad = " ".repeat(self.padding);
            let right_pad = " ".repeat(self.padding);
            lines.push(format!(
                "{}{}{}{}{}{}{}{}{}",
                border_fg, left, reset, left_pad, title_fg, truncated, right_pad, border_fg, right
            ));

            // Blank line after title
            lines.push(format!(
                "{}{}{}{}{}",
                border_fg,
                left,
                " ".repeat(inner_w),
                right,
                reset
            ));
        }

        // Content lines (with horizontal padding)
        let content_start = lines.len();
        let content_w = inner_w.saturating_sub(self.padding * 2);
        let left_pad = " ".repeat(self.padding);
        for line in &self.content {
            let display_line = if measure_text(line) > content_w {
                // Truncate if too long
                let mut truncated = String::new();
                let mut width = 0;
                for c in line.chars() {
                    let cw = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
                    if width + cw > content_w.saturating_sub(1) {
                        truncated.push('…');
                        break;
                    }
                    truncated.push(c);
                    width += cw;
                }
                truncated
            } else {
                line.clone()
            };

            let right_pad = inner_w.saturating_sub(self.padding + measure_text(&display_line));
            lines.push(format!(
                "{}{}{}{}{}{}{}{}{}",
                border_fg,
                left,
                reset,
                left_pad,
                content_fg,
                display_line,
                " ".repeat(right_pad),
                border_fg,
                right
            ));
        }

        // Fill remaining content area with blank lines
        let content_lines_used = lines.len() - content_start;
        let remaining = self.inner_height().saturating_sub(content_lines_used);
        for _ in 0..remaining {
            lines.push(format!(
                "{}{}{}{}{}",
                border_fg,
                left,
                " ".repeat(inner_w),
                right,
                reset
            ));
        }

        // Footer hints
        if !self.hints.is_empty() {
            // Blank line before hints
            lines.push(format!(
                "{}{}{}{}{}",
                border_fg,
                left,
                " ".repeat(inner_w),
                right,
                reset
            ));

            // Hints line (right-aligned)
            let hint_key_fg = self.hint_key_color.to_ansi_fg();
            let hint_desc_fg = self.hint_desc_color.to_ansi_fg();

            let mut hint_parts = Vec::new();
            let mut hint_len = 0;
            for (i, hint) in self.hints.iter().enumerate() {
                if i > 0 {
                    hint_parts.push(format!("{}  ", reset));
                    hint_len += 2;
                }
                hint_parts.push(format!(
                    "{}{}{} {}{}",
                    hint_key_fg, hint.shortcut, reset, hint_desc_fg, hint.description
                ));
                hint_len += hint.shortcut.len() + 1 + hint.description.len();
            }
            let hints_str: String = hint_parts.concat();
            // Right-align hints with padding on both sides
            let left_fill = inner_w.saturating_sub(hint_len + self.padding);

            lines.push(format!(
                "{}{}{}{}{}{}{}{}",
                border_fg,
                left,
                " ".repeat(left_fill),
                hints_str,
                reset,
                " ".repeat(self.padding),
                border_fg,
                right
            ));
        }

        // Bottom border
        lines.push(format!(
            "{}{}{}{}{}",
            border_fg,
            bl,
            h.repeat(inner_w),
            br,
            reset
        ));

        lines
    }

    /// Render the modal as a string with \r\n line endings.
    pub fn render(&self) -> String {
        self.render_lines().join("\r\n")
    }

    /// Render the modal overlaid on background content.
    ///
    /// The modal is centered on a terminal of `term_width` x `term_height`.
    /// The background content is shown around the modal.
    pub fn render_overlay(
        &self,
        term_width: usize,
        term_height: usize,
        background: &str,
    ) -> String {
        let modal_lines = self.render_lines();
        let modal_height = modal_lines.len();
        let modal_width = self.width;

        // Calculate centering offsets
        let start_row = term_height.saturating_sub(modal_height) / 2;
        let start_col = term_width.saturating_sub(modal_width) / 2;

        // Split background into lines
        let bg_lines: Vec<&str> = background.lines().collect();

        let mut output = String::new();

        for row in 0..term_height {
            let line = if row >= start_row && row < start_row + modal_height {
                // This row contains part of the modal
                let modal_row = row - start_row;
                let modal_line = &modal_lines[modal_row];

                // Get background line
                let bg_line = bg_lines.get(row).copied().unwrap_or("");

                // Compose: bg prefix + modal + bg suffix
                compose_overlay_line(bg_line, modal_line, start_col, modal_width, term_width)
            } else {
                // Pure background line
                let bg_line = bg_lines.get(row).copied().unwrap_or("");
                pad_to_width(bg_line, term_width)
            };

            output.push_str(&line);
            if row < term_height - 1 {
                output.push_str("\r\n");
            }
        }

        output
    }
}

/// Compose a line with modal overlay on background.
fn compose_overlay_line(
    _bg_line: &str, // TODO: Could blend background with modal edges
    modal_line: &str,
    start_col: usize,
    modal_width: usize,
    term_width: usize,
) -> String {
    // For simplicity, we'll extract visual portions of the background
    // This is tricky with ANSI codes, so we'll use a simpler approach:
    // Just take the modal line and pad it to position

    let reset = "\x1b[0m";
    let mut result = String::new();

    // Left padding (could show background here, but ANSI makes it complex)
    result.push_str(&" ".repeat(start_col));

    // Modal content
    result.push_str(modal_line);
    result.push_str(reset);

    // Right padding
    let right_start = start_col + modal_width;
    if right_start < term_width {
        result.push_str(&" ".repeat(term_width - right_start));
    }

    result
}

/// Pad a line to a given width.
fn pad_to_width(line: &str, width: usize) -> String {
    let line_width = measure_text(line);
    if line_width >= width {
        line.to_string()
    } else {
        format!("{}{}", line, " ".repeat(width - line_width))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modal_creation() {
        let modal = Modal::new(40, 10).title("Test").content("Hello world");

        assert_eq!(modal.width, 40);
        assert_eq!(modal.height, 10);
        assert_eq!(modal.title, "Test");
    }

    #[test]
    fn test_modal_render() {
        let modal = Modal::new(40, 8)
            .title("Error")
            .content("Something went wrong")
            .footer_hint("esc", "close");

        let output = modal.render();
        assert!(output.contains("Error"));
        assert!(output.contains("Something went wrong"));
        assert!(output.contains("esc"));
        assert!(output.contains("close"));
    }

    #[test]
    fn test_modal_multiline_content() {
        let modal = Modal::new(40, 10)
            .title("Info")
            .content("Line 1\nLine 2\nLine 3");

        let lines = modal.render_lines();
        // Should have content spread across multiple lines
        let content_text: String = lines.join("");
        assert!(content_text.contains("Line 1"));
        assert!(content_text.contains("Line 2"));
        assert!(content_text.contains("Line 3"));
    }

    #[test]
    fn test_modal_border_styles() {
        let single = Modal::new(20, 5).border(ModalBorder::Single);
        let double = Modal::new(20, 5).border(ModalBorder::Double);
        let rounded = Modal::new(20, 5).border(ModalBorder::Rounded);

        let single_out = single.render();
        let double_out = double.render();
        let rounded_out = rounded.render();

        assert!(single_out.contains("┌"));
        assert!(double_out.contains("╔"));
        assert!(rounded_out.contains("╭"));
    }
}
