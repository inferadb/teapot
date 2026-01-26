//! Form system (Huh equivalent).
//!
//! Provides declarative form building with validation.
//!
//! # Example
//!
//! ```no_run
//! use teapot::forms::{Form, Group, Field};
//! use teapot::runtime::Program;
//!
//! let form = Form::new()
//!     .title("User Registration")
//!     .group(
//!         Group::new()
//!             .title("Personal Info")
//!             .field(Field::input().key("name").title("Your name").required(true).build())
//!             .field(Field::input().key("email").title("Email").build())
//!     )
//!     .group(
//!         Group::new()
//!             .title("Preferences")
//!             .field(Field::select()
//!                 .key("theme")
//!                 .title("Theme")
//!                 .options(vec!["Light".to_string(), "Dark".to_string(), "System".to_string()])
//!                 .build())
//!             .field(Field::confirm().key("newsletter").title("Subscribe to newsletter?").build())
//!     );
//!
//! // Run the form with Program
//! Program::new(form).run();
//! ```

mod field;
mod form;
mod group;
mod validation;

// Re-export field builders
#[allow(deprecated)]
pub use field::{
    ConfirmField, Field, FieldKind, FieldValue, FilePickerField, InputField, MultiSelectField,
    Note, NoteField, SelectField,
};
pub use form::{Form, FormLayout, FormMsg, FormResults};
pub use group::Group;
pub use validation::{Validator, ValidatorFn};
