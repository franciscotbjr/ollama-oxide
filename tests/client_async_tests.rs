// Client Async API Tests - Phase 0 TDD
// These tests validate the async API functionality using mocking

use ollama_oxide::{ClientConfig, OllamaApiAsync, OllamaClient};
use std::sync::Arc;
use std::time::Duration;

// Note: These tests use mockito for HTTP mocking
// They test the async API behavior without requiring a real Ollama server

#[tokio::test]
async fn test_version_async_successful() {
    // This test will use mockito to mock the HTTP response
    // For now, we're testing the structure - implementation will add mocking
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/api/version")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"version":"0.12.6"}"#)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(30),
        3,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let result = client.version().await;

    assert!(result.is_ok());
    let version = result.unwrap();
    assert_eq!(version.version, "0.12.6");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_version_async_different_version() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/api/version")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"version":"1.0.0"}"#)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(30),
        3,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let result = client.version().await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().version, "1.0.0");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_version_async_retry_logic_success_on_second_attempt() {
    let mut server = mockito::Server::new_async().await;

    // First request fails, second succeeds
    let mock_fail = server
        .mock("GET", "/api/version")
        .with_status(500)
        .expect(1)
        .create_async()
        .await;

    let mock_success = server
        .mock("GET", "/api/version")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"version":"0.12.6"}"#)
        .expect(1)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        3,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let result = client.version().await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().version, "0.12.6");

    mock_fail.assert_async().await;
    mock_success.assert_async().await;
}

#[tokio::test]
async fn test_version_async_max_retries_exceeded() {
    let mut server = mockito::Server::new_async().await;

    // All requests fail
    let mock = server
        .mock("GET", "/api/version")
        .with_status(500)
        .expect(4) // max_retries + 1 (initial attempt + 3 retries)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(1),
        3,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let result = client.version().await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_version_async_json_deserialization_error() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/api/version")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"invalid":"json"}"#) // Missing "version" field
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(30),
        3,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let result = client.version().await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_version_async_invalid_json() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/api/version")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"not valid json"#)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(30),
        3,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let result = client.version().await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_version_async_concurrent_calls() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/api/version")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"version":"0.12.6"}"#)
        .expect(10)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(30),
        3,
    ).unwrap();

    let client = Arc::new(OllamaClient::new(config).unwrap());
    let mut tasks = vec![];

    // Spawn 10 concurrent tasks
    for _ in 0..10 {
        let client_clone = Arc::clone(&client);
        let task = tokio::spawn(async move {
            let result = client_clone.version().await;
            assert!(result.is_ok());
            result.unwrap()
        });
        tasks.push(task);
    }

    // Wait for all tasks
    for task in tasks {
        let version = task.await.unwrap();
        assert_eq!(version.version, "0.12.6");
    }

    mock.assert_async().await;
}

#[tokio::test]
async fn test_version_async_404_error() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/api/version")
        .with_status(404)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(30),
        0, // No retries for this test
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let result = client.version().await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_version_async_with_zero_retries() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/api/version")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"version":"0.12.6"}"#)
        .expect(1) // Should only try once with zero retries
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(30),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let result = client.version().await;

    assert!(result.is_ok());
    mock.assert_async().await;
}
