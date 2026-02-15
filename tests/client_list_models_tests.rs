//! Tests for list_models API methods

use ollama_oxide::{ClientConfig, OllamaApiAsync, OllamaApiSync, OllamaClient};
use std::time::Duration;

// ============================================================================
// Async API Tests
// ============================================================================

#[tokio::test]
async fn test_list_models_async_with_mock() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/api/tags")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "models": [
                {
                    "name": "test-model",
                    "size": 1000000
                }
            ]
        }"#,
        )
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let response = client.list_models().await.unwrap();

    assert_eq!(response.models.len(), 1);
    assert_eq!(response.models[0].name, "test-model");
    assert_eq!(response.models[0].size, Some(1000000));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_models_async_empty_response() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/api/tags")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"models": []}"#)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let response = client.list_models().await.unwrap();

    assert!(response.models.is_empty());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_models_async_full_response() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/api/tags")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "models": [
                {
                    "name": "gemma3",
                    "modified_at": "2025-10-03T23:34:03.409490317-07:00",
                    "size": 3338801804,
                    "digest": "a2af6cc3eb7fa8be8504abaf9b04e88f17a119ec3f04a3addf55f92841195f5a",
                    "details": {
                        "format": "gguf",
                        "family": "gemma",
                        "families": ["gemma"],
                        "parameter_size": "4.3B",
                        "quantization_level": "Q4_K_M"
                    }
                }
            ]
        }"#,
        )
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let response = client.list_models().await.unwrap();

    assert_eq!(response.models.len(), 1);

    let model = &response.models[0];
    assert_eq!(model.name, "gemma3");
    assert_eq!(
        model.modified_at,
        Some("2025-10-03T23:34:03.409490317-07:00".to_string())
    );
    assert_eq!(model.size, Some(3338801804));
    assert!(model.details.is_some());

    let details = model.details.as_ref().unwrap();
    assert_eq!(details.format, Some("gguf".to_string()));
    assert_eq!(details.family, Some("gemma".to_string()));
    assert_eq!(details.families, Some(vec!["gemma".to_string()]));
    assert_eq!(details.parameter_size, Some("4.3B".to_string()));
    assert_eq!(details.quantization_level, Some("Q4_K_M".to_string()));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_models_async_multiple_models() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/api/tags")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "models": [
                {"name": "llama3", "size": 4000000000},
                {"name": "gemma3", "size": 3000000000},
                {"name": "mistral", "size": 7000000000}
            ]
        }"#,
        )
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let response = client.list_models().await.unwrap();

    assert_eq!(response.models.len(), 3);
    assert_eq!(response.models[0].name, "llama3");
    assert_eq!(response.models[1].name, "gemma3");
    assert_eq!(response.models[2].name, "mistral");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_models_async_retry_on_error() {
    let mut server = mockito::Server::new_async().await;

    // First request fails, second succeeds
    let mock_fail = server
        .mock("GET", "/api/tags")
        .with_status(500)
        .expect(1)
        .create_async()
        .await;

    let mock_success = server
        .mock("GET", "/api/tags")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"models": [{"name": "recovered"}]}"#)
        .expect(1)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        1,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let response = client.list_models().await.unwrap();

    assert_eq!(response.models.len(), 1);
    assert_eq!(response.models[0].name, "recovered");

    mock_fail.assert_async().await;
    mock_success.assert_async().await;
}

// ============================================================================
// Sync API Tests
// ============================================================================

#[test]
fn test_list_models_sync_with_mock() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("GET", "/api/tags")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "models": [
                {
                    "name": "sync-model",
                    "size": 2000000
                }
            ]
        }"#,
        )
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let response = client.list_models_blocking().unwrap();

    assert_eq!(response.models.len(), 1);
    assert_eq!(response.models[0].name, "sync-model");
    assert_eq!(response.models[0].size, Some(2000000));

    mock.assert();
}

#[test]
fn test_list_models_sync_empty_response() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("GET", "/api/tags")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"models": []}"#)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let response = client.list_models_blocking().unwrap();

    assert!(response.models.is_empty());
    mock.assert();
}

#[test]
fn test_list_models_sync_full_response() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("GET", "/api/tags")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "models": [
                {
                    "name": "llama3.2",
                    "modified_at": "2025-01-14T10:00:00Z",
                    "size": 5000000000,
                    "digest": "deadbeef1234",
                    "details": {
                        "format": "gguf",
                        "family": "llama",
                        "families": ["llama", "instruct"],
                        "parameter_size": "8B",
                        "quantization_level": "Q4_0"
                    }
                }
            ]
        }"#,
        )
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let response = client.list_models_blocking().unwrap();

    assert_eq!(response.models.len(), 1);

    let model = &response.models[0];
    assert_eq!(model.name, "llama3.2");
    assert!(model.details.is_some());

    let details = model.details.as_ref().unwrap();
    assert_eq!(
        details.families,
        Some(vec!["llama".to_string(), "instruct".to_string()])
    );

    mock.assert();
}

#[test]
fn test_list_models_sync_retry_on_error() {
    let mut server = mockito::Server::new();

    // First request fails, second succeeds
    let mock_fail = server
        .mock("GET", "/api/tags")
        .with_status(500)
        .expect(1)
        .create();

    let mock_success = server
        .mock("GET", "/api/tags")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"models": [{"name": "sync-recovered"}]}"#)
        .expect(1)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        1,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let response = client.list_models_blocking().unwrap();

    assert_eq!(response.models.len(), 1);
    assert_eq!(response.models[0].name, "sync-recovered");

    mock_fail.assert();
    mock_success.assert();
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_list_models_async_max_retries_exceeded() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/api/tags")
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
    let result = client.list_models().await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[test]
fn test_list_models_sync_max_retries_exceeded() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("GET", "/api/tags")
        .with_status(500)
        .expect(3) // Initial + 2 retries
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(1),
        2,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let result = client.list_models_blocking();

    assert!(result.is_err());
    mock.assert();
}
