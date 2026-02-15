//! Example demonstrating ollama-oxide library usage
//!
//! This example shows both async and sync usage of the Ollama API client.

use ollama_oxide::{ClientConfig, OllamaApiAsync, OllamaApiSync, OllamaClient};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Ollama Oxide Example ===\n");

    // Example 1: Async usage with default configuration
    println!("1. Async API with default configuration:");
    async_example_default().await?;

    println!("\n---\n");

    // Example 2: Async usage with custom configuration
    println!("2. Async API with custom configuration:");
    async_example_custom().await?;

    println!("\n---\n");

    // Example 3: Sync (blocking) usage
    // Note: We run the sync example in a separate thread because mixing
    // sync blocking calls with async runtime in the same thread can cause
    // runtime conflicts. In a real application, you would typically use
    // either the async API OR the sync API, not both together.
    println!("3. Sync (blocking) API:");
    let handle = std::thread::spawn(|| {
        if let Err(e) = sync_example() {
            eprintln!("  Error in sync example: {}", e);
        }
    });
    handle.join().unwrap();

    println!("\n=== All examples completed successfully! ===");
    Ok(())
}

/// Example: Async API with default configuration
async fn async_example_default() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with default settings (http://localhost:11434)
    let client = OllamaClient::default()?;

    println!("  Created client with default config");
    println!("  Fetching Ollama version...");

    match client.version().await {
        Ok(response) => {
            println!("  ✓ Success! Ollama version: {}", response.version);
        }
        Err(e) => {
            println!("  ✗ Error: {}", e);
            println!("  (Make sure Ollama is running on http://localhost:11434)");
        }
    }

    Ok(())
}

/// Example: Async API with custom configuration
async fn async_example_custom() -> Result<(), Box<dyn std::error::Error>> {
    // Create custom configuration
    let config = ClientConfig::new(
        "http://localhost:11434".to_string(),
        Duration::from_secs(10),
        5,
    )?;

    let client = OllamaClient::new(config)?;

    println!("  Created client with custom config:");
    println!("    - Base URL: http://localhost:11434");
    println!("    - Timeout: 10 seconds");
    println!("    - Max retries: 5");
    println!("  Fetching Ollama version...");

    match client.version().await {
        Ok(response) => {
            println!("  ✓ Success! Ollama version: {}", response.version);
        }
        Err(e) => {
            println!("  ✗ Error: {}", e);
        }
    }

    Ok(())
}

/// Example: Sync (blocking) API
fn sync_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with base URL helper
    let client = OllamaClient::with_base_url("http://localhost:11434")?;

    println!("  Created client using with_base_url helper");
    println!("  Fetching Ollama version (blocking)...");

    match client.version_blocking() {
        Ok(response) => {
            println!("  ✓ Success! Ollama version: {}", response.version);
        }
        Err(e) => {
            println!("  ✗ Error: {}", e);
            println!("  (Make sure Ollama is running on http://localhost:11434)");
        }
    }

    Ok(())
}
