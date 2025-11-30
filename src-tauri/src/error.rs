use serde::Serialize;
use std::fmt;

/// User-friendly error types for Cortex
#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum CortexError {
    /// Database-related errors
    DatabaseError { message: String },

    /// File system errors
    PermissionDenied { path: String, suggestion: String },
    FileNotFound { path: String },

    /// Indexing errors
    ExtractionFailed { path: String, error: String },
    IndexingInProgress,

    /// Search errors
    SearchTimeout,
    InvalidQuery { query: String, reason: String },

    /// General errors
    Internal { message: String },
}

impl fmt::Display for CortexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DatabaseError { message } => {
                write!(f, "Database error: {}", message)
            }
            Self::PermissionDenied { path, suggestion } => {
                write!(f, "Cannot access {}\n\n→ {}", path, suggestion)
            }
            Self::FileNotFound { path } => {
                write!(f, "File not found: {}", path)
            }
            Self::ExtractionFailed { path, error } => {
                write!(f, "Failed to extract content from {}: {}", path, error)
            }
            Self::IndexingInProgress => {
                write!(f, "Indexing is already in progress")
            }
            Self::SearchTimeout => {
                write!(f, "Search took too long. Try a more specific query.")
            }
            Self::InvalidQuery { query, reason } => {
                write!(f, "Invalid query '{}': {}", query, reason)
            }
            Self::Internal { message } => {
                write!(f, "Internal error: {}", message)
            }
        }
    }
}

impl std::error::Error for CortexError {}

impl From<rusqlite::Error> for CortexError {
    fn from(err: rusqlite::Error) -> Self {
        CortexError::DatabaseError {
            message: err.to_string(),
        }
    }
}

impl From<std::io::Error> for CortexError {
    fn from(err: std::io::Error) -> Self {
        CortexError::Internal {
            message: err.to_string(),
        }
    }
}

pub type Result<T> = std::result::Result<T, CortexError>;
