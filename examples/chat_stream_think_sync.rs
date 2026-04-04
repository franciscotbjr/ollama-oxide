//! Example: Chat completion with streaming and thinking enabled (blocking)
//!
//! Prints reasoning (`thinking`) then the assistant reply (`content`) as NDJSON chunks arrive.
//!
//! Run with: `cargo run --example chat_stream_think_sync`
//!
//! Requires a running Ollama server and a model that supports the `think` option (e.g. some
//! reasoning models). If the model ignores `think`, you may see only `content`.
//!
//! Empty `thinking` / `content` strings are skipped so placeholder `""` in the stream does not
//! flip to `[response]` before the real answer text arrives.

use std::io::Write;

use ollama_oxide::{
    ChatMessage, ChatRequest, ChatResponse, OllamaApiSync, OllamaClient, ThinkSetting,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::default()?;
    let model = "qwen3:0.6b";

    let request = ChatRequest::new(
        model,
        [ChatMessage::user(
            "What is the sky's color in Venus? Explain briefly.",
        )],
    )
    .with_think(ThinkSetting::enabled());

    let stream = client.chat_stream_blocking(&request)?;
    let mut last: Option<ChatResponse> = None;
    let mut started_thinking = false;
    let mut started_content = false;

    for event in stream {
        let chunk = event?;
        if let Some(t) = chunk.thinking().filter(|s| !s.is_empty()) {
            if !started_thinking {
                println!("[thinking]");
                started_thinking = true;
            }
            print!("{}", t);
            let _ = std::io::stdout().flush();
        }
        if let Some(text) = chunk.content().filter(|s| !s.is_empty()) {
            if !started_content {
                if started_thinking {
                    println!();
                }
                println!("[response]");
                started_content = true;
            }
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
