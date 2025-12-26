//! Border styles for boxes and containers.

/// Border style configuration.
#[derive(Debug, Clone, Default)]
pub struct Border {
    /// Border style to use.
    pub style: BorderStyle,
    /// Top border.
    pub top: bool,
    /// Bottom border.
    pub bottom: bool,
    /// Left border.
    pub left: bool,
    /// Right border.
    pub right: bool,
}

impl Border {
    /// Create a new border with all sides.
    pub fn all(style: BorderStyle) -> Self {
        Self {
            style,
            top: true,
            bottom: true,
            left: true,
            right: true,
        }
    }

    /// Create a border with no sides.
    pub fn none() -> Self {
        Self::default()
    }

    /// Create a border with only top and bottom.
    pub fn horizontal(style: BorderStyle) -> Self {
        Self {
            style,
            top: true,
            bottom: true,
            left: false,
            right: false,
        }
    }

    /// Create a border with only left and right.
    pub fn vertical(style: BorderStyle) -> Self {
        Self {
            style,
            top: false,
            bottom: false,
            left: true,
            right: true,
        }
    }

    /// Get the character set for this border.
    pub fn chars(&self) -> &BorderChars {
        self.style.chars()
    }
}

/// Predefined border styles.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BorderStyle {
    /// No border.
    #[default]
    None,
    /// ASCII border (+, -, |).
    Ascii,
    /// Single line border (─, │, ┌, ┐, └, ┘).
    Single,
    /// Double line border (═, ║, ╔, ╗, ╚, ╝).
    Double,
    /// Rounded border (─, │, ╭, ╮, ╰, ╯).
    Rounded,
    /// Heavy/thick border (━, ┃, ┏, ┓, ┗, ┛).
    Heavy,
    /// Block border using full blocks.
    Block,
}

/// Characters used for drawing borders.
#[derive(Debug, Clone)]
pub struct BorderChars {
    pub top: char,
    pub bottom: char,
    pub left: char,
    pub right: char,
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub middle_left: char,
    pub middle_right: char,
    pub middle_top: char,
    pub middle_bottom: char,
    pub cross: char,
}

impl BorderStyle {
    /// Get the character set for this style.
    pub fn chars(&self) -> &'static BorderChars {
        match self {
            BorderStyle::None => &NONE_CHARS,
            BorderStyle::Ascii => &ASCII_CHARS,
            BorderStyle::Single => &SINGLE_CHARS,
            BorderStyle::Double => &DOUBLE_CHARS,
            BorderStyle::Rounded => &ROUNDED_CHARS,
            BorderStyle::Heavy => &HEAVY_CHARS,
            BorderStyle::Block => &BLOCK_CHARS,
        }
    }
}

const NONE_CHARS: BorderChars = BorderChars {
    top: ' ',
    bottom: ' ',
    left: ' ',
    right: ' ',
    top_left: ' ',
    top_right: ' ',
    bottom_left: ' ',
    bottom_right: ' ',
    middle_left: ' ',
    middle_right: ' ',
    middle_top: ' ',
    middle_bottom: ' ',
    cross: ' ',
};

const ASCII_CHARS: BorderChars = BorderChars {
    top: '-',
    bottom: '-',
    left: '|',
    right: '|',
    top_left: '+',
    top_right: '+',
    bottom_left: '+',
    bottom_right: '+',
    middle_left: '+',
    middle_right: '+',
    middle_top: '+',
    middle_bottom: '+',
    cross: '+',
};

const SINGLE_CHARS: BorderChars = BorderChars {
    top: '─',
    bottom: '─',
    left: '│',
    right: '│',
    top_left: '┌',
    top_right: '┐',
    bottom_left: '└',
    bottom_right: '┘',
    middle_left: '├',
    middle_right: '┤',
    middle_top: '┬',
    middle_bottom: '┴',
    cross: '┼',
};

const DOUBLE_CHARS: BorderChars = BorderChars {
    top: '═',
    bottom: '═',
    left: '║',
    right: '║',
    top_left: '╔',
    top_right: '╗',
    bottom_left: '╚',
    bottom_right: '╝',
    middle_left: '╠',
    middle_right: '╣',
    middle_top: '╦',
    middle_bottom: '╩',
    cross: '╬',
};

const ROUNDED_CHARS: BorderChars = BorderChars {
    top: '─',
    bottom: '─',
    left: '│',
    right: '│',
    top_left: '╭',
    top_right: '╮',
    bottom_left: '╰',
    bottom_right: '╯',
    middle_left: '├',
    middle_right: '┤',
    middle_top: '┬',
    middle_bottom: '┴',
    cross: '┼',
};

const HEAVY_CHARS: BorderChars = BorderChars {
    top: '━',
    bottom: '━',
    left: '┃',
    right: '┃',
    top_left: '┏',
    top_right: '┓',
    bottom_left: '┗',
    bottom_right: '┛',
    middle_left: '┣',
    middle_right: '┫',
    middle_top: '┳',
    middle_bottom: '┻',
    cross: '╋',
};

const BLOCK_CHARS: BorderChars = BorderChars {
    top: '█',
    bottom: '█',
    left: '█',
    right: '█',
    top_left: '█',
    top_right: '█',
    bottom_left: '█',
    bottom_right: '█',
    middle_left: '█',
    middle_right: '█',
    middle_top: '█',
    middle_bottom: '█',
    cross: '█',
};

/// Render a string inside a box.
pub fn boxed(content: &str, style: BorderStyle, width: usize) -> String {
    let chars = style.chars();
    let inner_width = width.saturating_sub(2);

    let mut lines = Vec::new();

    // Top border
    lines.push(format!(
        "{}{}{}",
        chars.top_left,
        chars.top.to_string().repeat(inner_width),
        chars.top_right
    ));

    // Content lines
    for line in content.lines() {
        let padded = super::pad_right(line, inner_width);
        let truncated = super::truncate(&padded, inner_width);
        lines.push(format!("{}{}{}", chars.left, truncated, chars.right));
    }

    // Bottom border
    lines.push(format!(
        "{}{}{}",
        chars.bottom_left,
        chars.bottom.to_string().repeat(inner_width),
        chars.bottom_right
    ));

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boxed_ascii() {
        let result = boxed("hello", BorderStyle::Ascii, 10);
        assert!(result.contains("+--------+"));
        assert!(result.contains("|hello   |"));
    }

    #[test]
    fn test_boxed_single() {
        let result = boxed("test", BorderStyle::Single, 8);
        assert!(result.contains("┌──────┐"));
        assert!(result.contains("│test  │"));
        assert!(result.contains("└──────┘"));
    }

    #[test]
    fn test_boxed_rounded() {
        let result = boxed("hi", BorderStyle::Rounded, 6);
        assert!(result.contains("╭────╮"));
        assert!(result.contains("╰────╯"));
    }
}
