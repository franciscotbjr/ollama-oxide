//! Example: Chat completion with streaming (async)
//!
//! Prints assistant tokens as they arrive (NDJSON stream).
//!
//! Run with: `cargo run --example chat_stream_async`
//!
//! Requires a running Ollama server and an installed model (e.g. `qwen3:0.6b`).

use std::io::Write;

use ollama_oxide::{ChatMessage, ChatRequest, ChatResponse, OllamaApiAsync, OllamaClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::default()?;
    let model = "qwen3:0.6b";

    let request = ChatRequest::new(
        model,
        [ChatMessage::user("What is the sky's color in Venus?")],
    );

    let stream = client.chat_stream(&request).await?;
    let mut last: Option<ChatResponse> = None;

    while let Some(event) = stream.next().await {
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
