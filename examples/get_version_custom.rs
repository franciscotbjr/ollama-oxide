//! Example: Get Ollama server version with custom configuration
//!
//! This example demonstrates how to configure the client with custom
//! timeout, retry settings, and base URL.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example get_version_custom
//! ```

use ollama_oxide::{ClientConfig, OllamaApiAsync, OllamaClient, Result};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Create client with custom configuration
    let config = ClientConfig::new(
        "http://localhost:11434".to_string(),
        Duration::from_secs(10),
        5,
    )?;

    let client = OllamaClient::new(config)?;

    // Get Ollama server version
    let response = client.version().await?;

    println!("Ollama version: {}", response.version);

    Ok(())
}
