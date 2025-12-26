//! Reusable UI components (Bubbles equivalent).
//!
//! This module provides composable widgets that implement the Model trait:
//!
//! - [`Spinner`] - Animated loading indicator
//! - [`Progress`] - Progress bar
//! - [`TextInput`] - Single-line text input
//! - [`Select`] - Single option selection
//! - [`MultiSelect`] - Multiple option selection
//! - [`Confirm`] - Yes/No confirmation
//! - [`Viewport`] - Scrollable content area

pub mod confirm;
pub mod multi_select;
pub mod progress;
pub mod select;
pub mod spinner;
pub mod text_input;
pub mod viewport;

pub use confirm::{Confirm, ConfirmMsg};
pub use multi_select::{MultiSelect, MultiSelectMsg};
pub use progress::{Progress, ProgressMsg};
pub use select::{Select, SelectMsg};
pub use spinner::{Spinner, SpinnerMsg, SpinnerStyle};
pub use text_input::{TextInput, TextInputMsg};
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
