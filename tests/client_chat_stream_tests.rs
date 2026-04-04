//! Integration tests for streaming chat (POST /api/chat, NDJSON).

use ollama_oxide::{
    ChatMessage, ChatRequest, ClientConfig, Error, OllamaApiAsync, OllamaApiSync, OllamaClient,
};
use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn test_chat_stream_async_happy_path() {
    let mut server = mockito::Server::new_async().await;

    let body = concat!(
        r#"{"model":"m","message":{"role":"assistant","content":"He"},"done":false}"#,
        "\n",
        r#"{"model":"m","message":{"role":"assistant","content":"llo"},"done":false}"#,
        "\n",
        r#"{"model":"m","message":{"role":"assistant","content":"Hello"},"done":true,"total_duration":100,"eval_count":3}"#,
        "\n",
    );

    let mock = server
        .mock("POST", "/api/chat")
        .match_body(mockito::Matcher::Json(json!({
            "model": "qwen3:0.6b",
            "messages": [{"role": "user", "content": "Hi"}],
            "stream": true
        })))
        .with_status(200)
        .with_header("content-type", "application/x-ndjson")
        .with_body(body)
        .create_async()
        .await;

    let config = ClientConfig::new(server.url(), Duration::from_secs(5), 0).unwrap();
    let client = OllamaClient::new(config).unwrap();
    let request = ChatRequest::new("qwen3:0.6b", [ChatMessage::user("Hi")]);

    let stream = client.chat_stream(&request).await.unwrap();
    let events = stream.collect().await.unwrap();

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].content(), Some("He"));
    assert_eq!(events[1].content(), Some("llo"));
    assert_eq!(events[2].content(), Some("Hello"));
    assert!(events[2].is_done());
    assert_eq!(events[2].total_duration, Some(100));
    assert_eq!(events[2].eval_count, Some(3));

    mock.assert_async().await;
}

#[test]
fn test_chat_stream_blocking_happy_path() {
    let mut server = mockito::Server::new();

    let body = concat!(
        r#"{"model":"m","message":{"role":"assistant","content":"A"},"done":false}"#,
        "\n",
        r#"{"model":"m","message":{"role":"assistant","content":"B"},"done":true}"#,
        "\n",
    );

    let mock = server
        .mock("POST", "/api/chat")
        .match_body(mockito::Matcher::Json(json!({
            "model": "m",
            "messages": [{"role": "user", "content": "x"}],
            "stream": true
        })))
        .with_status(200)
        .with_header("content-type", "application/x-ndjson")
        .with_body(body)
        .create();

    let config = ClientConfig::new(server.url(), Duration::from_secs(5), 0).unwrap();
    let client = OllamaClient::new(config).unwrap();
    let request = ChatRequest::new("m", [ChatMessage::user("x")]);

    let stream = client.chat_stream_blocking(&request).unwrap();
    let events: Result<Vec<_>, _> = stream.collect();
    let events = events.unwrap();

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].content(), Some("A"));
    assert_eq!(events[1].content(), Some("B"));
    assert!(events[1].is_done());

    mock.assert();
}

#[tokio::test]
async fn test_chat_stream_async_invalid_json_mid_stream() {
    let mut server = mockito::Server::new_async().await;

    let body = concat!(
        r#"{"model":"m","message":{"content":"ok"},"done":false}"#,
        "\n",
        "not-json\n",
    );

    let mock = server
        .mock("POST", "/api/chat")
        .with_status(200)
        .with_body(body)
        .create_async()
        .await;

    let config = ClientConfig::new(server.url(), Duration::from_secs(5), 0).unwrap();
    let client = OllamaClient::new(config).unwrap();
    let request = ChatRequest::new("m", [ChatMessage::user("Hi")]);

    let stream = client.chat_stream(&request).await.unwrap();
    assert!(stream.next().await.unwrap().is_ok());
    let err = stream.next().await.unwrap().unwrap_err();
    assert!(matches!(err, Error::StreamError(_)));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_chat_stream_async_server_error() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/chat")
        .with_status(500)
        .create_async()
        .await;

    let config = ClientConfig::new(server.url(), Duration::from_secs(5), 0).unwrap();
    let client = OllamaClient::new(config).unwrap();
    let request = ChatRequest::new("m", [ChatMessage::user("Hi")]);

    let result = client.chat_stream(&request).await;
    assert!(result.is_err());
    match result.err().expect("err") {
        Error::HttpStatusError(500) => {}
        e => panic!("expected HttpStatusError(500), got {:?}", e),
    }

    mock.assert_async().await;
}

#[tokio::test]
async fn test_chat_stream_async_client_error() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/chat")
        .with_status(404)
        .create_async()
        .await;

    let config = ClientConfig::new(server.url(), Duration::from_secs(5), 0).unwrap();
    let client = OllamaClient::new(config).unwrap();
    let request = ChatRequest::new("missing", [ChatMessage::user("Hi")]);

    let result = client.chat_stream(&request).await;
    assert!(matches!(
        result.err().expect("err"),
        Error::HttpStatusError(404)
    ));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_chat_stream_async_empty_body() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/chat")
        .with_status(200)
        .with_body("")
        .create_async()
        .await;

    let config = ClientConfig::new(server.url(), Duration::from_secs(5), 0).unwrap();
    let client = OllamaClient::new(config).unwrap();
    let request = ChatRequest::new("m", [ChatMessage::user("Hi")]);

    let stream = client.chat_stream(&request).await.unwrap();
    let events = stream.collect().await.unwrap();
    assert!(events.is_empty());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_chat_stream_async_single_line_done() {
    let mut server = mockito::Server::new_async().await;

    let body = r#"{"model":"m","message":{"role":"assistant","content":"Only"},"done":true,"done_reason":"stop"}
"#;

    let mock = server
        .mock("POST", "/api/chat")
        .with_status(200)
        .with_body(body)
        .create_async()
        .await;

    let config = ClientConfig::new(server.url(), Duration::from_secs(5), 0).unwrap();
    let client = OllamaClient::new(config).unwrap();
    let request = ChatRequest::new("m", [ChatMessage::user("Hi")]);

    let stream = client.chat_stream(&request).await.unwrap();
    let events = stream.collect().await.unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].content(), Some("Only"));
    assert!(events[0].is_done());

    mock.assert_async().await;
}

#[test]
fn test_chat_stream_types_are_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ollama_oxide::ChatStream>();
    assert_send_sync::<ollama_oxide::ChatStreamBlocking>();
}

#[test]
fn test_chat_request_with_stream_builder() {
    let r = ChatRequest::new("m", [ChatMessage::user("x")]).with_stream(true);
    assert_eq!(r.stream, Some(true));
}
