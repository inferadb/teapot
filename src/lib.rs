// Library crate: public API may not be used internally
#![allow(dead_code)]

//! # Ferment
//!
//! A Rust-native terminal UI framework following the Elm Architecture,
//! inspired by the Charm.sh ecosystem (Bubble Tea, Bubbles, Huh).
//!
//! The name "Ferment" is a nod to the bubbly nature of Bubble Tea—fermentation
//! creates bubbles, after all.
//!
//! ## Architecture
//!
//! This framework implements the Model-Update-View pattern:
//!
//! - **Model**: Your application state
//! - **Update**: Handle messages and update state
//! - **View**: Render state as a string
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use ferment::{Model, Program, Cmd, KeyCode, Event};
//!
//! struct Counter {
//!     count: i32,
//! }
//!
//! enum Msg {
//!     Increment,
//!     Decrement,
//!     Quit,
//! }
//!
//! impl Model for Counter {
//!     type Message = Msg;
//!
//!     fn init(&self) -> Option<Cmd<Self::Message>> {
//!         None
//!     }
//!
//!     fn update(&mut self, msg: Self::Message) -> Option<Cmd<Self::Message>> {
//!         match msg {
//!             Msg::Increment => self.count += 1,
//!             Msg::Decrement => self.count -= 1,
//!             Msg::Quit => return Some(Cmd::quit()),
//!         }
//!         None
//!     }
//!
//!     fn view(&self) -> String {
//!         format!("Count: {}\n\nPress +/- to change, q to quit", self.count)
//!     }
//!
//!     fn handle_event(&self, event: Event) -> Option<Self::Message> {
//!         match event {
//!             Event::Key(key) => match key.code {
//!                 KeyCode::Char('+') | KeyCode::Char('=') => Some(Msg::Increment),
//!                 KeyCode::Char('-') => Some(Msg::Decrement),
//!                 KeyCode::Char('q') | KeyCode::Esc => Some(Msg::Quit),
//!                 _ => None,
//!             },
//!             _ => None,
//!         }
//!     }
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let counter = Counter { count: 0 };
//!     Program::new(counter).run()?;
//!     Ok(())
//! }
//! ```
//!
//! ## Components
//!
//! The `components` module provides reusable UI widgets:
//!
//! - [`Spinner`] - Animated loading indicators
//! - [`Progress`] - Progress bars
//! - [`TextInput`] - Single-line text input
//! - [`TextArea`] - Multi-line text input
//! - [`Select`] - Single option selection
//! - [`MultiSelect`] - Multiple option selection
//! - [`Confirm`] - Yes/No confirmation
//! - [`List`] - Filterable, paginated list
//! - [`Table`] - Scrollable data table
//! - [`MultiProgress`] - Multiple parallel progress bars
//!
//! ## Forms
//!
//! The `forms` module provides declarative form building:
//!
//! ```rust,ignore
//! use ferment::forms::{Form, Group, Input, Select, Confirm};
//!
//! let form = Form::new()
//!     .group(
//!         Group::new()
//!             .field(Input::new("name").title("Your name"))
//!             .field(Select::new("color").title("Favorite color")
//!                 .options(["Red", "Green", "Blue"]))
//!     )
//!     .group(
//!         Group::new()
//!             .field(Confirm::new("agree").title("Do you agree?"))
//!     );
//! ```
//!
//! ## CI/Script Compatibility
//!
//! The framework automatically detects non-interactive environments and
//! adjusts behavior accordingly (no animations, no prompts, clear errors).
//!
//! ## Accessible Mode
//!
//! For screen reader users and other accessible environments, set the
//! `ACCESSIBLE=1` environment variable. This enables:
//!
//! - Plain text prompts without ANSI formatting
//! - Numbered options for selection components
//! - Line-based input instead of raw terminal mode
//!
//! Forms can be run in accessible mode:
//!
//! ```rust,ignore
//! let mut form = Form::new()
//!     .group(Group::new()
//!         .field(InputField::new("name").title("Your Name").build()));
//!
//! if let Some(results) = form.run_accessible()? {
//!     println!("Name: {}", results.get_string("name").unwrap_or(""));
//! }
//! ```
//!
//! Components implement the [`Accessible`] trait for custom accessible handling.

pub mod components;
pub mod forms;
pub mod output;
pub mod runtime;
pub mod style;
pub mod terminal;
pub mod util;

// Re-export core types at crate root
// Re-export commonly used components
pub use components::{
    BadgeVariant, Column, Confirm, FilePicker, FooterHints, List, MultiProgress, MultiSelect,
    Progress, Select, Spinner, StatusBadge, Tab, TabBar, Table, TaskList, TaskProgressView,
    TaskStep, TextArea, TextInput, TitleBar,
};
// Re-export form types
pub use forms::{Form, Group};
// Re-export cmd module for Bubble Tea-style function calls
pub use runtime::cmd;
pub use runtime::{Accessible, AccessibleInput, Cmd, Model, Program, ProgramOptions, Sub};
// Re-export ANSI constants for convenience
pub use style::{
    BLINK, BOLD, CLEAR_LINE, CR, CURSOR_UP, DIM, HIDDEN, ITALIC, RESET, REVERSE, STRIKETHROUGH,
    UNDERLINE,
};
// Re-export style types
pub use style::{Border, Color, Style};
pub use terminal::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent};
// Re-export utility types
pub use util::{ManagedWorker, ScrollState, WorkerHandle};
