//! Multi-progress component for parallel operations.
//!
//! Displays multiple progress bars for tracking parallel tasks.
//!
//! # Example
//!
//! ```rust
//! use teapot::components::MultiProgress;
//!
//! let mp = MultiProgress::new()
//!     .add_task("download", "Downloading files...", 100)
//!     .add_task("compile", "Compiling...", 50)
//!     .add_task("test", "Running tests...", 200);
//! ```

use std::collections::HashMap;

use crate::{
    runtime::{Cmd, Model},
    style::Color,
    terminal::Event,
};

/// Message type for multi-progress.
#[derive(Debug, Clone)]
pub enum MultiProgressMsg {
    /// Set progress for a task.
    SetProgress { id: String, current: u64 },
    /// Increment progress for a task.
    Increment { id: String, amount: u64 },
    /// Set message for a task.
    SetMessage { id: String, message: String },
    /// Mark a task as complete.
    Complete { id: String },
    /// Mark a task as failed.
    Fail { id: String, error: String },
    /// Add a new task.
    AddTask { id: String, message: String, total: u64 },
    /// Remove a task.
    RemoveTask { id: String },
}

/// Status of a task in the multi-progress.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TaskStatus {
    /// Task is in progress.
    InProgress,
    /// Task completed successfully.
    Completed,
    /// Task failed with an error.
    Failed(String),
}

/// A single task within the multi-progress.
#[derive(Debug, Clone)]
#[must_use = "components do nothing unless used in a view or run with Program"]
pub struct Task {
    /// Unique identifier for the task.
    pub id: String,
    /// Display message for the task.
    pub message: String,
    /// Current progress value.
    pub current: u64,
    /// Total progress value.
    pub total: u64,
    /// Task status.
    pub status: TaskStatus,
}

impl Task {
    /// Create a new task.
    pub fn new(id: impl Into<String>, message: impl Into<String>, total: u64) -> Self {
        Self {
            id: id.into(),
            message: message.into(),
            current: 0,
            total,
            status: TaskStatus::InProgress,
        }
    }

    /// Get the progress percentage.
    pub fn percentage(&self) -> f64 {
        if self.total == 0 { 100.0 } else { (self.current as f64 / self.total as f64) * 100.0 }
    }

    /// Check if task is complete (either success or failure).
    pub fn is_done(&self) -> bool {
        matches!(self.status, TaskStatus::Completed | TaskStatus::Failed(_))
    }
}

/// A multi-progress component for tracking parallel operations.
#[derive(Debug, Clone)]
#[must_use = "components do nothing unless used in a view or run with Program"]
pub struct MultiProgress {
    tasks: Vec<Task>,
    task_index: HashMap<String, usize>,
    title: String,
    width: usize,
    show_percentage: bool,
    show_count: bool,
    filled_char: char,
    empty_char: char,
    in_progress_color: Color,
    completed_color: Color,
    failed_color: Color,
    empty_color: Color,
    remove_completed: bool,
    show_summary: bool,
}

impl Default for MultiProgress {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiProgress {
    /// Create a new multi-progress.
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            task_index: HashMap::new(),
            title: String::new(),
            width: 30,
            show_percentage: true,
            show_count: false,
            filled_char: '█',
            empty_char: '░',
            in_progress_color: Color::Cyan,
            completed_color: Color::Green,
            failed_color: Color::Red,
            empty_color: Color::BrightBlack,
            remove_completed: false,
            show_summary: true,
        }
    }

    /// Set a title for the multi-progress display.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Add a task.
    pub fn add_task(
        mut self,
        id: impl Into<String>,
        message: impl Into<String>,
        total: u64,
    ) -> Self {
        let task = Task::new(id, message, total);
        let idx = self.tasks.len();
        self.task_index.insert(task.id.clone(), idx);
        self.tasks.push(task);
        self
    }

    /// Set the progress bar width.
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Set whether to show percentage.
    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    /// Set whether to show count.
    pub fn show_count(mut self, show: bool) -> Self {
        self.show_count = show;
        self
    }

    /// Set the filled character.
    pub fn filled_char(mut self, c: char) -> Self {
        self.filled_char = c;
        self
    }

    /// Set the empty character.
    pub fn empty_char(mut self, c: char) -> Self {
        self.empty_char = c;
        self
    }

    /// Set the in-progress color.
    pub fn in_progress_color(mut self, color: Color) -> Self {
        self.in_progress_color = color;
        self
    }

    /// Set the completed color.
    pub fn completed_color(mut self, color: Color) -> Self {
        self.completed_color = color;
        self
    }

    /// Set the failed color.
    pub fn failed_color(mut self, color: Color) -> Self {
        self.failed_color = color;
        self
    }

    /// Set whether to remove completed tasks from display.
    pub fn remove_completed(mut self, remove: bool) -> Self {
        self.remove_completed = remove;
        self
    }

    /// Set whether to show a summary line.
    pub fn show_summary(mut self, show: bool) -> Self {
        self.show_summary = show;
        self
    }

    /// Get a task by ID.
    pub fn get_task(&self, id: &str) -> Option<&Task> {
        self.task_index.get(id).and_then(|&idx| self.tasks.get(idx))
    }

    /// Get a mutable task by ID.
    fn get_task_mut(&mut self, id: &str) -> Option<&mut Task> {
        self.task_index.get(id).copied().and_then(move |idx| self.tasks.get_mut(idx))
    }

    /// Get all tasks.
    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    /// Get the number of tasks.
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    /// Get the number of completed tasks.
    pub fn completed_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.status == TaskStatus::Completed).count()
    }

    /// Get the number of failed tasks.
    pub fn failed_count(&self) -> usize {
        self.tasks.iter().filter(|t| matches!(t.status, TaskStatus::Failed(_))).count()
    }

    /// Get the number of in-progress tasks.
    pub fn in_progress_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.status == TaskStatus::InProgress).count()
    }

    /// Check if all tasks are done.
    pub fn is_all_done(&self) -> bool {
        self.tasks.iter().all(|t| t.is_done())
    }

    /// Get overall progress percentage.
    pub fn overall_percentage(&self) -> f64 {
        if self.tasks.is_empty() {
            return 100.0;
        }

        let total: u64 = self.tasks.iter().map(|t| t.total).sum();
        let current: u64 = self.tasks.iter().map(|t| t.current).sum();

        if total == 0 { 100.0 } else { (current as f64 / total as f64) * 100.0 }
    }

    /// Set progress for a task.
    pub fn set_progress(&mut self, id: &str, current: u64) {
        if let Some(task) = self.get_task_mut(id) {
            task.current = current.min(task.total);
            if task.current >= task.total && task.status == TaskStatus::InProgress {
                task.status = TaskStatus::Completed;
            }
        }
    }

    /// Increment progress for a task.
    pub fn increment(&mut self, id: &str, amount: u64) {
        if let Some(task) = self.get_task_mut(id) {
            task.current = (task.current + amount).min(task.total);
            if task.current >= task.total && task.status == TaskStatus::InProgress {
                task.status = TaskStatus::Completed;
            }
        }
    }

    /// Set message for a task.
    pub fn set_message(&mut self, id: &str, message: String) {
        if let Some(task) = self.get_task_mut(id) {
            task.message = message;
        }
    }

    /// Mark a task as complete.
    pub fn complete_task(&mut self, id: &str) {
        if let Some(task) = self.get_task_mut(id) {
            task.current = task.total;
            task.status = TaskStatus::Completed;
        }
    }

    /// Mark a task as failed.
    pub fn fail_task(&mut self, id: &str, error: String) {
        if let Some(task) = self.get_task_mut(id) {
            task.status = TaskStatus::Failed(error);
        }
    }

    /// Add a new task dynamically.
    pub fn add_task_dynamic(&mut self, id: String, message: String, total: u64) {
        if !self.task_index.contains_key(&id) {
            let task = Task::new(id.clone(), message, total);
            let idx = self.tasks.len();
            self.task_index.insert(id, idx);
            self.tasks.push(task);
        }
    }

    /// Remove a task.
    pub fn remove_task(&mut self, id: &str) {
        if let Some(&idx) = self.task_index.get(id) {
            drop(self.tasks.remove(idx));
            self.task_index.remove(id);
            // Rebuild index
            self.task_index.clear();
            for (i, task) in self.tasks.iter().enumerate() {
                self.task_index.insert(task.id.clone(), i);
            }
        }
    }

    /// Render a single progress bar.
    fn render_bar(&self, task: &Task) -> String {
        let pct = task.percentage();
        let filled_count = ((pct / 100.0) * self.width as f64).round() as usize;
        let empty_count = self.width.saturating_sub(filled_count);

        let filled = self.filled_char.to_string().repeat(filled_count);
        let empty = self.empty_char.to_string().repeat(empty_count);

        let bar_color = match &task.status {
            TaskStatus::InProgress => self.in_progress_color.clone(),
            TaskStatus::Completed => self.completed_color.clone(),
            TaskStatus::Failed(_) => self.failed_color.clone(),
        };

        format!(
            "{}{}{}{}{}",
            bar_color.to_ansi_fg(),
            filled,
            self.empty_color.to_ansi_fg(),
            empty,
            "\x1b[0m"
        )
    }
}

impl Model for MultiProgress {
    type Message = MultiProgressMsg;

    fn init(&self) -> Option<Cmd<Self::Message>> {
        None
    }

    fn update(&mut self, msg: Self::Message) -> Option<Cmd<Self::Message>> {
        match msg {
            MultiProgressMsg::SetProgress { id, current } => {
                self.set_progress(&id, current);
            },
            MultiProgressMsg::Increment { id, amount } => {
                self.increment(&id, amount);
            },
            MultiProgressMsg::SetMessage { id, message } => {
                self.set_message(&id, message);
            },
            MultiProgressMsg::Complete { id } => {
                self.complete_task(&id);
            },
            MultiProgressMsg::Fail { id, error } => {
                self.fail_task(&id, error);
            },
            MultiProgressMsg::AddTask { id, message, total } => {
                self.add_task_dynamic(id, message, total);
            },
            MultiProgressMsg::RemoveTask { id } => {
                self.remove_task(&id);
            },
        }

        // Optionally remove completed tasks
        if self.remove_completed {
            let completed_ids: Vec<_> = self
                .tasks
                .iter()
                .filter(|t| t.status == TaskStatus::Completed)
                .map(|t| t.id.clone())
                .collect();
            for id in completed_ids {
                self.remove_task(&id);
            }
        }

        None
    }

    fn view(&self) -> String {
        let mut output = String::new();

        // Title
        if !self.title.is_empty() {
            output.push_str(&format!("{}\n", self.title));
        }

        if self.tasks.is_empty() {
            output.push_str(&format!("{}(no tasks){}", Color::BrightBlack.to_ansi_fg(), "\x1b[0m"));
            return output;
        }

        // Render each task
        for (i, task) in self.tasks.iter().enumerate() {
            // Status indicator
            let status_indicator = match &task.status {
                TaskStatus::InProgress => {
                    format!("{}◐{}", self.in_progress_color.to_ansi_fg(), "\x1b[0m")
                },
                TaskStatus::Completed => {
                    format!("{}✓{}", self.completed_color.to_ansi_fg(), "\x1b[0m")
                },
                TaskStatus::Failed(_) => {
                    format!("{}✗{}", self.failed_color.to_ansi_fg(), "\x1b[0m")
                },
            };

            // Task line
            let bar = self.render_bar(task);

            let mut parts = Vec::new();
            parts.push(status_indicator);
            parts.push(task.message.clone());
            parts.push(format!("[{}]", bar));

            if self.show_percentage {
                parts.push(format!("{:.0}%", task.percentage()));
            }

            if self.show_count {
                parts.push(format!("{}/{}", task.current, task.total));
            }

            output.push_str(&parts.join(" "));

            // Show error message if failed
            if let TaskStatus::Failed(error) = &task.status {
                output.push_str(&format!(
                    "\n    {}{}{}",
                    self.failed_color.to_ansi_fg(),
                    error,
                    "\x1b[0m"
                ));
            }

            if i < self.tasks.len() - 1 {
                output.push('\n');
            }
        }

        // Summary
        if self.show_summary && !self.tasks.is_empty() {
            output.push_str(&format!(
                "\n\n{}─────────────────────────────{}",
                Color::BrightBlack.to_ansi_fg(),
                "\x1b[0m"
            ));

            let completed = self.completed_count();
            let failed = self.failed_count();
            let total = self.task_count();

            output.push_str(&format!(
                "\n{}{}/{} completed{}",
                if completed == total && failed == 0 {
                    self.completed_color.to_ansi_fg()
                } else {
                    Color::Default.to_ansi_fg()
                },
                completed,
                total,
                "\x1b[0m"
            ));

            if failed > 0 {
                output.push_str(&format!(
                    " ({}{} failed{})",
                    self.failed_color.to_ansi_fg(),
                    failed,
                    "\x1b[0m"
                ));
            }

            output.push_str(&format!(" • {:.0}% overall", self.overall_percentage()));
        }

        output
    }

    fn handle_event(&self, _event: Event) -> Option<Self::Message> {
        // MultiProgress is typically driven by external messages, not events
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_progress_creation() {
        let mp =
            MultiProgress::new().add_task("task1", "Task 1", 100).add_task("task2", "Task 2", 50);

        assert_eq!(mp.task_count(), 2);
        assert_eq!(mp.completed_count(), 0);
        assert_eq!(mp.in_progress_count(), 2);
    }

    #[test]
    fn test_task_progress() {
        let mut mp = MultiProgress::new().add_task("task1", "Task 1", 100);

        mp.set_progress("task1", 50);
        assert_eq!(mp.get_task("task1").unwrap().current, 50);

        mp.increment("task1", 25);
        assert_eq!(mp.get_task("task1").unwrap().current, 75);
    }

    #[test]
    fn test_task_completion() {
        let mut mp = MultiProgress::new().add_task("task1", "Task 1", 100);

        assert!(!mp.is_all_done());
        mp.complete_task("task1");
        assert!(mp.is_all_done());
        assert_eq!(mp.completed_count(), 1);
    }

    #[test]
    fn test_task_failure() {
        let mut mp = MultiProgress::new().add_task("task1", "Task 1", 100);

        mp.fail_task("task1", "Something went wrong".to_string());
        assert!(mp.is_all_done());
        assert_eq!(mp.failed_count(), 1);
    }

    #[test]
    fn test_overall_percentage() {
        let mut mp =
            MultiProgress::new().add_task("task1", "Task 1", 100).add_task("task2", "Task 2", 100);

        mp.set_progress("task1", 50);
        mp.set_progress("task2", 50);
        assert_eq!(mp.overall_percentage(), 50.0);

        mp.complete_task("task1");
        assert_eq!(mp.overall_percentage(), 75.0);
    }

    #[test]
    fn test_dynamic_tasks() {
        let mut mp = MultiProgress::new();

        mp.add_task_dynamic("task1".to_string(), "Task 1".to_string(), 100);
        assert_eq!(mp.task_count(), 1);

        mp.remove_task("task1");
        assert_eq!(mp.task_count(), 0);
    }
}
