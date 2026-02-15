//! Tests for list_running_models API methods (GET /api/ps)

use ollama_oxide::{ClientConfig, OllamaApiAsync, OllamaApiSync, OllamaClient};
use std::time::Duration;

// ============================================================================
// Async API Tests
// ============================================================================

#[tokio::test]
async fn test_list_running_models_async_with_mock() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/api/ps")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "models": [
                {
                    "model": "test-model",
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
    let response = client.list_running_models().await.unwrap();

    assert_eq!(response.models.len(), 1);
    assert_eq!(response.models[0].model, "test-model");
    assert_eq!(response.models[0].size, Some(1000000));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_running_models_async_empty_response() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/api/ps")
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
    let response = client.list_running_models().await.unwrap();

    assert!(response.models.is_empty());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_running_models_async_full_response() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/api/ps")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "models": [
                {
                    "model": "gemma3",
                    "size": 6591830464,
                    "digest": "a2af6cc3eb7fa8be8504abaf9b04e88f17a119ec3f04a3addf55f92841195f5a",
                    "details": {
                        "format": "gguf",
                        "family": "gemma3",
                        "families": ["gemma3"],
                        "parameter_size": "4.3B",
                        "quantization_level": "Q4_K_M"
                    },
                    "expires_at": "2025-10-17T16:47:07.93355-07:00",
                    "size_vram": 5333539264,
                    "context_length": 4096
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
    let response = client.list_running_models().await.unwrap();

    assert_eq!(response.models.len(), 1);

    let model = &response.models[0];
    assert_eq!(model.model, "gemma3");
    assert_eq!(model.size, Some(6591830464));
    assert_eq!(model.size_vram, Some(5333539264));
    assert_eq!(model.context_length, Some(4096));
    assert_eq!(
        model.expires_at,
        Some("2025-10-17T16:47:07.93355-07:00".to_string())
    );
    assert!(model.details.is_some());

    let details = model.details.as_ref().unwrap();
    assert_eq!(details.format, Some("gguf".to_string()));
    assert_eq!(details.family, Some("gemma3".to_string()));
    assert_eq!(details.families, Some(vec!["gemma3".to_string()]));
    assert_eq!(details.parameter_size, Some("4.3B".to_string()));
    assert_eq!(details.quantization_level, Some("Q4_K_M".to_string()));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_running_models_async_multiple_models() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/api/ps")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "models": [
                {"model": "llama3", "size": 4000000000, "size_vram": 3500000000},
                {"model": "gemma3", "size": 3000000000, "context_length": 8192},
                {"model": "mistral", "size": 7000000000, "expires_at": "2025-01-15T00:00:00Z"}
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
    let response = client.list_running_models().await.unwrap();

    assert_eq!(response.models.len(), 3);
    assert_eq!(response.models[0].model, "llama3");
    assert_eq!(response.models[0].size_vram, Some(3500000000));
    assert_eq!(response.models[1].model, "gemma3");
    assert_eq!(response.models[1].context_length, Some(8192));
    assert_eq!(response.models[2].model, "mistral");
    assert_eq!(
        response.models[2].expires_at,
        Some("2025-01-15T00:00:00Z".to_string())
    );

    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_running_models_async_retry_on_error() {
    let mut server = mockito::Server::new_async().await;

    // First request fails, second succeeds
    let mock_fail = server
        .mock("GET", "/api/ps")
        .with_status(500)
        .expect(1)
        .create_async()
        .await;

    let mock_success = server
        .mock("GET", "/api/ps")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"models": [{"model": "recovered"}]}"#)
        .expect(1)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        1,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let response = client.list_running_models().await.unwrap();

    assert_eq!(response.models.len(), 1);
    assert_eq!(response.models[0].model, "recovered");

    mock_fail.assert_async().await;
    mock_success.assert_async().await;
}

// ============================================================================
// Sync API Tests
// ============================================================================

#[test]
fn test_list_running_models_sync_with_mock() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("GET", "/api/ps")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "models": [
                {
                    "model": "sync-model",
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
    let response = client.list_running_models_blocking().unwrap();

    assert_eq!(response.models.len(), 1);
    assert_eq!(response.models[0].model, "sync-model");
    assert_eq!(response.models[0].size, Some(2000000));

    mock.assert();
}

#[test]
fn test_list_running_models_sync_empty_response() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("GET", "/api/ps")
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
    let response = client.list_running_models_blocking().unwrap();

    assert!(response.models.is_empty());
    mock.assert();
}

#[test]
fn test_list_running_models_sync_full_response() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("GET", "/api/ps")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "models": [
                {
                    "model": "llama3.2",
                    "size": 5000000000,
                    "digest": "deadbeef1234",
                    "details": {
                        "format": "gguf",
                        "family": "llama",
                        "families": ["llama", "instruct"],
                        "parameter_size": "8B",
                        "quantization_level": "Q4_0"
                    },
                    "expires_at": "2025-01-14T10:00:00Z",
                    "size_vram": 4500000000,
                    "context_length": 8192
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
    let response = client.list_running_models_blocking().unwrap();

    assert_eq!(response.models.len(), 1);

    let model = &response.models[0];
    assert_eq!(model.model, "llama3.2");
    assert_eq!(model.size_vram, Some(4500000000));
    assert_eq!(model.context_length, Some(8192));
    assert!(model.details.is_some());

    let details = model.details.as_ref().unwrap();
    assert_eq!(
        details.families,
        Some(vec!["llama".to_string(), "instruct".to_string()])
    );

    mock.assert();
}

#[test]
fn test_list_running_models_sync_retry_on_error() {
    let mut server = mockito::Server::new();

    // First request fails, second succeeds
    let mock_fail = server
        .mock("GET", "/api/ps")
        .with_status(500)
        .expect(1)
        .create();

    let mock_success = server
        .mock("GET", "/api/ps")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"models": [{"model": "sync-recovered"}]}"#)
        .expect(1)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        1,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let response = client.list_running_models_blocking().unwrap();

    assert_eq!(response.models.len(), 1);
    assert_eq!(response.models[0].model, "sync-recovered");

    mock_fail.assert();
    mock_success.assert();
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_list_running_models_async_max_retries_exceeded() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/api/ps")
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
    let result = client.list_running_models().await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[test]
fn test_list_running_models_sync_max_retries_exceeded() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("GET", "/api/ps")
        .with_status(500)
        .expect(3) // Initial + 2 retries
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(1),
        2,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let result = client.list_running_models_blocking();

    assert!(result.is_err());
    mock.assert();
}

// ============================================================================
// RunningModel-specific Field Tests
// ============================================================================

#[tokio::test]
async fn test_list_running_models_async_vram_and_context() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/api/ps")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "models": [
                {
                    "model": "gpu-model",
                    "size": 8000000000,
                    "size_vram": 7500000000,
                    "context_length": 16384
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
    let response = client.list_running_models().await.unwrap();

    let model = &response.models[0];
    assert_eq!(model.model, "gpu-model");
    assert_eq!(model.size, Some(8000000000));
    assert_eq!(model.size_vram, Some(7500000000));
    assert_eq!(model.context_length, Some(16384));

    mock.assert_async().await;
}

#[test]
fn test_list_running_models_sync_expires_at() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("GET", "/api/ps")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "models": [
                {
                    "model": "expiring-model",
                    "expires_at": "2025-12-31T23:59:59.999Z"
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
    let response = client.list_running_models_blocking().unwrap();

    let model = &response.models[0];
    assert_eq!(model.model, "expiring-model");
    assert_eq!(
        model.expires_at,
        Some("2025-12-31T23:59:59.999Z".to_string())
    );

    mock.assert();
}
