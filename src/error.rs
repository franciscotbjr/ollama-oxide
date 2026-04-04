//! Error types for ollama-oxide library
//!
//! This module defines the error types used throughout the library,
//! including conversions from external error types and the Result type alias.

use thiserror::Error;

/// Error type for all ollama-oxide operations
#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    HttpError(String),

    #[error("HTTP status error: {0}")]
    HttpStatusError(u16),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("API error: {message}")]
    ApiError { message: String },

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Invalid URL: {0}")]
    InvalidUrlError(#[from] url::ParseError),

    #[error("Request timeout after {0} seconds")]
    TimeoutError(u64),

    #[error("Maximum retry attempts ({0}) exceeded")]
    MaxRetriesExceededError(u32),

    /// Error while reading or parsing a streaming (NDJSON) response.
    #[error("Stream error: {0}")]
    StreamError(String),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::HttpError(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerializationError(err.to_string())
    }
}

/// Result type alias for ollama-oxide operations
pub type Result<T> = std::result::Result<T, Error>;
