//! Unified error type for the docrafter ecosystem.

use std::io;
use std::path::PathBuf;

/// Result alias used across docrafter crates.
pub type Result<T> = std::result::Result<T, Error>;

/// Top-level error for public APIs.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// I/O failure while reading or writing a document.
    #[error("I/O error at {path}: {source}")]
    Io {
        /// Path involved in the operation.
        path: PathBuf,
        /// Underlying I/O error.
        source: io::Error,
    },

    /// Plain I/O without path context.
    #[error("I/O error: {0}")]
    IoPlain(#[from] io::Error),

    /// Invalid user input (color, length, style, etc.).
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// PDF-specific failure.
    #[error("PDF error: {0}")]
    Pdf(String),

    /// Layout or rendering failure.
    #[error("layout error: {0}")]
    Layout(String),

    /// Font parsing or embedding failure.
    #[error("font error: {0}")]
    Font(String),

    /// DOCX (OOXML) failure.
    #[error("DOCX error: {0}")]
    Docx(String),

    /// ODT (OpenDocument) failure.
    #[error("ODT error: {0}")]
    Odt(String),
}

impl Error {
    /// Wrap an I/O error with the path that caused it.
    #[must_use]
    pub fn io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
