//! Key binding utilities.

use crate::terminal::KeyCode;

/// A key binding.
#[derive(Debug, Clone)]
pub struct KeyBinding {
    /// The key code.
    pub key: KeyCode,
    /// The action description.
    pub description: String,
}

impl KeyBinding {
    /// Create a new key binding.
    pub fn new(key: KeyCode, description: impl Into<String>) -> Self {
        Self { key, description: description.into() }
    }

    /// Get a display string for the key.
    pub fn key_display(&self) -> String {
        match self.key {
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Enter => "enter".to_string(),
            KeyCode::Esc => "esc".to_string(),
            KeyCode::Up => "↑".to_string(),
            KeyCode::Down => "↓".to_string(),
            KeyCode::Left => "←".to_string(),
            KeyCode::Right => "→".to_string(),
            KeyCode::Tab => "tab".to_string(),
            KeyCode::BackTab => "shift+tab".to_string(),
            KeyCode::Backspace => "backspace".to_string(),
            KeyCode::Delete => "delete".to_string(),
            KeyCode::Home => "home".to_string(),
            KeyCode::End => "end".to_string(),
            KeyCode::PageUp => "pgup".to_string(),
            KeyCode::PageDown => "pgdn".to_string(),
            KeyCode::F(n) => format!("F{}", n),
            _ => "?".to_string(),
        }
    }
}

/// A collection of key bindings.
#[derive(Debug, Clone, Default)]
pub struct KeyBindings {
    bindings: Vec<KeyBinding>,
}

impl KeyBindings {
    /// Create a new empty collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a key binding.
    pub fn add(mut self, key: KeyCode, description: impl Into<String>) -> Self {
        self.bindings.push(KeyBinding::new(key, description));
        self
    }

    /// Get all bindings.
    pub fn bindings(&self) -> &[KeyBinding] {
        &self.bindings
    }

    /// Render as a help line.
    pub fn render_short(&self) -> String {
        self.bindings
            .iter()
            .map(|b| format!("{} {}", b.key_display(), b.description))
            .collect::<Vec<_>>()
            .join(" • ")
    }

    /// Render as a help table.
    pub fn render_full(&self) -> String {
        let max_key_width = self.bindings.iter().map(|b| b.key_display().len()).max().unwrap_or(0);

        self.bindings
            .iter()
            .map(|b| {
                let key = b.key_display();
                let padding = " ".repeat(max_key_width - key.len());
                format!(
                    "  {}{}{}{} {}",
                    crate::style::Color::Cyan.to_ansi_fg(),
                    key,
                    "\x1b[0m",
                    padding,
                    b.description
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_binding_display() {
        let binding = KeyBinding::new(KeyCode::Enter, "Submit");
        assert_eq!(binding.key_display(), "enter");
    }

    #[test]
    fn test_key_bindings_render() {
        let bindings = KeyBindings::new().add(KeyCode::Enter, "submit").add(KeyCode::Esc, "cancel");

        let short = bindings.render_short();
        assert!(short.contains("enter submit"));
        assert!(short.contains("esc cancel"));
    }
}
