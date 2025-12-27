//! Color definitions with adaptive color support.
//!
//! Supports ANSI 16, 256, true color, and adaptive colors for light/dark terminals.

use std::env;

/// A terminal color.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Color {
    /// Use the terminal's default color.
    #[default]
    Default,
    /// Black (ANSI 0).
    Black,
    /// Red (ANSI 1).
    Red,
    /// Green (ANSI 2).
    Green,
    /// Yellow (ANSI 3).
    Yellow,
    /// Blue (ANSI 4).
    Blue,
    /// Magenta (ANSI 5).
    Magenta,
    /// Cyan (ANSI 6).
    Cyan,
    /// White (ANSI 7).
    White,
    /// Bright black (ANSI 8).
    BrightBlack,
    /// Bright red (ANSI 9).
    BrightRed,
    /// Bright green (ANSI 10).
    BrightGreen,
    /// Bright yellow (ANSI 11).
    BrightYellow,
    /// Bright blue (ANSI 12).
    BrightBlue,
    /// Bright magenta (ANSI 13).
    BrightMagenta,
    /// Bright cyan (ANSI 14).
    BrightCyan,
    /// Bright white (ANSI 15).
    BrightWhite,
    /// ANSI 256 color by index.
    Ansi256(u8),
    /// True color RGB.
    Rgb(u8, u8, u8),
    /// Adaptive color that changes based on terminal background.
    Adaptive {
        /// Color to use on light backgrounds.
        light: Box<Color>,
        /// Color to use on dark backgrounds.
        dark: Box<Color>,
    },
    /// Complete color with explicit values for each color profile.
    Complete {
        /// True color (24-bit) value.
        true_color: (u8, u8, u8),
        /// ANSI 256 (8-bit) value.
        ansi256: u8,
        /// ANSI 16 (4-bit) value.
        ansi: u8,
    },
}

impl Color {
    /// Create an adaptive color that changes based on terminal background.
    ///
    /// # Example
    /// ```
    /// use ferment::style::Color;
    ///
    /// let color = Color::adaptive(Color::Black, Color::White);
    /// ```
    pub fn adaptive(light: Color, dark: Color) -> Self {
        Color::Adaptive {
            light: Box::new(light),
            dark: Box::new(dark),
        }
    }

    /// Create a complete color with explicit values for each color profile.
    ///
    /// # Example
    /// ```
    /// use ferment::style::Color;
    ///
    /// let color = Color::complete((255, 0, 0), 196, 1);
    /// ```
    pub fn complete(true_color: (u8, u8, u8), ansi256: u8, ansi: u8) -> Self {
        Color::Complete {
            true_color,
            ansi256,
            ansi,
        }
    }

    /// Create a color from a hex string like "#FF5500" or "FF5500".
    pub fn hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

        Some(Color::Rgb(r, g, b))
    }

    /// Alias for `hex` to match Lip Gloss naming.
    pub fn from_hex(hex: &str) -> Option<Self> {
        Self::hex(hex)
    }

    /// Resolve adaptive colors to their concrete value based on terminal background.
    pub fn resolve(&self) -> &Color {
        match self {
            Color::Adaptive { light, dark } => {
                if has_dark_background() {
                    dark.resolve()
                } else {
                    light.resolve()
                }
            }
            _ => self,
        }
    }

    /// Get the ANSI escape sequence for this color as foreground.
    pub fn to_ansi_fg(&self) -> String {
        self.resolve().to_ansi_fg_inner()
    }

    fn to_ansi_fg_inner(&self) -> String {
        match self {
            Color::Default => "\x1b[39m".to_string(),
            Color::Black => "\x1b[30m".to_string(),
            Color::Red => "\x1b[31m".to_string(),
            Color::Green => "\x1b[32m".to_string(),
            Color::Yellow => "\x1b[33m".to_string(),
            Color::Blue => "\x1b[34m".to_string(),
            Color::Magenta => "\x1b[35m".to_string(),
            Color::Cyan => "\x1b[36m".to_string(),
            Color::White => "\x1b[37m".to_string(),
            Color::BrightBlack => "\x1b[90m".to_string(),
            Color::BrightRed => "\x1b[91m".to_string(),
            Color::BrightGreen => "\x1b[92m".to_string(),
            Color::BrightYellow => "\x1b[93m".to_string(),
            Color::BrightBlue => "\x1b[94m".to_string(),
            Color::BrightMagenta => "\x1b[95m".to_string(),
            Color::BrightCyan => "\x1b[96m".to_string(),
            Color::BrightWhite => "\x1b[97m".to_string(),
            Color::Ansi256(n) => format!("\x1b[38;5;{}m", n),
            Color::Rgb(r, g, b) => format!("\x1b[38;2;{};{};{}m", r, g, b),
            Color::Adaptive { dark, .. } => dark.to_ansi_fg_inner(),
            Color::Complete { true_color, .. } => {
                format!(
                    "\x1b[38;2;{};{};{}m",
                    true_color.0, true_color.1, true_color.2
                )
            }
        }
    }

    /// Get the ANSI escape sequence for this color as background.
    pub fn to_ansi_bg(&self) -> String {
        self.resolve().to_ansi_bg_inner()
    }

    fn to_ansi_bg_inner(&self) -> String {
        match self {
            Color::Default => "\x1b[49m".to_string(),
            Color::Black => "\x1b[40m".to_string(),
            Color::Red => "\x1b[41m".to_string(),
            Color::Green => "\x1b[42m".to_string(),
            Color::Yellow => "\x1b[43m".to_string(),
            Color::Blue => "\x1b[44m".to_string(),
            Color::Magenta => "\x1b[45m".to_string(),
            Color::Cyan => "\x1b[46m".to_string(),
            Color::White => "\x1b[47m".to_string(),
            Color::BrightBlack => "\x1b[100m".to_string(),
            Color::BrightRed => "\x1b[101m".to_string(),
            Color::BrightGreen => "\x1b[102m".to_string(),
            Color::BrightYellow => "\x1b[103m".to_string(),
            Color::BrightBlue => "\x1b[104m".to_string(),
            Color::BrightMagenta => "\x1b[105m".to_string(),
            Color::BrightCyan => "\x1b[106m".to_string(),
            Color::BrightWhite => "\x1b[107m".to_string(),
            Color::Ansi256(n) => format!("\x1b[48;5;{}m", n),
            Color::Rgb(r, g, b) => format!("\x1b[48;2;{};{};{}m", r, g, b),
            Color::Adaptive { dark, .. } => dark.to_ansi_bg_inner(),
            Color::Complete { true_color, .. } => {
                format!(
                    "\x1b[48;2;{};{};{}m",
                    true_color.0, true_color.1, true_color.2
                )
            }
        }
    }

    /// Convert to crossterm color.
    pub fn to_crossterm(&self) -> crossterm::style::Color {
        match self.resolve() {
            Color::Default => crossterm::style::Color::Reset,
            Color::Black => crossterm::style::Color::Black,
            Color::Red => crossterm::style::Color::DarkRed,
            Color::Green => crossterm::style::Color::DarkGreen,
            Color::Yellow => crossterm::style::Color::DarkYellow,
            Color::Blue => crossterm::style::Color::DarkBlue,
            Color::Magenta => crossterm::style::Color::DarkMagenta,
            Color::Cyan => crossterm::style::Color::DarkCyan,
            Color::White => crossterm::style::Color::Grey,
            Color::BrightBlack => crossterm::style::Color::DarkGrey,
            Color::BrightRed => crossterm::style::Color::Red,
            Color::BrightGreen => crossterm::style::Color::Green,
            Color::BrightYellow => crossterm::style::Color::Yellow,
            Color::BrightBlue => crossterm::style::Color::Blue,
            Color::BrightMagenta => crossterm::style::Color::Magenta,
            Color::BrightCyan => crossterm::style::Color::Cyan,
            Color::BrightWhite => crossterm::style::Color::White,
            Color::Ansi256(n) => crossterm::style::Color::AnsiValue(*n),
            Color::Rgb(r, g, b) => crossterm::style::Color::Rgb {
                r: *r,
                g: *g,
                b: *b,
            },
            Color::Complete { true_color, .. } => crossterm::style::Color::Rgb {
                r: true_color.0,
                g: true_color.1,
                b: true_color.2,
            },
            Color::Adaptive { .. } => crossterm::style::Color::Reset,
        }
    }
}

// TRON-inspired color palette for InferaDB
impl Color {
    /// TRON cyan - primary brand color.
    pub const TRON_CYAN: Color = Color::Rgb(0, 204, 255);

    /// TRON bright cyan - accents.
    pub const TRON_BRIGHT_CYAN: Color = Color::Rgb(102, 255, 255);

    /// TRON green - success states.
    pub const TRON_GREEN: Color = Color::Rgb(0, 255, 136);

    /// TRON red - error states.
    pub const TRON_RED: Color = Color::Rgb(255, 68, 68);

    /// TRON yellow - warning states.
    pub const TRON_YELLOW: Color = Color::Rgb(255, 204, 0);

    /// TRON magenta - special values.
    pub const TRON_MAGENTA: Color = Color::Rgb(204, 0, 204);

    /// TRON dim - secondary info.
    pub const TRON_DIM: Color = Color::BrightBlack;
}

/// Check if the terminal has a dark background.
///
/// This checks the `COLORFGBG` environment variable and defaults to true (dark).
pub fn has_dark_background() -> bool {
    // Check COLORFGBG environment variable (format: "fg;bg")
    if let Ok(val) = env::var("COLORFGBG") {
        if let Some(bg) = val.split(';').nth(1) {
            if let Ok(bg_num) = bg.parse::<u8>() {
                // Light backgrounds are typically 7 (white) or 15 (bright white)
                return bg_num != 7 && bg_num != 15;
            }
        }
    }

    // Default to dark background (most common)
    true
}

/// Get the terminal's color profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorProfile {
    /// No color support (1-bit).
    Ascii,
    /// ANSI 16 colors (4-bit).
    Ansi,
    /// ANSI 256 colors (8-bit).
    Ansi256,
    /// True color (24-bit).
    TrueColor,
}

impl ColorProfile {
    /// Detect the current terminal's color profile.
    pub fn detect() -> Self {
        // Check for NO_COLOR
        if env::var("NO_COLOR").is_ok() {
            return ColorProfile::Ascii;
        }

        // Check for COLORTERM
        if let Ok(val) = env::var("COLORTERM") {
            if val == "truecolor" || val == "24bit" {
                return ColorProfile::TrueColor;
            }
        }

        // Check TERM
        if let Ok(term) = env::var("TERM") {
            if term.contains("256color") || term.contains("256") {
                return ColorProfile::Ansi256;
            }
            if term == "dumb" {
                return ColorProfile::Ascii;
            }
        }

        // Default to ANSI 256 for most modern terminals
        ColorProfile::Ansi256
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_hex() {
        assert_eq!(Color::hex("#FF5500"), Some(Color::Rgb(255, 85, 0)));
        assert_eq!(Color::hex("00FF00"), Some(Color::Rgb(0, 255, 0)));
        assert_eq!(Color::hex("invalid"), None);
        assert_eq!(Color::hex("#FFF"), None);
    }

    #[test]
    fn test_ansi_codes() {
        assert_eq!(Color::Red.to_ansi_fg(), "\x1b[31m");
        assert_eq!(Color::Red.to_ansi_bg(), "\x1b[41m");
        assert_eq!(Color::Ansi256(196).to_ansi_fg(), "\x1b[38;5;196m");
        assert_eq!(Color::Rgb(255, 128, 0).to_ansi_fg(), "\x1b[38;2;255;128;0m");
    }

    #[test]
    fn test_adaptive_color() {
        let color = Color::adaptive(Color::Black, Color::White);
        // Should resolve to one of the colors
        let resolved = color.resolve();
        assert!(matches!(resolved, Color::Black | Color::White));
    }

    #[test]
    fn test_complete_color() {
        let color = Color::complete((255, 0, 0), 196, 1);
        assert_eq!(color.to_ansi_fg(), "\x1b[38;2;255;0;0m");
    }

    #[test]
    fn test_default() {
        assert_eq!(Color::default(), Color::Default);
    }
}
