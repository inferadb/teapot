//! Core runtime implementing the Elm Architecture.
//!
//! This module provides:
//! - [`Model`] - The core trait for application state
//! - [`Cmd`] - Commands for side effects
//! - [`Program`] - The runtime that manages the event loop

mod command;
mod message;
mod program;

pub use command::Cmd;
pub use message::CommonMsg;
pub use program::{Program, ProgramOptions};

use crate::terminal::Event;

/// The core Model trait - equivalent to `tea.Model` in Bubble Tea.
///
/// Every TUI application implements this trait on their state type.
/// The framework calls these methods in a loop:
///
/// 1. `init()` - Called once at startup
/// 2. `handle_event()` - Convert terminal events to messages
/// 3. `update()` - Handle messages and update state
/// 4. `view()` - Render the current state
///
/// # Example
///
/// ```rust
/// use ferment::{Model, Cmd, Event, KeyCode};
///
/// struct App {
///     text: String,
/// }
///
/// enum Msg {
///     Append(char),
///     Clear,
///     Quit,
/// }
///
/// impl Model for App {
///     type Message = Msg;
///
///     fn init(&self) -> Option<Cmd<Self::Message>> {
///         None // No startup command
///     }
///
///     fn update(&mut self, msg: Self::Message) -> Option<Cmd<Self::Message>> {
///         match msg {
///             Msg::Append(c) => self.text.push(c),
///             Msg::Clear => self.text.clear(),
///             Msg::Quit => return Some(Cmd::quit()),
///         }
///         None
///     }
///
///     fn view(&self) -> String {
///         format!("Text: {}", self.text)
///     }
///
///     fn handle_event(&self, event: Event) -> Option<Self::Message> {
///         match event {
///             Event::Key(key) => match key.code {
///                 KeyCode::Char('c') if key.modifiers.contains(ferment::KeyModifiers::CONTROL) => {
///                     Some(Msg::Clear)
///                 }
///                 KeyCode::Char(c) => Some(Msg::Append(c)),
///                 KeyCode::Esc => Some(Msg::Quit),
///                 _ => None,
///             },
///             _ => None,
///         }
///     }
/// }
/// ```
pub trait Model: Sized {
    /// The message type for this model.
    ///
    /// Messages represent events and actions that can update the model.
    type Message: Send + 'static;

    /// Initialize the model, returning an optional startup command.
    ///
    /// This is called once when the program starts. Use it to:
    /// - Start timers
    /// - Fetch initial data
    /// - Begin animations
    ///
    /// Return `None` if no startup action is needed.
    fn init(&self) -> Option<Cmd<Self::Message>>;

    /// Handle a message and update the model state.
    ///
    /// This is the core of your application logic. Messages come from:
    /// - User input (via `handle_event`)
    /// - Command completions
    /// - Timers
    ///
    /// Return an optional command for side effects.
    fn update(&mut self, msg: Self::Message) -> Option<Cmd<Self::Message>>;

    /// Render the model as a string for display.
    ///
    /// This is called after every update to redraw the UI.
    /// The returned string is the complete UI representation.
    ///
    /// # Performance
    ///
    /// This is called frequently, so avoid expensive operations.
    /// The framework handles diffing and efficient updates.
    fn view(&self) -> String;

    /// Convert terminal events to messages.
    ///
    /// Override this to handle keyboard, mouse, and resize events.
    /// Return `Some(msg)` to trigger an update, or `None` to ignore.
    ///
    /// The default implementation ignores all events.
    fn handle_event(&self, _event: Event) -> Option<Self::Message> {
        None
    }

    /// Whether this model should receive tick updates.
    ///
    /// If true, the model will receive periodic tick messages.
    /// Override this if your model needs animation frames.
    fn wants_tick(&self) -> bool {
        false
    }
}
