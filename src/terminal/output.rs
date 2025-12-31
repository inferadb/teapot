//! Terminal output utilities.

use std::io::{self, Write};

use crossterm::{cursor, execute, style, terminal};

/// Terminal output writer with cursor and style control.
pub struct TerminalOutput {
    buffer: String,
}

impl TerminalOutput {
    /// Create a new terminal output.
    pub fn new() -> Self {
        Self { buffer: String::new() }
    }

    /// Write a string to the buffer.
    pub fn write(&mut self, s: &str) {
        self.buffer.push_str(s);
    }

    /// Write a line to the buffer.
    pub fn writeln(&mut self, s: &str) {
        self.buffer.push_str(s);
        self.buffer.push('\n');
    }

    /// Clear the buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Get the buffer contents.
    pub fn contents(&self) -> &str {
        &self.buffer
    }

    /// Flush the buffer to stdout.
    pub fn flush(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();
        write!(stdout, "{}", self.buffer)?;
        stdout.flush()?;
        self.buffer.clear();
        Ok(())
    }

    /// Move cursor to position and clear from there.
    pub fn clear_from(&self, row: u16) -> io::Result<()> {
        execute!(
            io::stdout(),
            cursor::MoveTo(0, row),
            terminal::Clear(terminal::ClearType::FromCursorDown)
        )
    }

    /// Move cursor to the beginning of a line.
    pub fn move_to_line(&self, row: u16) -> io::Result<()> {
        execute!(io::stdout(), cursor::MoveTo(0, row))
    }

    /// Set text color.
    pub fn set_foreground(&self, color: style::Color) -> io::Result<()> {
        execute!(io::stdout(), style::SetForegroundColor(color))
    }

    /// Reset text styling.
    pub fn reset_style(&self) -> io::Result<()> {
        execute!(io::stdout(), style::ResetColor)
    }
}

impl Default for TerminalOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl Write for TerminalOutput {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s =
            std::str::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        self.buffer.push_str(s);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        TerminalOutput::flush(self)
    }
}
