//! Form container and orchestration.

use std::collections::HashMap;

use super::field::FieldValue;
use super::group::{Group, GroupMsg};
use crate::runtime::{Cmd, Model};
use crate::style::Color;
use crate::terminal::{Event, KeyCode};

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
                        let result = group
                            .update(group_msg)
                            .map(|c| c.map(move |m| FormMsg::Group(idx, m)));
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
            }
            FormMsg::NextGroup => {
                if self.is_last_group() {
                    self.submitted = true;
                } else {
                    self.next_group();
                }
                None
            }
            FormMsg::PrevGroup => {
                self.prev_group();
                None
            }
            FormMsg::Submit => {
                self.submitted = true;
                Some(Cmd::quit())
            }
            FormMsg::Cancel => {
                self.cancelled = true;
                Some(Cmd::quit())
            }
        }
    }

    fn view(&self) -> String {
        if self.submitted {
            return self.view_results();
        }

        let mut output = String::new();

        // Form title
        if let Some(title) = &self.title {
            output.push_str(&format!(
                "{}\x1b[1m{}\x1b[0m\n",
                Color::Cyan.to_ansi_fg(),
                title
            ));
        }

        // Form description
        if let Some(desc) = &self.description {
            output.push_str(&format!(
                "{}{}{}\n",
                Color::BrightBlack.to_ansi_fg(),
                desc,
                "\x1b[0m"
            ));
        }

        if self.title.is_some() || self.description.is_some() {
            output.push('\n');
        }

        // Current group
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

        output
    }

    fn handle_event(&self, event: Event) -> Option<Self::Message> {
        // Check for form-level shortcuts
        if let Event::Key(key) = &event {
            if key.code == KeyCode::Esc {
                return Some(FormMsg::Cancel);
            }
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

        output.push_str(&format!(
            "{}✓ Form completed{}\n\n",
            Color::Green.to_ansi_fg(),
            "\x1b[0m"
        ));

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
                    }
                    FieldValue::StringList(list) => list.join(", "),
                    FieldValue::Int(n) => n.to_string(),
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

#[cfg(test)]
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
}
