//! Tests for delete_model API methods (DELETE /api/delete)

use ollama_oxide::{ClientConfig, DeleteRequest, OllamaApiAsync, OllamaApiSync, OllamaClient};
use std::time::Duration;

// ============================================================================
// Async API Tests
// ============================================================================

#[tokio::test]
async fn test_delete_model_async_success() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("DELETE", "/api/delete")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "model": "llama3.1-backup"
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
    let request = DeleteRequest::new("llama3.1-backup");
    let result = client.delete_model(&request).await;

    assert!(result.is_ok());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_delete_model_async_model_not_found() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("DELETE", "/api/delete")
        .with_status(404)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = DeleteRequest::new("nonexistent");
    let result = client.delete_model(&request).await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_delete_model_async_retry_on_server_error() {
    let mut server = mockito::Server::new_async().await;

    let mock_fail = server
        .mock("DELETE", "/api/delete")
        .with_status(500)
        .expect(1)
        .create_async()
        .await;

    let mock_success = server
        .mock("DELETE", "/api/delete")
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
    let request = DeleteRequest::new("model");
    let result = client.delete_model(&request).await;

    assert!(result.is_ok());
    mock_fail.assert_async().await;
    mock_success.assert_async().await;
}

// ============================================================================
// Sync API Tests
// ============================================================================

#[test]
fn test_delete_model_sync_success() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("DELETE", "/api/delete")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "model": "gemma3-backup"
        })))
        .with_status(200)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = DeleteRequest::new("gemma3-backup");
    let result = client.delete_model_blocking(&request);

    assert!(result.is_ok());
    mock.assert();
}

#[test]
fn test_delete_model_sync_model_not_found() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("DELETE", "/api/delete")
        .with_status(404)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = DeleteRequest::new("missing");
    let result = client.delete_model_blocking(&request);

    assert!(result.is_err());
    mock.assert();
}

#[test]
fn test_delete_model_sync_retry_on_server_error() {
    let mut server = mockito::Server::new();

    let mock_fail = server
        .mock("DELETE", "/api/delete")
        .with_status(500)
        .expect(1)
        .create();

    let mock_success = server
        .mock("DELETE", "/api/delete")
        .with_status(200)
        .expect(1)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        1,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = DeleteRequest::new("model");
    let result = client.delete_model_blocking(&request);

    assert!(result.is_ok());
    mock_fail.assert();
    mock_success.assert();
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_delete_model_async_max_retries_exceeded() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("DELETE", "/api/delete")
        .with_status(500)
        .expect(3)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(1),
        2,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = DeleteRequest::new("model");
    let result = client.delete_model(&request).await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[test]
fn test_delete_model_sync_max_retries_exceeded() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("DELETE", "/api/delete")
        .with_status(500)
        .expect(3)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(1),
        2,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = DeleteRequest::new("model");
    let result = client.delete_model_blocking(&request);

    assert!(result.is_err());
    mock.assert();
}

// ============================================================================
// DeleteRequest Type Tests
// ============================================================================

#[test]
fn test_delete_request_debug_impl() {
    let request = DeleteRequest::new("test-model");
    let debug_str = format!("{:?}", request);
    assert!(debug_str.contains("test-model"));
}

#[test]
fn test_delete_request_clone_impl() {
    let request = DeleteRequest::new("original");
    let cloned = request.clone();
    assert_eq!(request, cloned);
}
