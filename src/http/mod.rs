//! HTTP client for Ollama API
//!
//! This module provides the HTTP client implementation for interacting with
//! the Ollama API, including both async and sync (blocking) interfaces.
//!
//! # Components
//!
//! - [`ClientConfig`] - Configuration for the HTTP client
//! - [`OllamaClient`] - The main HTTP client
//! - [`OllamaApiAsync`] - Async API trait
//! - [`OllamaApiSync`] - Sync (blocking) API trait
//!
//! # Examples
//!
//! ## Async Usage
//!
//! ```no_run
//! use ollama_oxide::{OllamaClient, OllamaApiAsync};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = OllamaClient::default()?;
//!     let version = client.version().await?;
//!     println!("Version: {}", version.version);
//!     Ok(())
//! }
//! ```
//!
//! ## Sync Usage
//!
//! ```no_run
//! use ollama_oxide::{OllamaClient, OllamaApiSync};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = OllamaClient::default()?;
//!     let version = client.version_blocking()?;
//!     println!("Version: {}", version.version);
//!     Ok(())
//! }
//! ```

mod api_async;
mod api_sync;
mod client;
mod client_config;
pub(crate) mod endpoints;

pub use api_async::OllamaApiAsync;
pub use api_sync::OllamaApiSync;
pub use client::OllamaClient;
pub use client_config::ClientConfig;
