//! Comprehensive text styling with Lip Gloss-inspired API.
//!
//! Provides CSS-like styling with padding, margins, borders, dimensions, and alignment.

use super::{
    border::{Border, BorderStyle},
    color::Color,
    strip_ansi, width as str_width,
};

/// Position for alignment.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Position {
    /// Top or left alignment.
    #[default]
    Top,
    /// Center alignment.
    Center,
    /// Bottom or right alignment.
    Bottom,
}

impl Position {
    /// Left alias for Top.
    pub const LEFT: Position = Position::Top;
    /// Right alias for Bottom.
    pub const RIGHT: Position = Position::Bottom;
}

/// Spacing values (top, right, bottom, left).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Spacing {
    pub top: usize,
    pub right: usize,
    pub bottom: usize,
    pub left: usize,
}

impl Spacing {
    /// Create spacing with all sides equal.
    pub fn all(value: usize) -> Self {
        Self { top: value, right: value, bottom: value, left: value }
    }

    /// Create spacing from CSS-style shorthand.
    ///
    /// - 1 value: all sides
    /// - 2 values: vertical, horizontal
    /// - 3 values: top, horizontal, bottom
    /// - 4 values: top, right, bottom, left
    pub fn from_values(values: &[usize]) -> Self {
        match values.len() {
            0 => Self::default(),
            1 => Self::all(values[0]),
            2 => Self { top: values[0], right: values[1], bottom: values[0], left: values[1] },
            3 => Self { top: values[0], right: values[1], bottom: values[2], left: values[1] },
            _ => Self { top: values[0], right: values[1], bottom: values[2], left: values[3] },
        }
    }

    /// Get horizontal spacing (left + right).
    pub fn horizontal(&self) -> usize {
        self.left + self.right
    }

    /// Get vertical spacing (top + bottom).
    pub fn vertical(&self) -> usize {
        self.top + self.bottom
    }
}

/// Text style with colors, attributes, padding, margins, borders, and dimensions.
#[derive(Debug, Clone, Default)]
pub struct Style {
    // Colors
    foreground: Option<Color>,
    background: Option<Color>,

    // Text attributes
    bold: Option<bool>,
    dim: Option<bool>,
    italic: Option<bool>,
    underline: Option<bool>,
    blink: Option<bool>,
    reverse: Option<bool>,
    strikethrough: Option<bool>,

    // Spacing
    padding: Spacing,
    margin: Spacing,

    // Dimensions
    width: Option<usize>,
    height: Option<usize>,
    max_width: Option<usize>,
    max_height: Option<usize>,

    // Alignment
    align_horizontal: Position,
    align_vertical: Position,

    // Border
    border: Option<Border>,
    border_foreground: Option<Color>,
    border_background: Option<Color>,

    // Rendering options
    inline: bool,
}

impl Style {
    /// Create a new empty style.
    pub fn new() -> Self {
        Self::default()
    }

    // ========== Colors ==========

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

    /// Unset the foreground color.
    pub fn unset_foreground(mut self) -> Self {
        self.foreground = None;
        self
    }

    /// Unset the background color.
    pub fn unset_background(mut self) -> Self {
        self.background = None;
        self
    }

    // ========== Text Attributes ==========

    /// Set bold attribute.
    pub fn bold(mut self, v: bool) -> Self {
        self.bold = Some(v);
        self
    }

    /// Unset bold attribute.
    pub fn unset_bold(mut self) -> Self {
        self.bold = None;
        self
    }

    /// Set dim attribute.
    pub fn dim(mut self, v: bool) -> Self {
        self.dim = Some(v);
        self
    }

    /// Unset dim attribute.
    pub fn unset_dim(mut self) -> Self {
        self.dim = None;
        self
    }

    /// Set italic attribute.
    pub fn italic(mut self, v: bool) -> Self {
        self.italic = Some(v);
        self
    }

    /// Unset italic attribute.
    pub fn unset_italic(mut self) -> Self {
        self.italic = None;
        self
    }

    /// Set underline attribute.
    pub fn underline(mut self, v: bool) -> Self {
        self.underline = Some(v);
        self
    }

    /// Unset underline attribute.
    pub fn unset_underline(mut self) -> Self {
        self.underline = None;
        self
    }

    /// Set blink attribute.
    pub fn blink(mut self, v: bool) -> Self {
        self.blink = Some(v);
        self
    }

    /// Unset blink attribute.
    pub fn unset_blink(mut self) -> Self {
        self.blink = None;
        self
    }

    /// Set reverse attribute.
    pub fn reverse(mut self, v: bool) -> Self {
        self.reverse = Some(v);
        self
    }

    /// Unset reverse attribute.
    pub fn unset_reverse(mut self) -> Self {
        self.reverse = None;
        self
    }

    /// Set strikethrough attribute.
    pub fn strikethrough(mut self, v: bool) -> Self {
        self.strikethrough = Some(v);
        self
    }

    /// Unset strikethrough attribute.
    pub fn unset_strikethrough(mut self) -> Self {
        self.strikethrough = None;
        self
    }

    // ========== Padding ==========

    /// Set padding using CSS-style shorthand.
    ///
    /// - 1 value: all sides
    /// - 2 values: vertical, horizontal
    /// - 3 values: top, horizontal, bottom
    /// - 4 values: top, right, bottom, left
    pub fn padding(mut self, values: &[usize]) -> Self {
        self.padding = Spacing::from_values(values);
        self
    }

    /// Set top padding.
    pub fn padding_top(mut self, v: usize) -> Self {
        self.padding.top = v;
        self
    }

    /// Set right padding.
    pub fn padding_right(mut self, v: usize) -> Self {
        self.padding.right = v;
        self
    }

    /// Set bottom padding.
    pub fn padding_bottom(mut self, v: usize) -> Self {
        self.padding.bottom = v;
        self
    }

    /// Set left padding.
    pub fn padding_left(mut self, v: usize) -> Self {
        self.padding.left = v;
        self
    }

    /// Unset all padding.
    pub fn unset_padding(mut self) -> Self {
        self.padding = Spacing::default();
        self
    }

    /// Get padding values.
    pub fn get_padding(&self) -> Spacing {
        self.padding
    }

    /// Get horizontal padding (left + right).
    pub fn get_horizontal_padding(&self) -> usize {
        self.padding.horizontal()
    }

    /// Get vertical padding (top + bottom).
    pub fn get_vertical_padding(&self) -> usize {
        self.padding.vertical()
    }

    // ========== Margin ==========

    /// Set margin using CSS-style shorthand.
    ///
    /// - 1 value: all sides
    /// - 2 values: vertical, horizontal
    /// - 3 values: top, horizontal, bottom
    /// - 4 values: top, right, bottom, left
    pub fn margin(mut self, values: &[usize]) -> Self {
        self.margin = Spacing::from_values(values);
        self
    }

    /// Set top margin.
    pub fn margin_top(mut self, v: usize) -> Self {
        self.margin.top = v;
        self
    }

    /// Set right margin.
    pub fn margin_right(mut self, v: usize) -> Self {
        self.margin.right = v;
        self
    }

    /// Set bottom margin.
    pub fn margin_bottom(mut self, v: usize) -> Self {
        self.margin.bottom = v;
        self
    }

    /// Set left margin.
    pub fn margin_left(mut self, v: usize) -> Self {
        self.margin.left = v;
        self
    }

    /// Unset all margin.
    pub fn unset_margin(mut self) -> Self {
        self.margin = Spacing::default();
        self
    }

    /// Get margin values.
    pub fn get_margin(&self) -> Spacing {
        self.margin
    }

    /// Get horizontal margin (left + right).
    pub fn get_horizontal_margin(&self) -> usize {
        self.margin.horizontal()
    }

    /// Get vertical margin (top + bottom).
    pub fn get_vertical_margin(&self) -> usize {
        self.margin.vertical()
    }

    // ========== Dimensions ==========

    /// Set the width (content will be padded/truncated to fit).
    pub fn width(mut self, w: usize) -> Self {
        self.width = Some(w);
        self
    }

    /// Set the height (content will be padded/truncated to fit).
    pub fn height(mut self, h: usize) -> Self {
        self.height = Some(h);
        self
    }

    /// Set the maximum width.
    pub fn max_width(mut self, w: usize) -> Self {
        self.max_width = Some(w);
        self
    }

    /// Set the maximum height.
    pub fn max_height(mut self, h: usize) -> Self {
        self.max_height = Some(h);
        self
    }

    /// Unset width.
    pub fn unset_width(mut self) -> Self {
        self.width = None;
        self
    }

    /// Unset height.
    pub fn unset_height(mut self) -> Self {
        self.height = None;
        self
    }

    /// Get width.
    pub fn get_width(&self) -> Option<usize> {
        self.width
    }

    /// Get height.
    pub fn get_height(&self) -> Option<usize> {
        self.height
    }

    /// Get max width.
    pub fn get_max_width(&self) -> Option<usize> {
        self.max_width
    }

    /// Get max height.
    pub fn get_max_height(&self) -> Option<usize> {
        self.max_height
    }

    // ========== Alignment ==========

    /// Set horizontal and vertical alignment.
    pub fn align(mut self, h: Position, v: Position) -> Self {
        self.align_horizontal = h;
        self.align_vertical = v;
        self
    }

    /// Set horizontal alignment.
    pub fn align_horizontal(mut self, pos: Position) -> Self {
        self.align_horizontal = pos;
        self
    }

    /// Set vertical alignment.
    pub fn align_vertical(mut self, pos: Position) -> Self {
        self.align_vertical = pos;
        self
    }

    /// Get horizontal alignment.
    pub fn get_align_horizontal(&self) -> Position {
        self.align_horizontal
    }

    /// Get vertical alignment.
    pub fn get_align_vertical(&self) -> Position {
        self.align_vertical
    }

    // ========== Border ==========

    /// Set the border.
    pub fn border(mut self, style: BorderStyle) -> Self {
        self.border = Some(Border::all(style));
        self
    }

    /// Set a custom border configuration.
    pub fn border_custom(mut self, border: Border) -> Self {
        self.border = Some(border);
        self
    }

    /// Set border foreground color.
    pub fn border_foreground(mut self, color: Color) -> Self {
        self.border_foreground = Some(color);
        self
    }

    /// Set border background color.
    pub fn border_background(mut self, color: Color) -> Self {
        self.border_background = Some(color);
        self
    }

    /// Unset border.
    pub fn unset_border(mut self) -> Self {
        self.border = None;
        self
    }

    /// Get border.
    pub fn get_border(&self) -> Option<&Border> {
        self.border.as_ref()
    }

    // ========== Rendering Options ==========

    /// Set inline rendering (ignores margins and padding height).
    pub fn inline(mut self, v: bool) -> Self {
        self.inline = v;
        self
    }

    /// Get inline setting.
    pub fn get_inline(&self) -> bool {
        self.inline
    }

    // ========== Frame Size ==========

    /// Get horizontal frame size (padding + border + margin).
    pub fn get_horizontal_frame_size(&self) -> usize {
        let border = if self.border.is_some() { 2 } else { 0 };
        self.padding.horizontal() + self.margin.horizontal() + border
    }

    /// Get vertical frame size (padding + border + margin).
    pub fn get_vertical_frame_size(&self) -> usize {
        let border = if self.border.is_some() { 2 } else { 0 };
        self.padding.vertical() + self.margin.vertical() + border
    }

    /// Get total frame size (horizontal, vertical).
    pub fn get_frame_size(&self) -> (usize, usize) {
        (self.get_horizontal_frame_size(), self.get_vertical_frame_size())
    }

    // ========== Inheritance ==========

    /// Inherit unset properties from another style.
    ///
    /// Only properties that are not set on this style will be inherited.
    /// Margins and padding are NOT inherited.
    pub fn inherit(mut self, parent: &Style) -> Self {
        if self.foreground.is_none() {
            self.foreground = parent.foreground.clone();
        }
        if self.background.is_none() {
            self.background = parent.background.clone();
        }
        if self.bold.is_none() {
            self.bold = parent.bold;
        }
        if self.dim.is_none() {
            self.dim = parent.dim;
        }
        if self.italic.is_none() {
            self.italic = parent.italic;
        }
        if self.underline.is_none() {
            self.underline = parent.underline;
        }
        if self.blink.is_none() {
            self.blink = parent.blink;
        }
        if self.reverse.is_none() {
            self.reverse = parent.reverse;
        }
        if self.strikethrough.is_none() {
            self.strikethrough = parent.strikethrough;
        }
        self
    }

    // ========== Rendering ==========

    /// Apply this style to a string.
    pub fn render(&self, text: &str) -> String {
        if self.inline {
            return self.render_inline(text);
        }

        let mut result = text.to_string();

        // Apply text styling
        result = self.apply_text_style(&result);

        // Apply padding
        result = self.apply_padding(&result);

        // Apply dimensions
        result = self.apply_dimensions(&result);

        // Apply border
        result = self.apply_border(&result);

        // Apply margin
        result = self.apply_margin(&result);

        // Apply max constraints
        result = self.apply_max_constraints(&result);

        result
    }

    fn render_inline(&self, text: &str) -> String {
        self.apply_text_style(text)
    }

    fn apply_text_style(&self, text: &str) -> String {
        let mut codes = Vec::new();

        if self.bold == Some(true) {
            codes.push("1");
        }
        if self.dim == Some(true) {
            codes.push("2");
        }
        if self.italic == Some(true) {
            codes.push("3");
        }
        if self.underline == Some(true) {
            codes.push("4");
        }
        if self.blink == Some(true) {
            codes.push("5");
        }
        if self.reverse == Some(true) {
            codes.push("7");
        }
        if self.strikethrough == Some(true) {
            codes.push("9");
        }

        if codes.is_empty() && self.foreground.is_none() && self.background.is_none() {
            return text.to_string();
        }

        let mut result = String::new();

        if !codes.is_empty() {
            result.push_str(&format!("\x1b[{}m", codes.join(";")));
        }

        if let Some(ref fg) = self.foreground {
            result.push_str(&fg.to_ansi_fg());
        }
        if let Some(ref bg) = self.background {
            result.push_str(&bg.to_ansi_bg());
        }

        result.push_str(text);
        result.push_str("\x1b[0m");

        result
    }

    fn apply_padding(&self, text: &str) -> String {
        if self.padding == Spacing::default() {
            return text.to_string();
        }

        let lines: Vec<&str> = text.lines().collect();
        let max_width = lines.iter().map(|l| str_width(l)).max().unwrap_or(0);
        let padded_width = max_width + self.padding.left + self.padding.right;

        let mut result = Vec::new();

        // Top padding
        for _ in 0..self.padding.top {
            result.push(" ".repeat(padded_width));
        }

        // Content with left/right padding
        for line in &lines {
            let line_width = str_width(line);
            let right_pad = padded_width - self.padding.left - line_width;
            result.push(format!(
                "{}{}{}",
                " ".repeat(self.padding.left),
                line,
                " ".repeat(right_pad)
            ));
        }

        // Bottom padding
        for _ in 0..self.padding.bottom {
            result.push(" ".repeat(padded_width));
        }

        result.join("\n")
    }

    fn apply_dimensions(&self, text: &str) -> String {
        let mut lines: Vec<String> = text.lines().map(String::from).collect();
        let _current_width = lines.iter().map(|l| str_width(l)).max().unwrap_or(0);

        // Apply width
        if let Some(target_width) = self.width {
            for line in &mut lines {
                let line_width = str_width(line);
                if line_width < target_width {
                    // Align content
                    let padding = target_width - line_width;
                    match self.align_horizontal {
                        Position::Top => {
                            // Left align
                            line.push_str(&" ".repeat(padding));
                        },
                        Position::Center => {
                            let left = padding / 2;
                            let right = padding - left;
                            *line = format!("{}{}{}", " ".repeat(left), line, " ".repeat(right));
                        },
                        Position::Bottom => {
                            // Right align
                            *line = format!("{}{}", " ".repeat(padding), line);
                        },
                    }
                } else if line_width > target_width {
                    *line = truncate_to_width(line, target_width);
                }
            }
        }

        // Apply height
        if let Some(target_height) = self.height {
            let content_width = lines.iter().map(|l| str_width(l)).max().unwrap_or(0);
            let current_height = lines.len();

            if current_height < target_height {
                let padding = target_height - current_height;
                let empty_line = " ".repeat(content_width);

                match self.align_vertical {
                    Position::Top => {
                        for _ in 0..padding {
                            lines.push(empty_line.clone());
                        }
                    },
                    Position::Center => {
                        let top = padding / 2;
                        let bottom = padding - top;
                        let mut new_lines = Vec::new();
                        for _ in 0..top {
                            new_lines.push(empty_line.clone());
                        }
                        new_lines.extend(lines);
                        for _ in 0..bottom {
                            new_lines.push(empty_line.clone());
                        }
                        lines = new_lines;
                    },
                    Position::Bottom => {
                        let mut new_lines = Vec::new();
                        for _ in 0..padding {
                            new_lines.push(empty_line.clone());
                        }
                        new_lines.extend(lines);
                        lines = new_lines;
                    },
                }
            } else if current_height > target_height {
                lines.truncate(target_height);
            }
        }

        lines.join("\n")
    }

    fn apply_border(&self, text: &str) -> String {
        let border = match &self.border {
            Some(b) => b,
            None => return text.to_string(),
        };

        let chars = border.chars();
        let lines: Vec<&str> = text.lines().collect();
        let content_width = lines.iter().map(|l| str_width(l)).max().unwrap_or(0);

        let mut result = Vec::new();

        // Build border style codes
        let border_style_start = self.get_border_style_start();
        let border_style_end =
            if border_style_start.is_empty() { String::new() } else { "\x1b[0m".to_string() };

        // Top border
        if border.top {
            let line = format!(
                "{}{}{}{}{}",
                border_style_start,
                if border.left { chars.top_left.to_string() } else { String::new() },
                chars.top.to_string().repeat(content_width),
                if border.right { chars.top_right.to_string() } else { String::new() },
                border_style_end
            );
            result.push(line);
        }

        // Content with side borders
        for content_line in &lines {
            let padded =
                format!("{}{}", content_line, " ".repeat(content_width - str_width(content_line)));
            let line = format!(
                "{}{}{}{}{}{}{}",
                if border.left { &border_style_start } else { "" },
                if border.left { chars.left.to_string() } else { String::new() },
                if border.left { &border_style_end } else { "" },
                padded,
                if border.right { &border_style_start } else { "" },
                if border.right { chars.right.to_string() } else { String::new() },
                if border.right { &border_style_end } else { "" },
            );
            result.push(line);
        }

        // Bottom border
        if border.bottom {
            let line = format!(
                "{}{}{}{}{}",
                border_style_start,
                if border.left { chars.bottom_left.to_string() } else { String::new() },
                chars.bottom.to_string().repeat(content_width),
                if border.right { chars.bottom_right.to_string() } else { String::new() },
                border_style_end
            );
            result.push(line);
        }

        result.join("\n")
    }

    fn get_border_style_start(&self) -> String {
        let mut codes = String::new();
        if let Some(ref fg) = self.border_foreground {
            codes.push_str(&fg.to_ansi_fg());
        }
        if let Some(ref bg) = self.border_background {
            codes.push_str(&bg.to_ansi_bg());
        }
        codes
    }

    fn apply_margin(&self, text: &str) -> String {
        if self.margin == Spacing::default() {
            return text.to_string();
        }

        let lines: Vec<&str> = text.lines().collect();
        let max_width = lines.iter().map(|l| str_width(l)).max().unwrap_or(0);
        let _margined_width = max_width + self.margin.left + self.margin.right;

        let mut result = Vec::new();

        // Top margin
        for _ in 0..self.margin.top {
            result.push(String::new());
        }

        // Content with left/right margin
        for line in &lines {
            result.push(format!("{}{}", " ".repeat(self.margin.left), line,));
        }

        // Bottom margin
        for _ in 0..self.margin.bottom {
            result.push(String::new());
        }

        result.join("\n")
    }

    fn apply_max_constraints(&self, text: &str) -> String {
        let mut lines: Vec<String> = text.lines().map(String::from).collect();

        // Apply max width
        if let Some(max_w) = self.max_width {
            for line in &mut lines {
                if str_width(line) > max_w {
                    *line = truncate_to_width(line, max_w);
                }
            }
        }

        // Apply max height
        if let Some(max_h) = self.max_height
            && lines.len() > max_h
        {
            lines.truncate(max_h);
        }

        lines.join("\n")
    }
}

/// Truncate a string to a specific display width.
fn truncate_to_width(s: &str, max_width: usize) -> String {
    let stripped = strip_ansi(s);
    if str_width(&stripped) <= max_width {
        return s.to_string();
    }

    if max_width == 0 {
        return String::new();
    }

    let mut result = String::new();
    let mut current_width = 0;

    for c in stripped.chars() {
        let char_width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if current_width + char_width > max_width {
            break;
        }
        result.push(c);
        current_width += char_width;
    }

    result
}

// ========== Convenience Color Constructors ==========

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

// ========== Standalone Functions ==========

/// Create a styled string with foreground color.
pub fn colored(text: &str, color: Color) -> String {
    Style::new().fg(color).render(text)
}

/// Create a bold string.
pub fn bold(text: &str) -> String {
    Style::new().bold(true).render(text)
}

/// Create a dim string.
pub fn dim(text: &str) -> String {
    Style::new().dim(true).render(text)
}

/// Create an underlined string.
pub fn underline(text: &str) -> String {
    Style::new().underline(true).render(text)
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
        let result = Style::new().bold(true).render("text");
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
        let result = Style::new().bold(true).fg(Color::Green).render("text");
        assert!(result.contains("\x1b[1m"));
        assert!(result.contains("\x1b[32m"));
    }

    #[test]
    fn test_padding_shorthand() {
        let style = Style::new().padding(&[2]);
        assert_eq!(style.padding.top, 2);
        assert_eq!(style.padding.right, 2);
        assert_eq!(style.padding.bottom, 2);
        assert_eq!(style.padding.left, 2);

        let style = Style::new().padding(&[1, 2]);
        assert_eq!(style.padding.top, 1);
        assert_eq!(style.padding.right, 2);
        assert_eq!(style.padding.bottom, 1);
        assert_eq!(style.padding.left, 2);

        let style = Style::new().padding(&[1, 2, 3, 4]);
        assert_eq!(style.padding.top, 1);
        assert_eq!(style.padding.right, 2);
        assert_eq!(style.padding.bottom, 3);
        assert_eq!(style.padding.left, 4);
    }

    #[test]
    fn test_margin_shorthand() {
        let style = Style::new().margin(&[2]);
        assert_eq!(style.margin.top, 2);
        assert_eq!(style.margin.right, 2);
    }

    #[test]
    fn test_dimensions() {
        let style = Style::new().width(10);
        let result = style.render("hi");
        assert_eq!(str_width(&result), 10);
    }

    #[test]
    fn test_inherit() {
        let parent = Style::new().fg(Color::Red).bold(true);
        let child = Style::new().italic(true).inherit(&parent);

        assert!(child.foreground.is_some());
        assert_eq!(child.bold, Some(true));
        assert_eq!(child.italic, Some(true));
    }

    #[test]
    fn test_unset() {
        let style = Style::new().bold(true).unset_bold();
        assert!(style.bold.is_none());
    }

    #[test]
    fn test_alignment() {
        let style = Style::new().width(10).align_horizontal(Position::Center);
        let result = style.render("hi");
        // "hi" should be centered in 10 chars: "    hi    "
        assert!(result.starts_with("    "));
    }

    #[test]
    fn test_border() {
        let style = Style::new().border(BorderStyle::Single);
        let result = style.render("hi");
        assert!(result.contains("┌"));
        assert!(result.contains("└"));
    }

    #[test]
    fn test_frame_size() {
        let style = Style::new().padding(&[1, 2]).margin(&[3, 4]).border(BorderStyle::Single);

        // padding: top=1, right=2, bottom=1, left=2 -> h=4, v=2
        // margin: top=3, right=4, bottom=3, left=4 -> h=8, v=6
        // border: h=2, v=2
        assert_eq!(style.get_horizontal_frame_size(), 4 + 8 + 2);
        assert_eq!(style.get_vertical_frame_size(), 2 + 6 + 2);
    }
}
