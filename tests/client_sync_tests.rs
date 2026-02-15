// Client Sync API Tests - Phase 0 TDD
// These tests validate the sync (blocking) API functionality

use ollama_oxide::{ClientConfig, OllamaApiSync, OllamaClient};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// Note: These tests use mockito for HTTP mocking
// They test the sync API behavior without requiring a real Ollama server

#[test]
fn test_version_sync_successful() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("GET", "/api/version")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"version":"0.12.6"}"#)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(30),
        3,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let result = client.version_blocking();

    assert!(result.is_ok());
    let version = result.unwrap();
    assert_eq!(version.version, "0.12.6");

    mock.assert();
}

#[test]
fn test_version_sync_different_version() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("GET", "/api/version")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"version":"1.0.0"}"#)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(30),
        3,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let result = client.version_blocking();

    assert!(result.is_ok());
    assert_eq!(result.unwrap().version, "1.0.0");

    mock.assert();
}

#[test]
fn test_version_sync_retry_logic_success_on_second_attempt() {
    let mut server = mockito::Server::new();

    // First request fails, second succeeds
    let mock_fail = server
        .mock("GET", "/api/version")
        .with_status(500)
        .expect(1)
        .create();

    let mock_success = server
        .mock("GET", "/api/version")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"version":"0.12.6"}"#)
        .expect(1)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        3,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let result = client.version_blocking();

    assert!(result.is_ok());
    assert_eq!(result.unwrap().version, "0.12.6");

    mock_fail.assert();
    mock_success.assert();
}

#[test]
fn test_version_sync_max_retries_exceeded() {
    let mut server = mockito::Server::new();

    // All requests fail
    let mock = server
        .mock("GET", "/api/version")
        .with_status(500)
        .expect(4) // max_retries + 1
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(1),
        3,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let result = client.version_blocking();

    assert!(result.is_err());
    mock.assert();
}

#[test]
fn test_version_sync_json_deserialization_error() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("GET", "/api/version")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"invalid":"json"}"#)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(30),
        3,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let result = client.version_blocking();

    assert!(result.is_err());
    mock.assert();
}

#[test]
fn test_version_sync_invalid_json() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("GET", "/api/version")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"not valid json"#)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(30),
        3,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let result = client.version_blocking();

    assert!(result.is_err());
    mock.assert();
}

#[test]
fn test_version_sync_concurrent_calls_from_threads() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("GET", "/api/version")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"version":"0.12.6"}"#)
        .expect(10)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(30),
        3,
    ).unwrap();

    let client = Arc::new(OllamaClient::new(config).unwrap());
    let mut handles = vec![];

    // Spawn 10 threads
    for _ in 0..10 {
        let client_clone = Arc::clone(&client);
        let handle = thread::spawn(move || {
            let result = client_clone.version_blocking();
            assert!(result.is_ok());
            result.unwrap()
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        let version = handle.join().unwrap();
        assert_eq!(version.version, "0.12.6");
    }

    mock.assert();
}

#[test]
fn test_version_sync_404_error() {
    let mut server = mockito::Server::new();

    let mock = server.mock("GET", "/api/version").with_status(404).create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(30),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let result = client.version_blocking();

    assert!(result.is_err());
    mock.assert();
}

#[test]
fn test_version_sync_with_zero_retries() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("GET", "/api/version")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"version":"0.12.6"}"#)
        .expect(1)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(30),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let result = client.version_blocking();

    assert!(result.is_ok());
    mock.assert();
}

#[test]
fn test_version_sync_500_error() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("GET", "/api/version")
        .with_status(500)
        .expect(4)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(1),
        3,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let result = client.version_blocking();

    assert!(result.is_err());
    mock.assert();
}
