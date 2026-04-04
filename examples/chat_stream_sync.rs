//! Example: Chat completion with streaming (blocking)
//!
//! Prints assistant tokens as they arrive (NDJSON stream).
//!
//! Run with: `cargo run --example chat_stream_sync`
//!
//! Requires a running Ollama server and an installed model (e.g. `qwen3:0.6b`).

use std::io::Write;

use ollama_oxide::{ChatMessage, ChatRequest, ChatResponse, OllamaApiSync, OllamaClient};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::default()?;
    let model = "qwen3:0.6b";

    let request = ChatRequest::new(
        model,
        [ChatMessage::user("What is the sky's color in Venus?")],
    );

    let stream = client.chat_stream_blocking(&request)?;
    let mut last: Option<ChatResponse> = None;

    for event in stream {
        let chunk = event?;
        if let Some(text) = chunk.content() {
            print!("{}", text);
            let _ = std::io::stdout().flush();
        }
        last = Some(chunk);
    }

    println!();
    if let Some(resp) = last {
        if let Some(ns) = resp.total_duration {
            println!("total_duration (ns): {}", ns);
        }
    }

    Ok(())
}
