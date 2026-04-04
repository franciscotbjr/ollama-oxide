//! Streaming response types for NDJSON APIs (e.g. `POST /api/chat` with `stream: true`).

use std::io::{BufRead, BufReader};

use crate::{ChatResponse, Error, Result};

/// Async stream of [`ChatResponse`] events from a streaming chat request.
///
/// Each [`next`](Self::next) yields one NDJSON line deserialized as [`ChatResponse`].
/// When the server closes the body, [`next`](Self::next) returns `None`.
///
/// # Examples
///
/// ```no_run
/// use ollama_oxide::{ChatMessage, ChatRequest, OllamaApiAsync, OllamaClient};
///
/// #[tokio::main]
/// async fn main() -> ollama_oxide::Result<()> {
///     let client = OllamaClient::default()?;
///     let request = ChatRequest::new("qwen3:0.6b", [ChatMessage::user("Hi!")]);
///     let stream = client.chat_stream(&request).await?;
///     while let Some(event) = stream.next().await {
///         let chunk = event?;
///         if let Some(s) = chunk.content() {
///             print!("{}", s);
///         }
///     }
///     Ok(())
/// }
/// ```
pub struct ChatStream {
    rx: tokio::sync::Mutex<tokio::sync::mpsc::Receiver<Result<ChatResponse>>>,
}

impl ChatStream {
    /// Wraps a channel receiver produced by the HTTP client streaming helper.
    pub(crate) fn new(rx: tokio::sync::mpsc::Receiver<Result<ChatResponse>>) -> Self {
        Self {
            rx: tokio::sync::Mutex::new(rx),
        }
    }

    /// Returns the next event, or `None` when the stream has ended.
    pub async fn next(&self) -> Option<Result<ChatResponse>> {
        self.rx.lock().await.recv().await
    }

    /// Collects all events into a vector, stopping on the first error.
    pub async fn collect(self) -> Result<Vec<ChatResponse>> {
        let mut out = Vec::new();
        let mut rx = self.rx.into_inner();
        while let Some(item) = rx.recv().await {
            match item {
                Ok(v) => out.push(v),
                Err(e) => return Err(e),
            }
        }
        Ok(out)
    }
}

/// Blocking iterator over [`ChatResponse`] events from a streaming chat request.
///
/// Implements [`Iterator`] so you can use `for`/`while let` over events.
///
/// # Examples
///
/// ```no_run
/// use ollama_oxide::{ChatMessage, ChatRequest, OllamaApiSync, OllamaClient};
///
/// fn main() -> ollama_oxide::Result<()> {
///     let client = OllamaClient::default()?;
///     let request = ChatRequest::new("qwen3:0.6b", [ChatMessage::user("Hi!")]);
///     let stream = client.chat_stream_blocking(&request)?;
///     for event in stream {
///         let chunk = event?;
///         if let Some(s) = chunk.content() {
///             print!("{}", s);
///         }
///     }
///     Ok(())
/// }
/// ```
pub struct ChatStreamBlocking {
    lines: std::io::Lines<BufReader<reqwest::blocking::Response>>,
}

impl ChatStreamBlocking {
    /// Builds a line iterator over the blocking response body.
    pub(crate) fn new(response: reqwest::blocking::Response) -> Self {
        Self {
            lines: BufReader::new(response).lines(),
        }
    }
}

impl Iterator for ChatStreamBlocking {
    type Item = Result<ChatResponse>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.lines.next() {
                None => return None,
                Some(Err(e)) => return Some(Err(Error::StreamError(e.to_string()))),
                Some(Ok(line)) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    return Some(
                        serde_json::from_str::<ChatResponse>(trimmed)
                            .map_err(|e| Error::StreamError(e.to_string())),
                    );
                }
            }
        }
    }
}
