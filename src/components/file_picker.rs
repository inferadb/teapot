//! File picker component.
//!
//! A component for browsing and selecting files or directories.
//!
//! # Example
//!
//! ```rust
//! use teapot::components::FilePicker;
//!
//! let picker = FilePicker::new()
//!     .title("Select a file")
//!     .directory("/home/user")
//!     .show_hidden(false);
//! ```

use std::path::{Path, PathBuf};

use crate::{
    runtime::{Cmd, Model, accessible::Accessible},
    style::Color,
    terminal::{Event, KeyCode},
};

/// Message type for file picker.
#[derive(Debug, Clone)]
pub enum FilePickerMsg {
    /// Move selection up.
    Up,
    /// Move selection down.
    Down,
    /// Enter directory or select file.
    Enter,
    /// Go to parent directory.
    Back,
    /// Toggle showing hidden files.
    ToggleHidden,
    /// Submit selection.
    Submit,
    /// Cancel selection.
    Cancel,
}

/// An entry in the file picker.
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// The file/directory name.
    pub name: String,
    /// Full path to the entry.
    pub path: PathBuf,
    /// Whether this is a directory.
    pub is_dir: bool,
    /// File size (for files).
    pub size: Option<u64>,
}

impl FileEntry {
    fn from_path(path: &Path) -> Option<Self> {
        let name = path.file_name()?.to_string_lossy().to_string();
        let metadata = path.metadata().ok()?;

        Some(Self {
            name,
            path: path.to_path_buf(),
            is_dir: metadata.is_dir(),
            size: if metadata.is_file() { Some(metadata.len()) } else { None },
        })
    }
}

/// A file picker component for selecting files or directories.
#[derive(Debug, Clone)]
pub struct FilePicker {
    title: String,
    current_dir: PathBuf,
    entries: Vec<FileEntry>,
    cursor: usize,
    show_hidden: bool,
    dirs_only: bool,
    files_only: bool,
    extensions: Vec<String>,
    selected: Option<PathBuf>,
    submitted: bool,
    cancelled: bool,
    focused: bool,
    height: usize,
    scroll_offset: usize,
}

impl Default for FilePicker {
    fn default() -> Self {
        Self::new()
    }
}

impl FilePicker {
    /// Create a new file picker.
    pub fn new() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));

        let mut picker = Self {
            title: String::new(),
            current_dir: current_dir.clone(),
            entries: Vec::new(),
            cursor: 0,
            show_hidden: false,
            dirs_only: false,
            files_only: false,
            extensions: Vec::new(),
            selected: None,
            submitted: false,
            cancelled: false,
            focused: true,
            height: 10,
            scroll_offset: 0,
        };

        picker.refresh_entries();
        picker
    }

    /// Set the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the starting directory.
    pub fn directory(mut self, dir: impl Into<PathBuf>) -> Self {
        self.current_dir = dir.into();
        self.refresh_entries();
        self
    }

    /// Show hidden files (those starting with .).
    pub fn show_hidden(mut self, show: bool) -> Self {
        self.show_hidden = show;
        self.refresh_entries();
        self
    }

    /// Only show directories.
    pub fn dirs_only(mut self) -> Self {
        self.dirs_only = true;
        self.files_only = false;
        self.refresh_entries();
        self
    }

    /// Only show files.
    pub fn files_only(mut self) -> Self {
        self.files_only = true;
        self.dirs_only = false;
        self.refresh_entries();
        self
    }

    /// Filter by file extensions (e.g., ["rs", "toml"]).
    pub fn extensions<I, S>(mut self, exts: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.extensions = exts.into_iter().map(|s| s.into()).collect();
        self.refresh_entries();
        self
    }

    /// Set the visible height (number of items to show).
    pub fn height(mut self, height: usize) -> Self {
        self.height = height.max(3);
        self
    }

    /// Set focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Get the selected path.
    pub fn selected(&self) -> Option<&PathBuf> {
        self.selected.as_ref()
    }

    /// Check if submitted.
    pub fn is_submitted(&self) -> bool {
        self.submitted
    }

    /// Check if cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    /// Get the current directory.
    pub fn current_dir(&self) -> &Path {
        &self.current_dir
    }

    /// Refresh the list of entries from the filesystem.
    fn refresh_entries(&mut self) {
        self.entries.clear();
        self.cursor = 0;
        self.scroll_offset = 0;

        let Ok(read_dir) = std::fs::read_dir(&self.current_dir) else {
            return;
        };

        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for entry in read_dir.flatten() {
            let path = entry.path();

            // Get file name
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };

            // Skip hidden files if not showing
            if !self.show_hidden && name.starts_with('.') {
                continue;
            }

            let Some(file_entry) = FileEntry::from_path(&path) else {
                continue;
            };

            // Apply filters
            if self.dirs_only && !file_entry.is_dir {
                continue;
            }

            if self.files_only && file_entry.is_dir {
                // Still show directories for navigation
                dirs.push(file_entry);
                continue;
            }

            // Extension filter (only for files)
            if !file_entry.is_dir && !self.extensions.is_empty() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                if !self.extensions.iter().any(|e| e.to_lowercase() == ext) {
                    continue;
                }
            }

            if file_entry.is_dir {
                dirs.push(file_entry);
            } else {
                files.push(file_entry);
            }
        }

        // Sort: directories first, then files, both alphabetically
        dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        self.entries.extend(dirs);
        self.entries.extend(files);
    }

    /// Move cursor up.
    fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.adjust_scroll();
        }
    }

    /// Move cursor down.
    fn move_down(&mut self) {
        if self.cursor < self.entries.len().saturating_sub(1) {
            self.cursor += 1;
            self.adjust_scroll();
        }
    }

    /// Adjust scroll offset to keep cursor visible.
    fn adjust_scroll(&mut self) {
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        } else if self.cursor >= self.scroll_offset + self.height {
            self.scroll_offset = self.cursor - self.height + 1;
        }
    }

    /// Enter directory or select file.
    fn enter(&mut self) {
        if let Some(entry) = self.entries.get(self.cursor) {
            if entry.is_dir {
                self.current_dir = entry.path.clone();
                self.refresh_entries();
            } else {
                self.selected = Some(entry.path.clone());
                self.submitted = true;
            }
        }
    }

    /// Go to parent directory.
    fn go_back(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.refresh_entries();
        }
    }

    /// Format file size for display.
    fn format_size(size: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if size >= GB {
            format!("{:.1}G", size as f64 / GB as f64)
        } else if size >= MB {
            format!("{:.1}M", size as f64 / MB as f64)
        } else if size >= KB {
            format!("{:.1}K", size as f64 / KB as f64)
        } else {
            format!("{}B", size)
        }
    }
}

impl Model for FilePicker {
    type Message = FilePickerMsg;

    fn init(&self) -> Option<Cmd<Self::Message>> {
        None
    }

    fn update(&mut self, msg: Self::Message) -> Option<Cmd<Self::Message>> {
        match msg {
            FilePickerMsg::Up => self.move_up(),
            FilePickerMsg::Down => self.move_down(),
            FilePickerMsg::Enter => self.enter(),
            FilePickerMsg::Back => self.go_back(),
            FilePickerMsg::ToggleHidden => {
                self.show_hidden = !self.show_hidden;
                self.refresh_entries();
            },
            FilePickerMsg::Submit => {
                if let Some(entry) = self.entries.get(self.cursor) {
                    if self.dirs_only || !entry.is_dir {
                        self.selected = Some(entry.path.clone());
                        self.submitted = true;
                    }
                }
            },
            FilePickerMsg::Cancel => {
                self.cancelled = true;
            },
        }
        None
    }

    fn view(&self) -> String {
        let mut output = String::new();

        // Title
        if !self.title.is_empty() {
            output.push_str(&format!("{}{}{}", Color::Cyan.to_ansi_fg(), self.title, "\x1b[0m"));
            output.push('\n');
        }

        // Current directory
        output.push_str(&format!(
            "{}{}{}",
            Color::BrightBlack.to_ansi_fg(),
            self.current_dir.display(),
            "\x1b[0m"
        ));
        output.push('\n');

        // Entries
        if self.entries.is_empty() {
            output.push_str(&format!("{}(empty){}", Color::BrightBlack.to_ansi_fg(), "\x1b[0m"));
        } else {
            let visible_end = (self.scroll_offset + self.height).min(self.entries.len());

            for (i, entry) in self.entries[self.scroll_offset..visible_end].iter().enumerate() {
                let idx = self.scroll_offset + i;
                let is_selected = idx == self.cursor && self.focused;

                // Cursor indicator
                if is_selected {
                    output.push_str(&format!("{}❯ ", Color::Magenta.to_ansi_fg()));
                } else {
                    output.push_str("  ");
                }

                // Directory indicator
                if entry.is_dir {
                    output.push_str(&format!("{}📁 ", Color::Blue.to_ansi_fg()));
                } else {
                    output.push_str("   ");
                }

                // Name
                if is_selected {
                    output.push_str(&format!(
                        "\x1b[1m{}{}\x1b[0m",
                        entry.name,
                        if entry.is_dir { "/" } else { "" }
                    ));
                } else {
                    output.push_str(&format!(
                        "{}{}",
                        entry.name,
                        if entry.is_dir { "/" } else { "" }
                    ));
                }

                // Size (for files)
                if let Some(size) = entry.size {
                    output.push_str(&format!(
                        "  {}{}{}",
                        Color::BrightBlack.to_ansi_fg(),
                        Self::format_size(size),
                        "\x1b[0m"
                    ));
                }

                output.push('\n');
            }

            // Scroll indicator
            if self.entries.len() > self.height {
                let position = if self.entries.len() > 1 {
                    self.cursor * 100 / (self.entries.len() - 1)
                } else {
                    100
                };
                output.push_str(&format!(
                    "\n{}({}/{} - {}%){}",
                    Color::BrightBlack.to_ansi_fg(),
                    self.cursor + 1,
                    self.entries.len(),
                    position,
                    "\x1b[0m"
                ));
            }
        }

        // Help
        output.push_str(&format!(
            "\n\n{}↑/↓: navigate  Enter: select  Backspace: parent  .: toggle hidden  Esc: cancel{}",
            Color::BrightBlack.to_ansi_fg(),
            "\x1b[0m"
        ));

        output
    }

    fn handle_event(&self, event: Event) -> Option<Self::Message> {
        if !self.focused {
            return None;
        }

        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => Some(FilePickerMsg::Up),
                KeyCode::Down | KeyCode::Char('j') => Some(FilePickerMsg::Down),
                KeyCode::Enter => Some(FilePickerMsg::Enter),
                KeyCode::Backspace => Some(FilePickerMsg::Back),
                KeyCode::Char('.') => Some(FilePickerMsg::ToggleHidden),
                KeyCode::Esc => Some(FilePickerMsg::Cancel),
                _ => None,
            }
        } else {
            None
        }
    }
}

impl Accessible for FilePicker {
    type Message = FilePickerMsg;

    fn accessible_prompt(&self) -> String {
        let mut prompt = String::new();

        if !self.title.is_empty() {
            prompt.push_str(&format!("{}\n", self.title));
        }

        prompt.push_str(&format!("Current directory: {}\n\n", self.current_dir.display()));

        if self.entries.is_empty() {
            prompt.push_str("(empty directory)\n");
        } else {
            for (i, entry) in self.entries.iter().enumerate() {
                let type_indicator = if entry.is_dir { "[DIR]" } else { "[FILE]" };
                prompt.push_str(&format!("{}: {} {}\n", i + 1, type_indicator, entry.name));
            }
        }

        prompt.push_str("\nEnter number to select, 'b' for parent directory, or 'q' to cancel: ");
        prompt
    }

    fn parse_accessible_input(&self, input: &str) -> Option<Self::Message> {
        let trimmed = input.trim().to_lowercase();

        if trimmed == "b" || trimmed == "back" {
            return Some(FilePickerMsg::Back);
        }

        if trimmed == "q" || trimmed == "quit" || trimmed == "cancel" {
            return Some(FilePickerMsg::Cancel);
        }

        if let Ok(num) = trimmed.parse::<usize>() {
            if num > 0 && num <= self.entries.len() {
                // Select and enter
                return Some(FilePickerMsg::Enter);
            }
        }

        None
    }

    fn is_accessible_complete(&self) -> bool {
        self.submitted || self.cancelled
    }
}

impl FilePicker {
    /// Apply accessible input and return true if selection is complete.
    pub fn apply_accessible_input(&mut self, input: &str) -> bool {
        let trimmed = input.trim().to_lowercase();

        if trimmed == "b" || trimmed == "back" {
            self.go_back();
            return false;
        }

        if trimmed == "q" || trimmed == "quit" || trimmed == "cancel" {
            self.cancelled = true;
            return true;
        }

        if let Ok(num) = trimmed.parse::<usize>() {
            if num > 0 && num <= self.entries.len() {
                self.cursor = num - 1;
                self.enter();
                return self.submitted;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_picker_creation() {
        let picker = FilePicker::new().title("Select file").height(5);

        assert!(!picker.is_submitted());
        assert!(!picker.is_cancelled());
        assert!(picker.selected().is_none());
    }

    #[test]
    fn test_format_size() {
        assert_eq!(FilePicker::format_size(500), "500B");
        assert_eq!(FilePicker::format_size(1500), "1.5K");
        assert_eq!(FilePicker::format_size(1500000), "1.4M");
        assert_eq!(FilePicker::format_size(1500000000), "1.4G");
    }

    #[test]
    fn test_file_picker_navigation() {
        let mut picker = FilePicker::new();

        // Add some test entries manually
        picker.entries = vec![
            FileEntry {
                name: "dir1".to_string(),
                path: PathBuf::from("/test/dir1"),
                is_dir: true,
                size: None,
            },
            FileEntry {
                name: "file1.txt".to_string(),
                path: PathBuf::from("/test/file1.txt"),
                is_dir: false,
                size: Some(1024),
            },
        ];

        assert_eq!(picker.cursor, 0);

        picker.update(FilePickerMsg::Down);
        assert_eq!(picker.cursor, 1);

        picker.update(FilePickerMsg::Up);
        assert_eq!(picker.cursor, 0);
    }

    #[test]
    fn test_file_picker_cancel() {
        let mut picker = FilePicker::new();

        picker.update(FilePickerMsg::Cancel);
        assert!(picker.is_cancelled());
    }
}
