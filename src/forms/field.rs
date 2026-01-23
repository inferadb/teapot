//! Form field types.

use std::{path::PathBuf, sync::Arc};

use super::validation::Validator;
use crate::{
    components::{Confirm, FilePicker, MultiSelect, Select, TextInput},
    runtime::{Cmd, Model, accessible::Accessible},
    terminal::{Event, KeyCode},
};

/// A dynamic string function for titles and descriptions.
pub type DynamicString = Arc<dyn Fn() -> String + Send + Sync>;

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
    /// A file/directory path.
    Path(PathBuf),
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

    /// Get as path.
    pub fn as_path(&self) -> Option<&PathBuf> {
        match self {
            FieldValue::Path(p) => Some(p),
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
    /// Display-only note.
    Note,
    /// File picker field.
    FilePicker,
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
    /// File picker message.
    FilePicker(crate::components::file_picker::FilePickerMsg),
    /// Note acknowledged (continue to next field).
    NoteAck,
}

/// A form field.
pub struct Field {
    /// Field identifier.
    pub key: String,
    /// Field title (static).
    pub title: String,
    /// Dynamic title function (overrides static title if set).
    pub title_fn: Option<DynamicString>,
    /// Field description (static).
    pub description: Option<String>,
    /// Dynamic description function (overrides static description if set).
    pub description_fn: Option<DynamicString>,
    /// Whether the field is required.
    pub required: bool,
    /// The field kind and state.
    pub inner: FieldInner,
}

impl Clone for Field {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            title: self.title.clone(),
            title_fn: self.title_fn.clone(),
            description: self.description.clone(),
            description_fn: self.description_fn.clone(),
            required: self.required,
            inner: self.inner.clone(),
        }
    }
}

impl std::fmt::Debug for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Field")
            .field("key", &self.key)
            .field("title", &self.title)
            .field("title_fn", &self.title_fn.as_ref().map(|_| "<fn>"))
            .field("description", &self.description)
            .field("description_fn", &self.description_fn.as_ref().map(|_| "<fn>"))
            .field("required", &self.required)
            .field("inner", &self.inner)
            .finish()
    }
}

impl Field {
    /// Get the effective title (dynamic or static).
    pub fn get_title(&self) -> String {
        if let Some(f) = &self.title_fn { f() } else { self.title.clone() }
    }

    /// Get the effective description (dynamic or static).
    pub fn get_description(&self) -> Option<String> {
        if let Some(f) = &self.description_fn { Some(f()) } else { self.description.clone() }
    }
}

/// A display-only note field.
#[derive(Debug, Clone)]
pub struct Note {
    content: String,
    acknowledged: bool,
    focused: bool,
}

impl Note {
    /// Create a new note.
    pub fn new(content: impl Into<String>) -> Self {
        Self { content: content.into(), acknowledged: false, focused: true }
    }

    /// Check if acknowledged.
    pub fn is_acknowledged(&self) -> bool {
        self.acknowledged
    }

    /// Acknowledge the note.
    pub fn acknowledge(&mut self) {
        self.acknowledged = true;
    }

    /// Set focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Get the content.
    pub fn content(&self) -> &str {
        &self.content
    }
}

/// Inner field state.
#[derive(Debug, Clone)]
pub enum FieldInner {
    Input(TextInput),
    Select(Select<String>),
    MultiSelect(MultiSelect<String>),
    Confirm(Confirm),
    Note(Note),
    FilePicker(FilePicker),
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
            },
            FieldInner::MultiSelect(select) => {
                FieldValue::StringList(select.selected().iter().map(|s| (*s).clone()).collect())
            },
            FieldInner::Confirm(confirm) => FieldValue::Bool(confirm.value()),
            FieldInner::Note(_) => FieldValue::None, // Notes have no value
            FieldInner::FilePicker(picker) => {
                if let Some(path) = picker.selected() {
                    FieldValue::Path(path.clone())
                } else {
                    FieldValue::None
                }
            },
        }
    }

    /// Check if the field is submitted.
    pub fn is_submitted(&self) -> bool {
        match &self.inner {
            FieldInner::Input(input) => input.is_submitted(),
            FieldInner::Select(select) => select.is_submitted(),
            FieldInner::MultiSelect(select) => select.is_submitted(),
            FieldInner::Confirm(confirm) => confirm.is_submitted(),
            FieldInner::Note(note) => note.is_acknowledged(),
            FieldInner::FilePicker(picker) => picker.is_submitted(),
        }
    }

    /// Check if the field is cancelled.
    pub fn is_cancelled(&self) -> bool {
        match &self.inner {
            FieldInner::Input(_) => false, // Input doesn't have cancel
            FieldInner::Select(select) => select.is_cancelled(),
            FieldInner::MultiSelect(select) => select.is_cancelled(),
            FieldInner::Confirm(confirm) => confirm.is_cancelled(),
            FieldInner::Note(_) => false, // Notes can't be cancelled
            FieldInner::FilePicker(picker) => picker.is_cancelled(),
        }
    }

    /// Set focus state.
    pub fn set_focused(&mut self, focused: bool) {
        match &mut self.inner {
            FieldInner::Input(input) => input.set_focused(focused),
            FieldInner::Select(select) => select.set_focused(focused),
            FieldInner::MultiSelect(select) => select.set_focused(focused),
            FieldInner::Confirm(confirm) => confirm.set_focused(focused),
            FieldInner::Note(note) => note.set_focused(focused),
            FieldInner::FilePicker(picker) => picker.set_focused(focused),
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
            FieldInner::Note(_) => None, // Notes don't need initialization
            FieldInner::FilePicker(picker) => picker.init().map(|c| c.map(FieldMsg::FilePicker)),
        }
    }

    fn update(&mut self, msg: Self::Message) -> Option<Cmd<Self::Message>> {
        match (&mut self.inner, msg) {
            (FieldInner::Input(input), FieldMsg::Input(msg)) => {
                input.update(msg).map(|c| c.map(FieldMsg::Input))
            },
            (FieldInner::Select(select), FieldMsg::Select(msg)) => {
                select.update(msg).map(|c| c.map(FieldMsg::Select))
            },
            (FieldInner::MultiSelect(select), FieldMsg::MultiSelect(msg)) => {
                select.update(msg).map(|c| c.map(FieldMsg::MultiSelect))
            },
            (FieldInner::Confirm(confirm), FieldMsg::Confirm(msg)) => {
                confirm.update(msg).map(|c| c.map(FieldMsg::Confirm))
            },
            (FieldInner::FilePicker(picker), FieldMsg::FilePicker(msg)) => {
                picker.update(msg).map(|c| c.map(FieldMsg::FilePicker))
            },
            (FieldInner::Note(note), FieldMsg::NoteAck) => {
                note.acknowledge();
                None
            },
            _ => None,
        }
    }

    fn view(&self) -> String {
        let mut output = String::new();

        // Title (dynamic or static)
        let title = self.get_title();
        if !title.is_empty() {
            output.push_str(&title);
            if self.required {
                output.push_str(" *");
            }
            output.push('\n');
        }

        // Description (dynamic or static)
        if let Some(desc) = self.get_description() {
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
            FieldInner::FilePicker(picker) => output.push_str(&picker.view()),
            FieldInner::Note(note) => {
                output.push_str(&format!(
                    "{}{}{}\n\n{}Press Enter to continue{}",
                    crate::style::Color::BrightBlack.to_ansi_fg(),
                    note.content(),
                    "\x1b[0m",
                    crate::style::Color::BrightBlack.to_ansi_fg(),
                    "\x1b[0m"
                ));
            },
        }

        output
    }

    fn handle_event(&self, event: Event) -> Option<Self::Message> {
        match &self.inner {
            FieldInner::Input(input) => input.handle_event(event).map(FieldMsg::Input),
            FieldInner::Select(select) => select.handle_event(event).map(FieldMsg::Select),
            FieldInner::MultiSelect(select) => {
                select.handle_event(event).map(FieldMsg::MultiSelect)
            },
            FieldInner::Confirm(confirm) => confirm.handle_event(event).map(FieldMsg::Confirm),
            FieldInner::FilePicker(picker) => picker.handle_event(event).map(FieldMsg::FilePicker),
            FieldInner::Note(_) => {
                // Notes acknowledge on Enter or Space
                if let Event::Key(key) = event
                    && matches!(key.code, KeyCode::Enter | KeyCode::Char(' '))
                {
                    return Some(FieldMsg::NoteAck);
                }
                None
            },
        }
    }
}

impl Accessible for Field {
    type Message = FieldMsg;

    fn accessible_prompt(&self) -> String {
        let mut prompt = String::new();

        // Title (dynamic or static)
        let title = self.get_title();
        if !title.is_empty() {
            prompt.push_str(&title);
            if self.required {
                prompt.push_str(" *");
            }
            prompt.push('\n');
        }

        // Description (dynamic or static)
        if let Some(desc) = self.get_description() {
            prompt.push_str(&desc);
            prompt.push('\n');
        }

        // Delegate to inner component
        match &self.inner {
            FieldInner::Input(input) => prompt.push_str(&input.accessible_prompt()),
            FieldInner::Select(select) => prompt.push_str(&select.accessible_prompt()),
            FieldInner::MultiSelect(select) => prompt.push_str(&select.accessible_prompt()),
            FieldInner::Confirm(confirm) => prompt.push_str(&confirm.accessible_prompt()),
            FieldInner::FilePicker(picker) => prompt.push_str(&picker.accessible_prompt()),
            FieldInner::Note(note) => {
                prompt.push_str(note.content());
                prompt.push_str("\n\nPress Enter to continue: ");
            },
        }

        prompt
    }

    fn parse_accessible_input(&self, input: &str) -> Option<Self::Message> {
        match &self.inner {
            FieldInner::Input(inner) => inner.parse_accessible_input(input).map(FieldMsg::Input),
            FieldInner::Select(inner) => inner.parse_accessible_input(input).map(FieldMsg::Select),
            FieldInner::MultiSelect(inner) => {
                inner.parse_accessible_input(input).map(FieldMsg::MultiSelect)
            },
            FieldInner::Confirm(inner) => {
                inner.parse_accessible_input(input).map(FieldMsg::Confirm)
            },
            FieldInner::FilePicker(inner) => {
                inner.parse_accessible_input(input).map(FieldMsg::FilePicker)
            },
            FieldInner::Note(_) => Some(FieldMsg::NoteAck), // Any input acknowledges the note
        }
    }

    fn is_accessible_complete(&self) -> bool {
        self.is_submitted() || self.is_cancelled()
    }
}

impl Field {
    /// Apply accessible input to this field.
    ///
    /// Returns true if the field is complete.
    pub fn apply_accessible_input(&mut self, input: &str) -> bool {
        match &mut self.inner {
            FieldInner::Input(inner) => {
                // For text input, set value and submit
                let trimmed = input.trim();
                if !trimmed.is_empty() {
                    inner.update(crate::components::text_input::TextInputMsg::SetValue(
                        trimmed.to_string(),
                    ));
                }
                inner.update(crate::components::text_input::TextInputMsg::Submit);
                true
            },
            FieldInner::Select(inner) => inner.apply_accessible_input(input),
            FieldInner::MultiSelect(inner) => inner.apply_accessible_input(input),
            FieldInner::Confirm(inner) => inner.apply_accessible_input(input),
            FieldInner::FilePicker(inner) => inner.apply_accessible_input(input),
            FieldInner::Note(note) => {
                // Any input acknowledges the note
                note.acknowledge();
                true
            },
        }
    }
}

/// Builder for text input fields.
pub struct InputField {
    key: String,
    title: String,
    title_fn: Option<DynamicString>,
    description: Option<String>,
    description_fn: Option<DynamicString>,
    placeholder: String,
    default: String,
    required: bool,
    hidden: bool,
    validators: Vec<Validator<String>>,
}

impl Clone for InputField {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            title: self.title.clone(),
            title_fn: self.title_fn.clone(),
            description: self.description.clone(),
            description_fn: self.description_fn.clone(),
            placeholder: self.placeholder.clone(),
            default: self.default.clone(),
            required: self.required,
            hidden: self.hidden,
            validators: self.validators.clone(),
        }
    }
}

impl std::fmt::Debug for InputField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputField")
            .field("key", &self.key)
            .field("title", &self.title)
            .field("title_fn", &self.title_fn.as_ref().map(|_| "<fn>"))
            .field("description", &self.description)
            .field("description_fn", &self.description_fn.as_ref().map(|_| "<fn>"))
            .field("placeholder", &self.placeholder)
            .field("default", &self.default)
            .field("required", &self.required)
            .field("hidden", &self.hidden)
            .finish()
    }
}

impl InputField {
    /// Create a new input field.
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            title: String::new(),
            title_fn: None,
            description: None,
            description_fn: None,
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

    /// Set a dynamic title function.
    ///
    /// The function is called each time the field is rendered,
    /// allowing the title to update based on external state.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::sync::Arc;
    /// use std::sync::atomic::{AtomicUsize, Ordering};
    ///
    /// let counter = Arc::new(AtomicUsize::new(0));
    /// let counter_clone = counter.clone();
    ///
    /// let field = InputField::new("name")
    ///     .title_fn(move || format!("Name (attempt {})", counter_clone.load(Ordering::SeqCst)))
    ///     .build();
    /// ```
    pub fn title_fn<F>(mut self, f: F) -> Self
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        self.title_fn = Some(Arc::new(f));
        self
    }

    /// Set the description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set a dynamic description function.
    pub fn description_fn<F>(mut self, f: F) -> Self
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        self.description_fn = Some(Arc::new(f));
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
        let input =
            TextInput::new().placeholder(self.placeholder).value(self.default).hidden(self.hidden);

        Field {
            key: self.key,
            title: self.title,
            title_fn: self.title_fn,
            description: self.description,
            description_fn: self.description_fn,
            required: self.required,
            inner: FieldInner::Input(input),
        }
    }
}

/// Builder for select fields.
pub struct SelectField {
    key: String,
    title: String,
    title_fn: Option<DynamicString>,
    description: Option<String>,
    description_fn: Option<DynamicString>,
    options: Vec<String>,
    default: Option<usize>,
}

impl Clone for SelectField {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            title: self.title.clone(),
            title_fn: self.title_fn.clone(),
            description: self.description.clone(),
            description_fn: self.description_fn.clone(),
            options: self.options.clone(),
            default: self.default,
        }
    }
}

impl std::fmt::Debug for SelectField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectField")
            .field("key", &self.key)
            .field("title", &self.title)
            .field("options", &self.options)
            .finish()
    }
}

impl SelectField {
    /// Create a new select field.
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            title: String::new(),
            title_fn: None,
            description: None,
            description_fn: None,
            options: Vec::new(),
            default: None,
        }
    }

    /// Set the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set a dynamic title function.
    pub fn title_fn<F>(mut self, f: F) -> Self
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        self.title_fn = Some(Arc::new(f));
        self
    }

    /// Set the description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set a dynamic description function.
    pub fn description_fn<F>(mut self, f: F) -> Self
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        self.description_fn = Some(Arc::new(f));
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
            title_fn: self.title_fn,
            description: self.description,
            description_fn: self.description_fn,
            required: false,
            inner: FieldInner::Select(select),
        }
    }
}

/// Builder for multi-select fields.
pub struct MultiSelectField {
    key: String,
    title: String,
    title_fn: Option<DynamicString>,
    description: Option<String>,
    description_fn: Option<DynamicString>,
    options: Vec<String>,
    min: Option<usize>,
    max: Option<usize>,
}

impl Clone for MultiSelectField {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            title: self.title.clone(),
            title_fn: self.title_fn.clone(),
            description: self.description.clone(),
            description_fn: self.description_fn.clone(),
            options: self.options.clone(),
            min: self.min,
            max: self.max,
        }
    }
}

impl std::fmt::Debug for MultiSelectField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiSelectField")
            .field("key", &self.key)
            .field("title", &self.title)
            .field("options", &self.options)
            .finish()
    }
}

impl MultiSelectField {
    /// Create a new multi-select field.
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            title: String::new(),
            title_fn: None,
            description: None,
            description_fn: None,
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

    /// Set a dynamic title function.
    pub fn title_fn<F>(mut self, f: F) -> Self
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        self.title_fn = Some(Arc::new(f));
        self
    }

    /// Set the description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set a dynamic description function.
    pub fn description_fn<F>(mut self, f: F) -> Self
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        self.description_fn = Some(Arc::new(f));
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
            title_fn: self.title_fn,
            description: self.description,
            description_fn: self.description_fn,
            required: self.min.is_some() && self.min.unwrap() > 0,
            inner: FieldInner::MultiSelect(select),
        }
    }
}

/// Builder for confirm fields.
pub struct ConfirmField {
    key: String,
    title: String,
    title_fn: Option<DynamicString>,
    description: Option<String>,
    description_fn: Option<DynamicString>,
    default: bool,
}

impl Clone for ConfirmField {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            title: self.title.clone(),
            title_fn: self.title_fn.clone(),
            description: self.description.clone(),
            description_fn: self.description_fn.clone(),
            default: self.default,
        }
    }
}

impl std::fmt::Debug for ConfirmField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfirmField")
            .field("key", &self.key)
            .field("title", &self.title)
            .field("default", &self.default)
            .finish()
    }
}

impl ConfirmField {
    /// Create a new confirm field.
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            title: String::new(),
            title_fn: None,
            description: None,
            description_fn: None,
            default: false,
        }
    }

    /// Set the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set a dynamic title function.
    pub fn title_fn<F>(mut self, f: F) -> Self
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        self.title_fn = Some(Arc::new(f));
        self
    }

    /// Set the description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set a dynamic description function.
    pub fn description_fn<F>(mut self, f: F) -> Self
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        self.description_fn = Some(Arc::new(f));
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
            title_fn: self.title_fn,
            description: self.description,
            description_fn: self.description_fn,
            required: false,
            inner: FieldInner::Confirm(confirm),
        }
    }
}

/// Builder for note (display-only) fields.
///
/// Notes display information to the user and require acknowledgment
/// (pressing Enter) to proceed.
pub struct NoteField {
    title: String,
    title_fn: Option<DynamicString>,
    content: String,
    description: Option<String>,
    description_fn: Option<DynamicString>,
}

impl Clone for NoteField {
    fn clone(&self) -> Self {
        Self {
            title: self.title.clone(),
            title_fn: self.title_fn.clone(),
            content: self.content.clone(),
            description: self.description.clone(),
            description_fn: self.description_fn.clone(),
        }
    }
}

impl std::fmt::Debug for NoteField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NoteField")
            .field("title", &self.title)
            .field("content", &self.content)
            .finish()
    }
}

impl NoteField {
    /// Create a new note field with content.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            title: String::new(),
            title_fn: None,
            content: content.into(),
            description: None,
            description_fn: None,
        }
    }

    /// Set the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set a dynamic title function.
    pub fn title_fn<F>(mut self, f: F) -> Self
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        self.title_fn = Some(Arc::new(f));
        self
    }

    /// Set the description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set a dynamic description function.
    pub fn description_fn<F>(mut self, f: F) -> Self
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        self.description_fn = Some(Arc::new(f));
        self
    }

    /// Set the content (alias for the constructor content).
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    /// Build the field.
    ///
    /// Note: Notes don't have keys since they don't produce values.
    /// A placeholder key "_note_{index}" will be used internally.
    pub fn build(self) -> Field {
        Field {
            key: format!(
                "_note_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            ),
            title: self.title,
            title_fn: self.title_fn,
            description: self.description,
            description_fn: self.description_fn,
            required: false,
            inner: FieldInner::Note(Note::new(self.content)),
        }
    }

    /// Build the field with a specific key.
    pub fn build_with_key(self, key: impl Into<String>) -> Field {
        Field {
            key: key.into(),
            title: self.title,
            title_fn: self.title_fn,
            description: self.description,
            description_fn: self.description_fn,
            required: false,
            inner: FieldInner::Note(Note::new(self.content)),
        }
    }
}

/// Builder for file picker fields.
pub struct FilePickerField {
    key: String,
    title: String,
    title_fn: Option<DynamicString>,
    description: Option<String>,
    description_fn: Option<DynamicString>,
    directory: Option<PathBuf>,
    show_hidden: bool,
    dirs_only: bool,
    files_only: bool,
    extensions: Vec<String>,
    height: usize,
    required: bool,
}

impl Clone for FilePickerField {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            title: self.title.clone(),
            title_fn: self.title_fn.clone(),
            description: self.description.clone(),
            description_fn: self.description_fn.clone(),
            directory: self.directory.clone(),
            show_hidden: self.show_hidden,
            dirs_only: self.dirs_only,
            files_only: self.files_only,
            extensions: self.extensions.clone(),
            height: self.height,
            required: self.required,
        }
    }
}

impl std::fmt::Debug for FilePickerField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FilePickerField")
            .field("key", &self.key)
            .field("title", &self.title)
            .field("title_fn", &self.title_fn.as_ref().map(|_| "<fn>"))
            .field("description", &self.description)
            .field("description_fn", &self.description_fn.as_ref().map(|_| "<fn>"))
            .field("directory", &self.directory)
            .field("show_hidden", &self.show_hidden)
            .field("dirs_only", &self.dirs_only)
            .field("files_only", &self.files_only)
            .field("extensions", &self.extensions)
            .field("height", &self.height)
            .field("required", &self.required)
            .finish()
    }
}

impl FilePickerField {
    /// Create a new file picker field.
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            title: String::new(),
            title_fn: None,
            description: None,
            description_fn: None,
            directory: None,
            show_hidden: false,
            dirs_only: false,
            files_only: false,
            extensions: Vec::new(),
            height: 10,
            required: false,
        }
    }

    /// Set the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set a dynamic title function.
    pub fn title_fn<F>(mut self, f: F) -> Self
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        self.title_fn = Some(Arc::new(f));
        self
    }

    /// Set the description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set a dynamic description function.
    pub fn description_fn<F>(mut self, f: F) -> Self
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        self.description_fn = Some(Arc::new(f));
        self
    }

    /// Set the starting directory.
    pub fn directory(mut self, dir: impl Into<PathBuf>) -> Self {
        self.directory = Some(dir.into());
        self
    }

    /// Show hidden files.
    pub fn show_hidden(mut self, show: bool) -> Self {
        self.show_hidden = show;
        self
    }

    /// Only show directories.
    pub fn dirs_only(mut self) -> Self {
        self.dirs_only = true;
        self.files_only = false;
        self
    }

    /// Only show files.
    pub fn files_only(mut self) -> Self {
        self.files_only = true;
        self.dirs_only = false;
        self
    }

    /// Filter by file extensions.
    pub fn extensions<I, S>(mut self, exts: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.extensions = exts.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Set the visible height (number of items).
    pub fn height(mut self, height: usize) -> Self {
        self.height = height;
        self
    }

    /// Mark as required.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Build the field.
    pub fn build(self) -> Field {
        let mut picker = FilePicker::new().height(self.height);

        if !self.title.is_empty() {
            picker = picker.title(&self.title);
        }

        if let Some(dir) = self.directory {
            picker = picker.directory(dir);
        }

        picker = picker.show_hidden(self.show_hidden);

        if self.dirs_only {
            picker = picker.dirs_only();
        } else if self.files_only {
            picker = picker.files_only();
        }

        if !self.extensions.is_empty() {
            picker = picker.extensions(self.extensions);
        }

        Field {
            key: self.key,
            title: String::new(), // Title is in the FilePicker component
            title_fn: self.title_fn,
            description: self.description,
            description_fn: self.description_fn,
            required: self.required,
            inner: FieldInner::FilePicker(picker),
        }
    }
}
