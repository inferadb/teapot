//! Form field types.

use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

/// Counter for generating unique note field keys.
static NOTE_COUNTER: AtomicU64 = AtomicU64::new(0);

use crate::{
    components::{Confirm, FilePicker, MultiSelect, Select, TextInput},
    runtime::{Cmd, Model, accessible::Accessible},
    terminal::{Event, KeyCode},
};

/// A dynamic string function for titles and descriptions.
pub type DynamicString = Arc<dyn Fn() -> String + Send + Sync>;

/// A field value that can be stored in form results.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

// ============================================================================
// Field Builders (bon-generated)
// ============================================================================

#[bon::bon]
impl Field {
    /// Create an input field using the builder pattern.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use teapot::forms::Field;
    ///
    /// let field = Field::input()
    ///     .key("username")
    ///     .title("Username")
    ///     .placeholder("Enter your username")
    ///     .required(true)
    ///     .build();
    /// ```
    #[builder(on(String, into), finish_fn = build)]
    pub fn input(
        key: String,
        #[builder(default)] title: String,
        description: Option<String>,
        #[builder(default)] placeholder: String,
        #[builder(default = String::new())] default_value: String,
        #[builder(default)] required: bool,
        #[builder(default)] hidden: bool,
        title_fn: Option<DynamicString>,
        description_fn: Option<DynamicString>,
    ) -> Self {
        let input = TextInput::new().placeholder(placeholder).value(default_value).hidden(hidden);

        Field {
            key,
            title,
            title_fn,
            description,
            description_fn,
            required,
            inner: FieldInner::Input(input),
        }
    }

    /// Create a select field using the builder pattern.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use teapot::forms::Field;
    ///
    /// let field = Field::select()
    ///     .key("color")
    ///     .title("Favorite Color")
    ///     .options(vec!["Red".to_string(), "Green".to_string(), "Blue".to_string()])
    ///     .build();
    /// ```
    #[builder(on(String, into), finish_fn = build)]
    pub fn select(
        key: String,
        #[builder(default)] title: String,
        description: Option<String>,
        #[builder(default)] options: Vec<String>,
        _default_index: Option<usize>,
        title_fn: Option<DynamicString>,
        description_fn: Option<DynamicString>,
    ) -> Self {
        let select = Select::new(&title).options(options);

        Field {
            key,
            title: String::new(), // Title is in the Select component
            title_fn,
            description,
            description_fn,
            required: false,
            inner: FieldInner::Select(select),
        }
    }

    /// Create a multi-select field using the builder pattern.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use teapot::forms::Field;
    ///
    /// let field = Field::multi_select()
    ///     .key("languages")
    ///     .title("Programming Languages")
    ///     .options(vec!["Rust".to_string(), "Python".to_string(), "TypeScript".to_string()])
    ///     .min(1)
    ///     .max(3)
    ///     .build();
    /// ```
    #[builder(on(String, into), finish_fn = build)]
    pub fn multi_select(
        key: String,
        #[builder(default)] title: String,
        description: Option<String>,
        #[builder(default)] options: Vec<String>,
        min: Option<usize>,
        max: Option<usize>,
        title_fn: Option<DynamicString>,
        description_fn: Option<DynamicString>,
    ) -> Self {
        let mut select = MultiSelect::new(&title).options(options);

        if let Some(min_val) = min {
            select = select.min(min_val);
        }
        if let Some(max_val) = max {
            select = select.max(max_val);
        }

        Field {
            key,
            title: String::new(), // Title is in the MultiSelect component
            title_fn,
            description,
            description_fn,
            required: min.is_some_and(|m| m > 0),
            inner: FieldInner::MultiSelect(select),
        }
    }

    /// Create a confirm field using the bon builder pattern.
    ///
    /// # Example
    /// ```no_run
    /// use teapot::forms::Field;
    ///
    /// let field = Field::confirm()
    ///     .key("agree")
    ///     .title("Do you agree to the terms?")
    ///     .default(false)
    ///     .build();
    /// ```
    #[builder(on(String, into), finish_fn = build)]
    pub fn confirm(
        key: String,
        #[builder(default)] title: String,
        description: Option<String>,
        #[builder(default)] default: bool,
        title_fn: Option<DynamicString>,
        description_fn: Option<DynamicString>,
    ) -> Self {
        let confirm = Confirm::new(&title).default(default);

        Field {
            key,
            title: String::new(), // Title is in the Confirm component
            title_fn,
            description,
            description_fn,
            required: false,
            inner: FieldInner::Confirm(confirm),
        }
    }

    /// Create a note field using the bon builder pattern.
    ///
    /// Notes are display-only fields that show information to the user.
    /// They don't have a key since they don't produce values; a placeholder
    /// key will be generated automatically unless `key` is provided.
    ///
    /// # Example
    /// ```no_run
    /// use teapot::forms::Field;
    ///
    /// let field = Field::note()
    ///     .content("This is important information")
    ///     .title("Notice")
    ///     .build();
    /// ```
    #[builder(on(String, into), finish_fn = build)]
    pub fn note(
        content: String,
        #[builder(default)] title: String,
        description: Option<String>,
        key: Option<String>,
        title_fn: Option<DynamicString>,
        description_fn: Option<DynamicString>,
    ) -> Self {
        let generated_key = key
            .unwrap_or_else(|| format!("_note_{}", NOTE_COUNTER.fetch_add(1, Ordering::Relaxed)));

        Field {
            key: generated_key,
            title,
            title_fn,
            description,
            description_fn,
            required: false,
            inner: FieldInner::Note(Note::new(content)),
        }
    }

    /// Create a file picker field using the bon builder pattern.
    ///
    /// # Example
    /// ```no_run
    /// use teapot::forms::Field;
    ///
    /// let field = Field::file_picker()
    ///     .key("config")
    ///     .title("Select Config File")
    ///     .extensions(vec!["toml".to_string(), "yaml".to_string()])
    ///     .build();
    /// ```
    #[builder(on(String, into), finish_fn = build)]
    pub fn file_picker(
        key: String,
        #[builder(default)] title: String,
        description: Option<String>,
        directory: Option<std::path::PathBuf>,
        #[builder(default)] show_hidden: bool,
        #[builder(default)] dirs_only: bool,
        #[builder(default)] files_only: bool,
        #[builder(default)] extensions: Vec<String>,
        #[builder(default = 10)] height: usize,
        #[builder(default)] required: bool,
        title_fn: Option<DynamicString>,
        description_fn: Option<DynamicString>,
    ) -> Self {
        let mut picker = FilePicker::new().height(height);

        if !title.is_empty() {
            picker = picker.title(&title);
        }

        if let Some(dir) = directory {
            picker = picker.directory(dir);
        }

        picker = picker.show_hidden(show_hidden);

        if dirs_only {
            picker = picker.dirs_only();
        } else if files_only {
            picker = picker.files_only();
        }

        if !extensions.is_empty() {
            picker = picker.extensions(extensions);
        }

        Field {
            key,
            title: String::new(), // Title is in the FilePicker component
            title_fn,
            description,
            description_fn,
            required,
            inner: FieldInner::FilePicker(picker),
        }
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
