//! Common message types used by the runtime.

use std::time::Instant;

use crate::terminal::Event;

/// Common messages that can be used by any model.
///
/// These are pre-defined messages for common scenarios.
/// You can use these directly or create your own message types.
#[derive(Debug, Clone)]
pub enum CommonMsg {
    /// A terminal event occurred (key press, mouse, resize).
    Event(Event),

    /// A tick timer fired.
    Tick(Instant),

    /// Request to quit the application.
    Quit,

    /// The terminal was resized.
    Resize { width: u16, height: u16 },

    /// Focus was gained.
    FocusGained,

    /// Focus was lost.
    FocusLost,
}

impl CommonMsg {
    /// Check if this is a quit message.
    pub fn is_quit(&self) -> bool {
        matches!(self, CommonMsg::Quit)
    }

    /// Check if this is a resize message.
    pub fn is_resize(&self) -> bool {
        matches!(self, CommonMsg::Resize { .. })
    }
}
