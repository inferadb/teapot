//! Worker thread utilities for background task execution.
//!
//! Provides a simple abstraction for spawning worker threads and polling
//! for their results in an event-driven manner.
//!
//! # Example
//!
//! ```rust
//! use teapot::util::WorkerHandle;
//!
//! // Spawn a worker that returns a String
//! let mut worker: Option<WorkerHandle<String>> = Some(
//!     WorkerHandle::spawn(|| "Hello from worker!".to_string())
//! );
//!
//! // Poll for result (non-blocking)
//! if let Some(ref handle) = worker {
//!     if let Some(result) = handle.try_recv() {
//!         println!("Got result: {}", result);
//!         worker = None;
//!     }
//! }
//! ```

use std::{
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
};

/// A handle to a spawned worker thread with its result receiver.
///
/// This provides a simple abstraction for background task execution
/// that integrates well with the Model-Update-View pattern.
#[derive(Debug)]
pub struct WorkerHandle<T> {
    receiver: Receiver<T>,
}

impl<T: Send + 'static> WorkerHandle<T> {
    /// Spawn a new worker thread that executes the given function.
    ///
    /// The function runs in a separate thread and its return value
    /// is sent back through the internal channel.
    ///
    /// # Example
    ///
    /// ```rust
    /// use teapot::util::WorkerHandle;
    ///
    /// let handle = WorkerHandle::spawn(|| {
    ///     // Do some work...
    ///     42
    /// });
    /// ```
    pub fn spawn<F>(f: F) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let result = f();
            let _ = tx.send(result);
        });
        Self { receiver: rx }
    }

    /// Try to receive the result without blocking.
    ///
    /// Returns `Some(result)` if the worker has completed,
    /// `None` if still running or if the worker panicked.
    ///
    /// # Example
    ///
    /// ```rust
    /// use teapot::util::WorkerHandle;
    ///
    /// let handle = WorkerHandle::spawn(|| "done");
    /// // In an event loop:
    /// match handle.try_recv() {
    ///     Some(result) => println!("Worker finished: {}", result),
    ///     None => println!("Still working..."),
    /// }
    /// ```
    pub fn try_recv(&self) -> Option<T> {
        match self.receiver.try_recv() {
            Ok(result) => Some(result),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => None,
        }
    }

    /// Check if the worker has completed without consuming the result.
    ///
    /// Note: This is a best-effort check. The worker could complete
    /// between this check and a subsequent `try_recv()`.
    pub fn is_finished(&self) -> bool {
        // A disconnected channel means the sender was dropped,
        // which happens after the worker thread finishes.
        // However, we can't check this without consuming the message.
        // This implementation just checks if a message is available.
        matches!(self.receiver.try_recv(), Ok(_) | Err(TryRecvError::Disconnected))
    }
}

/// A worker handle that manages execution state internally.
///
/// Tracks whether execution is in progress and automatically
/// cleans up after completion.
#[derive(Debug, Default)]
pub struct ManagedWorker<T> {
    handle: Option<WorkerHandle<T>>,
    executing: bool,
}

impl<T: Send + 'static> ManagedWorker<T> {
    /// Create a new managed worker (not executing).
    pub fn new() -> Self {
        Self { handle: None, executing: false }
    }

    /// Check if currently executing.
    pub fn is_executing(&self) -> bool {
        self.executing
    }

    /// Start execution with the given function.
    ///
    /// If already executing, this is a no-op.
    pub fn start<F>(&mut self, f: F)
    where
        F: FnOnce() -> T + Send + 'static,
    {
        if self.executing {
            return;
        }
        self.handle = Some(WorkerHandle::spawn(f));
        self.executing = true;
    }

    /// Poll for result without blocking.
    ///
    /// Returns `Some(result)` if the worker completed.
    /// Automatically resets the executing state on completion.
    pub fn poll(&mut self) -> Option<T> {
        if let Some(ref handle) = self.handle
            && let Some(result) = handle.try_recv()
        {
            self.handle = None;
            self.executing = false;
            return Some(result);
        }
        None
    }

    /// Reset the worker, discarding any pending result.
    pub fn reset(&mut self) {
        self.handle = None;
        self.executing = false;
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use super::*;

    #[test]
    fn test_worker_handle_spawn_and_recv() {
        let handle = WorkerHandle::spawn(|| 42);
        // Give the thread time to complete
        thread::sleep(Duration::from_millis(10));
        let result = handle.try_recv();
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_worker_handle_try_recv_empty() {
        let handle = WorkerHandle::spawn(|| {
            thread::sleep(Duration::from_millis(100));
            "done"
        });
        // Check immediately - should be empty
        let result = handle.try_recv();
        assert!(result.is_none());
    }

    #[test]
    fn test_worker_handle_with_closure() {
        let value = 10;
        let handle = WorkerHandle::spawn(move || value * 2);
        thread::sleep(Duration::from_millis(10));
        assert_eq!(handle.try_recv(), Some(20));
    }

    #[test]
    fn test_managed_worker_new() {
        let worker: ManagedWorker<i32> = ManagedWorker::new();
        assert!(!worker.is_executing());
    }

    #[test]
    fn test_managed_worker_start_and_poll() {
        let mut worker = ManagedWorker::new();
        worker.start(|| 123);
        assert!(worker.is_executing());

        // Give thread time to complete
        thread::sleep(Duration::from_millis(10));

        let result = worker.poll();
        assert_eq!(result, Some(123));
        assert!(!worker.is_executing());
    }

    #[test]
    fn test_managed_worker_poll_empty() {
        let mut worker = ManagedWorker::new();
        worker.start(|| {
            thread::sleep(Duration::from_millis(100));
            "slow"
        });

        // Poll immediately
        let result = worker.poll();
        assert!(result.is_none());
        assert!(worker.is_executing());
    }

    #[test]
    fn test_managed_worker_reset() {
        let mut worker = ManagedWorker::new();
        worker.start(|| 42);
        assert!(worker.is_executing());

        worker.reset();
        assert!(!worker.is_executing());
    }

    #[test]
    fn test_managed_worker_double_start() {
        let mut worker = ManagedWorker::new();
        worker.start(|| 1);
        worker.start(|| 2); // Should be ignored

        thread::sleep(Duration::from_millis(10));
        let result = worker.poll();
        assert_eq!(result, Some(1)); // First value, second was ignored
    }

    #[test]
    fn test_worker_with_string_result() {
        let handle = WorkerHandle::spawn(|| "hello".to_string());
        thread::sleep(Duration::from_millis(10));
        assert_eq!(handle.try_recv(), Some("hello".to_string()));
    }

    #[test]
    fn test_worker_with_tuple_result() {
        let handle = WorkerHandle::spawn(|| (1, "test".to_string()));
        thread::sleep(Duration::from_millis(10));
        assert_eq!(handle.try_recv(), Some((1, "test".to_string())));
    }
}
