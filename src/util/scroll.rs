//! Scroll state management for scrollable views.
//!
//! Provides a reusable state container for managing vertical and horizontal
//! scrolling with row selection in table-like views.

/// State for managing scrollable content with row selection.
///
/// This utility encapsulates the common scroll state pattern used by
/// table views, list views, and other scrollable components.
///
/// # Example
///
/// ```rust
/// use teapot::util::ScrollState;
///
/// let mut scroll = ScrollState::new();
///
/// // Navigate through 100 rows with 20 visible
/// scroll.select_next(100, 20);
/// scroll.select_prev();
/// scroll.page_down(100, 20);
///
/// // Use with a table
/// let selected = scroll.selected();
/// let offset = scroll.offset();
/// ```
#[derive(Debug, Clone, Default)]
pub struct ScrollState {
    /// Currently selected row index.
    selected: usize,
    /// Vertical scroll offset.
    offset: usize,
    /// Horizontal scroll offset.
    h_offset: usize,
}

impl ScrollState {
    /// Create a new scroll state at position 0.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected row index.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Get the vertical scroll offset.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Get the horizontal scroll offset.
    pub fn h_offset(&self) -> usize {
        self.h_offset
    }

    /// Set the selected row directly.
    pub fn set_selected(&mut self, row: usize) {
        self.selected = row;
    }

    /// Set the vertical offset directly.
    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }

    /// Set the horizontal offset directly.
    pub fn set_h_offset(&mut self, offset: usize) {
        self.h_offset = offset;
    }

    /// Reset scroll state to initial position.
    pub fn reset(&mut self) {
        self.selected = 0;
        self.offset = 0;
        self.h_offset = 0;
    }

    /// Move selection up by one row.
    ///
    /// Automatically adjusts vertical scroll offset to keep selection visible.
    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            if self.selected < self.offset {
                self.offset = self.selected;
            }
        }
    }

    /// Move selection down by one row.
    ///
    /// # Arguments
    ///
    /// * `row_count` - Total number of rows in the content
    /// * `visible` - Number of rows visible in the viewport
    pub fn select_next(&mut self, row_count: usize, visible: usize) {
        if self.selected < row_count.saturating_sub(1) {
            self.selected += 1;
            if self.selected >= self.offset + visible {
                self.offset = self.selected.saturating_sub(visible - 1);
            }
        }
    }

    /// Page up - move selection up by nearly a full page.
    ///
    /// # Arguments
    ///
    /// * `visible` - Number of rows visible in the viewport
    pub fn page_up(&mut self, visible: usize) {
        let page_size = visible.saturating_sub(1);
        self.selected = self.selected.saturating_sub(page_size);
        if self.selected < self.offset {
            self.offset = self.selected;
        }
    }

    /// Page down - move selection down by nearly a full page.
    ///
    /// # Arguments
    ///
    /// * `row_count` - Total number of rows in the content
    /// * `visible` - Number of rows visible in the viewport
    pub fn page_down(&mut self, row_count: usize, visible: usize) {
        let page_size = visible.saturating_sub(1);
        self.selected = (self.selected + page_size).min(row_count.saturating_sub(1));
        if self.selected >= self.offset + visible {
            self.offset = self.selected.saturating_sub(visible - 1);
        }
    }

    /// Scroll left by the given step size.
    ///
    /// # Arguments
    ///
    /// * `step` - Number of columns to scroll left
    pub fn scroll_left(&mut self, step: usize) {
        self.h_offset = self.h_offset.saturating_sub(step);
    }

    /// Scroll right by the given step size.
    ///
    /// # Arguments
    ///
    /// * `step` - Number of columns to scroll right
    /// * `max` - Maximum horizontal offset (content_width - viewport_width)
    pub fn scroll_right(&mut self, step: usize, max: usize) {
        if self.h_offset + step <= max {
            self.h_offset += step;
        } else {
            self.h_offset = max;
        }
    }

    /// Clamp scroll positions to valid ranges.
    ///
    /// Call this after content changes to ensure positions are valid.
    ///
    /// # Arguments
    ///
    /// * `row_count` - Total number of rows in the content
    /// * `visible` - Number of rows visible in the viewport
    pub fn clamp(&mut self, row_count: usize, visible: usize) {
        let max_scroll = row_count.saturating_sub(visible);
        self.offset = self.offset.min(max_scroll);
        self.selected = self.selected.min(row_count.saturating_sub(1));
    }

    /// Clamp horizontal scroll position.
    ///
    /// # Arguments
    ///
    /// * `max` - Maximum horizontal offset
    pub fn clamp_horizontal(&mut self, max: usize) {
        self.h_offset = self.h_offset.min(max);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state() {
        let state = ScrollState::new();
        assert_eq!(state.selected(), 0);
        assert_eq!(state.offset(), 0);
        assert_eq!(state.h_offset(), 0);
    }

    #[test]
    fn test_select_next() {
        let mut state = ScrollState::new();
        // 10 rows, 5 visible
        state.select_next(10, 5);
        assert_eq!(state.selected(), 1);
        assert_eq!(state.offset(), 0);

        // Select past visible area
        for _ in 0..5 {
            state.select_next(10, 5);
        }
        assert_eq!(state.selected(), 6);
        assert_eq!(state.offset(), 2); // offset adjusted to keep selection visible

        // Can't go past end
        for _ in 0..10 {
            state.select_next(10, 5);
        }
        assert_eq!(state.selected(), 9);
    }

    #[test]
    fn test_select_prev() {
        let mut state = ScrollState::new();
        state.set_selected(5);
        state.set_offset(3);

        state.select_prev();
        assert_eq!(state.selected(), 4);
        assert_eq!(state.offset(), 3); // offset unchanged

        // Select past visible area going up
        state.select_prev();
        state.select_prev();
        assert_eq!(state.selected(), 2);
        assert_eq!(state.offset(), 2); // offset adjusted

        // Can't go below 0
        for _ in 0..10 {
            state.select_prev();
        }
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn test_page_down() {
        let mut state = ScrollState::new();
        // 20 rows, 5 visible
        state.page_down(20, 5);
        assert_eq!(state.selected(), 4); // page_size = visible - 1 = 4
        assert_eq!(state.offset(), 0); // still in view

        state.page_down(20, 5);
        assert_eq!(state.selected(), 8);
        assert_eq!(state.offset(), 4); // adjusted to keep in view
    }

    #[test]
    fn test_page_up() {
        let mut state = ScrollState::new();
        state.set_selected(10);
        state.set_offset(6);

        state.page_up(5); // 5 visible
        assert_eq!(state.selected(), 6); // page_size = 4
        assert_eq!(state.offset(), 6);

        state.page_up(5);
        assert_eq!(state.selected(), 2);
        assert_eq!(state.offset(), 2); // adjusted
    }

    #[test]
    fn test_scroll_horizontal() {
        let mut state = ScrollState::new();

        state.scroll_right(4, 20);
        assert_eq!(state.h_offset(), 4);

        state.scroll_right(4, 20);
        assert_eq!(state.h_offset(), 8);

        // Can't go past max
        state.scroll_right(100, 20);
        assert_eq!(state.h_offset(), 20);

        state.scroll_left(4);
        assert_eq!(state.h_offset(), 16);

        // Can't go below 0
        state.scroll_left(100);
        assert_eq!(state.h_offset(), 0);
    }

    #[test]
    fn test_clamp() {
        let mut state = ScrollState::new();
        state.set_selected(50);
        state.set_offset(40);

        // Content shrinks to 10 rows, 5 visible
        state.clamp(10, 5);
        assert_eq!(state.selected(), 9); // clamped to last row
        assert_eq!(state.offset(), 5); // clamped to max scroll
    }

    #[test]
    fn test_reset() {
        let mut state = ScrollState::new();
        state.set_selected(10);
        state.set_offset(5);
        state.set_h_offset(8);

        state.reset();
        assert_eq!(state.selected(), 0);
        assert_eq!(state.offset(), 0);
        assert_eq!(state.h_offset(), 0);
    }
}
