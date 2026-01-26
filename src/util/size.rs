//! Size and measurement utilities.

use unicode_width::UnicodeWidthStr;

/// Measure the display width of text.
pub fn measure_text(text: &str) -> usize {
    // Strip ANSI escape codes
    let stripped = crate::style::strip_ansi(text);
    UnicodeWidthStr::width(stripped.as_str())
}

/// Wrap text to a given width.
pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for word in text.split_whitespace() {
        let word_width = UnicodeWidthStr::width(word);

        if current_width == 0 {
            // First word on line
            current_line = word.to_string();
            current_width = word_width;
        } else if current_width + 1 + word_width <= width {
            // Word fits on current line
            current_line.push(' ');
            current_line.push_str(word);
            current_width += 1 + word_width;
        } else {
            // Start new line
            lines.push(current_line);
            current_line = word.to_string();
            current_width = word_width;
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measure_text() {
        assert_eq!(measure_text("hello"), 5);
        assert_eq!(measure_text("日本語"), 6);
        assert_eq!(measure_text("\x1b[31mred\x1b[0m"), 3);
    }

    #[test]
    fn test_wrap_text() {
        let wrapped = wrap_text("hello world foo bar", 10);
        assert_eq!(wrapped, vec!["hello", "world foo", "bar"]);
    }
}
