//! Form system (Huh equivalent).
//!
//! Provides declarative form building with validation.
//!
//! # Example
//!
//! ```rust,ignore
//! use ferment::forms::{Form, Group, Input, Select, Confirm};
//!
//! let form = Form::new()
//!     .title("User Registration")
//!     .group(
//!         Group::new()
//!             .title("Personal Info")
//!             .field(Input::new("name").title("Your name").required())
//!             .field(Input::new("email").title("Email").validate(|v| {
//!                 if v.contains('@') { Ok(()) } else { Err("Invalid email") }
//!             }))
//!     )
//!     .group(
//!         Group::new()
//!             .title("Preferences")
//!             .field(Select::new("theme").title("Theme")
//!                 .options(["Light", "Dark", "System"]))
//!             .field(Confirm::new("newsletter").title("Subscribe to newsletter?"))
//!     );
//!
//! let results = form.run()?;
//! let name = results.get_string("name")?;
//! let theme = results.get_string("theme")?;
//! ```

mod field;
mod form;
mod group;
mod validation;

// Re-export field builders
pub use field::{
    ConfirmField, FilePickerField, InputField, MultiSelectField, NoteField, SelectField,
};
pub use field::{Field, FieldKind, FieldValue, Note};
pub use form::{Form, FormLayout, FormMsg, FormResults};
pub use group::Group;
pub use validation::{Validator, ValidatorFn};
