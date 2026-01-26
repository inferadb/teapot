//! Error types for Teapot.
//!
//! This module provides structured error types for the Teapot TUI framework,
//! enabling better error handling and user-friendly error messages.

use std::io;

/// The main error type for Teapot operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Terminal I/O error.
    #[error("terminal I/O error: {0}")]
    Io(#[from] io::Error),

    /// Form was cancelled by the user.
    #[error("form cancelled by user")]
    Cancelled,

    /// A required field was not filled.
    #[error("required field '{field}' was not filled")]
    RequiredField {
        /// The key of the required field.
        field: String,
    },

    /// Field validation failed.
    #[error("validation failed for field '{field}': {message}")]
    Validation {
        /// The key of the field that failed validation.
        field: String,
        /// The validation error message.
        message: String,
    },

    /// Terminal is not interactive (e.g., running in CI).
    #[error("terminal is not interactive")]
    NotInteractive,

    /// Terminal size could not be determined.
    #[error("could not determine terminal size")]
    TerminalSize,

    /// An external process failed.
    #[error("external process failed: {0}")]
    ProcessFailed(String),
}

/// A specialized Result type for Teapot operations.
pub type Result<T> = std::result::Result<T, Error>;
