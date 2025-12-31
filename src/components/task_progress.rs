//! Task progress view for multi-step operations.
//!
//! A full-screen TUI component showing step-by-step progress with:
//! - Title bar with configurable title and subtitle
//! - Task list with animated spinners and completion status
//! - Worker thread execution for async steps
//! - Optional confirmation modal before starting
//! - Error modal on failure
//! - Footer with keyboard hints
//!
//! # Example
//!
//! ```rust,ignore
//! use teapot::components::{TaskProgressView, TaskStep};
//!
//! let steps = vec![
//!     TaskStep::with_executor("Clone repository", || {
//!         // Clone logic...
//!         Ok(None)
//!     }),
//!     TaskStep::with_executor("Build project", || {
//!         // Build logic...
//!         Ok(Some("Built successfully".to_string()))
//!     }),
//! ];
//!
//! let view = TaskProgressView::builder(steps)
//!     .title("My Application")
//!     .subtitle("Install")
//!     .auto_start()
//!     .build();
//!
//! Program::new(view).with_alt_screen().run()?;
//! ```

use std::{any::Any, sync::Arc, time::Duration};

use crate::{
    components::{FooterHints, Modal, ModalBorder, TaskList, TitleBar},
    runtime::{Cmd, Model, Sub},
    style::Color,
    terminal::{Event, KeyCode},
    util::WorkerHandle,
};

// ============================================================================
// Core Types
// ============================================================================

/// Type alias for step executor function.
/// Returns Ok(detail) on success, or Err(error_message) on failure.
/// Ok(Some(reason)) indicates the step was skipped with that reason.
pub type StepExecutor = Arc<dyn Fn() -> Result<Option<String>, String> + Send + Sync>;

/// A step in the task progress.
#[derive(Clone)]
pub struct TaskStep {
    /// Step name displayed to user.
    pub name: String,
    /// Optional executor function for this step.
    pub executor: Option<StepExecutor>,
}

impl std::fmt::Debug for TaskStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskStep")
            .field("name", &self.name)
            .field("executor", &self.executor.is_some())
            .finish()
    }
}

impl TaskStep {
    /// Create a new task step without an executor.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), executor: None }
    }

    /// Create a step with an executor function.
    ///
    /// The executor should return:
    /// - `Ok(None)` for success
    /// - `Ok(Some(reason))` to indicate the step was skipped
    /// - `Err(message)` for failure
    pub fn with_executor<F>(name: impl Into<String>, executor: F) -> Self
    where
        F: Fn() -> Result<Option<String>, String> + Send + Sync + 'static,
    {
        Self { name: name.into(), executor: Some(Arc::new(executor)) }
    }
}

/// Result of running a step.
#[derive(Debug, Clone)]
pub enum StepResult {
    /// Step completed successfully.
    Success(Option<String>),
    /// Step was skipped (with reason).
    Skipped(String),
    /// Step failed with error.
    Failure(String),
}

/// Phase of the task progress operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    /// Showing confirmation modal (if configured).
    Confirming,
    /// Ready to start but not yet started.
    Ready,
    /// Running steps.
    Running,
    /// All steps completed (success or failure).
    Completed,
}

// ============================================================================
// Messages
// ============================================================================

/// Message type for task progress view.
#[derive(Debug, Clone)]
pub enum TaskProgressMsg {
    /// Advance spinner animation and poll for worker results.
    Tick,
    /// Start the execution process.
    Start,
    /// User confirmed (when confirmation modal is shown).
    Confirm,
    /// User cancelled/declined.
    Cancel,
    /// Run a specific step by index.
    RunStep(usize),
    /// A step completed with result (internal).
    StepCompleted(usize, StepResult),
    /// Start a task manually (for external control mode).
    StartTask(usize),
    /// Complete a task with result (for external control mode).
    CompleteTask(usize, StepResult),
    /// Close error modal.
    CloseModal,
    /// User pressed quit/cancel key.
    Quit,
    /// Terminal resized.
    Resize(u16, u16),
}

// ============================================================================
// Configuration
// ============================================================================

/// Configuration for footer hints.
#[derive(Debug, Clone)]
pub struct HintConfig {
    /// Hints shown during confirmation phase.
    pub confirming: Vec<(String, String)>,
    /// Hints shown while running.
    pub running: Vec<(String, String)>,
    /// Hints shown when completed.
    pub completed: Vec<(String, String)>,
}

impl Default for HintConfig {
    fn default() -> Self {
        Self {
            confirming: vec![
                ("y".to_string(), "confirm".to_string()),
                ("n".to_string(), "cancel".to_string()),
            ],
            running: vec![("q".to_string(), "cancel".to_string())],
            completed: vec![("q".to_string(), "quit".to_string())],
        }
    }
}

/// Type alias for the content function used in confirmation modals.
pub type ConfirmationContentFn = Box<dyn Fn(&dyn Any) -> Vec<String> + Send + Sync>;

/// Configuration for confirmation modal.
pub struct ConfirmationConfig {
    /// Modal title.
    pub title: String,
    /// Title color.
    pub title_color: Color,
    /// Border color.
    pub border_color: Color,
    /// Function to generate content lines from the context.
    pub content_fn: ConfirmationContentFn,
}

impl std::fmt::Debug for ConfirmationConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfirmationConfig")
            .field("title", &self.title)
            .field("title_color", &self.title_color)
            .field("border_color", &self.border_color)
            .finish()
    }
}

/// Full configuration for task progress view.
#[derive(Debug, Default)]
pub struct TaskProgressConfig {
    /// Whether to auto-start execution.
    pub auto_start: bool,
    /// Whether to allow external control via StartTask/CompleteTask messages.
    pub external_control: bool,
    /// Footer hints configuration.
    pub hints: HintConfig,
}

// ============================================================================
// Builder
// ============================================================================

/// Builder for TaskProgressView.
pub struct TaskProgressViewBuilder {
    steps: Vec<TaskStep>,
    title: String,
    subtitle: String,
    config: TaskProgressConfig,
    confirmation: Option<ConfirmationConfig>,
    context: Option<Box<dyn Any + Send + Sync>>,
}

impl TaskProgressViewBuilder {
    /// Create a new builder with the given steps.
    pub fn new(steps: Vec<TaskStep>) -> Self {
        Self {
            steps,
            title: String::new(),
            subtitle: String::new(),
            config: TaskProgressConfig::default(),
            confirmation: None,
            context: None,
        }
    }

    /// Create a new builder with steps and a context for confirmation.
    pub fn with_context<C: Any + Send + Sync>(steps: Vec<TaskStep>, context: C) -> Self {
        Self {
            steps,
            title: String::new(),
            subtitle: String::new(),
            config: TaskProgressConfig::default(),
            confirmation: None,
            context: Some(Box::new(context)),
        }
    }

    /// Set the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the subtitle.
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = subtitle.into();
        self
    }

    /// Enable auto-start (begin executing immediately).
    pub fn auto_start(mut self) -> Self {
        self.config.auto_start = true;
        self
    }

    /// Enable external control mode.
    pub fn external_control(mut self) -> Self {
        self.config.external_control = true;
        self
    }

    /// Set confirmation modal configuration.
    pub fn with_confirmation(mut self, config: ConfirmationConfig) -> Self {
        self.confirmation = Some(config);
        self
    }

    /// Set footer hints for the confirming phase.
    pub fn hints_confirming(mut self, hints: Vec<(&str, &str)>) -> Self {
        self.config.hints.confirming =
            hints.into_iter().map(|(k, d)| (k.to_string(), d.to_string())).collect();
        self
    }

    /// Set footer hints for the running phase.
    pub fn hints_running(mut self, hints: Vec<(&str, &str)>) -> Self {
        self.config.hints.running =
            hints.into_iter().map(|(k, d)| (k.to_string(), d.to_string())).collect();
        self
    }

    /// Set footer hints for the completed phase.
    pub fn hints_completed(mut self, hints: Vec<(&str, &str)>) -> Self {
        self.config.hints.completed =
            hints.into_iter().map(|(k, d)| (k.to_string(), d.to_string())).collect();
        self
    }

    /// Build the TaskProgressView.
    pub fn build(self) -> TaskProgressView {
        let initial_phase =
            if self.confirmation.is_some() { Phase::Confirming } else { Phase::Ready };

        TaskProgressView::new_internal(
            self.steps,
            self.title,
            self.subtitle,
            self.config,
            self.confirmation,
            self.context,
            initial_phase,
        )
    }
}

// ============================================================================
// TaskProgressView
// ============================================================================

/// Result message from a worker thread.
type WorkerResult = (usize, StepResult);

/// A full-screen view for multi-step task execution with progress display.
pub struct TaskProgressView {
    // Display
    title: String,
    subtitle: String,

    // Tasks
    task_list: TaskList,
    executors: Vec<Option<StepExecutor>>,
    current_step: usize,

    // State
    phase: Phase,
    worker: Option<WorkerHandle<WorkerResult>>,

    // Terminal
    width: u16,
    height: u16,

    // Modals
    error_modal: Option<(String, String)>,

    // Exit state
    should_quit: bool,
    was_cancelled: bool,

    // Configuration
    config: TaskProgressConfig,
    confirmation: Option<ConfirmationConfig>,
    confirmation_context: Option<Box<dyn Any + Send + Sync>>,
}

impl TaskProgressView {
    /// Create a builder for a new TaskProgressView with the given steps.
    ///
    /// Use the builder pattern for configuration:
    /// ```rust,ignore
    /// TaskProgressView::builder(steps)
    ///     .title("My App")
    ///     .auto_start()
    ///     .build()
    /// ```
    pub fn builder(steps: Vec<TaskStep>) -> TaskProgressViewBuilder {
        TaskProgressViewBuilder::new(steps)
    }

    /// Create a new TaskProgressView with steps and a context.
    ///
    /// The context is used by the confirmation modal's content_fn.
    pub fn with_context<C: Any + Send + Sync>(
        steps: Vec<TaskStep>,
        context: C,
    ) -> TaskProgressViewBuilder {
        TaskProgressViewBuilder::with_context(steps, context)
    }

    /// Internal constructor.
    fn new_internal(
        steps: Vec<TaskStep>,
        title: String,
        subtitle: String,
        config: TaskProgressConfig,
        confirmation: Option<ConfirmationConfig>,
        context: Option<Box<dyn Any + Send + Sync>>,
        initial_phase: Phase,
    ) -> Self {
        let mut task_list = TaskList::new();
        let mut executors = Vec::new();

        for step in steps {
            task_list = task_list.add_task(&step.name);
            executors.push(step.executor);
        }

        let (width, height) = crate::terminal::size().unwrap_or((80, 24));

        Self {
            title,
            subtitle,
            task_list,
            executors,
            current_step: 0,
            phase: initial_phase,
            worker: None,
            width,
            height,
            error_modal: None,
            should_quit: false,
            was_cancelled: false,
            config,
            confirmation,
            confirmation_context: context,
        }
    }

    // ========================================================================
    // Public API
    // ========================================================================

    /// Check if the view should quit.
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Check if the operation was cancelled by user.
    pub fn was_cancelled(&self) -> bool {
        self.was_cancelled
    }

    /// Check if completed successfully.
    pub fn is_success(&self) -> bool {
        self.phase == Phase::Completed
            && self.task_list.is_all_complete()
            && !self.task_list.has_failure()
    }

    /// Check if there was a failure.
    pub fn has_failure(&self) -> bool {
        self.task_list.has_failure()
    }

    /// Start a task by index (for external control mode).
    pub fn start_task(&mut self, index: usize) {
        self.task_list.start_task(index);
    }

    /// Complete a task with result (for external control mode).
    pub fn complete_task(&mut self, index: usize, result: StepResult) {
        match result {
            StepResult::Success(detail) => {
                self.task_list.complete_task(index, detail);
            },
            StepResult::Skipped(reason) => {
                self.task_list.skip_task(index, Some(reason));
            },
            StepResult::Failure(error) => {
                self.task_list.fail_task(index, Some(error.clone()));
                if let Some(task) = self.task_list.get(index) {
                    self.error_modal = Some((task.name.clone(), error));
                }
            },
        }

        // Update phase if all tasks are complete
        if self.task_list.is_all_complete() {
            self.phase = Phase::Completed;
        }
    }

    /// Check if a task is currently running.
    pub fn is_running(&self) -> bool {
        self.task_list.is_running()
    }

    /// Check if all tasks are complete.
    pub fn is_all_complete(&self) -> bool {
        self.task_list.is_all_complete()
    }

    /// Get current task index.
    pub fn current_task_index(&self) -> Option<usize> {
        self.task_list.current_task_index()
    }

    /// Get the current phase.
    pub fn phase(&self) -> &Phase {
        &self.phase
    }

    // ========================================================================
    // Private Methods
    // ========================================================================

    /// Render the title bar with dimmed slashes.
    fn render_title_bar(&self) -> String {
        if self.title.is_empty() {
            return String::new();
        }

        let mut bar = TitleBar::new(&self.title).width(self.width as usize);
        if !self.subtitle.is_empty() {
            bar = bar.subtitle(&self.subtitle);
        }
        bar.render()
    }

    /// Render the footer with right-aligned hints.
    fn render_footer(&self) -> String {
        let hints = match self.phase {
            Phase::Confirming => &self.config.hints.confirming,
            Phase::Ready | Phase::Running => &self.config.hints.running,
            Phase::Completed => &self.config.hints.completed,
        };

        FooterHints::new()
            .hints(hints.iter().map(|(k, d)| (k.as_str(), d.as_str())).collect())
            .width(self.width as usize)
            .with_separator()
            .render()
    }

    /// Spawn a worker thread to execute a step.
    fn spawn_step_worker(&self, index: usize) -> WorkerHandle<WorkerResult> {
        if let Some(Some(executor)) = self.executors.get(index) {
            let executor = Arc::clone(executor);
            WorkerHandle::spawn(move || {
                let result = executor();
                let step_result = match result {
                    Ok(None) => StepResult::Success(None),
                    Ok(Some(reason)) => StepResult::Skipped(reason),
                    Err(error) => StepResult::Failure(error),
                };
                (index, step_result)
            })
        } else {
            // No executor - auto-succeed
            WorkerHandle::spawn(move || (index, StepResult::Success(None)))
        }
    }

    /// Poll for worker result.
    fn poll_worker_result(&mut self) -> Option<WorkerResult> {
        if let Some(ref handle) = self.worker {
            if let Some(result) = handle.try_recv() {
                self.worker = None;
                return Some(result);
            }
        }
        None
    }

    /// Check if currently executing a step.
    fn is_executing(&self) -> bool {
        self.worker.is_some()
    }

    /// Render the confirmation modal.
    fn render_confirm_modal(&self, background: &str) -> String {
        let Some(ref conf) = self.confirmation else {
            return background.to_string();
        };

        let content_lines = if let Some(ref ctx) = self.confirmation_context {
            (conf.content_fn)(ctx.as_ref())
        } else {
            vec![]
        };

        let modal_width = 60.min(self.width as usize - 4);
        let modal_height = (content_lines.len() + 8).min(self.height as usize - 4);

        let mut all_lines = vec!["This will:".to_string(), String::new()];
        for line in &content_lines {
            all_lines.push(format!("  • {}", line));
        }
        all_lines.push(String::new());
        all_lines.push("Are you sure you want to continue?".to_string());

        let modal = Modal::new(modal_width, modal_height)
            .border(ModalBorder::Rounded)
            .border_color(conf.border_color.clone())
            .title(&conf.title)
            .title_color(conf.title_color.clone())
            .content(all_lines.join("\n"))
            .footer_hints(vec![("y", "confirm"), ("n", "cancel")]);

        modal.render_overlay(self.width as usize, self.height as usize, background)
    }

    /// Render the error modal.
    fn render_error_modal(&self, background: &str) -> String {
        if let Some((ref task_name, ref error_msg)) = self.error_modal {
            let modal_width = 60.min(self.width as usize - 4);
            let modal_height = 10.min(self.height as usize - 4);

            let modal = Modal::new(modal_width, modal_height)
                .border(ModalBorder::Rounded)
                .border_color(Color::Red)
                .title("Error")
                .title_color(Color::Red)
                .content(format!("Failed: {}\n\n{}", task_name, error_msg))
                .footer_hint("esc", "close");

            modal.render_overlay(self.width as usize, self.height as usize, background)
        } else {
            background.to_string()
        }
    }

    /// Start execution of steps.
    fn start_execution(&mut self) -> Option<Cmd<TaskProgressMsg>> {
        if !self.executors.is_empty() {
            self.phase = Phase::Running;
            self.current_step = 0;
            self.task_list.start_task(0);
            self.worker = Some(self.spawn_step_worker(0));
            return Some(Cmd::tick(Duration::from_millis(80), |_| TaskProgressMsg::Tick));
        }
        None
    }
}

impl Model for TaskProgressView {
    type Message = TaskProgressMsg;

    fn init(&self) -> Option<Cmd<Self::Message>> {
        if self.config.auto_start && self.phase == Phase::Ready {
            // Schedule start after brief delay for initial render
            Some(Cmd::tick(Duration::from_millis(100), |_| TaskProgressMsg::Start))
        } else {
            None
        }
    }

    fn update(&mut self, msg: Self::Message) -> Option<Cmd<Self::Message>> {
        match msg {
            TaskProgressMsg::Tick => {
                // Forward tick to task list for spinner animation
                self.task_list.update(crate::components::TaskListMsg::Tick);

                // Poll for worker result if we're executing
                if let Some((index, result)) = self.poll_worker_result() {
                    match &result {
                        StepResult::Success(detail) => {
                            self.task_list.complete_task(index, detail.clone());
                        },
                        StepResult::Skipped(reason) => {
                            self.task_list.skip_task(index, Some(reason.clone()));
                        },
                        StepResult::Failure(error) => {
                            self.task_list.fail_task(index, Some(error.clone()));
                            if let Some(task) = self.task_list.get(index) {
                                self.error_modal = Some((task.name.clone(), error.clone()));
                            }
                            self.phase = Phase::Completed;
                            return None;
                        },
                    }

                    // Start next step if there is one
                    let next_step = index + 1;
                    if next_step < self.executors.len() {
                        self.current_step = next_step;
                        self.task_list.start_task(next_step);
                        self.worker = Some(self.spawn_step_worker(next_step));
                        return Some(Cmd::tick(Duration::from_millis(80), |_| {
                            TaskProgressMsg::Tick
                        }));
                    } else {
                        self.phase = Phase::Completed;
                    }
                }
                None
            },
            TaskProgressMsg::Start => {
                if self.phase == Phase::Ready {
                    return self.start_execution();
                }
                None
            },
            TaskProgressMsg::Confirm => {
                if self.phase == Phase::Confirming {
                    return self.start_execution();
                }
                None
            },
            TaskProgressMsg::Cancel => {
                self.was_cancelled = true;
                self.should_quit = true;
                Some(Cmd::quit())
            },
            TaskProgressMsg::RunStep(index) => {
                if self.config.external_control
                    && index < self.executors.len()
                    && !self.is_executing()
                {
                    self.task_list.start_task(index);
                    self.worker = Some(self.spawn_step_worker(index));
                }
                None
            },
            TaskProgressMsg::StepCompleted(index, result) => {
                // Manual completion (for external control)
                match &result {
                    StepResult::Success(detail) => {
                        self.task_list.complete_task(index, detail.clone());
                    },
                    StepResult::Skipped(reason) => {
                        self.task_list.skip_task(index, Some(reason.clone()));
                    },
                    StepResult::Failure(error) => {
                        self.task_list.fail_task(index, Some(error.clone()));
                        if let Some(task) = self.task_list.get(index) {
                            self.error_modal = Some((task.name.clone(), error.clone()));
                        }
                    },
                }
                None
            },
            TaskProgressMsg::StartTask(index) => {
                if self.config.external_control {
                    self.start_task(index);
                }
                None
            },
            TaskProgressMsg::CompleteTask(index, result) => {
                if self.config.external_control {
                    self.complete_task(index, result);
                }
                None
            },
            TaskProgressMsg::CloseModal => {
                self.error_modal = None;
                None
            },
            TaskProgressMsg::Quit => {
                if self.error_modal.is_some() {
                    self.error_modal = None;
                    None
                } else {
                    self.should_quit = true;
                    if self.phase != Phase::Completed {
                        self.was_cancelled = true;
                    }
                    Some(Cmd::quit())
                }
            },
            TaskProgressMsg::Resize(w, h) => {
                self.width = w;
                self.height = h;
                None
            },
        }
    }

    fn view(&self) -> String {
        let mut output = String::new();

        // Title bar
        output.push_str(&self.render_title_bar());
        output.push_str("\r\n\r\n");

        // Task list
        output.push_str(&self.task_list.render());

        // Calculate remaining space for padding
        let title_lines = 2;
        let task_lines = self.task_list.line_count();
        let footer_lines = 2;
        let content_lines = title_lines + task_lines;
        let available = self.height as usize;

        if available > content_lines + footer_lines {
            let padding = available - content_lines - footer_lines;
            for _ in 0..padding {
                output.push_str("\r\n");
            }
        }

        // Footer (hidden hints when modal is showing)
        let has_modal = self.phase == Phase::Confirming || self.error_modal.is_some();
        if has_modal {
            let dim = Color::BrightBlack.to_ansi_fg();
            let reset = "\x1b[0m";
            output.push_str(&format!(
                "{}{}{}\r\n{}",
                dim,
                "─".repeat(self.width as usize),
                reset,
                " ".repeat(self.width as usize)
            ));
        } else {
            output.push_str(&self.render_footer());
        }

        // Overlay modal based on state
        if self.phase == Phase::Confirming {
            self.render_confirm_modal(&output)
        } else if self.error_modal.is_some() {
            self.render_error_modal(&output)
        } else {
            output
        }
    }

    fn handle_event(&self, event: Event) -> Option<Self::Message> {
        match event {
            Event::Key(key) => {
                // Error modal takes precedence
                if self.error_modal.is_some() {
                    return match key.code {
                        KeyCode::Esc => Some(TaskProgressMsg::CloseModal),
                        _ => None,
                    };
                }

                match self.phase {
                    Phase::Confirming => match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => Some(TaskProgressMsg::Confirm),
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            Some(TaskProgressMsg::Cancel)
                        },
                        KeyCode::Char('q') => Some(TaskProgressMsg::Cancel),
                        _ => None,
                    },
                    Phase::Ready | Phase::Running | Phase::Completed => match key.code {
                        KeyCode::Char('q') => Some(TaskProgressMsg::Quit),
                        _ => None,
                    },
                }
            },
            Event::Resize { width, height } => Some(TaskProgressMsg::Resize(width, height)),
            _ => None,
        }
    }

    fn subscriptions(&self) -> Sub<Self::Message> {
        // Keep ticking while executing
        if self.is_executing() || self.task_list.is_running() {
            Sub::interval("task-progress-spinner", Duration::from_millis(80), || {
                TaskProgressMsg::Tick
            })
        } else {
            Sub::none()
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_step_creation() {
        let step = TaskStep::new("Test step");
        assert_eq!(step.name, "Test step");
        assert!(step.executor.is_none());
    }

    #[test]
    fn test_task_step_with_executor() {
        let step = TaskStep::with_executor("Test step", || Ok(None));
        assert_eq!(step.name, "Test step");
        assert!(step.executor.is_some());
    }

    #[test]
    fn test_builder_pattern() {
        let steps = vec![TaskStep::new("Step 1"), TaskStep::new("Step 2")];
        let view = TaskProgressView::builder(steps)
            .title("Test Title")
            .subtitle("Test Subtitle")
            .auto_start()
            .build();

        assert_eq!(view.title, "Test Title");
        assert_eq!(view.subtitle, "Test Subtitle");
        assert!(view.config.auto_start);
    }

    #[test]
    fn test_initial_phase_with_confirmation() {
        let steps = vec![TaskStep::new("Step 1")];
        let view = TaskProgressView::with_context(steps, ())
            .with_confirmation(ConfirmationConfig {
                title: "Confirm".to_string(),
                title_color: Color::Yellow,
                border_color: Color::Yellow,
                content_fn: Box::new(|_| vec!["Test".to_string()]),
            })
            .build();

        assert_eq!(view.phase, Phase::Confirming);
    }

    #[test]
    fn test_initial_phase_without_confirmation() {
        let steps = vec![TaskStep::new("Step 1")];
        let view = TaskProgressView::builder(steps).auto_start().build();

        assert_eq!(view.phase, Phase::Ready);
    }

    #[test]
    fn test_external_control() {
        let steps = vec![TaskStep::new("Step 1")];
        let mut view = TaskProgressView::builder(steps).external_control().build();

        view.start_task(0);
        assert!(view.is_running());

        view.complete_task(0, StepResult::Success(Some("Done".to_string())));
        assert!(!view.is_running());
        assert!(view.is_all_complete());
    }

    #[test]
    fn test_failure_handling() {
        let steps = vec![TaskStep::new("Step 1")];
        let mut view = TaskProgressView::builder(steps).external_control().build();

        view.start_task(0);
        view.complete_task(0, StepResult::Failure("Error!".to_string()));

        assert!(view.has_failure());
        assert!(view.error_modal.is_some());
    }

    #[test]
    fn test_step_result_skipped() {
        let steps = vec![TaskStep::new("Step 1")];
        let mut view = TaskProgressView::builder(steps).external_control().build();

        view.start_task(0);
        view.complete_task(0, StepResult::Skipped("Already exists".to_string()));

        assert!(view.is_all_complete());
        assert!(!view.has_failure());
    }
}
