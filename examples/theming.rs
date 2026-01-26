//! Adaptive theming example demonstrating terminal-aware color selection.
//!
//! Run with: cargo run --example theming

use teapot::style::{BorderStyle, Color, ColorProfile, Style, has_dark_background};

fn main() {
    println!("\n=== Teapot Adaptive Theming Demo ===\n");

    // Detect terminal capabilities
    demo_detection();

    // Adaptive colors
    demo_adaptive_colors();

    // Complete color specs
    demo_complete_colors();

    // Theme system
    demo_theme_system();

    println!("=== End Demo ===\n");
}

fn demo_detection() {
    println!("--- Terminal Detection ---\n");

    // Detect background
    let is_dark = has_dark_background();
    println!("Background: {}", if is_dark { "Dark" } else { "Light" });

    // Detect color profile
    let profile = ColorProfile::detect();
    let profile_name = match profile {
        ColorProfile::TrueColor => "True Color (16M colors)",
        ColorProfile::Ansi256 => "ANSI 256 colors",
        ColorProfile::Ansi => "ANSI 16 colors",
        ColorProfile::Ascii => "ASCII (no color support)",
    };
    println!("Color profile: {}", profile_name);

    // Check environment
    if std::env::var("NO_COLOR").is_ok() {
        println!("NO_COLOR is set - colors disabled");
    }
    if std::env::var("COLORTERM").is_ok() {
        println!("COLORTERM: {}", std::env::var("COLORTERM").unwrap_or_default());
    }

    println!();
}

fn demo_adaptive_colors() {
    println!("--- Adaptive Colors ---\n");

    // Colors that adapt to terminal background
    let text_color = Color::Adaptive {
        light: Box::new(Color::Ansi256(235)), // Very dark gray for light bg
        dark: Box::new(Color::Ansi256(253)),  // Very light gray for dark bg
    };

    let accent_color = Color::Adaptive {
        light: Box::new(Color::Ansi256(27)), // Bright blue for light bg
        dark: Box::new(Color::Ansi256(123)), // Cyan for dark bg
    };

    let muted_color = Color::Adaptive {
        light: Box::new(Color::Ansi256(245)), // Medium gray for light bg
        dark: Box::new(Color::Ansi256(240)),  // Darker gray for dark bg
    };

    println!("{}", Style::new().fg(text_color).render("Primary text - adapts to background"));
    println!(
        "{}",
        Style::new().bold(true).fg(accent_color).render("Accent text - high visibility")
    );
    println!("{}", Style::new().fg(muted_color).render("Muted text - secondary information"));
    println!();
}

fn demo_complete_colors() {
    println!("--- Complete Color Specifications ---\n");

    // Colors with fallbacks for all terminal types
    // true_color is a tuple (r, g, b)
    let orange = Color::Complete {
        true_color: (255, 102, 0), // Exact color for modern terminals
        ansi256: 208,              // Best 256-color approximation
        ansi: 3,                   // Yellow fallback for 16-color
    };

    let purple = Color::Complete {
        true_color: (153, 51, 255),
        ansi256: 135,
        ansi: 5, // Magenta fallback
    };

    let teal = Color::Complete {
        true_color: (0, 168, 150),
        ansi256: 37,
        ansi: 6, // Cyan fallback
    };

    println!("{}", Style::new().bold(true).fg(orange).render("Orange: (255,102,0) → 208 → yellow"));
    println!(
        "{}",
        Style::new().bold(true).fg(purple).render("Purple: (153,51,255) → 135 → magenta")
    );
    println!("{}", Style::new().bold(true).fg(teal).render("Teal: (0,168,150) → 37 → cyan"));
    println!();
}

fn demo_theme_system() {
    println!("--- Theme System ---\n");

    // Define a theme
    let theme = Theme::detect();

    // Apply theme to UI elements
    let header = theme.header_style().render("Application Header");

    let content = theme.content_style().render(
        "This is the main content area.\n\
         The theme automatically adapts to your terminal.",
    );

    let success = theme.success_style().render("✓ Operation completed");
    let warning = theme.warning_style().render("⚠ Check configuration");
    let error = theme.error_style().render("✗ Connection failed");

    let footer = theme.muted_style().render("Press q to quit • ? for help");

    println!("{}\n", header);
    println!("{}\n", content);
    println!("{}", success);
    println!("{}", warning);
    println!("{}\n", error);
    println!("{}\n", footer);

    // Show theme info
    println!("Current theme: {}", if theme.is_dark { "Dark" } else { "Light" });
}

/// A complete theme system that adapts to terminal capabilities.
struct Theme {
    is_dark: bool,
    primary: Color,
    accent: Color,
    success: Color,
    warning: Color,
    error: Color,
    muted: Color,
}

impl Theme {
    /// Create a theme that adapts to the terminal.
    fn detect() -> Self {
        let is_dark = has_dark_background();

        if is_dark { Self::dark() } else { Self::light() }
    }

    /// Dark theme for dark terminal backgrounds.
    fn dark() -> Self {
        Self {
            is_dark: true,
            primary: Color::Ansi256(255), // White
            accent: Color::Ansi256(39),   // Bright cyan
            success: Color::Ansi256(114), // Green
            warning: Color::Ansi256(220), // Yellow
            error: Color::Ansi256(196),   // Red
            muted: Color::Ansi256(242),   // Dark gray
        }
    }

    /// Light theme for light terminal backgrounds.
    fn light() -> Self {
        Self {
            is_dark: false,
            primary: Color::Ansi256(232), // Near black
            accent: Color::Ansi256(27),   // Blue
            success: Color::Ansi256(28),  // Dark green
            warning: Color::Ansi256(172), // Orange
            error: Color::Ansi256(160),   // Dark red
            muted: Color::Ansi256(247),   // Light gray
        }
    }

    fn header_style(&self) -> Style {
        Style::new().bold(true).fg(self.accent.clone()).border(BorderStyle::Double).padding(&[0, 1])
    }

    fn content_style(&self) -> Style {
        Style::new().fg(self.primary.clone()).border(BorderStyle::Rounded).padding(&[1, 2])
    }

    fn success_style(&self) -> Style {
        Style::new().bold(true).fg(self.success.clone())
    }

    fn warning_style(&self) -> Style {
        Style::new().bold(true).fg(self.warning.clone())
    }

    fn error_style(&self) -> Style {
        Style::new().bold(true).fg(self.error.clone())
    }

    fn muted_style(&self) -> Style {
        Style::new().fg(self.muted.clone())
    }
}
