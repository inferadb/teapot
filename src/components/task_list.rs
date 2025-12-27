//! Task list component for step-by-step operations.
//!
//! A component that displays a list of tasks with their status,
//! showing spinners for running tasks and checkmarks/crosses for completed ones.
//!
//! # Example
//!
//! ```rust,ignore
//! use ferment::components::{TaskList, TaskItem, TaskState};
//!
//! let mut list = TaskList::new()
//!     .add_task("Clone repository")
//!     .add_task("Install dependencies")
//!     .add_task("Build project");
//!
//! // Start the first task
//! list.start_task(0);
//!
//! // Complete it with a detail
//! list.complete_task(0, Some("/path/to/repo"));
//! ```

use std::time::{Duration, Instant};

use crate::runtime::{Cmd, Model, Sub};
use crate::style::Color;
use crate::terminal::Event;

/// Spinner frames for running tasks.
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// State of a task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskState {
    /// Task is waiting to start.
    Pending,
    /// Task is currently running.
    Running,
    /// Task completed successfully.
    Success,
    /// Task failed.
    Failure,
    /// Task was skipped.
    Skipped,
}

impl TaskState {
    /// Get the icon for this state.
    pub fn icon(&self, spinner_frame: usize) -> &str {
        match self {
            TaskState::Pending => "○",
            TaskState::Running => SPINNER_FRAMES[spinner_frame % SPINNER_FRAMES.len()],
            TaskState::Success => "✓",
            TaskState::Failure => "✗",
            TaskState::Skipped => "⊘",
        }
    }

    /// Get the color for this state.
    pub fn color(&self) -> Color {
        match self {
            TaskState::Pending => Color::BrightBlack,
            TaskState::Running => Color::Cyan,
            TaskState::Success => Color::Green,
            TaskState::Failure => Color::Red,
            TaskState::Skipped => Color::Yellow,
        }
    }

    /// Is this state a terminal state (completed)?
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            TaskState::Success | TaskState::Failure | TaskState::Skipped
        )
    }
}

/// A single task in the list.
#[derive(Debug, Clone)]
pub struct TaskItem {
    /// Task name/description.
    pub name: String,
    /// Current state.
    pub state: TaskState,
    /// Optional detail text (shown on next line when completed).
    pub detail: Option<String>,
    /// Optional error message (for failures).
    pub error: Option<String>,
}

impl TaskItem {
    /// Create a new pending task.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            state: TaskState::Pending,
            detail: None,
            error: None,
        }
    }

    /// Set the detail text.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// Message type for task list.
#[derive(Debug, Clone)]
pub enum TaskListMsg {
    /// Advance spinner animation.
    Tick,
}

/// A list of tasks with status indicators.
#[derive(Debug, Clone)]
pub struct TaskList {
    /// The tasks.
    tasks: Vec<TaskItem>,
    /// Current spinner frame.
    spinner_frame: usize,
    /// Last tick time.
    last_tick: Option<Instant>,
    /// Width for rendering.
    width: usize,
    /// Whether all tasks are complete.
    all_complete: bool,
}

impl Default for TaskList {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskList {
    /// Create a new empty task list.
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            spinner_frame: 0,
            last_tick: None,
            width: 80,
            all_complete: false,
        }
    }

    /// Set the width for rendering.
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Add a task to the list.
    pub fn add_task(mut self, name: impl Into<String>) -> Self {
        self.tasks.push(TaskItem::new(name));
        self
    }

    /// Add a task with a detail.
    pub fn add_task_with_detail(
        mut self,
        name: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        self.tasks.push(TaskItem::new(name).with_detail(detail));
        self
    }

    /// Get the number of tasks.
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// Check if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Get a task by index.
    pub fn get(&self, index: usize) -> Option<&TaskItem> {
        self.tasks.get(index)
    }

    /// Get a mutable task by index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut TaskItem> {
        self.tasks.get_mut(index)
    }

    /// Start a task (set to Running).
    pub fn start_task(&mut self, index: usize) {
        if let Some(task) = self.tasks.get_mut(index) {
            task.state = TaskState::Running;
        }
        self.update_completion_status();
    }

    /// Complete a task successfully.
    pub fn complete_task(&mut self, index: usize, detail: Option<String>) {
        if let Some(task) = self.tasks.get_mut(index) {
            task.state = TaskState::Success;
            if let Some(d) = detail {
                task.detail = Some(d);
            }
        }
        self.update_completion_status();
    }

    /// Fail a task.
    pub fn fail_task(&mut self, index: usize, error: Option<String>) {
        if let Some(task) = self.tasks.get_mut(index) {
            task.state = TaskState::Failure;
            task.error = error;
        }
        self.update_completion_status();
    }

    /// Skip a task.
    pub fn skip_task(&mut self, index: usize, reason: Option<String>) {
        if let Some(task) = self.tasks.get_mut(index) {
            task.state = TaskState::Skipped;
            task.detail = reason;
        }
        self.update_completion_status();
    }

    /// Check if all tasks are complete.
    pub fn is_all_complete(&self) -> bool {
        self.all_complete
    }

    /// Check if any task has failed.
    pub fn has_failure(&self) -> bool {
        self.tasks.iter().any(|t| t.state == TaskState::Failure)
    }

    /// Get the first failed task.
    pub fn first_failure(&self) -> Option<&TaskItem> {
        self.tasks.iter().find(|t| t.state == TaskState::Failure)
    }

    /// Check if any task is currently running.
    pub fn is_running(&self) -> bool {
        self.tasks.iter().any(|t| t.state == TaskState::Running)
    }

    /// Get the index of the current running task.
    pub fn current_task_index(&self) -> Option<usize> {
        self.tasks
            .iter()
            .position(|t| t.state == TaskState::Running)
    }

    /// Update the completion status.
    fn update_completion_status(&mut self) {
        self.all_complete = self.tasks.iter().all(|t| t.state.is_terminal());
    }

    /// Render the task list as a string.
    pub fn render(&self) -> String {
        let mut output = String::new();
        let reset = "\x1b[0m";

        for task in &self.tasks {
            let icon = task.state.icon(self.spinner_frame);
            let color = task.state.color().to_ansi_fg();

            // Task line: icon + name
            output.push_str(&format!("{}{}{} {}\r\n", color, icon, reset, task.name));

            // Detail line (if present and task is complete or running)
            if let Some(ref detail) = task.detail {
                if task.state.is_terminal() || task.state == TaskState::Running {
                    let dim = Color::BrightBlack.to_ansi_fg();
                    output.push_str(&format!("{}  {}{}\r\n", dim, detail, reset));
                }
            }

            // Error line (if present)
            if let Some(ref error) = task.error {
                let red = Color::Red.to_ansi_fg();
                output.push_str(&format!("{}  {}{}\r\n", red, error, reset));
            }

            // Blank line after each task
            output.push_str("\r\n");
        }

        output
    }

    /// Count the lines rendered by this task list.
    pub fn line_count(&self) -> usize {
        let mut count = 0;
        for task in &self.tasks {
            count += 1; // Task line

            // Detail line
            if task.detail.is_some()
                && (task.state.is_terminal() || task.state == TaskState::Running)
            {
                count += 1;
            }

            // Error line
            if task.error.is_some() {
                count += 1;
            }

            count += 1; // Blank line
        }
        count
    }
}

impl Model for TaskList {
    type Message = TaskListMsg;

    fn init(&self) -> Option<Cmd<Self::Message>> {
        None
    }

    fn update(&mut self, msg: Self::Message) -> Option<Cmd<Self::Message>> {
        match msg {
            TaskListMsg::Tick => {
                self.spinner_frame = (self.spinner_frame + 1) % SPINNER_FRAMES.len();
                self.last_tick = Some(Instant::now());
            }
        }
        None
    }

    fn view(&self) -> String {
        self.render()
    }

    fn handle_event(&self, _event: Event) -> Option<Self::Message> {
        None
    }

    fn subscriptions(&self) -> Sub<Self::Message> {
        // Only animate if there are running tasks
        if self.is_running() {
            Sub::interval("task-spinner", Duration::from_millis(80), || {
                TaskListMsg::Tick
            })
        } else {
            Sub::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_list_creation() {
        let list = TaskList::new().add_task("Task 1").add_task("Task 2");

        assert_eq!(list.len(), 2);
        assert!(!list.is_all_complete());
    }

    #[test]
    fn test_task_state_transitions() {
        let mut list = TaskList::new().add_task("Task 1").add_task("Task 2");

        // Initially pending
        assert_eq!(list.get(0).unwrap().state, TaskState::Pending);

        // Start task
        list.start_task(0);
        assert_eq!(list.get(0).unwrap().state, TaskState::Running);
        assert!(list.is_running());

        // Complete task
        list.complete_task(0, Some("/path".to_string()));
        assert_eq!(list.get(0).unwrap().state, TaskState::Success);
        assert_eq!(list.get(0).unwrap().detail, Some("/path".to_string()));

        // Still not all complete
        assert!(!list.is_all_complete());

        // Complete second task
        list.complete_task(1, None);
        assert!(list.is_all_complete());
    }

    #[test]
    fn test_task_failure() {
        let mut list = TaskList::new().add_task("Task 1");

        list.start_task(0);
        list.fail_task(0, Some("Something went wrong".to_string()));

        assert!(list.has_failure());
        assert!(list.is_all_complete());
        assert!(list.first_failure().is_some());
    }

    #[test]
    fn test_task_icons() {
        assert_eq!(TaskState::Pending.icon(0), "○");
        assert_eq!(TaskState::Success.icon(0), "✓");
        assert_eq!(TaskState::Failure.icon(0), "✗");
        assert_eq!(TaskState::Skipped.icon(0), "⊘");
        // Running should animate
        assert!(SPINNER_FRAMES.contains(&TaskState::Running.icon(0)));
    }
}
