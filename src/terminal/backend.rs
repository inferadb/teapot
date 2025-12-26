//! Terminal backend implementation.

use std::io::{self, Write};

use crossterm::{cursor, execute, terminal};

/// Terminal backend for rendering.
pub struct Backend {
    alt_screen: bool,
}

impl Backend {
    /// Create a new backend.
    pub fn new() -> Self {
        Self { alt_screen: false }
    }

    /// Enter raw mode.
    pub fn enable_raw_mode(&self) -> io::Result<()> {
        terminal::enable_raw_mode()
    }

    /// Leave raw mode.
    pub fn disable_raw_mode(&self) -> io::Result<()> {
        terminal::disable_raw_mode()
    }

    /// Enter alternate screen.
    pub fn enter_alt_screen(&mut self) -> io::Result<()> {
        execute!(io::stdout(), terminal::EnterAlternateScreen)?;
        self.alt_screen = true;
        Ok(())
    }

    /// Leave alternate screen.
    pub fn leave_alt_screen(&mut self) -> io::Result<()> {
        if self.alt_screen {
            execute!(io::stdout(), terminal::LeaveAlternateScreen)?;
            self.alt_screen = false;
        }
        Ok(())
    }

    /// Hide the cursor.
    pub fn hide_cursor(&self) -> io::Result<()> {
        execute!(io::stdout(), cursor::Hide)
    }

    /// Show the cursor.
    pub fn show_cursor(&self) -> io::Result<()> {
        execute!(io::stdout(), cursor::Show)
    }

    /// Move the cursor to a position.
    pub fn move_cursor(&self, x: u16, y: u16) -> io::Result<()> {
        execute!(io::stdout(), cursor::MoveTo(x, y))
    }

    /// Clear the screen.
    pub fn clear(&self) -> io::Result<()> {
        execute!(io::stdout(), terminal::Clear(terminal::ClearType::All))
    }

    /// Get terminal size.
    pub fn size(&self) -> io::Result<(u16, u16)> {
        terminal::size()
    }

    /// Flush stdout.
    pub fn flush(&self) -> io::Result<()> {
        io::stdout().flush()
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Backend {
    fn drop(&mut self) {
        // Ensure we leave alt screen if we entered it
        let _ = self.leave_alt_screen();
    }
}
