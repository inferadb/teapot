//! Table component for displaying tabular data.
//!
//! A scrollable data table with keyboard navigation and column support.
//!
//! # Example
//!
//! ```rust
//! use teapot::components::{Table, Column};
//!
//! let table = Table::new()
//!     .columns(vec![
//!         Column::new("Name").width(20),
//!         Column::new("Age").width(5),
//!         Column::new("City").width(15),
//!     ])
//!     .rows(vec![
//!         vec!["Alice", "30", "New York"],
//!         vec!["Bob", "25", "Los Angeles"],
//!         vec!["Charlie", "35", "Chicago"],
//!     ])
//!     .height(10);
//! ```

use crate::{
    runtime::{Cmd, Model},
    style::Color,
    terminal::{Event, KeyCode, KeyModifiers},
};

/// Column alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    /// Left-aligned text.
    #[default]
    Left,
    /// Center-aligned text.
    Center,
    /// Right-aligned text.
    Right,
}

/// Column definition for the table.
#[derive(Debug, Clone)]
pub struct Column {
    /// Column header text.
    pub title: String,
    /// Fixed width (0 means auto).
    pub width: usize,
    /// Text alignment.
    pub align: Align,
    /// Whether this column should grow to fill remaining space.
    pub grow: bool,
}

impl Column {
    /// Create a new column with a title.
    pub fn new(title: impl Into<String>) -> Self {
        Self { title: title.into(), width: 0, align: Align::Left, grow: false }
    }

    /// Set the column width.
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Set the column alignment.
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Mark this column as the growth column.
    ///
    /// The growth column will expand to fill any remaining horizontal space
    /// after all other columns have been sized.
    pub fn grow(mut self) -> Self {
        self.grow = true;
        self
    }
}

/// Message type for table.
#[derive(Debug, Clone)]
pub enum TableMsg {
    /// Move selection up.
    Up,
    /// Move selection down.
    Down,
    /// Move to first row.
    First,
    /// Move to last row.
    Last,
    /// Page up.
    PageUp,
    /// Page down.
    PageDown,
    /// Move to previous column (for cell selection mode).
    Left,
    /// Move to next column (for cell selection mode).
    Right,
    /// Submit selection.
    Submit,
    /// Cancel selection.
    Cancel,
    /// Focus the table.
    Focus,
    /// Blur the table.
    Blur,
}

/// A scrollable table component.
#[derive(Debug, Clone)]
pub struct Table {
    columns: Vec<Column>,
    rows: Vec<Vec<String>>,
    cursor_row: usize,
    cursor_col: usize,
    offset: usize,
    height: usize,
    /// Maximum width for the table (0 = unlimited).
    width: usize,
    /// Horizontal scroll offset for wide tables.
    h_scroll_offset: usize,
    focused: bool,
    submitted: bool,
    cancelled: bool,
    cell_selection: bool,
    show_header: bool,
    show_borders: bool,
    header_color: Color,
    selected_row_color: Color,
    selected_cell_color: Color,
    border_color: Color,
    row_color: Color,
    alt_row_color: Option<Color>,
}

impl Default for Table {
    fn default() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            cursor_row: 0,
            cursor_col: 0,
            offset: 0,
            height: 10,
            width: 0,
            h_scroll_offset: 0,
            focused: true,
            submitted: false,
            cancelled: false,
            cell_selection: false,
            show_header: true,
            show_borders: true,
            header_color: Color::Cyan,
            selected_row_color: Color::Cyan,
            selected_cell_color: Color::Green,
            border_color: Color::BrightBlack,
            row_color: Color::Default,
            alt_row_color: None,
        }
    }
}

impl Table {
    /// Create a new empty table.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the columns.
    pub fn columns(mut self, columns: Vec<Column>) -> Self {
        self.columns = columns;
        self
    }

    /// Set the rows.
    pub fn rows<I, R, S>(mut self, rows: I) -> Self
    where
        I: IntoIterator<Item = R>,
        R: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.rows =
            rows.into_iter().map(|row| row.into_iter().map(|s| s.into()).collect()).collect();
        self
    }

    /// Set the visible height (number of rows).
    pub fn height(mut self, height: usize) -> Self {
        self.height = height;
        self
    }

    /// Set the maximum width for the table.
    ///
    /// When set, rows will be clipped or scrolled horizontally to fit.
    /// Use 0 (default) for unlimited width.
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Set the horizontal scroll offset.
    ///
    /// This determines how many characters of the table content are hidden
    /// on the left side when the table is wider than the available width.
    pub fn with_h_scroll_offset(mut self, offset: usize) -> Self {
        self.h_scroll_offset = offset;
        self
    }

    /// Enable cell selection mode (navigate to individual cells).
    pub fn cell_selection(mut self, enabled: bool) -> Self {
        self.cell_selection = enabled;
        self
    }

    /// Set whether to show the header row.
    pub fn show_header(mut self, show: bool) -> Self {
        self.show_header = show;
        self
    }

    /// Set whether to show borders.
    pub fn show_borders(mut self, show: bool) -> Self {
        self.show_borders = show;
        self
    }

    /// Set the header color.
    pub fn header_color(mut self, color: Color) -> Self {
        self.header_color = color;
        self
    }

    /// Set the selected row color.
    pub fn selected_row_color(mut self, color: Color) -> Self {
        self.selected_row_color = color;
        self
    }

    /// Set the selected cell color (for cell selection mode).
    pub fn selected_cell_color(mut self, color: Color) -> Self {
        self.selected_cell_color = color;
        self
    }

    /// Set the border color.
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    /// Enable alternating row colors.
    pub fn alt_row_color(mut self, color: Color) -> Self {
        self.alt_row_color = Some(color);
        self
    }

    /// Set the focus state.
    ///
    /// When focused is false, the table will not show selection highlighting,
    /// making it suitable for non-interactive CLI output.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Set the initial cursor row position.
    pub fn with_cursor_row(mut self, row: usize) -> Self {
        self.cursor_row = row;
        self
    }

    /// Set the scroll offset.
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    /// Render the table as a string.
    ///
    /// This is a convenience method that returns the table view as a string
    /// without requiring the `Model` trait to be in scope.
    ///
    /// # Example
    ///
    /// ```rust
    /// use teapot::components::{Column, Table};
    ///
    /// let table = Table::new()
    ///     .columns(vec![Column::new("Name"), Column::new("Value")])
    ///     .rows(vec![vec!["foo".to_string(), "bar".to_string()]])
    ///     .focused(false);
    ///
    /// println!("{}", table.render());
    /// ```
    pub fn render(&self) -> String {
        use crate::Model;
        Model::view(self)
    }

    /// Get the current row cursor position.
    pub fn cursor_row(&self) -> usize {
        self.cursor_row
    }

    /// Get the current column cursor position.
    pub fn cursor_col(&self) -> usize {
        self.cursor_col
    }

    /// Get the total number of rows.
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Get the total number of columns.
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Get the selected row if submitted.
    pub fn selected_row(&self) -> Option<&Vec<String>> {
        if self.submitted && !self.rows.is_empty() { self.rows.get(self.cursor_row) } else { None }
    }

    /// Get the current row regardless of submit state.
    pub fn current_row(&self) -> Option<&Vec<String>> {
        self.rows.get(self.cursor_row)
    }

    /// Get the selected cell value if in cell selection mode and submitted.
    pub fn selected_cell(&self) -> Option<&str> {
        if self.submitted && self.cell_selection { self.current_cell() } else { None }
    }

    /// Get the current cell value.
    pub fn current_cell(&self) -> Option<&str> {
        self.rows.get(self.cursor_row).and_then(|row| row.get(self.cursor_col)).map(|s| s.as_str())
    }

    /// Check if submitted.
    pub fn is_submitted(&self) -> bool {
        self.submitted
    }

    /// Check if cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    /// Set focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Set rows dynamically.
    pub fn set_rows<I, R, S>(&mut self, rows: I)
    where
        I: IntoIterator<Item = R>,
        R: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.rows =
            rows.into_iter().map(|row| row.into_iter().map(|s| s.into()).collect()).collect();
        self.cursor_row = self.cursor_row.min(self.rows.len().saturating_sub(1));
        self.ensure_visible();
    }

    fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.ensure_visible();
        }
    }

    fn move_down(&mut self) {
        if self.cursor_row < self.rows.len().saturating_sub(1) {
            self.cursor_row += 1;
            self.ensure_visible();
        }
    }

    fn move_left(&mut self) {
        if self.cell_selection && self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    fn move_right(&mut self) {
        if self.cell_selection && self.cursor_col < self.columns.len().saturating_sub(1) {
            self.cursor_col += 1;
        }
    }

    fn move_first(&mut self) {
        self.cursor_row = 0;
        self.offset = 0;
    }

    fn move_last(&mut self) {
        self.cursor_row = self.rows.len().saturating_sub(1);
        self.ensure_visible();
    }

    fn page_up(&mut self) {
        let page_size = self.height.saturating_sub(1);
        self.cursor_row = self.cursor_row.saturating_sub(page_size);
        self.ensure_visible();
    }

    fn page_down(&mut self) {
        let page_size = self.height.saturating_sub(1);
        self.cursor_row = (self.cursor_row + page_size).min(self.rows.len().saturating_sub(1));
        self.ensure_visible();
    }

    fn ensure_visible(&mut self) {
        if self.cursor_row < self.offset {
            self.offset = self.cursor_row;
        }
        if self.cursor_row >= self.offset + self.height {
            self.offset = self.cursor_row.saturating_sub(self.height - 1);
        }
    }

    fn visible_range(&self) -> (usize, usize) {
        let start = self.offset;
        let end = (self.offset + self.height).min(self.rows.len());
        (start, end)
    }

    /// Calculate column widths (auto-size if width is 0).
    ///
    /// If a column is marked as `grow`, it will expand to fill remaining space.
    fn calculate_widths(&self) -> Vec<usize> {
        // First pass: calculate base widths for all columns
        let mut widths: Vec<usize> = self
            .columns
            .iter()
            .enumerate()
            .map(|(col_idx, col)| {
                if col.width > 0 {
                    col.width
                } else {
                    // Auto-calculate based on content
                    let header_len = col.title.chars().count();
                    let max_data_len = self
                        .rows
                        .iter()
                        .filter_map(|row| row.get(col_idx))
                        .map(|s| s.chars().count())
                        .max()
                        .unwrap_or(0);
                    header_len.max(max_data_len).max(3) // Minimum 3 chars
                }
            })
            .collect();

        // If we have a table width and a growth column, expand it
        if self.width > 0 {
            if let Some(grow_idx) = self.columns.iter().position(|c| c.grow) {
                // Calculate current total width
                // Each column has: space + content + space (2 chars padding per column)
                let padding_per_col = 2;
                let current_width: usize =
                    widths.iter().sum::<usize>() + widths.len() * padding_per_col;

                // Calculate remaining space
                let remaining = self.width.saturating_sub(current_width);

                // Add remaining space to growth column
                if remaining > 0 {
                    widths[grow_idx] += remaining;
                }
            }
        }

        widths
    }

    /// Align text within a width.
    fn align_text(&self, text: &str, width: usize, align: Align) -> String {
        let text_len = text.chars().count();
        if text_len >= width {
            text.chars().take(width).collect()
        } else {
            let padding = width - text_len;
            match align {
                Align::Left => format!("{}{}", text, " ".repeat(padding)),
                Align::Right => format!("{}{}", " ".repeat(padding), text),
                Align::Center => {
                    let left_pad = padding / 2;
                    let right_pad = padding - left_pad;
                    format!("{}{}{}", " ".repeat(left_pad), text, " ".repeat(right_pad))
                },
            }
        }
    }

    /// Render a horizontal border line.
    fn render_border(&self, widths: &[usize], left: &str, mid: &str, right: &str) -> String {
        let mut line = format!("{}{}", self.border_color.to_ansi_fg(), left);
        for (i, width) in widths.iter().enumerate() {
            line.push_str(&"─".repeat(*width + 2));
            if i < widths.len() - 1 {
                line.push_str(mid);
            }
        }
        line.push_str(right);
        line.push_str("\x1b[0m");
        line
    }

    /// Calculate the total content width of the table.
    fn calculate_content_width(&self, widths: &[usize]) -> usize {
        if widths.is_empty() {
            return 0;
        }

        if self.show_borders {
            // │ cell1 │ cell2 │ ... │
            // Each cell has +3 (space before, after, and border)
            // Plus 1 for leading border
            widths.len() + widths.iter().sum::<usize>() + widths.len() * 2
        } else {
            // " cell1  cell2  ... " (space around each cell)
            widths.iter().sum::<usize>() + widths.len() * 2
        }
    }

    /// Get the total content width of the table.
    ///
    /// This is useful for determining if horizontal scrolling is needed.
    pub fn content_width(&self) -> usize {
        let widths = self.calculate_widths();
        self.calculate_content_width(&widths)
    }

    /// Apply horizontal scroll to a line of text.
    ///
    /// Strips ANSI codes for width calculation, applies scroll offset,
    /// then clips to max width.
    fn apply_h_scroll(&self, line: &str, max_width: usize) -> String {
        if max_width == 0 && self.h_scroll_offset == 0 {
            return line.to_string();
        }

        // We need to handle ANSI escape codes properly
        // Strategy: render the full line, then slice by visible characters
        let mut result = String::new();
        let mut visible_pos = 0;
        let mut in_escape = false;
        let mut escape_seq = String::new();
        let mut active_style = String::new();

        let target_start = self.h_scroll_offset;
        let target_end = if max_width > 0 { self.h_scroll_offset + max_width } else { usize::MAX };

        for ch in line.chars() {
            if in_escape {
                escape_seq.push(ch);
                if ch == 'm' {
                    in_escape = false;
                    // Track active style for restoration after scroll
                    if escape_seq.contains("\x1b[0m") {
                        active_style.clear();
                    } else {
                        active_style = escape_seq.clone();
                    }
                    // Always include escape sequences in output if we're in visible range
                    if visible_pos >= target_start && visible_pos < target_end {
                        result.push_str(&escape_seq);
                    }
                    escape_seq.clear();
                }
            } else if ch == '\x1b' {
                in_escape = true;
                escape_seq.push(ch);
            } else {
                // Visible character
                if visible_pos >= target_start && visible_pos < target_end {
                    // If this is the first visible character and we have an active style,
                    // apply it
                    if visible_pos == target_start && !active_style.is_empty() && result.is_empty()
                    {
                        result.push_str(&active_style);
                    }
                    result.push(ch);
                }
                visible_pos += 1;

                if visible_pos >= target_end {
                    break;
                }
            }
        }

        // Always reset at the end if we output anything
        if !result.is_empty() && !result.ends_with("\x1b[0m") {
            result.push_str("\x1b[0m");
        }

        result
    }
}

impl Model for Table {
    type Message = TableMsg;

    fn init(&self) -> Option<Cmd<Self::Message>> {
        None
    }

    fn update(&mut self, msg: Self::Message) -> Option<Cmd<Self::Message>> {
        match msg {
            TableMsg::Up => self.move_up(),
            TableMsg::Down => self.move_down(),
            TableMsg::Left => self.move_left(),
            TableMsg::Right => self.move_right(),
            TableMsg::First => self.move_first(),
            TableMsg::Last => self.move_last(),
            TableMsg::PageUp => self.page_up(),
            TableMsg::PageDown => self.page_down(),
            TableMsg::Submit => {
                if !self.rows.is_empty() {
                    self.submitted = true;
                }
            },
            TableMsg::Cancel => self.cancelled = true,
            TableMsg::Focus => self.focused = true,
            TableMsg::Blur => self.focused = false,
        }
        None
    }

    fn view(&self) -> String {
        if self.columns.is_empty() {
            return format!(
                " {}Nothing to display.{} ",
                Color::BrightBlack.to_ansi_fg(),
                "\x1b[0m"
            );
        }

        let widths = self.calculate_widths();
        let effective_width = if self.width > 0 { self.width } else { 0 };
        let needs_scroll = effective_width > 0 || self.h_scroll_offset > 0;
        let mut output = String::new();

        // Top border
        if self.show_borders {
            let border = self.render_border(&widths, "┌", "┬", "┐");
            if needs_scroll {
                output.push_str(&self.apply_h_scroll(&border, effective_width));
            } else {
                output.push_str(&border);
            }
            output.push('\n');
        }

        // Header row
        if self.show_header {
            let mut header_line = String::new();
            if self.show_borders {
                header_line.push_str(&format!("{}│{}", self.border_color.to_ansi_fg(), "\x1b[0m"));
            }

            for (i, col) in self.columns.iter().enumerate() {
                let text = self.align_text(&col.title, widths[i], col.align);
                header_line.push_str(&format!(
                    "{}\x1b[1m {} {}\x1b[0m",
                    self.header_color.to_ansi_fg(),
                    text,
                    "\x1b[0m"
                ));
                if self.show_borders {
                    header_line.push_str(&format!(
                        "{}│{}",
                        self.border_color.to_ansi_fg(),
                        "\x1b[0m"
                    ));
                }
            }
            if needs_scroll {
                output.push_str(&self.apply_h_scroll(&header_line, effective_width));
            } else {
                output.push_str(&header_line);
            }
            output.push('\n');

            // Header separator
            if self.show_borders {
                let sep = self.render_border(&widths, "├", "┼", "┤");
                if needs_scroll {
                    output.push_str(&self.apply_h_scroll(&sep, effective_width));
                } else {
                    output.push_str(&sep);
                }
                output.push('\n');
            }
        }

        // Data rows
        if self.rows.is_empty() {
            output.push_str(&format!("{}(no data){}", Color::BrightBlack.to_ansi_fg(), "\x1b[0m"));
        } else {
            let (start, end) = self.visible_range();

            // Scroll indicator (top)
            if start > 0 {
                output.push_str(&format!(
                    "{}  ↑ {} more rows{}",
                    Color::BrightBlack.to_ansi_fg(),
                    start,
                    "\x1b[0m\n"
                ));
            }

            for (view_idx, row_idx) in (start..end).enumerate() {
                let row = &self.rows[row_idx];
                // Only show selection when focused
                let is_selected_row = self.focused && row_idx == self.cursor_row;

                // Determine row color
                let row_color = if is_selected_row {
                    self.selected_row_color.clone()
                } else if let Some(ref alt) = self.alt_row_color {
                    if row_idx % 2 == 1 { alt.clone() } else { self.row_color.clone() }
                } else {
                    self.row_color.clone()
                };

                let mut row_line = String::new();

                if self.show_borders {
                    row_line.push_str(&format!("{}│{}", self.border_color.to_ansi_fg(), "\x1b[0m"));
                }

                for (col_idx, col) in self.columns.iter().enumerate() {
                    let cell_value = row.get(col_idx).map(|s| s.as_str()).unwrap_or("");
                    let text = self.align_text(cell_value, widths[col_idx], col.align);

                    let is_selected_cell =
                        self.cell_selection && is_selected_row && col_idx == self.cursor_col;

                    let cell_color =
                        if is_selected_cell { &self.selected_cell_color } else { &row_color };

                    if is_selected_row {
                        row_line.push_str(&format!(
                            "{}\x1b[7m {} \x1b[27m{}",
                            cell_color.to_ansi_fg(),
                            text,
                            "\x1b[0m"
                        ));
                    } else {
                        row_line.push_str(&format!(
                            "{} {} {}",
                            cell_color.to_ansi_fg(),
                            text,
                            "\x1b[0m"
                        ));
                    }

                    if self.show_borders {
                        row_line.push_str(&format!(
                            "{}│{}",
                            self.border_color.to_ansi_fg(),
                            "\x1b[0m"
                        ));
                    }
                }

                // Apply horizontal scroll to the row and pad to width if needed
                if needs_scroll {
                    output.push_str(&self.apply_h_scroll(&row_line, effective_width));
                } else {
                    output.push_str(&row_line);
                }

                if view_idx < (end - start - 1) || self.show_borders {
                    output.push('\n');
                }
            }

            // Scroll indicator (bottom)
            let remaining = self.rows.len().saturating_sub(end);
            if remaining > 0 {
                if !self.show_borders {
                    output.push('\n');
                }
                output.push_str(&format!(
                    "{}  ↓ {} more rows{}",
                    Color::BrightBlack.to_ansi_fg(),
                    remaining,
                    "\x1b[0m"
                ));
            } else if self.show_borders {
                // Bottom border (only if no scroll indicator)
                let border = self.render_border(&widths, "└", "┴", "┘");
                if needs_scroll {
                    output.push_str(&self.apply_h_scroll(&border, effective_width));
                } else {
                    output.push_str(&border);
                }
            }
        }

        output
    }

    fn handle_event(&self, event: Event) -> Option<Self::Message> {
        if !self.focused {
            return None;
        }

        match event {
            Event::Key(key) => {
                // Handle control keys
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    return match key.code {
                        KeyCode::Char('n') => Some(TableMsg::Down),
                        KeyCode::Char('p') => Some(TableMsg::Up),
                        _ => None,
                    };
                }

                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => Some(TableMsg::Up),
                    KeyCode::Down | KeyCode::Char('j') => Some(TableMsg::Down),
                    KeyCode::Left | KeyCode::Char('h') => Some(TableMsg::Left),
                    KeyCode::Right | KeyCode::Char('l') => Some(TableMsg::Right),
                    KeyCode::Home => Some(TableMsg::First),
                    KeyCode::End => Some(TableMsg::Last),
                    KeyCode::PageUp => Some(TableMsg::PageUp),
                    KeyCode::PageDown => Some(TableMsg::PageDown),
                    KeyCode::Enter | KeyCode::Char(' ') => Some(TableMsg::Submit),
                    KeyCode::Esc | KeyCode::Char('q') => Some(TableMsg::Cancel),
                    _ => None,
                }
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_creation() {
        let table = Table::new()
            .columns(vec![Column::new("Name").width(10), Column::new("Age").width(5)])
            .rows(vec![vec!["Alice", "30"], vec!["Bob", "25"]]);

        assert_eq!(table.column_count(), 2);
        assert_eq!(table.row_count(), 2);
        assert_eq!(table.cursor_row(), 0);
    }

    #[test]
    fn test_table_navigation() {
        let mut table = Table::new().columns(vec![Column::new("Name")]).rows(vec![
            vec!["A"],
            vec!["B"],
            vec!["C"],
        ]);

        assert_eq!(table.cursor_row(), 0);
        table.move_down();
        assert_eq!(table.cursor_row(), 1);
        table.move_down();
        assert_eq!(table.cursor_row(), 2);
        table.move_down(); // Should not go past end
        assert_eq!(table.cursor_row(), 2);
        table.move_up();
        assert_eq!(table.cursor_row(), 1);
    }

    #[test]
    fn test_table_cell_selection() {
        let mut table = Table::new()
            .columns(vec![Column::new("A"), Column::new("B"), Column::new("C")])
            .rows(vec![vec!["1", "2", "3"]])
            .cell_selection(true);

        assert_eq!(table.cursor_col(), 0);
        table.move_right();
        assert_eq!(table.cursor_col(), 1);
        table.move_right();
        assert_eq!(table.cursor_col(), 2);
        table.move_right(); // Should not go past end
        assert_eq!(table.cursor_col(), 2);
        table.move_left();
        assert_eq!(table.cursor_col(), 1);
    }

    #[test]
    fn test_table_submit() {
        let mut table =
            Table::new().columns(vec![Column::new("Name")]).rows(vec![vec!["Alice"], vec!["Bob"]]);

        table.move_down();
        assert!(table.selected_row().is_none());
        table.submitted = true;
        assert_eq!(table.selected_row(), Some(&vec!["Bob".to_string()]));
    }

    #[test]
    fn test_table_pagination() {
        let mut table = Table::new()
            .columns(vec![Column::new("Num")])
            .rows((1..=10).map(|i| vec![i.to_string()]).collect::<Vec<_>>())
            .height(3);

        assert_eq!(table.visible_range(), (0, 3));
        table.move_down();
        table.move_down();
        table.move_down();
        assert_eq!(table.cursor_row(), 3);
        assert_eq!(table.visible_range(), (1, 4));

        table.move_last();
        assert_eq!(table.cursor_row(), 9);
        assert_eq!(table.visible_range(), (7, 10));
    }

    #[test]
    fn test_column_alignment() {
        let table = Table::new();

        assert_eq!(table.align_text("Hi", 5, Align::Left), "Hi   ");
        assert_eq!(table.align_text("Hi", 5, Align::Right), "   Hi");
        assert_eq!(table.align_text("Hi", 5, Align::Center), " Hi  ");
    }

    #[test]
    fn test_auto_width() {
        let table = Table::new()
            .columns(vec![Column::new("Name"), Column::new("Age")])
            .rows(vec![vec!["Alexander", "30"]]);

        let widths = table.calculate_widths();
        assert_eq!(widths[0], 9); // "Alexander" is longest
        assert_eq!(widths[1], 3); // "Age" header (min 3)
    }
}
