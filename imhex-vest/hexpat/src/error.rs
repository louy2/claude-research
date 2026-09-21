//! Error type shared by every stage.

use std::fmt;

/// A diagnostic with an optional source location (1-based line and column).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub message: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Error { message: message.into(), line: None, column: None }
    }

    pub fn at(message: impl Into<String>, line: u32, column: u32) -> Self {
        Error { message: message.into(), line: Some(line), column: Some(column) }
    }

    /// Attaches a location if the error does not carry one yet.
    pub fn with_location(mut self, line: u32, column: u32) -> Self {
        if self.line.is_none() {
            self.line = Some(line);
            self.column = Some(column);
        }
        self
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.line, self.column) {
            (Some(l), Some(c)) => write!(f, "{}:{}: {}", l, c, self.message),
            _ => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

/// Shorthand for building an [`Error`] from format arguments.
#[macro_export]
macro_rules! err {
    ($($arg:tt)*) => { $crate::error::Error::new(format!($($arg)*)) };
}

/// Shorthand for returning early with an [`Error`].
#[macro_export]
macro_rules! bail {
    ($($arg:tt)*) => { return Err($crate::err!($($arg)*)) };
}
