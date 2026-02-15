//! Tests for copy_model API methods (POST /api/copy)

use ollama_oxide::{ClientConfig, CopyRequest, OllamaApiAsync, OllamaApiSync, OllamaClient};
use std::time::Duration;

// ============================================================================
// Async API Tests
// ============================================================================

#[tokio::test]
async fn test_copy_model_async_success() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/copy")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "source": "llama3.1",
            "destination": "llama3.1-backup"
        })))
        .with_status(200)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = CopyRequest::new("llama3.1", "llama3.1-backup");
    let result = client.copy_model(&request).await;

    assert!(result.is_ok());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_copy_model_async_model_not_found() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/copy")
        .with_status(404)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = CopyRequest::new("nonexistent", "backup");
    let result = client.copy_model(&request).await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_copy_model_async_retry_on_server_error() {
    let mut server = mockito::Server::new_async().await;

    let mock_fail = server
        .mock("POST", "/api/copy")
        .with_status(500)
        .expect(1)
        .create_async()
        .await;

    let mock_success = server
        .mock("POST", "/api/copy")
        .with_status(200)
        .expect(1)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        1,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = CopyRequest::new("model", "model-copy");
    let result = client.copy_model(&request).await;

    assert!(result.is_ok());
    mock_fail.assert_async().await;
    mock_success.assert_async().await;
}

#[tokio::test]
async fn test_copy_model_async_different_models() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/copy")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "source": "gemma3",
            "destination": "my-gemma"
        })))
        .with_status(200)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = CopyRequest::new("gemma3", "my-gemma");
    let result = client.copy_model(&request).await;

    assert!(result.is_ok());
    mock.assert_async().await;
}

// ============================================================================
// Sync API Tests
// ============================================================================

#[test]
fn test_copy_model_sync_success() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("POST", "/api/copy")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "source": "gemma3",
            "destination": "gemma3-backup"
        })))
        .with_status(200)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = CopyRequest::new("gemma3", "gemma3-backup");
    let result = client.copy_model_blocking(&request);

    assert!(result.is_ok());
    mock.assert();
}

#[test]
fn test_copy_model_sync_model_not_found() {
    let mut server = mockito::Server::new();

    let mock = server.mock("POST", "/api/copy").with_status(404).create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = CopyRequest::new("missing", "copy");
    let result = client.copy_model_blocking(&request);

    assert!(result.is_err());
    mock.assert();
}

#[test]
fn test_copy_model_sync_retry_on_server_error() {
    let mut server = mockito::Server::new();

    let mock_fail = server
        .mock("POST", "/api/copy")
        .with_status(500)
        .expect(1)
        .create();

    let mock_success = server
        .mock("POST", "/api/copy")
        .with_status(200)
        .expect(1)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        1,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = CopyRequest::new("model", "backup");
    let result = client.copy_model_blocking(&request);

    assert!(result.is_ok());
    mock_fail.assert();
    mock_success.assert();
}

#[test]
fn test_copy_model_sync_different_models() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("POST", "/api/copy")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "source": "mistral",
            "destination": "mistral-fine-tuned"
        })))
        .with_status(200)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = CopyRequest::new("mistral", "mistral-fine-tuned");
    let result = client.copy_model_blocking(&request);

    assert!(result.is_ok());
    mock.assert();
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_copy_model_async_max_retries_exceeded() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/copy")
        .with_status(500)
        .expect(3) // Initial + 2 retries
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(1),
        2,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = CopyRequest::new("model", "copy");
    let result = client.copy_model(&request).await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[test]
fn test_copy_model_sync_max_retries_exceeded() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("POST", "/api/copy")
        .with_status(500)
        .expect(3) // Initial + 2 retries
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(1),
        2,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = CopyRequest::new("model", "copy");
    let result = client.copy_model_blocking(&request);

    assert!(result.is_err());
    mock.assert();
}

#[tokio::test]
async fn test_copy_model_async_bad_request() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/copy")
        .with_status(400)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        2, // Should NOT retry on 4xx
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = CopyRequest::new("invalid", "also-invalid");
    let result = client.copy_model(&request).await;

    assert!(result.is_err());
    mock.assert_async().await; // Should only be called once (no retry)
}

#[test]
fn test_copy_model_sync_bad_request() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("POST", "/api/copy")
        .with_status(400)
        .expect(1) // Should only be called once (no retry on 4xx)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        2,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = CopyRequest::new("invalid", "also-invalid");
    let result = client.copy_model_blocking(&request);

    assert!(result.is_err());
    mock.assert();
}

// ============================================================================
// CopyRequest Tests
// ============================================================================

#[test]
fn test_copy_request_new_constructor() {
    let request = CopyRequest::new("source-model", "dest-model");
    assert_eq!(request.source, "source-model");
    assert_eq!(request.destination, "dest-model");
}

#[test]
fn test_copy_request_struct_init() {
    let request = CopyRequest {
        source: "my-source".to_string(),
        destination: "my-dest".to_string(),
    };
    assert_eq!(request.source, "my-source");
    assert_eq!(request.destination, "my-dest");
}

#[test]
fn test_copy_request_clone() {
    let request = CopyRequest::new("original", "copy");
    let cloned = request.clone();
    assert_eq!(request, cloned);
}

#[test]
fn test_copy_request_debug() {
    let request = CopyRequest::new("llama3.1", "llama3.1-backup");
    let debug_str = format!("{:?}", request);
    assert!(debug_str.contains("llama3.1"));
    assert!(debug_str.contains("llama3.1-backup"));
}
