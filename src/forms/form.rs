//! Form container and orchestration.

use std::collections::HashMap;

use super::{
    field::FieldValue,
    group::{Group, GroupMsg},
};
use crate::{
    runtime::{Cmd, Model, accessible::Accessible},
    style::{Color, Position, join_horizontal_with},
    terminal::{Event, KeyCode},
};

/// Form layout options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormLayout {
    /// Show one group at a time (default).
    #[default]
    Default,
    /// Show all groups stacked vertically.
    Stack,
    /// Show groups in columns.
    Columns(usize),
}

/// Form results containing all field values.
#[derive(Debug, Clone, Default)]
pub struct FormResults {
    values: HashMap<String, FieldValue>,
}

impl FormResults {
    /// Create new empty results.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a value.
    pub fn insert(&mut self, key: String, value: FieldValue) {
        self.values.insert(key, value);
    }

    /// Get a value by key.
    pub fn get(&self, key: &str) -> Option<&FieldValue> {
        self.values.get(key)
    }

    /// Get a string value.
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.values.get(key).and_then(|v| v.as_string())
    }

    /// Get a bool value.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.values.get(key).and_then(|v| v.as_bool())
    }

    /// Get a string list value.
    pub fn get_string_list(&self, key: &str) -> Option<&[String]> {
        self.values.get(key).and_then(|v| v.as_string_list())
    }

    /// Iterate over all values.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &FieldValue)> {
        self.values.iter()
    }
}

/// Message type for forms.
#[derive(Debug, Clone)]
pub enum FormMsg {
    /// Group message.
    Group(usize, GroupMsg),
    /// Move to next group.
    NextGroup,
    /// Move to previous group.
    PrevGroup,
    /// Submit the form.
    Submit,
    /// Cancel the form.
    Cancel,
}

/// A form with multiple groups of fields.
#[derive(Debug, Clone)]
pub struct Form {
    title: Option<String>,
    description: Option<String>,
    groups: Vec<Group>,
    current_group: usize,
    submitted: bool,
    cancelled: bool,
    accessible: bool,
    layout: FormLayout,
}

impl Default for Form {
    fn default() -> Self {
        Self::new()
    }
}

impl Form {
    /// Create a new empty form.
    pub fn new() -> Self {
        Self {
            title: None,
            description: None,
            groups: Vec::new(),
            current_group: 0,
            submitted: false,
            cancelled: false,
            accessible: std::env::var("ACCESSIBLE").is_ok(),
            layout: FormLayout::Default,
        }
    }

    /// Set the form title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the form description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a group to the form.
    pub fn group(mut self, group: Group) -> Self {
        self.groups.push(group);
        self
    }

    /// Enable accessible mode.
    pub fn accessible(mut self, accessible: bool) -> Self {
        self.accessible = accessible;
        self
    }

    /// Set the form layout.
    ///
    /// - `FormLayout::Default` - Show one group at a time (default)
    /// - `FormLayout::Stack` - Show all groups stacked vertically
    /// - `FormLayout::Columns(n)` - Show groups in n columns
    pub fn layout(mut self, layout: FormLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Get the current layout.
    pub fn get_layout(&self) -> FormLayout {
        self.layout
    }

    /// Check if the form is submitted.
    pub fn is_submitted(&self) -> bool {
        self.submitted
    }

    /// Check if the form is cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    /// Get the form results.
    pub fn results(&self) -> FormResults {
        let mut results = FormResults::new();

        for group in &self.groups {
            for field in group.fields() {
                results.insert(field.key().to_string(), field.value());
            }
        }

        results
    }

    /// Get the current group.
    pub fn current_group(&self) -> Option<&Group> {
        self.groups.get(self.current_group)
    }

    /// Get the current group mutably.
    pub fn current_group_mut(&mut self) -> Option<&mut Group> {
        self.groups.get_mut(self.current_group)
    }

    /// Move to next group.
    fn next_group(&mut self) -> bool {
        if self.current_group < self.groups.len().saturating_sub(1) {
            self.current_group += 1;
            if let Some(group) = self.groups.get_mut(self.current_group) {
                group.focus_first();
            }
            true
        } else {
            false
        }
    }

    /// Move to previous group.
    fn prev_group(&mut self) -> bool {
        if self.current_group > 0 {
            self.current_group -= 1;
            if let Some(group) = self.groups.get_mut(self.current_group) {
                group.focus_first();
            }
            true
        } else {
            false
        }
    }

    /// Check if on last group.
    fn is_last_group(&self) -> bool {
        self.current_group >= self.groups.len().saturating_sub(1)
    }

    /// Initialize the form.
    pub fn init_form(&mut self) {
        if let Some(group) = self.groups.get_mut(0) {
            group.focus_first();
        }
    }
}

impl Model for Form {
    type Message = FormMsg;

    fn init(&self) -> Option<Cmd<Self::Message>> {
        // Initialize current group
        if let Some(group) = self.groups.get(self.current_group) {
            let idx = self.current_group;
            group.init().map(|c| c.map(move |m| FormMsg::Group(idx, m)))
        } else {
            None
        }
    }

    fn update(&mut self, msg: Self::Message) -> Option<Cmd<Self::Message>> {
        match msg {
            FormMsg::Group(idx, group_msg) => {
                // Update the group and capture state before releasing borrow
                let (result, is_complete, is_cancelled) = {
                    if let Some(group) = self.groups.get_mut(idx) {
                        let result =
                            group.update(group_msg).map(|c| c.map(move |m| FormMsg::Group(idx, m)));
                        (result, group.is_complete(), group.is_cancelled())
                    } else {
                        return None;
                    }
                };

                // Now handle state changes without borrow conflicts
                if is_complete && !self.is_last_group() {
                    self.next_group();
                } else if is_complete && self.is_last_group() {
                    self.submitted = true;
                }

                if is_cancelled {
                    self.cancelled = true;
                }

                result
            },
            FormMsg::NextGroup => {
                if self.is_last_group() {
                    self.submitted = true;
                } else {
                    self.next_group();
                }
                None
            },
            FormMsg::PrevGroup => {
                self.prev_group();
                None
            },
            FormMsg::Submit => {
                self.submitted = true;
                Some(Cmd::quit())
            },
            FormMsg::Cancel => {
                self.cancelled = true;
                Some(Cmd::quit())
            },
        }
    }

    fn view(&self) -> String {
        if self.submitted {
            return self.view_results();
        }

        let mut output = String::new();

        // Form title
        if let Some(title) = &self.title {
            output.push_str(&format!("{}\x1b[1m{}\x1b[0m\n", Color::Cyan.to_ansi_fg(), title));
        }

        // Form description
        if let Some(desc) = &self.description {
            output.push_str(&format!("{}{}{}\n", Color::BrightBlack.to_ansi_fg(), desc, "\x1b[0m"));
        }

        if self.title.is_some() || self.description.is_some() {
            output.push('\n');
        }

        // Render groups based on layout
        match self.layout {
            FormLayout::Default => {
                // Show one group at a time
                if let Some(group) = self.groups.get(self.current_group) {
                    output.push_str(&group.view());
                }

                // Group progress (if multiple groups)
                if self.groups.len() > 1 {
                    output.push_str(&format!(
                        "\n{}Page {}/{}{}",
                        Color::BrightBlack.to_ansi_fg(),
                        self.current_group + 1,
                        self.groups.len(),
                        "\x1b[0m"
                    ));
                }
            },
            FormLayout::Stack => {
                // Show all groups stacked vertically
                for (i, group) in self.groups.iter().enumerate() {
                    if i > 0 {
                        output.push_str("\n\n");
                    }
                    output.push_str(&group.view());
                }
            },
            FormLayout::Columns(cols) => {
                // Show groups in columns
                let cols = cols.max(1);
                let group_views: Vec<String> = self.groups.iter().map(|g| g.view()).collect();

                for chunk in group_views.chunks(cols) {
                    let strs: Vec<&str> = chunk.iter().map(|s| s.as_str()).collect();
                    if !strs.is_empty() {
                        if !output.is_empty()
                            && !output.ends_with('\n')
                            && !output.ends_with("\n\n")
                        {
                            output.push_str("\n\n");
                        }
                        output.push_str(&join_horizontal_with(Position::Top, &strs));
                    }
                }
            },
        }

        output
    }

    fn handle_event(&self, event: Event) -> Option<Self::Message> {
        // Check for form-level shortcuts
        if let Event::Key(key) = &event
            && key.code == KeyCode::Esc
        {
            return Some(FormMsg::Cancel);
        }

        // Pass to current group
        if let Some(group) = self.groups.get(self.current_group) {
            let idx = self.current_group;
            group.handle_event(event).map(|m| FormMsg::Group(idx, m))
        } else {
            None
        }
    }
}

impl Form {
    /// View the results summary.
    fn view_results(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("{}✓ Form completed{}\n\n", Color::Green.to_ansi_fg(), "\x1b[0m"));

        for group in &self.groups {
            for field in group.fields() {
                let value_str = match field.value() {
                    FieldValue::String(s) => s,
                    FieldValue::Bool(b) => {
                        if b {
                            "Yes".to_string()
                        } else {
                            "No".to_string()
                        }
                    },
                    FieldValue::StringList(list) => list.join(", "),
                    FieldValue::Int(n) => n.to_string(),
                    FieldValue::Path(p) => p.display().to_string(),
                    FieldValue::None => "(empty)".to_string(),
                };

                output.push_str(&format!(
                    "  {}{}{}: {}\n",
                    Color::Cyan.to_ansi_fg(),
                    field.key(),
                    "\x1b[0m",
                    value_str
                ));
            }
        }

        output
    }
}

impl Accessible for Form {
    type Message = FormMsg;

    fn accessible_prompt(&self) -> String {
        let mut prompt = String::new();

        // Form title
        if let Some(title) = &self.title {
            prompt.push_str(&format!("=== {} ===\n", title));
        }

        // Form description
        if let Some(desc) = &self.description {
            prompt.push_str(&format!("{}\n", desc));
        }

        // Progress indicator
        if self.groups.len() > 1 {
            prompt.push_str(&format!("Page {}/{}\n", self.current_group + 1, self.groups.len()));
        }

        prompt.push('\n');

        // Current group and field prompt
        if let Some(group) = self.groups.get(self.current_group) {
            prompt.push_str(&group.accessible_prompt());
        }

        prompt
    }

    fn parse_accessible_input(&self, input: &str) -> Option<Self::Message> {
        if let Some(group) = self.groups.get(self.current_group) {
            let idx = self.current_group;
            group.parse_accessible_input(input).map(|m| FormMsg::Group(idx, m))
        } else {
            None
        }
    }

    fn is_accessible_complete(&self) -> bool {
        self.submitted || self.cancelled
    }
}

impl Form {
    /// Run the form in accessible mode.
    ///
    /// This method handles the complete accessible flow, prompting for
    /// each field and advancing through groups automatically.
    ///
    /// Returns the form results on success, or None if cancelled.
    pub fn run_accessible(&mut self) -> std::io::Result<Option<FormResults>> {
        use std::io::{self, BufRead, Write};

        self.init_form();

        loop {
            // Print the accessible prompt
            print!("{}", self.accessible_prompt());
            io::stdout().flush()?;

            // Read input
            let mut input = String::new();
            io::stdin().lock().read_line(&mut input)?;

            // Apply input to current field
            if let Some(group) = self.groups.get_mut(self.current_group) {
                let complete = group.apply_accessible_input(&input);

                if complete {
                    // Check if group is cancelled
                    if group.is_cancelled() {
                        self.cancelled = true;
                        return Ok(None);
                    }

                    // Move to next field or group
                    if group.is_complete() {
                        if self.is_last_group() {
                            self.submitted = true;
                            println!("\nForm completed!");
                            return Ok(Some(self.results()));
                        } else {
                            self.next_group();
                        }
                    }
                }
            }

            // Check if form is done
            if self.is_accessible_complete() {
                if self.cancelled {
                    return Ok(None);
                }
                return Ok(Some(self.results()));
            }
        }
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use crate::forms::{ConfirmField, InputField};

    #[test]
    fn test_form_creation() {
        let form = Form::new().title("Test Form").group(
            Group::new()
                .field(InputField::new("name").title("Name").build())
                .field(ConfirmField::new("agree").title("Agree?").build()),
        );

        assert!(!form.is_submitted());
        assert!(!form.is_cancelled());
    }

    #[test]
    fn test_form_results() {
        let mut results = FormResults::new();
        results.insert("name".to_string(), FieldValue::String("Alice".to_string()));
        results.insert("agree".to_string(), FieldValue::Bool(true));

        assert_eq!(results.get_string("name"), Some("Alice"));
        assert_eq!(results.get_bool("agree"), Some(true));
    }

    #[test]
    fn test_form_layout_default() {
        let form = Form::new()
            .layout(FormLayout::Default)
            .group(Group::new().field(InputField::new("a").build()))
            .group(Group::new().field(InputField::new("b").build()));

        assert_eq!(form.get_layout(), FormLayout::Default);
    }

    #[test]
    fn test_form_layout_stack() {
        let form = Form::new()
            .layout(FormLayout::Stack)
            .group(Group::new().field(InputField::new("a").build()))
            .group(Group::new().field(InputField::new("b").build()));

        assert_eq!(form.get_layout(), FormLayout::Stack);
        // In Stack mode, view should contain content from all groups
        let view = form.view();
        assert!(!view.is_empty());
    }

    #[test]
    fn test_form_layout_columns() {
        let form = Form::new()
            .layout(FormLayout::Columns(2))
            .group(Group::new().field(InputField::new("a").build()))
            .group(Group::new().field(InputField::new("b").build()));

        assert_eq!(form.get_layout(), FormLayout::Columns(2));
    }

    #[test]
    fn test_note_field() {
        use crate::forms::NoteField;

        let note_field = NoteField::new("This is an important note!").title("Notice").build();

        // Notes should not produce a value
        assert!(matches!(note_field.value(), FieldValue::None));
        // Notes start unsubmitted
        assert!(!note_field.is_submitted());
    }

    #[test]
    fn test_dynamic_title() {
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let field = InputField::new("test")
            .title_fn(move || format!("Attempt {}", counter_clone.load(Ordering::SeqCst)))
            .build();

        // Initial title should be "Attempt 0"
        assert_eq!(field.get_title(), "Attempt 0");

        // Update counter and check title updates
        counter.store(5, Ordering::SeqCst);
        assert_eq!(field.get_title(), "Attempt 5");
    }

    #[test]
    fn test_dynamic_description() {
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        };

        let show_hint = Arc::new(AtomicBool::new(false));
        let show_hint_clone = show_hint.clone();

        let field = InputField::new("password")
            .title("Password")
            .description_fn(move || {
                if show_hint_clone.load(Ordering::SeqCst) {
                    "Hint: It's your birthday!".to_string()
                } else {
                    "Enter your password".to_string()
                }
            })
            .build();

        // Initially no hint
        assert_eq!(field.get_description(), Some("Enter your password".to_string()));

        // Show hint
        show_hint.store(true, Ordering::SeqCst);
        assert_eq!(field.get_description(), Some("Hint: It's your birthday!".to_string()));
    }
}
