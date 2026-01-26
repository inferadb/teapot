//! # Teapot
//!
//! A Rust-native terminal UI framework following the Elm Architecture,
//! inspired by the Charm.sh ecosystem (Bubble Tea, Bubbles, Huh).
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
//! use teapot::{Model, Program, Cmd, KeyCode, Event};
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
//! The [`components`] module provides reusable UI widgets:
//!
//! - [`components::Spinner`] - Animated loading indicators
//! - [`components::Progress`] - Progress bars
//! - [`components::TextInput`] - Single-line text input
//! - [`components::TextArea`] - Multi-line text input
//! - [`components::Select`] - Single option selection
//! - [`components::MultiSelect`] - Multiple option selection
//! - [`components::Confirm`] - Yes/No confirmation
//! - [`components::List`] - Filterable, paginated list
//! - [`components::Table`] - Scrollable data table
//! - [`components::MultiProgress`] - Multiple parallel progress bars
//!
//! ## Forms
//!
//! The `forms` module provides declarative form building:
//!
//! ```no_run
//! use teapot::forms::{Form, Group, Field};
//!
//! let form = Form::new()
//!     .group(
//!         Group::new()
//!             .field(Field::input().key("name").title("Your name").build())
//!             .field(Field::select()
//!                 .key("color")
//!                 .title("Favorite color")
//!                 .options(vec!["Red".to_string(), "Green".to_string(), "Blue".to_string()])
//!                 .build())
//!     )
//!     .group(
//!         Group::new()
//!             .field(Field::confirm().key("agree").title("Do you agree?").build())
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
//! ```no_run
//! use teapot::forms::{Form, Group, Field};
//!
//! fn main() -> Result<(), teapot::Error> {
//!     let mut form = Form::new()
//!         .group(Group::new()
//!             .field(Field::input().key("name").title("Your Name").build()));
//!
//!     if let Some(results) = form.run_accessible()? {
//!         println!("Name: {}", results.get_string("name").unwrap_or(""));
//!     }
//!     Ok(())
//! }
//! ```
//!
//! Components implement the [`runtime::Accessible`] trait for custom accessible handling.

pub mod components;
pub mod error;
pub mod forms;
pub mod output;
pub mod runtime;
pub mod style;
pub mod terminal;
pub mod util;

// Core framework types only at crate root.
// Import components via `teapot::components::*`
// Import styles via `teapot::style::*`
// Import forms via `teapot::forms::*`
pub use error::{Error, Result};
pub use runtime::{Cmd, Model, Program, Sub};
pub use terminal::{Event, KeyCode, KeyModifiers};
