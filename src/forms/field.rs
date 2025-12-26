//! Form field types.

use super::validation::Validator;
use crate::components::{Confirm, MultiSelect, Select, TextInput};
use crate::runtime::{Cmd, Model};
use crate::terminal::Event;

/// A field value that can be stored in form results.
#[derive(Debug, Clone)]
pub enum FieldValue {
    /// A string value.
    String(String),
    /// A boolean value.
    Bool(bool),
    /// A list of selected strings.
    StringList(Vec<String>),
    /// An integer value.
    Int(i64),
    /// No value.
    None,
}

impl FieldValue {
    /// Get as string.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            FieldValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get as bool.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            FieldValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Get as string list.
    pub fn as_string_list(&self) -> Option<&[String]> {
        match self {
            FieldValue::StringList(list) => Some(list),
            _ => None,
        }
    }
}

/// The kind of field.
#[derive(Debug, Clone)]
pub enum FieldKind {
    /// Text input field.
    Input,
    /// Select field.
    Select,
    /// Multi-select field.
    MultiSelect,
    /// Confirmation field.
    Confirm,
}

/// Message type for fields.
#[derive(Debug, Clone)]
pub enum FieldMsg {
    /// Text input message.
    Input(crate::components::text_input::TextInputMsg),
    /// Select message.
    Select(crate::components::select::SelectMsg),
    /// Multi-select message.
    MultiSelect(crate::components::multi_select::MultiSelectMsg),
    /// Confirm message.
    Confirm(crate::components::confirm::ConfirmMsg),
}

/// A form field.
#[derive(Debug, Clone)]
pub struct Field {
    /// Field identifier.
    pub key: String,
    /// Field title.
    pub title: String,
    /// Field description.
    pub description: Option<String>,
    /// Whether the field is required.
    pub required: bool,
    /// The field kind and state.
    pub inner: FieldInner,
}

/// Inner field state.
#[derive(Debug, Clone)]
pub enum FieldInner {
    Input(TextInput),
    Select(Select<String>),
    MultiSelect(MultiSelect<String>),
    Confirm(Confirm),
}

impl Field {
    /// Get the field key.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Get the current value.
    pub fn value(&self) -> FieldValue {
        match &self.inner {
            FieldInner::Input(input) => FieldValue::String(input.get_value().to_string()),
            FieldInner::Select(select) => {
                if let Some(val) = select.selected() {
                    FieldValue::String(val.clone())
                } else if let Some(val) = select.current() {
                    FieldValue::String(val.clone())
                } else {
                    FieldValue::None
                }
            }
            FieldInner::MultiSelect(select) => {
                FieldValue::StringList(select.selected().iter().map(|s| (*s).clone()).collect())
            }
            FieldInner::Confirm(confirm) => FieldValue::Bool(confirm.value()),
        }
    }

    /// Check if the field is submitted.
    pub fn is_submitted(&self) -> bool {
        match &self.inner {
            FieldInner::Input(input) => input.is_submitted(),
            FieldInner::Select(select) => select.is_submitted(),
            FieldInner::MultiSelect(select) => select.is_submitted(),
            FieldInner::Confirm(confirm) => confirm.is_submitted(),
        }
    }

    /// Check if the field is cancelled.
    pub fn is_cancelled(&self) -> bool {
        match &self.inner {
            FieldInner::Input(_) => false, // Input doesn't have cancel
            FieldInner::Select(select) => select.is_cancelled(),
            FieldInner::MultiSelect(select) => select.is_cancelled(),
            FieldInner::Confirm(confirm) => confirm.is_cancelled(),
        }
    }

    /// Set focus state.
    pub fn set_focused(&mut self, focused: bool) {
        match &mut self.inner {
            FieldInner::Input(input) => input.set_focused(focused),
            FieldInner::Select(select) => select.set_focused(focused),
            FieldInner::MultiSelect(select) => select.set_focused(focused),
            FieldInner::Confirm(confirm) => confirm.set_focused(focused),
        }
    }
}

impl Model for Field {
    type Message = FieldMsg;

    fn init(&self) -> Option<Cmd<Self::Message>> {
        match &self.inner {
            FieldInner::Input(input) => input.init().map(|c| c.map(FieldMsg::Input)),
            FieldInner::Select(select) => select.init().map(|c| c.map(FieldMsg::Select)),
            FieldInner::MultiSelect(select) => select.init().map(|c| c.map(FieldMsg::MultiSelect)),
            FieldInner::Confirm(confirm) => confirm.init().map(|c| c.map(FieldMsg::Confirm)),
        }
    }

    fn update(&mut self, msg: Self::Message) -> Option<Cmd<Self::Message>> {
        match (&mut self.inner, msg) {
            (FieldInner::Input(input), FieldMsg::Input(msg)) => {
                input.update(msg).map(|c| c.map(FieldMsg::Input))
            }
            (FieldInner::Select(select), FieldMsg::Select(msg)) => {
                select.update(msg).map(|c| c.map(FieldMsg::Select))
            }
            (FieldInner::MultiSelect(select), FieldMsg::MultiSelect(msg)) => {
                select.update(msg).map(|c| c.map(FieldMsg::MultiSelect))
            }
            (FieldInner::Confirm(confirm), FieldMsg::Confirm(msg)) => {
                confirm.update(msg).map(|c| c.map(FieldMsg::Confirm))
            }
            _ => None,
        }
    }

    fn view(&self) -> String {
        let mut output = String::new();

        // Title
        if !self.title.is_empty() {
            output.push_str(&self.title);
            if self.required {
                output.push_str(" *");
            }
            output.push('\n');
        }

        // Description
        if let Some(desc) = &self.description {
            output.push_str(&format!(
                "{}{}{}\n",
                crate::style::Color::BrightBlack.to_ansi_fg(),
                desc,
                "\x1b[0m"
            ));
        }

        // Field view
        match &self.inner {
            FieldInner::Input(input) => output.push_str(&input.view()),
            FieldInner::Select(select) => output.push_str(&select.view()),
            FieldInner::MultiSelect(select) => output.push_str(&select.view()),
            FieldInner::Confirm(confirm) => output.push_str(&confirm.view()),
        }

        output
    }

    fn handle_event(&self, event: Event) -> Option<Self::Message> {
        match &self.inner {
            FieldInner::Input(input) => input.handle_event(event).map(FieldMsg::Input),
            FieldInner::Select(select) => select.handle_event(event).map(FieldMsg::Select),
            FieldInner::MultiSelect(select) => {
                select.handle_event(event).map(FieldMsg::MultiSelect)
            }
            FieldInner::Confirm(confirm) => confirm.handle_event(event).map(FieldMsg::Confirm),
        }
    }
}

/// Builder for text input fields.
#[derive(Debug, Clone)]
pub struct InputField {
    key: String,
    title: String,
    description: Option<String>,
    placeholder: String,
    default: String,
    required: bool,
    hidden: bool,
    validators: Vec<Validator<String>>,
}

impl InputField {
    /// Create a new input field.
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            title: String::new(),
            description: None,
            placeholder: String::new(),
            default: String::new(),
            required: false,
            hidden: false,
            validators: Vec::new(),
        }
    }

    /// Set the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the placeholder.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set the default value.
    pub fn default(mut self, default: impl Into<String>) -> Self {
        self.default = default.into();
        self
    }

    /// Mark as required.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Mark as hidden (password).
    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    /// Add a validator.
    pub fn validate(mut self, validator: Validator<String>) -> Self {
        self.validators.push(validator);
        self
    }

    /// Build the field.
    pub fn build(self) -> Field {
        let input = TextInput::new()
            .placeholder(self.placeholder)
            .value(self.default)
            .hidden(self.hidden);

        Field {
            key: self.key,
            title: self.title,
            description: self.description,
            required: self.required,
            inner: FieldInner::Input(input),
        }
    }
}

/// Builder for select fields.
#[derive(Debug, Clone)]
pub struct SelectField {
    key: String,
    title: String,
    description: Option<String>,
    options: Vec<String>,
    default: Option<usize>,
}

impl SelectField {
    /// Create a new select field.
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            title: String::new(),
            description: None,
            options: Vec::new(),
            default: None,
        }
    }

    /// Set the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the options.
    pub fn options<I, S>(mut self, options: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.options = options.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Set the default index.
    pub fn default(mut self, index: usize) -> Self {
        self.default = Some(index);
        self
    }

    /// Build the field.
    pub fn build(self) -> Field {
        let select = Select::new(&self.title).options(self.options);

        Field {
            key: self.key,
            title: String::new(), // Title is in the Select component
            description: self.description,
            required: false,
            inner: FieldInner::Select(select),
        }
    }
}

/// Builder for multi-select fields.
#[derive(Debug, Clone)]
pub struct MultiSelectField {
    key: String,
    title: String,
    description: Option<String>,
    options: Vec<String>,
    min: Option<usize>,
    max: Option<usize>,
}

impl MultiSelectField {
    /// Create a new multi-select field.
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            title: String::new(),
            description: None,
            options: Vec::new(),
            min: None,
            max: None,
        }
    }

    /// Set the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the options.
    pub fn options<I, S>(mut self, options: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.options = options.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Set minimum selections.
    pub fn min(mut self, min: usize) -> Self {
        self.min = Some(min);
        self
    }

    /// Set maximum selections.
    pub fn max(mut self, max: usize) -> Self {
        self.max = Some(max);
        self
    }

    /// Build the field.
    pub fn build(self) -> Field {
        let mut select = MultiSelect::new(&self.title).options(self.options);

        if let Some(min) = self.min {
            select = select.min(min);
        }
        if let Some(max) = self.max {
            select = select.max(max);
        }

        Field {
            key: self.key,
            title: String::new(), // Title is in the MultiSelect component
            description: self.description,
            required: self.min.is_some() && self.min.unwrap() > 0,
            inner: FieldInner::MultiSelect(select),
        }
    }
}

/// Builder for confirm fields.
#[derive(Debug, Clone)]
pub struct ConfirmField {
    key: String,
    title: String,
    description: Option<String>,
    default: bool,
}

impl ConfirmField {
    /// Create a new confirm field.
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            title: String::new(),
            description: None,
            default: false,
        }
    }

    /// Set the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the default value.
    pub fn default(mut self, default: bool) -> Self {
        self.default = default;
        self
    }

    /// Build the field.
    pub fn build(self) -> Field {
        let confirm = Confirm::new(&self.title).default(self.default);

        Field {
            key: self.key,
            title: String::new(), // Title is in the Confirm component
            description: self.description,
            required: false,
            inner: FieldInner::Confirm(confirm),
        }
    }
}
