//! Animated spinner component.
//!
//! A spinner displays an animated loading indicator.
//!
//! # Example
//!
//! ```rust
//! use ferment::components::{Spinner, SpinnerStyle};
//!
//! let spinner = Spinner::new()
//!     .style(SpinnerStyle::Dots)
//!     .message("Loading...");
//! ```

use std::time::Duration;

use crate::{
    runtime::{Cmd, Model},
    style::Color,
    terminal::Event,
};

/// Predefined spinner animation styles.
#[derive(Debug, Clone, Copy, Default)]
pub enum SpinnerStyle {
    /// Classic line spinner: | / - \
    Line,
    /// Braille dots spinner.
    #[default]
    Dots,
    /// Growing dots: . .. ...
    GrowingDots,
    /// Circle quadrants.
    Circle,
    /// Box spinner.
    Box,
    /// Moon phases.
    Moon,
    /// Bouncing ball.
    Bounce,
    /// Arrow spinner.
    Arrow,
    /// Simple toggle.
    Toggle,
}

impl SpinnerStyle {
    /// Get the frames for this spinner style.
    pub fn frames(&self) -> &'static [&'static str] {
        match self {
            SpinnerStyle::Line => &["|", "/", "-", "\\"],
            SpinnerStyle::Dots => &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            SpinnerStyle::GrowingDots => &[".  ", ".. ", "...", " ..", "  .", "   "],
            SpinnerStyle::Circle => &["◐", "◓", "◑", "◒"],
            SpinnerStyle::Box => &["▖", "▘", "▝", "▗"],
            SpinnerStyle::Moon => &["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"],
            SpinnerStyle::Bounce => &["⠁", "⠂", "⠄", "⠂"],
            SpinnerStyle::Arrow => &["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"],
            SpinnerStyle::Toggle => &["⊶", "⊷"],
        }
    }

    /// Get the recommended interval for this spinner style.
    pub fn interval(&self) -> Duration {
        match self {
            SpinnerStyle::Dots => Duration::from_millis(80),
            SpinnerStyle::GrowingDots => Duration::from_millis(300),
            SpinnerStyle::Moon => Duration::from_millis(150),
            _ => Duration::from_millis(100),
        }
    }
}

/// Message type for spinner.
#[derive(Debug, Clone)]
pub enum SpinnerMsg {
    /// Advance to next frame.
    Tick,
    /// Update the message.
    SetMessage(String),
    /// Stop the spinner.
    Stop,
}

/// An animated spinner component.
#[derive(Debug, Clone)]
pub struct Spinner {
    style: SpinnerStyle,
    frame: usize,
    message: String,
    color: Color,
    running: bool,
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Spinner {
    /// Create a new spinner.
    pub fn new() -> Self {
        Self {
            style: SpinnerStyle::default(),
            frame: 0,
            message: String::new(),
            color: Color::Cyan,
            running: true,
        }
    }

    /// Set the spinner style.
    pub fn style(mut self, style: SpinnerStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the message to display next to the spinner.
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    /// Set the spinner color.
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Check if the spinner is running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Stop the spinner.
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Get the current frame character.
    pub fn current_frame(&self) -> &'static str {
        let frames = self.style.frames();
        frames[self.frame % frames.len()]
    }

    /// Advance to the next frame.
    pub fn tick(&mut self) {
        if self.running {
            let frames = self.style.frames();
            self.frame = (self.frame + 1) % frames.len();
        }
    }
}

impl Model for Spinner {
    type Message = SpinnerMsg;

    fn init(&self) -> Option<Cmd<Self::Message>> {
        if self.running {
            Some(Cmd::tick(self.style.interval(), |_| SpinnerMsg::Tick))
        } else {
            None
        }
    }

    fn update(&mut self, msg: Self::Message) -> Option<Cmd<Self::Message>> {
        match msg {
            SpinnerMsg::Tick => {
                self.tick();
                if self.running {
                    Some(Cmd::tick(self.style.interval(), |_| SpinnerMsg::Tick))
                } else {
                    None
                }
            },
            SpinnerMsg::SetMessage(message) => {
                self.message = message;
                None
            },
            SpinnerMsg::Stop => {
                self.running = false;
                None
            },
        }
    }

    fn view(&self) -> String {
        if !self.running {
            return String::new();
        }

        let frame = self.current_frame();
        let colored_frame = format!("{}{}{}", self.color.to_ansi_fg(), frame, "\x1b[0m");

        if self.message.is_empty() {
            colored_frame
        } else {
            format!("{} {}", colored_frame, self.message)
        }
    }

    fn handle_event(&self, _event: Event) -> Option<Self::Message> {
        None
    }

    fn wants_tick(&self) -> bool {
        self.running
    }
}

/// A multi-spinner for parallel operations.
#[derive(Debug, Clone)]
pub struct MultiSpinner {
    spinners: Vec<(String, Spinner)>,
}

impl MultiSpinner {
    /// Create a new multi-spinner.
    pub fn new() -> Self {
        Self { spinners: Vec::new() }
    }

    /// Add a spinner with a key.
    pub fn add(&mut self, key: impl Into<String>, spinner: Spinner) {
        self.spinners.push((key.into(), spinner));
    }

    /// Get a spinner by key.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Spinner> {
        self.spinners.iter_mut().find(|(k, _)| k == key).map(|(_, s)| s)
    }

    /// Stop all spinners.
    pub fn stop_all(&mut self) {
        for (_, spinner) in &mut self.spinners {
            spinner.stop();
        }
    }

    /// Tick all spinners.
    pub fn tick_all(&mut self) {
        for (_, spinner) in &mut self.spinners {
            spinner.tick();
        }
    }

    /// Render all spinners.
    pub fn view(&self) -> String {
        self.spinners.iter().map(|(_, spinner)| spinner.view()).collect::<Vec<_>>().join("\n")
    }
}

impl Default for MultiSpinner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_creation() {
        let spinner = Spinner::new().message("Loading...");
        assert!(spinner.is_running());
        assert_eq!(spinner.message, "Loading...");
    }

    #[test]
    fn test_spinner_tick() {
        let mut spinner = Spinner::new().style(SpinnerStyle::Line);
        assert_eq!(spinner.current_frame(), "|");
        spinner.tick();
        assert_eq!(spinner.current_frame(), "/");
        spinner.tick();
        assert_eq!(spinner.current_frame(), "-");
    }

    #[test]
    fn test_spinner_stop() {
        let mut spinner = Spinner::new();
        assert!(spinner.is_running());
        spinner.stop();
        assert!(!spinner.is_running());
    }

    #[test]
    fn test_spinner_view() {
        let spinner = Spinner::new().style(SpinnerStyle::Line).message("Loading");
        let view = spinner.view();
        assert!(view.contains("Loading"));
    }
}
