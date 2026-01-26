//! Program runner that manages the event loop.
//!
//! # Fluent API Design
//!
//! [`Program`] uses fluent method chaining for configuration. This pattern was evaluated
//! for `bon` builder macro conversion but **intentionally kept** because the Program is
//! used directly after configuration (no intermediate builder type). See PRD.md Task 10.

use std::{
    collections::HashMap,
    io::{self, Write},
    time::{Duration, Instant},
};

use crossterm::{
    cursor,
    event::{self, Event as CrosstermEvent, KeyEventKind},
    execute,
    terminal::{self, ClearType},
};

use super::{Model, command::CmdResult, subscription::SubEntry};
use crate::{Cmd, terminal::Event};

/// Type alias for pending tick entries: (scheduled_time, interval, message_generator)
type PendingTick<M> = (Instant, Duration, Box<dyn Fn(Instant) -> M + Send>);

/// Active subscription with next scheduled fire time
struct ActiveSub<M> {
    next_fire: Instant,
    interval: Duration,
    msg_fn: Box<dyn Fn() -> M + Send>,
}

/// Options for configuring the program runtime.
#[derive(Debug, Clone)]
pub struct ProgramOptions {
    /// Enable alternate screen mode (full-screen TUI).
    pub alt_screen: bool,

    /// Enable mouse capture.
    pub mouse: bool,

    /// Enable bracketed paste mode.
    pub bracketed_paste: bool,

    /// Enable focus change events.
    pub focus_change: bool,

    /// Frame rate for rendering (frames per second).
    pub fps: u32,

    /// Enable accessible mode (text-based prompts instead of TUI).
    pub accessible: bool,

    /// Respect NO_COLOR environment variable.
    pub respect_no_color: bool,

    /// Disable animations and spinners.
    pub reduce_motion: bool,

    /// Tick duration for models that want periodic updates.
    pub tick_rate: Duration,
}

impl Default for ProgramOptions {
    fn default() -> Self {
        Self {
            alt_screen: false,
            mouse: false,
            bracketed_paste: false,
            focus_change: false,
            fps: 60,
            accessible: std::env::var("ACCESSIBLE").is_ok(),
            respect_no_color: true,
            reduce_motion: std::env::var("REDUCE_MOTION").is_ok(),
            tick_rate: Duration::from_millis(100),
        }
    }
}

impl ProgramOptions {
    /// Create options for a full-screen TUI application.
    pub fn fullscreen() -> Self {
        Self { alt_screen: true, mouse: true, ..Default::default() }
    }

    /// Create options for an inline TUI (no alternate screen).
    pub fn inline() -> Self {
        Self::default()
    }
}

/// A message filter function that can modify or block messages.
///
/// Return `Some(msg)` to pass the message through (possibly modified),
/// or `None` to block the message from reaching the model.
pub type MessageFilter<M, Msg> = Box<dyn Fn(&M, Msg) -> Option<Msg> + Send>;

/// The program runtime that manages the event loop.
///
/// The program orchestrates:
/// - Terminal setup and teardown
/// - Event polling and dispatch
/// - Command execution
/// - View rendering
///
/// # Example
///
/// ```rust,no_run
/// use teapot::{Model, Program, Cmd, Event, KeyCode};
///
/// struct App { count: i32 }
/// enum Msg { Inc, Quit }
///
/// impl Model for App {
///     type Message = Msg;
///     fn init(&self) -> Option<Cmd<Self::Message>> { None }
///     fn update(&mut self, msg: Msg) -> Option<Cmd<Self::Message>> {
///         match msg {
///             Msg::Inc => self.count += 1,
///             Msg::Quit => return Some(Cmd::quit()),
///         }
///         None
///     }
///     fn view(&self) -> String { format!("Count: {}", self.count) }
///     fn handle_event(&self, event: Event) -> Option<Msg> {
///         match event {
///             Event::Key(k) if k.code == KeyCode::Char('q') => Some(Msg::Quit),
///             Event::Key(k) if k.code == KeyCode::Enter => Some(Msg::Inc),
///             _ => None,
///         }
///     }
/// }
///
/// let app = App { count: 0 };
/// Program::new(app).run().unwrap();
/// ```
pub struct Program<M: Model> {
    model: M,
    options: ProgramOptions,
    last_view: String,
    filter: Option<MessageFilter<M, M::Message>>,
}

impl<M: Model> Program<M> {
    /// Create a new program with the given model.
    pub fn new(model: M) -> Self {
        Self { model, options: ProgramOptions::default(), last_view: String::new(), filter: None }
    }

    /// Configure the program with custom options.
    pub fn with_options(mut self, options: ProgramOptions) -> Self {
        self.options = options;
        self
    }

    /// Add a message filter.
    ///
    /// The filter function is called before each message reaches the model's
    /// update function. It can:
    /// - Pass the message through unchanged: `Some(msg)`
    /// - Modify the message: `Some(modified_msg)`
    /// - Block the message: `None`
    ///
    /// This is useful for:
    /// - Logging all messages
    /// - Blocking certain inputs in specific states
    /// - Transforming messages globally
    ///
    /// # Example
    ///
    /// ```no_run
    /// use teapot::runtime::{Program, Model, Cmd};
    /// use teapot::terminal::Event;
    ///
    /// struct MyModel { has_unsaved_changes: bool }
    ///
    /// #[derive(Debug)]
    /// enum Msg { Quit }
    ///
    /// impl Model for MyModel {
    ///     type Message = Msg;
    ///     fn init(&self) -> Option<Cmd<Self::Message>> { None }
    ///     fn update(&mut self, _msg: Self::Message) -> Option<Cmd<Self::Message>> { None }
    ///     fn view(&self) -> String { String::new() }
    ///     fn handle_event(&self, _event: Event) -> Option<Self::Message> { None }
    /// }
    ///
    /// let model = MyModel { has_unsaved_changes: false };
    /// Program::new(model)
    ///     .with_filter(|model, msg| {
    ///         eprintln!("Message received: {:?}", msg);
    ///         if matches!(msg, Msg::Quit) && model.has_unsaved_changes {
    ///             None
    ///         } else {
    ///             Some(msg)
    ///         }
    ///     })
    ///     .run();
    /// ```
    pub fn with_filter<F>(mut self, filter: F) -> Self
    where
        F: Fn(&M, M::Message) -> Option<M::Message> + Send + 'static,
    {
        self.filter = Some(Box::new(filter));
        self
    }

    /// Enable alternate screen mode.
    pub fn with_alt_screen(mut self) -> Self {
        self.options.alt_screen = true;
        self
    }

    /// Enable mouse capture.
    pub fn with_mouse(mut self) -> Self {
        self.options.mouse = true;
        self
    }

    /// Set the frame rate.
    pub fn with_fps(mut self, fps: u32) -> Self {
        self.options.fps = fps.clamp(1, 120);
        self
    }

    /// Enable bracketed paste mode.
    ///
    /// When enabled, pasted text is delivered as a single `Event::Paste(String)`
    /// instead of individual key events.
    pub fn with_bracketed_paste(mut self) -> Self {
        self.options.bracketed_paste = true;
        self
    }

    /// Enable focus change events.
    ///
    /// When enabled, the program will receive `Event::FocusGained` and
    /// `Event::FocusLost` events when the terminal gains/loses focus.
    pub fn with_focus_change(mut self) -> Self {
        self.options.focus_change = true;
        self
    }

    /// Enable accessible mode.
    ///
    /// In accessible mode, the program renders text-based output suitable
    /// for screen readers instead of the full TUI.
    pub fn with_accessible(mut self) -> Self {
        self.options.accessible = true;
        self
    }

    /// Enable reduced motion mode.
    ///
    /// When enabled, animations and spinners are disabled or simplified.
    /// This respects the REDUCE_MOTION environment variable by default.
    pub fn with_reduce_motion(mut self) -> Self {
        self.options.reduce_motion = true;
        self
    }

    /// Set the tick rate for periodic updates.
    pub fn with_tick_rate(mut self, duration: Duration) -> Self {
        self.options.tick_rate = duration;
        self
    }

    /// Check if running in an interactive terminal.
    pub fn is_interactive() -> bool {
        use std::io::IsTerminal;
        std::io::stdin().is_terminal()
            && std::io::stdout().is_terminal()
            && std::env::var("CI").is_err()
    }

    /// Apply the message filter if one is set.
    ///
    /// Returns `Some(msg)` if the message should be processed,
    /// or `None` if it was blocked by the filter.
    fn apply_filter(&self, msg: M::Message) -> Option<M::Message> {
        match &self.filter {
            Some(filter) => filter(&self.model, msg),
            None => Some(msg),
        }
    }

    /// Run the program, blocking until it exits.
    ///
    /// Returns the final model state.
    pub fn run(mut self) -> io::Result<M> {
        if !Self::is_interactive() || self.options.accessible {
            return self.run_non_interactive();
        }

        self.setup_terminal()?;
        let result = self.run_interactive();
        self.teardown_terminal()?;

        result.map(|_| self.model)
    }

    /// Run in interactive mode with full TUI.
    fn run_interactive(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();
        let frame_duration = Duration::from_secs(1) / self.options.fps;

        // Create pending_ticks before init so Cmd::tick works from init()
        let mut pending_ticks: Vec<PendingTick<M::Message>> = Vec::new();
        let mut active_subs: HashMap<String, ActiveSub<M::Message>> = HashMap::new();

        // Run init command (may schedule ticks)
        if let Some(cmd) = self.model.init()
            && self.process_command_with_ticks(cmd, &mut pending_ticks)?
        {
            return Ok(());
        }

        // Initial render
        self.render(&mut stdout)?;

        // Initialize subscriptions
        self.refresh_subscriptions(&mut active_subs);

        loop {
            let now = Instant::now();
            let mut messages = Vec::new();
            let mut needs_sub_refresh = false;

            // Check for pending ticks (from Cmd::tick)
            pending_ticks.retain(|(scheduled, _, msg_fn)| {
                if now >= *scheduled {
                    messages.push(msg_fn(now));
                    false
                } else {
                    true
                }
            });

            // Check for subscription fires
            for sub in active_subs.values_mut() {
                if now >= sub.next_fire {
                    messages.push((sub.msg_fn)());
                    sub.next_fire = now + sub.interval;
                }
            }

            // Process accumulated messages (applying filter)
            for msg in messages {
                // Apply the message filter
                let msg = match self.apply_filter(msg) {
                    Some(m) => m,
                    None => continue, // Message was blocked
                };

                if let Some(cmd) = self.model.update(msg)
                    && self.process_command_with_ticks(cmd, &mut pending_ticks)?
                {
                    return Ok(());
                }
                needs_sub_refresh = true;
                self.render(&mut stdout)?;
            }

            // Refresh subscriptions if model was updated
            if needs_sub_refresh {
                self.refresh_subscriptions(&mut active_subs);
            }

            // Calculate poll timeout (min of ticks, subs, and frame duration)
            let tick_timeout = pending_ticks
                .iter()
                .map(|(scheduled, ..)| scheduled.saturating_duration_since(now))
                .min();

            let sub_timeout =
                active_subs.values().map(|sub| sub.next_fire.saturating_duration_since(now)).min();

            let timeout = [tick_timeout, sub_timeout]
                .into_iter()
                .flatten()
                .min()
                .unwrap_or(frame_duration)
                .min(frame_duration);

            // Poll for events
            if event::poll(timeout)? {
                let crossterm_event = event::read()?;
                let event = Event::from(crossterm_event.clone());

                // Convert to message and update (applying filter)
                if let Some(msg) = self.model.handle_event(event) {
                    // Apply the message filter
                    if let Some(msg) = self.apply_filter(msg) {
                        if let Some(cmd) = self.model.update(msg)
                            && self.process_command_with_ticks(cmd, &mut pending_ticks)?
                        {
                            return Ok(());
                        }
                        // Refresh subscriptions after update
                        self.refresh_subscriptions(&mut active_subs);
                        self.render(&mut stdout)?;
                    }
                }

                // Handle special events
                match crossterm_event {
                    CrosstermEvent::Resize(..) => {
                        self.render(&mut stdout)?;
                    },
                    CrosstermEvent::Key(key) if key.kind == KeyEventKind::Press => {
                        // Ctrl+C handling as fallback
                        if key.code == crossterm::event::KeyCode::Char('c')
                            && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                        {
                            // Check if the model handled it
                            // If not, we could force quit here
                        }
                    },
                    _ => {},
                }
            }
        }
    }

    /// Refresh active subscriptions based on current model state.
    fn refresh_subscriptions(&self, active_subs: &mut HashMap<String, ActiveSub<M::Message>>) {
        let new_subs = self.model.subscriptions();
        let entries: Vec<SubEntry<M::Message>> = new_subs.into_entries();

        // Collect new subscription IDs
        let new_ids: std::collections::HashSet<_> = entries.iter().map(|e| e.id.clone()).collect();

        // Remove subscriptions that are no longer active
        active_subs.retain(|id, _| new_ids.contains(id));

        // Add or update subscriptions
        let now = Instant::now();
        for entry in entries {
            active_subs.entry(entry.id).or_insert_with(|| ActiveSub {
                next_fire: now + entry.interval,
                interval: entry.interval,
                msg_fn: entry.msg_fn,
            });
        }
    }

    /// Process a command, returning true if we should quit.
    fn process_command(&mut self, cmd: Cmd<M::Message>) -> io::Result<bool> {
        let mut pending_ticks = Vec::new();
        self.process_command_with_ticks(cmd, &mut pending_ticks)
    }

    /// Process a command with tick handling.
    fn process_command_with_ticks(
        &mut self,
        cmd: Cmd<M::Message>,
        pending_ticks: &mut Vec<PendingTick<M::Message>>,
    ) -> io::Result<bool> {
        match cmd.execute() {
            CmdResult::None => Ok(false),
            CmdResult::Quit => Ok(true),
            CmdResult::Message(msg) => {
                if let Some(next_cmd) = self.model.update(msg) {
                    self.process_command_with_ticks(next_cmd, pending_ticks)
                } else {
                    Ok(false)
                }
            },
            CmdResult::Tick { duration, msg_fn } => {
                let scheduled = Instant::now() + duration;
                pending_ticks.push((scheduled, duration, msg_fn));
                Ok(false)
            },
            CmdResult::Batch(cmds) => {
                for cmd in cmds {
                    if self.process_command_with_ticks(cmd, pending_ticks)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            },
            CmdResult::Sequence(cmds) => {
                for cmd in cmds {
                    if self.process_command_with_ticks(cmd, pending_ticks)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            },
            CmdResult::Async(_future) => {
                // For now, we don't support async commands in the sync runtime
                // This would require tokio integration
                Ok(false)
            },
            CmdResult::RunProcess { mut command, on_exit } => {
                // Teardown terminal before running external process
                self.teardown_terminal()?;

                // Run the external process
                let result = command.status();

                // Re-setup terminal after process completes
                self.setup_terminal()?;

                // Force a full redraw by clearing the last view
                self.last_view.clear();

                // Call the callback with the result
                let msg = on_exit(result);
                if let Some(next_cmd) = self.model.update(msg) {
                    self.process_command_with_ticks(next_cmd, pending_ticks)
                } else {
                    Ok(false)
                }
            },
        }
    }

    /// Run in non-interactive mode (CI, piped input).
    ///
    /// In non-interactive mode, the program displays the initial view
    /// and returns immediately. This is appropriate for:
    /// - CI environments
    /// - Piped input/output
    /// - Scripts
    ///
    /// For accessible mode with screen readers, use `Form::run_accessible()`
    /// or implement custom accessible handling using the `Accessible` trait.
    fn run_non_interactive(self) -> io::Result<M> {
        use super::accessible::strip_ansi;

        // In non-interactive mode, display a stripped view (no ANSI codes)
        // This ensures clean output for CI/scripts
        let view = self.model.view();
        let clean_view = if self.options.accessible { strip_ansi(&view) } else { view };
        println!("{}", clean_view);

        // Note: For interactive accessible mode, use Form::run_accessible()
        // or implement custom handling with the Accessible trait
        if self.options.accessible {
            eprintln!("Note: For interactive accessible mode, use Form::run_accessible()");
        }

        Ok(self.model)
    }

    /// Set up the terminal for TUI mode.
    fn setup_terminal(&self) -> io::Result<()> {
        terminal::enable_raw_mode()?;

        let mut stdout = io::stdout();

        if self.options.alt_screen {
            execute!(stdout, terminal::EnterAlternateScreen, cursor::MoveTo(0, 0))?;
        }

        if self.options.mouse {
            execute!(stdout, event::EnableMouseCapture)?;
        }

        if self.options.bracketed_paste {
            execute!(stdout, event::EnableBracketedPaste)?;
        }

        if self.options.focus_change {
            execute!(stdout, event::EnableFocusChange)?;
        }

        execute!(stdout, cursor::Hide)?;

        Ok(())
    }

    /// Tear down the terminal, restoring original state.
    fn teardown_terminal(&self) -> io::Result<()> {
        let mut stdout = io::stdout();

        execute!(stdout, cursor::Show)?;

        if self.options.focus_change {
            execute!(stdout, event::DisableFocusChange)?;
        }

        if self.options.bracketed_paste {
            execute!(stdout, event::DisableBracketedPaste)?;
        }

        if self.options.mouse {
            execute!(stdout, event::DisableMouseCapture)?;
        }

        if self.options.alt_screen {
            execute!(stdout, terminal::LeaveAlternateScreen)?;
        }

        terminal::disable_raw_mode()?;

        // Ensure cursor is at column 0 for clean output after TUI exits
        execute!(stdout, cursor::MoveToColumn(0))?;

        Ok(())
    }

    /// Render the current view.
    fn render(&mut self, stdout: &mut impl Write) -> io::Result<()> {
        let view = self.model.view();

        // Only redraw if the view changed
        if view != self.last_view {
            if self.options.alt_screen {
                execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
            } else {
                // For inline mode, clear previous lines
                let prev_lines = self.last_view.lines().count();
                if prev_lines > 0 {
                    execute!(
                        stdout,
                        cursor::MoveUp(prev_lines as u16),
                        terminal::Clear(ClearType::FromCursorDown)
                    )?;
                }
            }

            write!(stdout, "{}", view)?;
            stdout.flush()?;

            self.last_view = view;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestModel {
        count: i32,
    }

    enum TestMsg {
        Inc,
        #[allow(dead_code)]
        Quit,
    }

    impl Model for TestModel {
        type Message = TestMsg;

        fn init(&self) -> Option<Cmd<Self::Message>> {
            None
        }

        fn update(&mut self, msg: Self::Message) -> Option<Cmd<Self::Message>> {
            match msg {
                TestMsg::Inc => self.count += 1,
                TestMsg::Quit => return Some(Cmd::quit()),
            }
            None
        }

        fn view(&self) -> String {
            format!("Count: {}", self.count)
        }
    }

    #[test]
    fn test_program_creation() {
        let model = TestModel { count: 0 };
        let program = Program::new(model);
        assert_eq!(program.model.count, 0);
    }

    #[test]
    fn test_program_options() {
        let options = ProgramOptions::fullscreen();
        assert!(options.alt_screen);
        assert!(options.mouse);

        let options = ProgramOptions::inline();
        assert!(!options.alt_screen);
        assert!(!options.mouse);
    }

    #[test]
    fn test_program_builder_pattern() {
        let model = TestModel { count: 0 };
        let program = Program::new(model)
            .with_alt_screen()
            .with_mouse()
            .with_fps(30)
            .with_bracketed_paste()
            .with_focus_change()
            .with_reduce_motion()
            .with_tick_rate(Duration::from_millis(50));

        assert!(program.options.alt_screen);
        assert!(program.options.mouse);
        assert_eq!(program.options.fps, 30);
        assert!(program.options.bracketed_paste);
        assert!(program.options.focus_change);
        assert!(program.options.reduce_motion);
        assert_eq!(program.options.tick_rate, Duration::from_millis(50));
    }

    #[test]
    fn test_fps_clamping() {
        let model = TestModel { count: 0 };

        let program = Program::new(model).with_fps(0);
        assert_eq!(program.options.fps, 1);

        let model = TestModel { count: 0 };
        let program = Program::new(model).with_fps(200);
        assert_eq!(program.options.fps, 120);
    }

    #[test]
    fn test_message_filter() {
        let model = TestModel { count: 0 };

        // Test that filter is None by default
        let program = Program::new(model);
        assert!(program.filter.is_none());

        // Test that filter can be set
        let model = TestModel { count: 0 };
        let program = Program::new(model).with_filter(|_model, msg| {
            // Pass all messages through
            Some(msg)
        });
        assert!(program.filter.is_some());
    }

    #[test]
    fn test_apply_filter_pass_through() {
        let model = TestModel { count: 0 };
        let program = Program::new(model).with_filter(|_model, msg| Some(msg));

        // Filter should pass the message through
        let result = program.apply_filter(TestMsg::Inc);
        assert!(matches!(result, Some(TestMsg::Inc)));
    }

    #[test]
    fn test_apply_filter_block() {
        let model = TestModel { count: 0 };
        let program = Program::new(model).with_filter(|_model, _msg| {
            // Block all messages
            None
        });

        // Filter should block the message
        let result = program.apply_filter(TestMsg::Inc);
        assert!(result.is_none());
    }

    #[test]
    fn test_run_process_command() {
        use std::process::Command;

        // Just verify we can create a run_process command
        let mut cmd = Command::new("echo");
        cmd.arg("test");

        let _cmd: Cmd<TestMsg> = Cmd::run_process(cmd, |result| {
            if result.map(|s| s.success()).unwrap_or(false) { TestMsg::Inc } else { TestMsg::Quit }
        });
    }
}
