//! Reusable UI components (Bubbles equivalent).
//!
//! # Fluent API Design
//!
//! Components use a fluent API pattern with self-consuming setters:
//!
//! ```rust
//! use teapot::components::TextInput;
//!
//! let input = TextInput::new()
//!     .placeholder("Enter your name...")
//!     .prompt("> ");
//! ```
//!
//! This pattern is used consistently across all components.
//!
//! This module provides composable widgets that implement the Model trait:
//!
//! - [`Spinner`] - Animated loading indicator
//! - [`Progress`] - Progress bar
//! - [`TextInput`] - Single-line text input
//! - [`TextArea`] - Multi-line text input
//! - [`Select`] - Single option selection
//! - [`MultiSelect`] - Multiple option selection
//! - [`Confirm`] - Yes/No confirmation
//! - [`Viewport`] - Scrollable content area
//! - [`List`] - Filterable, paginated list
//! - [`Table`] - Scrollable data table
//! - [`MultiProgress`] - Multiple parallel progress bars
//! - [`FilePicker`] - File/directory browser
//! - [`TabBar`] - Horizontal tab bar with keyboard hints
//! - [`StatusBadge`] - Colored status indicator
//! - [`Modal`] - Centered overlay dialog
//! - [`TaskList`] - Step-by-step task list with spinners
//! - [`TaskProgressView`] - Full-screen task progress with worker execution
//! - [`TitleBar`] - Decorative title bar with slash separators
//! - [`FooterHints`] - Keyboard shortcut hints footer

pub mod confirm;
pub mod file_picker;
pub mod footer_hints;
pub mod list;
pub mod modal;
pub mod multi_progress;
pub mod multi_select;
pub mod progress;
pub mod select;
pub mod spinner;
pub mod status_badge;
pub mod tab_bar;
pub mod table;
pub mod task_list;
pub mod task_progress;
pub mod text_area;
pub mod text_input;
pub mod title_bar;
pub mod viewport;

pub use confirm::{Confirm, ConfirmMsg};
pub use file_picker::{FileEntry, FilePicker, FilePickerMsg};
pub use footer_hints::{FooterHints, FooterHintsMsg};
pub use list::{List, ListMsg};
pub use modal::{Modal, ModalBorder, ModalHint};
pub use multi_progress::{MultiProgress, MultiProgressMsg, Task, TaskStatus};
pub use multi_select::{MultiSelect, MultiSelectMsg};
pub use progress::{Progress, ProgressMsg};
pub use select::{Select, SelectMsg};
pub use spinner::{Spinner, SpinnerMsg, SpinnerStyle};
pub use status_badge::{BadgeVariant, StatusBadge, StatusBadgeMsg};
pub use tab_bar::{Tab, TabBar, TabBarMsg};
pub use table::{Align, Column, Table, TableMsg};
pub use task_list::{TaskItem, TaskList, TaskListMsg, TaskState};
pub use task_progress::{
    ConfirmationConfig, HintConfig, Phase, StepExecutor, StepResult, TaskProgressConfig,
    TaskProgressMsg, TaskProgressView, TaskStep,
};
pub use text_area::{CursorPos, TextArea, TextAreaMsg};
pub use text_input::{TextInput, TextInputMsg};
pub use title_bar::{TitleBar, TitleBarMsg};
pub use viewport::{Viewport, ViewportMsg};

/// Common component message types.
#[derive(Debug, Clone)]
pub enum ComponentMsg {
    /// The component was focused.
    Focus,
    /// The component was blurred.
    Blur,
    /// A tick for animation.
    Tick,
    /// The component was submitted.
    Submit,
    /// The component was cancelled.
    Cancel,
}
