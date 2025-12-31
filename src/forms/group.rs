//! Form groups (pages).

use super::field::{Field, FieldMsg};
use crate::{
    runtime::{Cmd, Model, accessible::Accessible},
    terminal::Event,
};

/// A group of form fields (like a page).
#[derive(Debug, Clone)]
pub struct Group {
    title: Option<String>,
    description: Option<String>,
    fields: Vec<Field>,
    current_field: usize,
}

impl Default for Group {
    fn default() -> Self {
        Self::new()
    }
}

impl Group {
    /// Create a new empty group.
    pub fn new() -> Self {
        Self { title: None, description: None, fields: Vec::new(), current_field: 0 }
    }

    /// Set the group title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the group description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a field to the group.
    pub fn field(mut self, field: Field) -> Self {
        self.fields.push(field);
        self
    }

    /// Get the fields.
    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    /// Get mutable fields.
    pub fn fields_mut(&mut self) -> &mut [Field] {
        &mut self.fields
    }

    /// Get the current field index.
    pub fn current_field(&self) -> usize {
        self.current_field
    }

    /// Get the current field.
    pub fn current(&self) -> Option<&Field> {
        self.fields.get(self.current_field)
    }

    /// Get the current field mutably.
    pub fn current_mut(&mut self) -> Option<&mut Field> {
        self.fields.get_mut(self.current_field)
    }

    /// Move to next field.
    pub fn next_field(&mut self) -> bool {
        if self.current_field < self.fields.len().saturating_sub(1) {
            if let Some(field) = self.fields.get_mut(self.current_field) {
                field.set_focused(false);
            }
            self.current_field += 1;
            if let Some(field) = self.fields.get_mut(self.current_field) {
                field.set_focused(true);
            }
            true
        } else {
            false
        }
    }

    /// Move to previous field.
    pub fn prev_field(&mut self) -> bool {
        if self.current_field > 0 {
            if let Some(field) = self.fields.get_mut(self.current_field) {
                field.set_focused(false);
            }
            self.current_field -= 1;
            if let Some(field) = self.fields.get_mut(self.current_field) {
                field.set_focused(true);
            }
            true
        } else {
            false
        }
    }

    /// Check if on last field.
    pub fn is_last_field(&self) -> bool {
        self.current_field >= self.fields.len().saturating_sub(1)
    }

    /// Check if group is complete (all fields submitted).
    pub fn is_complete(&self) -> bool {
        self.fields.iter().all(|f| f.is_submitted())
    }

    /// Check if any field was cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.fields.iter().any(|f| f.is_cancelled())
    }

    /// Initialize focus on first field.
    pub fn focus_first(&mut self) {
        self.current_field = 0;
        if let Some(field) = self.fields.get_mut(0) {
            field.set_focused(true);
        }
    }
}

/// Message type for groups.
#[derive(Debug, Clone)]
pub enum GroupMsg {
    /// Field message for specific field.
    Field(usize, FieldMsg),
    /// Move to next field.
    NextField,
    /// Move to previous field.
    PrevField,
}

impl Model for Group {
    type Message = GroupMsg;

    fn init(&self) -> Option<Cmd<Self::Message>> {
        // Initialize current field
        if let Some(field) = self.fields.get(self.current_field) {
            let idx = self.current_field;
            field.init().map(|c| c.map(move |m| GroupMsg::Field(idx, m)))
        } else {
            None
        }
    }

    fn update(&mut self, msg: Self::Message) -> Option<Cmd<Self::Message>> {
        match msg {
            GroupMsg::Field(idx, field_msg) => {
                if let Some(field) = self.fields.get_mut(idx) {
                    // Check if this was a submit
                    let was_submit = matches!(
                        &field_msg,
                        FieldMsg::Input(crate::components::text_input::TextInputMsg::Submit)
                            | FieldMsg::Select(crate::components::select::SelectMsg::Submit)
                            | FieldMsg::MultiSelect(
                                crate::components::multi_select::MultiSelectMsg::Submit
                            )
                            | FieldMsg::Confirm(crate::components::confirm::ConfirmMsg::Submit)
                    );

                    let result =
                        field.update(field_msg).map(|c| c.map(move |m| GroupMsg::Field(idx, m)));

                    // Auto-advance on submit
                    if was_submit && !self.is_last_field() {
                        self.next_field();
                    }

                    result
                } else {
                    None
                }
            },
            GroupMsg::NextField => {
                self.next_field();
                None
            },
            GroupMsg::PrevField => {
                self.prev_field();
                None
            },
        }
    }

    fn view(&self) -> String {
        let mut output = String::new();

        // Group title
        if let Some(title) = &self.title {
            output.push_str(&format!(
                "{}\x1b[1m{}\x1b[0m\n",
                crate::style::Color::Cyan.to_ansi_fg(),
                title
            ));
        }

        // Group description
        if let Some(desc) = &self.description {
            output.push_str(&format!(
                "{}{}{}\n",
                crate::style::Color::BrightBlack.to_ansi_fg(),
                desc,
                "\x1b[0m"
            ));
        }

        if self.title.is_some() || self.description.is_some() {
            output.push('\n');
        }

        // Current field only (one at a time like Huh)
        if let Some(field) = self.fields.get(self.current_field) {
            output.push_str(&field.view());
        }

        // Progress indicator
        let total = self.fields.len();
        if total > 1 {
            output.push_str(&format!(
                "\n\n{}({}/{}){}\n",
                crate::style::Color::BrightBlack.to_ansi_fg(),
                self.current_field + 1,
                total,
                "\x1b[0m"
            ));
        }

        output
    }

    fn handle_event(&self, event: Event) -> Option<Self::Message> {
        // Pass events to current field
        if let Some(field) = self.fields.get(self.current_field) {
            let idx = self.current_field;
            field.handle_event(event).map(|m| GroupMsg::Field(idx, m))
        } else {
            None
        }
    }
}

impl Accessible for Group {
    type Message = GroupMsg;

    fn accessible_prompt(&self) -> String {
        let mut prompt = String::new();

        // Group title
        if let Some(title) = &self.title {
            prompt.push_str(&format!("{}\n", title));
        }

        // Group description
        if let Some(desc) = &self.description {
            prompt.push_str(&format!("{}\n", desc));
        }

        if self.title.is_some() || self.description.is_some() {
            prompt.push('\n');
        }

        // Current field prompt
        if let Some(field) = self.fields.get(self.current_field) {
            prompt.push_str(&field.accessible_prompt());
        }

        // Progress indicator
        let total = self.fields.len();
        if total > 1 {
            prompt.push_str(&format!("(Field {}/{})\n", self.current_field + 1, total));
        }

        prompt
    }

    fn parse_accessible_input(&self, input: &str) -> Option<Self::Message> {
        if let Some(field) = self.fields.get(self.current_field) {
            let idx = self.current_field;
            field.parse_accessible_input(input).map(|m| GroupMsg::Field(idx, m))
        } else {
            None
        }
    }

    fn is_accessible_complete(&self) -> bool {
        self.is_complete() || self.is_cancelled()
    }
}

impl Group {
    /// Apply accessible input to the current field.
    ///
    /// Returns true if the current field is complete.
    pub fn apply_accessible_input(&mut self, input: &str) -> bool {
        if let Some(field) = self.fields.get_mut(self.current_field) {
            let complete = field.apply_accessible_input(input);

            if complete && !self.is_last_field() {
                // Auto-advance to next field
                self.next_field();
                false // More fields to go
            } else {
                complete
            }
        } else {
            false
        }
    }
}
