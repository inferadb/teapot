//! Multi-line text area component.
//!
//! A multi-line text input field with cursor support and scrolling.
//!
//! # Example
//!
//! ```rust
//! use teapot::components::TextArea;
//!
//! let textarea = TextArea::new()
//!     .placeholder("Enter your message...")
//!     .height(10)
//!     .width(60);
//! ```

use std::process::Command;

use crate::{
    runtime::{Cmd, Model, accessible::Accessible},
    style::Color,
    terminal::{Event, KeyCode, KeyModifiers},
};

/// Message type for text area.
#[derive(Debug, Clone)]
pub enum TextAreaMsg {
    /// Insert a character at cursor.
    InsertChar(char),
    /// Insert a newline at cursor.
    InsertNewline,
    /// Delete character before cursor.
    DeleteBack,
    /// Delete character at cursor.
    DeleteForward,
    /// Move cursor left.
    CursorLeft,
    /// Move cursor right.
    CursorRight,
    /// Move cursor up.
    CursorUp,
    /// Move cursor down.
    CursorDown,
    /// Move cursor to start of line.
    CursorLineStart,
    /// Move cursor to end of line.
    CursorLineEnd,
    /// Move cursor to start of text.
    CursorStart,
    /// Move cursor to end of text.
    CursorEnd,
    /// Delete word before cursor.
    DeleteWord,
    /// Delete entire line.
    DeleteLine,
    /// Clear all text.
    Clear,
    /// Submit the input (Ctrl+Enter or Esc based on config).
    Submit,
    /// Cancel editing.
    Cancel,
    /// Focus the input.
    Focus,
    /// Blur the input.
    Blur,
    /// Set the value.
    SetValue(String),
    /// Paste text.
    Paste(String),
    /// Page up.
    PageUp,
    /// Page down.
    PageDown,
    /// Open content in external editor.
    OpenEditor,
    /// Result from external editor (new content).
    EditorResult(String),
}

/// Cursor position in the text area.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[must_use = "components do nothing unless used in a view or run with Program"]
pub struct CursorPos {
    /// Row (line number, 0-indexed).
    pub row: usize,
    /// Column (character position in line, 0-indexed).
    pub col: usize,
}

/// A multi-line text area component.
#[derive(Debug, Clone)]
#[must_use = "components do nothing unless used in a view or run with Program"]
pub struct TextArea {
    lines: Vec<String>,
    cursor: CursorPos,
    scroll_offset: usize,
    placeholder: String,
    focused: bool,
    width: usize,
    height: usize,
    cursor_color: Color,
    text_color: Color,
    placeholder_color: Color,
    line_number_color: Color,
    submitted: bool,
    cancelled: bool,
    show_line_numbers: bool,
    max_lines: Option<usize>,
    validation_error: Option<String>,
    /// Custom editor command (overrides $EDITOR).
    editor: Option<String>,
    /// File extension for temp file when using external editor.
    editor_extension: String,
}

impl Default for TextArea {
    fn default() -> Self {
        Self::new()
    }
}

impl TextArea {
    /// Create a new text area.
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor: CursorPos::default(),
            scroll_offset: 0,
            placeholder: String::new(),
            focused: true,
            width: 80,
            height: 10,
            cursor_color: Color::Cyan,
            text_color: Color::Default,
            placeholder_color: Color::BrightBlack,
            line_number_color: Color::BrightBlack,
            submitted: false,
            cancelled: false,
            show_line_numbers: false,
            max_lines: None,
            validation_error: None,
            editor: None,
            editor_extension: "txt".to_string(),
        }
    }

    /// Set the placeholder text.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set the visible width in characters.
    pub fn width(mut self, width: usize) -> Self {
        self.width = width.max(10);
        self
    }

    /// Set the visible height in lines.
    pub fn height(mut self, height: usize) -> Self {
        self.height = height.max(1);
        self
    }

    /// Set the initial value.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.set_value_internal(value.into());
        self
    }

    /// Set the cursor color.
    pub fn cursor_color(mut self, color: Color) -> Self {
        self.cursor_color = color;
        self
    }

    /// Set the text color.
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// Enable or disable line numbers.
    pub fn show_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    /// Set maximum number of lines (None for unlimited).
    pub fn max_lines(mut self, max: Option<usize>) -> Self {
        self.max_lines = max;
        self
    }

    /// Set a custom editor command (overrides $EDITOR / $VISUAL).
    ///
    /// If not set, falls back to $VISUAL, then $EDITOR, then "vi".
    ///
    /// # Example
    ///
    /// ```no_run
    /// use teapot::components::TextArea;
    ///
    /// let textarea = TextArea::new()
    ///     .editor("code --wait")  // Use VS Code
    ///     .editor_extension("md"); // Use markdown extension
    /// ```
    pub fn editor(mut self, editor: impl Into<String>) -> Self {
        self.editor = Some(editor.into());
        self
    }

    /// Set the file extension for the temp file when using external editor.
    ///
    /// Defaults to "txt". This helps editors with syntax highlighting.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use teapot::components::TextArea;
    ///
    /// let textarea = TextArea::new()
    ///     .editor_extension("rs"); // For Rust code
    /// ```
    pub fn editor_extension(mut self, ext: impl Into<String>) -> Self {
        self.editor_extension = ext.into();
        self
    }

    /// Get the current value as a string.
    pub fn get_value(&self) -> String {
        self.lines.join("\n")
    }

    /// Get the lines.
    pub fn get_lines(&self) -> &[String] {
        &self.lines
    }

    /// Get the current cursor position.
    pub fn cursor_position(&self) -> CursorPos {
        self.cursor
    }

    /// Get the total number of lines.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Check if the input was submitted.
    pub fn is_submitted(&self) -> bool {
        self.submitted
    }

    /// Check if the input was cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    /// Check if the input is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Set focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Set a validation error.
    pub fn set_error(&mut self, error: impl Into<String>) {
        self.validation_error = Some(error.into());
    }

    /// Clear validation error.
    pub fn clear_error(&mut self) {
        self.validation_error = None;
    }

    /// Set value and update cursor.
    fn set_value_internal(&mut self, value: String) {
        self.lines = if value.is_empty() {
            vec![String::new()]
        } else {
            value.lines().map(String::from).collect()
        };
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        // Move cursor to end
        self.cursor.row = self.lines.len() - 1;
        self.cursor.col = self.lines[self.cursor.row].len();
        self.ensure_cursor_visible();
    }

    /// Get the current line.
    fn current_line(&self) -> &String {
        &self.lines[self.cursor.row]
    }

    /// Get the current line mutably.
    fn current_line_mut(&mut self) -> &mut String {
        &mut self.lines[self.cursor.row]
    }

    /// Get byte offset in current line from column position.
    fn col_to_byte_offset(&self, line: &str, col: usize) -> usize {
        line.chars().take(col).map(|c| c.len_utf8()).sum()
    }

    /// Get column position from byte offset.
    fn byte_offset_to_col(&self, line: &str, byte_offset: usize) -> usize {
        line[..byte_offset.min(line.len())].chars().count()
    }

    /// Clamp cursor column to current line length.
    fn clamp_cursor_col(&mut self) {
        let line_len = self.current_line().chars().count();
        self.cursor.col = self.cursor.col.min(line_len);
    }

    /// Ensure cursor is visible in the viewport.
    fn ensure_cursor_visible(&mut self) {
        if self.cursor.row < self.scroll_offset {
            self.scroll_offset = self.cursor.row;
        } else if self.cursor.row >= self.scroll_offset + self.height {
            self.scroll_offset = self.cursor.row.saturating_sub(self.height - 1);
        }
    }

    /// Insert a character at the cursor position.
    fn insert_char(&mut self, c: char) {
        let byte_offset = self.col_to_byte_offset(self.current_line(), self.cursor.col);
        self.current_line_mut().insert(byte_offset, c);
        self.cursor.col += 1;
        self.validation_error = None;
    }

    /// Insert a newline at the cursor position.
    fn insert_newline(&mut self) {
        // Check max lines limit
        if let Some(max) = self.max_lines
            && self.lines.len() >= max
        {
            return;
        }

        let byte_offset = self.col_to_byte_offset(self.current_line(), self.cursor.col);
        let current_line = self.current_line_mut();
        let remainder = current_line[byte_offset..].to_string();
        current_line.truncate(byte_offset);

        self.cursor.row += 1;
        self.cursor.col = 0;
        self.lines.insert(self.cursor.row, remainder);
        self.ensure_cursor_visible();
        self.validation_error = None;
    }

    /// Delete the character before the cursor.
    fn delete_back(&mut self) {
        if self.cursor.col > 0 {
            // Delete character in current line
            let cursor_col = self.cursor.col;
            let line = &self.lines[self.cursor.row];
            let byte_offset: usize = line.chars().take(cursor_col).map(|c| c.len_utf8()).sum();
            let prev_char_len =
                line[..byte_offset].chars().last().map(|c| c.len_utf8()).unwrap_or(0);
            let remove_at = byte_offset - prev_char_len;
            self.lines[self.cursor.row].remove(remove_at);
            self.cursor.col -= 1;
        } else if self.cursor.row > 0 {
            // Merge with previous line
            let current_line = self.lines.remove(self.cursor.row);
            self.cursor.row -= 1;
            self.cursor.col = self.current_line().chars().count();
            self.current_line_mut().push_str(&current_line);
            self.ensure_cursor_visible();
        }
        self.validation_error = None;
    }

    /// Delete the character at the cursor.
    fn delete_forward(&mut self) {
        let line_char_count = self.current_line().chars().count();
        if self.cursor.col < line_char_count {
            // Delete character in current line
            let byte_offset = self.col_to_byte_offset(self.current_line(), self.cursor.col);
            self.current_line_mut().remove(byte_offset);
        } else if self.cursor.row < self.lines.len() - 1 {
            // Merge next line into current
            let next_line = self.lines.remove(self.cursor.row + 1);
            self.current_line_mut().push_str(&next_line);
        }
        self.validation_error = None;
    }

    /// Move cursor left.
    fn cursor_left(&mut self) {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        } else if self.cursor.row > 0 {
            self.cursor.row -= 1;
            self.cursor.col = self.current_line().chars().count();
            self.ensure_cursor_visible();
        }
    }

    /// Move cursor right.
    fn cursor_right(&mut self) {
        let line_char_count = self.current_line().chars().count();
        if self.cursor.col < line_char_count {
            self.cursor.col += 1;
        } else if self.cursor.row < self.lines.len() - 1 {
            self.cursor.row += 1;
            self.cursor.col = 0;
            self.ensure_cursor_visible();
        }
    }

    /// Move cursor up.
    fn cursor_up(&mut self) {
        if self.cursor.row > 0 {
            self.cursor.row -= 1;
            self.clamp_cursor_col();
            self.ensure_cursor_visible();
        }
    }

    /// Move cursor down.
    fn cursor_down(&mut self) {
        if self.cursor.row < self.lines.len() - 1 {
            self.cursor.row += 1;
            self.clamp_cursor_col();
            self.ensure_cursor_visible();
        }
    }

    /// Move cursor to start of current line.
    fn cursor_line_start(&mut self) {
        self.cursor.col = 0;
    }

    /// Move cursor to end of current line.
    fn cursor_line_end(&mut self) {
        self.cursor.col = self.current_line().chars().count();
    }

    /// Move cursor to start of text.
    fn cursor_start(&mut self) {
        self.cursor.row = 0;
        self.cursor.col = 0;
        self.ensure_cursor_visible();
    }

    /// Move cursor to end of text.
    fn cursor_end(&mut self) {
        self.cursor.row = self.lines.len() - 1;
        self.cursor.col = self.current_line().chars().count();
        self.ensure_cursor_visible();
    }

    /// Delete word before cursor.
    fn delete_word(&mut self) {
        if self.cursor.col == 0 {
            // At start of line, merge with previous
            self.delete_back();
            return;
        }

        let line = self.current_line();
        let byte_offset = self.col_to_byte_offset(line, self.cursor.col);
        let before = &line[..byte_offset];
        let trimmed = before.trim_end();
        let word_start_byte = trimmed.rfind(char::is_whitespace).map(|i| i + 1).unwrap_or(0);
        let word_start_col = self.byte_offset_to_col(line, word_start_byte);

        // Remove from word_start to cursor
        let line = self.current_line_mut();
        let after = line[byte_offset..].to_string();
        line.truncate(word_start_byte);
        line.push_str(&after);
        self.cursor.col = word_start_col;
        self.validation_error = None;
    }

    /// Delete entire current line.
    fn delete_line(&mut self) {
        if self.lines.len() > 1 {
            self.lines.remove(self.cursor.row);
            if self.cursor.row >= self.lines.len() {
                self.cursor.row = self.lines.len() - 1;
            }
            self.clamp_cursor_col();
        } else {
            self.lines[0].clear();
            self.cursor.col = 0;
        }
        self.ensure_cursor_visible();
        self.validation_error = None;
    }

    /// Page up.
    fn page_up(&mut self) {
        let page_size = self.height.saturating_sub(1);
        self.cursor.row = self.cursor.row.saturating_sub(page_size);
        self.clamp_cursor_col();
        self.ensure_cursor_visible();
    }

    /// Page down.
    fn page_down(&mut self) {
        let page_size = self.height.saturating_sub(1);
        self.cursor.row = (self.cursor.row + page_size).min(self.lines.len().saturating_sub(1));
        self.clamp_cursor_col();
        self.ensure_cursor_visible();
    }

    /// Paste text at cursor.
    fn paste(&mut self, text: String) {
        for c in text.chars() {
            if c == '\n' {
                self.insert_newline();
            } else if !c.is_control() {
                self.insert_char(c);
            }
        }
    }

    /// Create a command to open the current content in an external editor.
    ///
    /// Uses `Cmd::run_process` to spawn the editor. When the editor closes,
    /// the file content is read back and returned as an `EditorResult` message.
    pub fn open_in_editor(&self) -> Cmd<TextAreaMsg> {
        let content = self.get_value();
        let extension = self.editor_extension.clone();
        let editor = self.editor.clone();

        // Get the editor command
        let editor_cmd = editor
            .or_else(|| std::env::var("VISUAL").ok())
            .or_else(|| std::env::var("EDITOR").ok())
            .unwrap_or_else(|| "vi".to_string());

        // Create temp file with content
        let temp_path =
            std::env::temp_dir().join(format!("teapot_edit_{}.{}", std::process::id(), extension));

        // Write content to temp file
        if std::fs::write(&temp_path, &content).is_err() {
            // If we can't write the file, return the original content
            return Cmd::perform(move || TextAreaMsg::EditorResult(content));
        }

        let temp_path_clone = temp_path.clone();

        // Parse the editor command (may include arguments like "code --wait")
        let parts: Vec<&str> = editor_cmd.split_whitespace().collect();
        let (program, args) =
            if parts.is_empty() { ("vi", Vec::new()) } else { (parts[0], parts[1..].to_vec()) };

        let mut cmd = Command::new(program);
        for arg in args {
            cmd.arg(arg);
        }
        cmd.arg(&temp_path);

        Cmd::run_process(cmd, move |result| {
            if result.is_ok() {
                // Read the edited content
                match std::fs::read_to_string(&temp_path_clone) {
                    Ok(new_content) => {
                        // Clean up temp file
                        let _ = std::fs::remove_file(&temp_path_clone);
                        TextAreaMsg::EditorResult(new_content)
                    },
                    Err(_) => {
                        // If read fails, keep original content
                        let _ = std::fs::remove_file(&temp_path_clone);
                        TextAreaMsg::EditorResult(content.clone())
                    },
                }
            } else {
                // Editor failed, keep original content
                let _ = std::fs::remove_file(&temp_path_clone);
                TextAreaMsg::EditorResult(content.clone())
            }
        })
    }

    /// Render a single line with cursor if applicable.
    fn render_line(&self, line_idx: usize, line: &str) -> String {
        let mut output = String::new();

        // Line number
        if self.show_line_numbers {
            let line_num = line_idx + 1;
            let num_width = self.lines.len().to_string().len().max(2);
            output.push_str(&format!(
                "{}{:>width$} │{} ",
                self.line_number_color.to_ansi_fg(),
                line_num,
                "\x1b[0m",
                width = num_width
            ));
        }

        let is_cursor_line = self.focused && line_idx == self.cursor.row;

        if is_cursor_line {
            // Render line with cursor
            let cursor_byte = self.col_to_byte_offset(line, self.cursor.col);
            let (before, after) = line.split_at(cursor_byte.min(line.len()));

            output.push_str(&format!("{}{}", self.text_color.to_ansi_fg(), before));

            // Cursor character (or space if at end)
            let cursor_char = after.chars().next().map(|c| {
                let mut buf = [0u8; 4];
                c.encode_utf8(&mut buf);
                c.to_string()
            });
            let (cursor_display, rest) = if let Some(ref c) = cursor_char {
                (c.as_str(), &after[c.len()..])
            } else {
                (" ", "")
            };

            output.push_str(&format!(
                "\x1b[7m{}{}\x1b[27m",
                self.cursor_color.to_ansi_fg(),
                cursor_display
            ));
            output.push_str(&format!("{}{}\x1b[0m", self.text_color.to_ansi_fg(), rest));
        } else {
            // Regular line
            output.push_str(&format!("{}{}{}", self.text_color.to_ansi_fg(), line, "\x1b[0m"));
        }

        output
    }
}

impl Model for TextArea {
    type Message = TextAreaMsg;

    fn init(&self) -> Option<Cmd<Self::Message>> {
        None
    }

    fn update(&mut self, msg: Self::Message) -> Option<Cmd<Self::Message>> {
        match msg {
            TextAreaMsg::InsertChar(c) => self.insert_char(c),
            TextAreaMsg::InsertNewline => self.insert_newline(),
            TextAreaMsg::DeleteBack => self.delete_back(),
            TextAreaMsg::DeleteForward => self.delete_forward(),
            TextAreaMsg::CursorLeft => self.cursor_left(),
            TextAreaMsg::CursorRight => self.cursor_right(),
            TextAreaMsg::CursorUp => self.cursor_up(),
            TextAreaMsg::CursorDown => self.cursor_down(),
            TextAreaMsg::CursorLineStart => self.cursor_line_start(),
            TextAreaMsg::CursorLineEnd => self.cursor_line_end(),
            TextAreaMsg::CursorStart => self.cursor_start(),
            TextAreaMsg::CursorEnd => self.cursor_end(),
            TextAreaMsg::DeleteWord => self.delete_word(),
            TextAreaMsg::DeleteLine => self.delete_line(),
            TextAreaMsg::Clear => {
                self.lines = vec![String::new()];
                self.cursor = CursorPos::default();
                self.scroll_offset = 0;
                self.validation_error = None;
            },
            TextAreaMsg::Submit => {
                self.submitted = true;
            },
            TextAreaMsg::Cancel => {
                self.cancelled = true;
            },
            TextAreaMsg::Focus => {
                self.focused = true;
            },
            TextAreaMsg::Blur => {
                self.focused = false;
            },
            TextAreaMsg::SetValue(value) => {
                self.set_value_internal(value);
            },
            TextAreaMsg::Paste(text) => {
                self.paste(text);
            },
            TextAreaMsg::PageUp => self.page_up(),
            TextAreaMsg::PageDown => self.page_down(),
            TextAreaMsg::OpenEditor => {
                return Some(self.open_in_editor());
            },
            TextAreaMsg::EditorResult(content) => {
                self.set_value_internal(content);
            },
        }
        None
    }

    fn view(&self) -> String {
        let mut output = String::new();

        // Check if empty and show placeholder
        let is_empty = self.lines.len() == 1 && self.lines[0].is_empty();

        if is_empty && !self.focused && !self.placeholder.is_empty() {
            output.push_str(&format!(
                "{}{}{}",
                self.placeholder_color.to_ansi_fg(),
                self.placeholder,
                "\x1b[0m"
            ));
            return output;
        }

        // Calculate visible range
        let visible_start = self.scroll_offset;
        let visible_end = (self.scroll_offset + self.height).min(self.lines.len());

        // Scroll indicator (top)
        if visible_start > 0 {
            output.push_str(&format!(
                "{}↑ {} more lines{}",
                Color::BrightBlack.to_ansi_fg(),
                visible_start,
                "\x1b[0m\n"
            ));
        }

        // Render visible lines
        for (view_idx, line_idx) in (visible_start..visible_end).enumerate() {
            let line = &self.lines[line_idx];
            output.push_str(&self.render_line(line_idx, line));

            if view_idx < visible_end - visible_start - 1 {
                output.push('\n');
            }
        }

        // Scroll indicator (bottom)
        let remaining = self.lines.len().saturating_sub(visible_end);
        if remaining > 0 {
            output.push_str(&format!(
                "\n{}↓ {} more lines{}",
                Color::BrightBlack.to_ansi_fg(),
                remaining,
                "\x1b[0m"
            ));
        }

        // Show validation error if present
        if let Some(ref error) = self.validation_error {
            output.push_str(&format!("\n{}✗ {}{}", Color::Red.to_ansi_fg(), error, "\x1b[0m"));
        }

        output
    }

    fn handle_event(&self, event: Event) -> Option<Self::Message> {
        if !self.focused {
            return None;
        }

        match event {
            Event::Key(key) => {
                // Handle control key combinations
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    return match key.code {
                        KeyCode::Char('a') => Some(TextAreaMsg::CursorLineStart),
                        KeyCode::Char('e') => Some(TextAreaMsg::CursorLineEnd),
                        KeyCode::Char('w') => Some(TextAreaMsg::DeleteWord),
                        KeyCode::Char('k') => Some(TextAreaMsg::DeleteLine),
                        KeyCode::Char('u') => Some(TextAreaMsg::Clear),
                        KeyCode::Char('n') => Some(TextAreaMsg::CursorDown),
                        KeyCode::Char('p') => Some(TextAreaMsg::CursorUp),
                        KeyCode::Char('o') => Some(TextAreaMsg::OpenEditor),
                        KeyCode::Enter => Some(TextAreaMsg::Submit),
                        KeyCode::Home => Some(TextAreaMsg::CursorStart),
                        KeyCode::End => Some(TextAreaMsg::CursorEnd),
                        _ => None,
                    };
                }

                match key.code {
                    KeyCode::Char(c) => Some(TextAreaMsg::InsertChar(c)),
                    KeyCode::Enter => Some(TextAreaMsg::InsertNewline),
                    KeyCode::Backspace => Some(TextAreaMsg::DeleteBack),
                    KeyCode::Delete => Some(TextAreaMsg::DeleteForward),
                    KeyCode::Left => Some(TextAreaMsg::CursorLeft),
                    KeyCode::Right => Some(TextAreaMsg::CursorRight),
                    KeyCode::Up => Some(TextAreaMsg::CursorUp),
                    KeyCode::Down => Some(TextAreaMsg::CursorDown),
                    KeyCode::Home => Some(TextAreaMsg::CursorLineStart),
                    KeyCode::End => Some(TextAreaMsg::CursorLineEnd),
                    KeyCode::PageUp => Some(TextAreaMsg::PageUp),
                    KeyCode::PageDown => Some(TextAreaMsg::PageDown),
                    KeyCode::Esc => Some(TextAreaMsg::Cancel),
                    KeyCode::Tab => Some(TextAreaMsg::InsertChar('\t')),
                    _ => None,
                }
            },
            Event::Paste(text) => Some(TextAreaMsg::Paste(text)),
            _ => None,
        }
    }
}

impl Accessible for TextArea {
    type Message = TextAreaMsg;

    fn accessible_prompt(&self) -> String {
        let mut prompt = String::new();

        // Placeholder/title
        if !self.placeholder.is_empty() {
            prompt.push_str(&format!(
                "? {} (multi-line, enter blank line to finish)\n",
                self.placeholder
            ));
        } else {
            prompt.push_str("? Enter text (multi-line, enter blank line to finish)\n");
        }

        // Show current content if any
        if !self.is_empty() {
            prompt.push_str("Current content:\n");
            for (i, line) in self.lines.iter().enumerate() {
                prompt.push_str(&format!("  {}: {}\n", i + 1, line));
            }
            prompt.push('\n');
        }

        prompt.push_str("> ");
        prompt
    }

    fn parse_accessible_input(&self, input: &str) -> Option<Self::Message> {
        let trimmed = input.trim();

        // Empty line signals end of input
        if trimmed.is_empty() {
            return Some(TextAreaMsg::Submit);
        }

        // Append the line
        Some(TextAreaMsg::Paste(input.trim_end_matches('\n').to_string()))
    }

    fn is_accessible_complete(&self) -> bool {
        self.submitted || self.cancelled
    }
}

impl TextArea {
    /// Apply accessible input line by line.
    ///
    /// In accessible mode, users enter text line by line.
    /// An empty line signals the end of input.
    ///
    /// Returns true if input is complete (empty line received).
    pub fn apply_accessible_input(&mut self, input: &str) -> bool {
        let trimmed = input.trim();

        // Empty line signals end of input
        if trimmed.is_empty() {
            self.submitted = true;
            return true;
        }

        // Add the line to content
        let content = input.trim_end_matches('\n').to_string();

        // If we have content on the last line, add a newline first
        if !self.lines.is_empty() && !self.current_line().is_empty() {
            self.insert_newline();
        }

        // Insert the content
        for c in content.chars() {
            if c == '\n' {
                self.insert_newline();
            } else {
                self.insert_char(c);
            }
        }

        false // Not complete, allow more input
    }

    /// Check if the text area is empty.
    fn is_empty(&self) -> bool {
        self.lines.len() == 1 && self.lines[0].is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_area_creation() {
        let textarea = TextArea::new().placeholder("Enter text...");
        assert_eq!(textarea.get_value(), "");
        assert_eq!(textarea.placeholder, "Enter text...");
        assert_eq!(textarea.line_count(), 1);
    }

    #[test]
    fn test_insert_char() {
        let mut textarea = TextArea::new();
        textarea.insert_char('a');
        textarea.insert_char('b');
        textarea.insert_char('c');
        assert_eq!(textarea.get_value(), "abc");
        assert_eq!(textarea.cursor.col, 3);
    }

    #[test]
    fn test_newline() {
        let mut textarea = TextArea::new();
        textarea.insert_char('a');
        textarea.insert_newline();
        textarea.insert_char('b');
        assert_eq!(textarea.get_value(), "a\nb");
        assert_eq!(textarea.line_count(), 2);
        assert_eq!(textarea.cursor.row, 1);
        assert_eq!(textarea.cursor.col, 1);
    }

    #[test]
    fn test_cursor_movement() {
        let mut textarea = TextArea::new().value("line1\nline2\nline3");
        // Cursor starts at end
        assert_eq!(textarea.cursor.row, 2);
        assert_eq!(textarea.cursor.col, 5);

        textarea.cursor_up();
        assert_eq!(textarea.cursor.row, 1);

        textarea.cursor_start();
        assert_eq!(textarea.cursor.row, 0);
        assert_eq!(textarea.cursor.col, 0);

        textarea.cursor_end();
        assert_eq!(textarea.cursor.row, 2);
        assert_eq!(textarea.cursor.col, 5);
    }

    #[test]
    fn test_delete_back() {
        let mut textarea = TextArea::new().value("ab\ncd");
        textarea.cursor.row = 1;
        textarea.cursor.col = 0;

        // Delete back at start of line merges with previous
        textarea.delete_back();
        assert_eq!(textarea.get_value(), "abcd");
        assert_eq!(textarea.line_count(), 1);
        assert_eq!(textarea.cursor.col, 2);
    }

    #[test]
    fn test_delete_forward() {
        let mut textarea = TextArea::new().value("ab\ncd");
        textarea.cursor.row = 0;
        textarea.cursor.col = 2;

        // Delete forward at end of line merges with next
        textarea.delete_forward();
        assert_eq!(textarea.get_value(), "abcd");
        assert_eq!(textarea.line_count(), 1);
    }

    #[test]
    fn test_cursor_clamp() {
        let mut textarea = TextArea::new().value("short\nlonger line");
        textarea.cursor.row = 1;
        textarea.cursor.col = 11; // At end of "longer line"

        textarea.cursor_up();
        // Should clamp to end of "short" (5 chars)
        assert_eq!(textarea.cursor.col, 5);
    }

    #[test]
    fn test_line_navigation() {
        let mut textarea = TextArea::new().value("first\nsecond\nthird");
        textarea.cursor_start();

        textarea.cursor_line_end();
        assert_eq!(textarea.cursor.col, 5);

        textarea.cursor_line_start();
        assert_eq!(textarea.cursor.col, 0);
    }

    #[test]
    fn test_paste_multiline() {
        let mut textarea = TextArea::new();
        textarea.paste("line1\nline2\nline3".to_string());
        assert_eq!(textarea.line_count(), 3);
        assert_eq!(textarea.get_value(), "line1\nline2\nline3");
    }

    #[test]
    fn test_delete_line() {
        let mut textarea = TextArea::new().value("one\ntwo\nthree");
        textarea.cursor.row = 1;
        textarea.delete_line();
        assert_eq!(textarea.get_value(), "one\nthree");
        assert_eq!(textarea.line_count(), 2);
    }

    #[test]
    fn test_max_lines() {
        let mut textarea = TextArea::new().max_lines(Some(2));
        textarea.insert_char('a');
        textarea.insert_newline();
        textarea.insert_char('b');
        textarea.insert_newline(); // Should be ignored
        textarea.insert_char('c');

        assert_eq!(textarea.line_count(), 2);
        assert_eq!(textarea.get_value(), "a\nbc");
    }

    #[test]
    fn test_accessible_prompt() {
        let textarea = TextArea::new().placeholder("Enter description");
        let prompt = textarea.accessible_prompt();
        assert!(prompt.contains("Enter description"));
        assert!(prompt.contains("multi-line"));
        assert!(prompt.contains("> "));
    }

    #[test]
    fn test_accessible_apply_input() {
        let mut textarea = TextArea::new();

        // Add first line
        assert!(!textarea.apply_accessible_input("First line"));
        assert!(!textarea.is_submitted());

        // Add second line
        assert!(!textarea.apply_accessible_input("Second line"));
        assert!(!textarea.is_submitted());

        // Empty line to finish
        assert!(textarea.apply_accessible_input(""));
        assert!(textarea.is_submitted());

        // Verify content
        let value = textarea.get_value();
        assert!(value.contains("First line"));
        assert!(value.contains("Second line"));
    }
}
