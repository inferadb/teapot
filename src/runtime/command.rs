//! Command type for side effects.
//!
//! Commands represent IO operations that produce messages when complete.
//! They are the only way to perform side effects in the Elm Architecture.

use std::{
    future::Future,
    pin::Pin,
    process::Command as ProcessCommand,
    time::{Duration, Instant},
};

/// A command representing an IO operation that produces a message.
///
/// Commands are returned from `Model::init` and `Model::update` to request
/// side effects. The runtime executes commands and delivers their results
/// as messages.
///
/// # Example
///
/// ```rust
/// use teapot::Cmd;
/// use std::time::Duration;
///
/// enum Msg {
///     Tick,
///     DataLoaded(String),
/// }
///
/// // Create a tick command
/// let tick_cmd: Cmd<Msg> = Cmd::tick(Duration::from_millis(100), |_| Msg::Tick);
///
/// // Create a quit command
/// let quit_cmd: Cmd<Msg> = Cmd::quit();
/// ```
#[must_use = "commands do nothing unless returned from update()"]
pub struct Cmd<M> {
    inner: CmdInner<M>,
}

enum CmdInner<M> {
    /// No-op command
    None,
    /// Quit the program
    Quit,
    /// A synchronous action that produces a message
    Sync(Box<dyn FnOnce() -> M + Send>),
    /// A tick timer
    Tick { duration: Duration, msg_fn: Box<dyn Fn(Instant) -> M + Send> },
    /// Batch of commands to run concurrently
    Batch(Vec<Cmd<M>>),
    /// Sequence of commands to run in order
    Sequence(Vec<Cmd<M>>),
    /// An async action
    Async(Pin<Box<dyn Future<Output = M> + Send>>),
    /// Run an external process (suspends the TUI)
    RunProcess {
        command: ProcessCommand,
        on_exit: Box<dyn FnOnce(std::io::Result<std::process::ExitStatus>) -> M + Send>,
    },
}

impl<M> Cmd<M> {
    /// Create a no-op command.
    ///
    /// This is useful when you need to return a command but have nothing to do.
    #[inline]
    pub fn none() -> Self {
        Self { inner: CmdInner::None }
    }

    /// Create a command to quit the program.
    ///
    /// # Example
    ///
    /// ```rust
    /// use teapot::Cmd;
    ///
    /// enum Msg { Quit }
    ///
    /// fn handle_quit() -> Cmd<Msg> {
    ///     Cmd::quit()
    /// }
    /// ```
    #[inline]
    pub fn quit() -> Self {
        Self { inner: CmdInner::Quit }
    }

    /// Create a tick command that fires after a duration.
    ///
    /// The message function receives the instant when the tick fired.
    /// To create a recurring tick, return another tick command from update.
    ///
    /// # Example
    ///
    /// ```rust
    /// use teapot::Cmd;
    /// use std::time::Duration;
    ///
    /// enum Msg { Tick }
    ///
    /// let cmd: Cmd<Msg> = Cmd::tick(Duration::from_millis(100), |_| Msg::Tick);
    /// ```
    pub fn tick<F>(duration: Duration, msg_fn: F) -> Self
    where
        F: Fn(Instant) -> M + Send + 'static,
    {
        Self { inner: CmdInner::Tick { duration, msg_fn: Box::new(msg_fn) } }
    }

    /// Create a command from a synchronous function.
    ///
    /// The function is called once and its return value becomes the message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use teapot::Cmd;
    ///
    /// enum Msg { Data(String) }
    ///
    /// let cmd: Cmd<Msg> = Cmd::perform(|| Msg::Data("loaded".to_string()));
    /// ```
    pub fn perform<F>(f: F) -> Self
    where
        F: FnOnce() -> M + Send + 'static,
    {
        Self { inner: CmdInner::Sync(Box::new(f)) }
    }

    /// Create a command from an async future.
    ///
    /// The future is executed and its result becomes the message.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use teapot::Cmd;
    ///
    /// enum Msg { Data(String) }
    ///
    /// async fn fetch_data() -> Msg {
    ///     // ... async operation
    ///     Msg::Data("loaded".to_string())
    /// }
    ///
    /// let cmd: Cmd<Msg> = Cmd::perform_async(fetch_data());
    /// ```
    pub fn perform_async<F>(future: F) -> Self
    where
        F: Future<Output = M> + Send + 'static,
    {
        Self { inner: CmdInner::Async(Box::pin(future)) }
    }

    /// Run an external process, suspending the TUI.
    ///
    /// This command temporarily exits the TUI, runs the specified process,
    /// waits for it to complete, and then restores the TUI. The provided
    /// function receives the process exit status.
    ///
    /// This is useful for launching editors, pagers, or other interactive
    /// programs that need full terminal control.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use teapot::Cmd;
    /// use std::process::Command;
    ///
    /// enum Msg { EditorClosed(bool) }
    ///
    /// // Open a file in the user's default editor
    /// let mut command = Command::new("vim");
    /// command.arg("file.txt");
    /// let cmd: Cmd<Msg> = Cmd::run_process(
    ///     command,
    ///     |result| Msg::EditorClosed(result.map(|s| s.success()).unwrap_or(false))
    /// );
    /// ```
    pub fn run_process<F>(command: ProcessCommand, on_exit: F) -> Self
    where
        F: FnOnce(std::io::Result<std::process::ExitStatus>) -> M + Send + 'static,
    {
        Self { inner: CmdInner::RunProcess { command, on_exit: Box::new(on_exit) } }
    }

    /// Batch multiple commands to run concurrently.
    ///
    /// All commands execute simultaneously with no ordering guarantees.
    ///
    /// # Example
    ///
    /// ```rust
    /// use teapot::Cmd;
    ///
    /// enum Msg { A, B }
    ///
    /// let cmd: Cmd<Msg> = Cmd::batch(vec![
    ///     Cmd::perform(|| Msg::A),
    ///     Cmd::perform(|| Msg::B),
    /// ]);
    /// ```
    pub fn batch(cmds: Vec<Cmd<M>>) -> Self {
        Self { inner: CmdInner::Batch(cmds) }
    }

    /// Sequence commands to run in order.
    ///
    /// Each command completes before the next starts.
    ///
    /// # Example
    ///
    /// ```rust
    /// use teapot::Cmd;
    ///
    /// enum Msg { First, Second }
    ///
    /// let cmd: Cmd<Msg> = Cmd::sequence(vec![
    ///     Cmd::perform(|| Msg::First),
    ///     Cmd::perform(|| Msg::Second),
    /// ]);
    /// ```
    pub fn sequence(cmds: Vec<Cmd<M>>) -> Self {
        Self { inner: CmdInner::Sequence(cmds) }
    }

    /// Transform the message type of this command.
    ///
    /// This is useful for composing commands from child components.
    ///
    /// # Example
    ///
    /// ```rust
    /// use teapot::Cmd;
    ///
    /// enum ChildMsg { Done }
    /// enum ParentMsg { Child(ChildMsg) }
    ///
    /// let child_cmd: Cmd<ChildMsg> = Cmd::perform(|| ChildMsg::Done);
    /// let parent_cmd: Cmd<ParentMsg> = child_cmd.map(ParentMsg::Child);
    /// ```
    pub fn map<N, F>(self, f: F) -> Cmd<N>
    where
        F: Fn(M) -> N + Send + Sync + Clone + 'static,
        M: Send + 'static,
        N: Send + 'static,
    {
        match self.inner {
            CmdInner::None => Cmd::none(),
            CmdInner::Quit => Cmd::quit(),
            CmdInner::Sync(action) => {
                let f = f.clone();
                Cmd::perform(move || f(action()))
            },
            CmdInner::Tick { duration, msg_fn } => {
                Cmd::tick(duration, move |instant| f(msg_fn(instant)))
            },
            CmdInner::Batch(cmds) => {
                Cmd::batch(cmds.into_iter().map(|c| c.map(f.clone())).collect())
            },
            CmdInner::Sequence(cmds) => {
                Cmd::sequence(cmds.into_iter().map(|c| c.map(f.clone())).collect())
            },
            CmdInner::Async(future) => {
                let f = f.clone();
                Cmd::perform_async(async move { f(future.await) })
            },
            CmdInner::RunProcess { command, on_exit } => {
                let f = f.clone();
                Cmd::run_process(command, move |result| f(on_exit(result)))
            },
        }
    }

    /// Execute this command synchronously, returning messages.
    ///
    /// This is used by the runtime to process commands.
    pub(crate) fn execute(self) -> CmdResult<M> {
        match self.inner {
            CmdInner::None => CmdResult::None,
            CmdInner::Quit => CmdResult::Quit,
            CmdInner::Sync(f) => CmdResult::Message(f()),
            CmdInner::Tick { duration, msg_fn } => CmdResult::Tick { duration, msg_fn },
            CmdInner::Batch(cmds) => CmdResult::Batch(cmds),
            CmdInner::Sequence(cmds) => CmdResult::Sequence(cmds),
            CmdInner::Async(future) => CmdResult::Async(future),
            CmdInner::RunProcess { command, on_exit } => CmdResult::RunProcess { command, on_exit },
        }
    }
}

impl<M> Default for Cmd<M> {
    fn default() -> Self {
        Self::none()
    }
}

impl<M> std::fmt::Debug for Cmd<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            CmdInner::None => write!(f, "Cmd::None"),
            CmdInner::Quit => write!(f, "Cmd::Quit"),
            CmdInner::Sync(_) => write!(f, "Cmd::Sync(...)"),
            CmdInner::Tick { duration, .. } => write!(f, "Cmd::Tick({:?})", duration),
            CmdInner::Batch(cmds) => write!(f, "Cmd::Batch({} cmds)", cmds.len()),
            CmdInner::Sequence(cmds) => write!(f, "Cmd::Sequence({} cmds)", cmds.len()),
            CmdInner::Async(_) => write!(f, "Cmd::Async(...)"),
            CmdInner::RunProcess { .. } => write!(f, "Cmd::RunProcess(...)"),
        }
    }
}

/// Internal result of executing a command.
pub(crate) enum CmdResult<M> {
    None,
    Quit,
    Message(M),
    Tick {
        duration: Duration,
        msg_fn: Box<dyn Fn(Instant) -> M + Send>,
    },
    Batch(Vec<Cmd<M>>),
    Sequence(Vec<Cmd<M>>),
    Async(Pin<Box<dyn Future<Output = M> + Send>>),
    RunProcess {
        command: ProcessCommand,
        on_exit: Box<dyn FnOnce(std::io::Result<std::process::ExitStatus>) -> M + Send>,
    },
}

// ============================================================================
// Module-level command functions (Bubble Tea style)
// ============================================================================

/// Create a no-op command.
///
/// Module-level function equivalent to `Cmd::none()`.
///
/// # Example
///
/// ```rust
/// use teapot::runtime::cmd;
///
/// enum Msg { Done }
///
/// let cmd: teapot::Cmd<Msg> = cmd::none();
/// ```
pub fn none<M>() -> Cmd<M> {
    Cmd::none()
}

/// Create a command to quit the program.
///
/// Module-level function equivalent to `Cmd::quit()`.
///
/// # Example
///
/// ```rust
/// use teapot::runtime::cmd;
///
/// enum Msg { Quit }
///
/// let cmd: teapot::Cmd<Msg> = cmd::quit();
/// ```
pub fn quit<M>() -> Cmd<M> {
    Cmd::quit()
}

/// Batch multiple commands to run concurrently.
///
/// Module-level function equivalent to `Cmd::batch()`.
///
/// # Example
///
/// ```rust
/// use teapot::runtime::cmd;
///
/// enum Msg { A, B }
///
/// let cmd: teapot::Cmd<Msg> = cmd::batch(vec![
///     cmd::none(),
///     cmd::quit(),
/// ]);
/// ```
pub fn batch<M>(cmds: Vec<Cmd<M>>) -> Cmd<M> {
    Cmd::batch(cmds)
}

/// Sequence commands to run in order.
///
/// Module-level function equivalent to `Cmd::sequence()`.
///
/// # Example
///
/// ```rust
/// use teapot::runtime::cmd;
///
/// enum Msg { First, Second }
///
/// let cmd: teapot::Cmd<Msg> = cmd::sequence(vec![
///     cmd::none(),
///     cmd::quit(),
/// ]);
/// ```
pub fn sequence<M>(cmds: Vec<Cmd<M>>) -> Cmd<M> {
    Cmd::sequence(cmds)
}

/// Create a tick command that fires after a duration.
///
/// Module-level function equivalent to `Cmd::tick()`.
///
/// # Example
///
/// ```rust
/// use teapot::runtime::cmd;
/// use std::time::Duration;
///
/// enum Msg { Tick }
///
/// let cmd: teapot::Cmd<Msg> = cmd::tick(Duration::from_millis(100), |_| Msg::Tick);
/// ```
pub fn tick<M, F>(duration: Duration, msg_fn: F) -> Cmd<M>
where
    F: Fn(Instant) -> M + Send + 'static,
{
    Cmd::tick(duration, msg_fn)
}

/// Run an external process, suspending the TUI.
///
/// Module-level function equivalent to `Cmd::run_process()`.
///
/// # Example
///
/// ```no_run
/// use teapot::runtime::cmd;
/// use std::process::Command;
///
/// enum Msg { EditorClosed(bool) }
///
/// let mut command = Command::new("vim");
/// command.arg("file.txt");
/// let cmd: teapot::Cmd<Msg> = cmd::run_process(
///     command,
///     |result| Msg::EditorClosed(result.map(|s| s.success()).unwrap_or(false))
/// );
/// ```
pub fn run_process<M, F>(command: ProcessCommand, on_exit: F) -> Cmd<M>
where
    F: FnOnce(std::io::Result<std::process::ExitStatus>) -> M + Send + 'static,
{
    Cmd::run_process(command, on_exit)
}
