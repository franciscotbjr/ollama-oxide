//! Tests for show_model API methods (POST /api/show)

use ollama_oxide::{
    ClientConfig, OllamaApiAsync, OllamaApiSync, OllamaClient, ShowModelDetails, ShowRequest,
    ShowResponse,
};
use std::time::Duration;

// ============================================================================
// Async API Tests
// ============================================================================

#[tokio::test]
async fn test_show_model_async_success() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/show")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "model": "llama3.1"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "license": "MIT License",
                "parameters": "temperature 0.7",
                "template": "{{ .System }}\n{{ .Prompt }}",
                "capabilities": ["completion", "vision"],
                "details": {
                    "format": "gguf",
                    "family": "llama",
                    "parameter_size": "8B",
                    "quantization_level": "Q4_K_M"
                },
                "model_info": {
                    "general.architecture": "llama",
                    "llama.context_length": 8192
                },
                "modified_at": "2024-01-15T10:30:00Z"
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
    let request = ShowRequest::new("llama3.1");
    let result = client.show_model(&request).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.license, Some("MIT License".to_string()));
    assert_eq!(response.parameters, Some("temperature 0.7".to_string()));
    assert!(response.capabilities.is_some());
    assert!(response.has_capability("completion"));
    assert!(response.has_capability("vision"));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_show_model_async_verbose() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/show")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "model": "gemma3",
            "verbose": true
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"license": "Apache 2.0", "capabilities": ["completion"]}"#)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = ShowRequest::verbose("gemma3");
    let result = client.show_model(&request).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.license, Some("Apache 2.0".to_string()));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_show_model_async_model_not_found() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/show")
        .with_status(404)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = ShowRequest::new("nonexistent");
    let result = client.show_model(&request).await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_show_model_async_retry_on_server_error() {
    let mut server = mockito::Server::new_async().await;

    let mock_fail = server
        .mock("POST", "/api/show")
        .with_status(500)
        .expect(1)
        .create_async()
        .await;

    let mock_success = server
        .mock("POST", "/api/show")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"capabilities": ["completion"]}"#)
        .expect(1)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        1,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = ShowRequest::new("llama3.1");
    let result = client.show_model(&request).await;

    assert!(result.is_ok());
    mock_fail.assert_async().await;
    mock_success.assert_async().await;
}

#[tokio::test]
async fn test_show_model_async_minimal_response() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/show")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{}"#)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = ShowRequest::new("minimal-model");
    let result = client.show_model(&request).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.license.is_none());
    assert!(response.capabilities.is_none());
    mock.assert_async().await;
}

// ============================================================================
// Sync API Tests
// ============================================================================

#[test]
fn test_show_model_sync_success() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("POST", "/api/show")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "model": "gemma3"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "license": "Apache 2.0",
                "capabilities": ["completion"],
                "details": {
                    "format": "gguf",
                    "family": "gemma3",
                    "parameter_size": "4.3B"
                }
            }"#,
        )
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = ShowRequest::new("gemma3");
    let result = client.show_model_blocking(&request);

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.license, Some("Apache 2.0".to_string()));
    assert!(response.details.is_some());
    let details = response.details.unwrap();
    assert_eq!(details.family, Some("gemma3".to_string()));
    mock.assert();
}

#[test]
fn test_show_model_sync_verbose() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("POST", "/api/show")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "model": "mistral",
            "verbose": true
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"template": "{{ .Prompt }}"}"#)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = ShowRequest::verbose("mistral");
    let result = client.show_model_blocking(&request);

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.template, Some("{{ .Prompt }}".to_string()));
    mock.assert();
}

#[test]
fn test_show_model_sync_model_not_found() {
    let mut server = mockito::Server::new();

    let mock = server.mock("POST", "/api/show").with_status(404).create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = ShowRequest::new("missing-model");
    let result = client.show_model_blocking(&request);

    assert!(result.is_err());
    mock.assert();
}

#[test]
fn test_show_model_sync_retry_on_server_error() {
    let mut server = mockito::Server::new();

    let mock_fail = server
        .mock("POST", "/api/show")
        .with_status(500)
        .expect(1)
        .create();

    let mock_success = server
        .mock("POST", "/api/show")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"capabilities": ["completion"]}"#)
        .expect(1)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        1,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = ShowRequest::new("llama3.1");
    let result = client.show_model_blocking(&request);

    assert!(result.is_ok());
    mock_fail.assert();
    mock_success.assert();
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_show_model_async_max_retries_exceeded() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/show")
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
    let request = ShowRequest::new("llama3.1");
    let result = client.show_model(&request).await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[test]
fn test_show_model_sync_max_retries_exceeded() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("POST", "/api/show")
        .with_status(500)
        .expect(3) // Initial + 2 retries
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(1),
        2,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = ShowRequest::new("llama3.1");
    let result = client.show_model_blocking(&request);

    assert!(result.is_err());
    mock.assert();
}

#[tokio::test]
async fn test_show_model_async_bad_request() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/show")
        .with_status(400)
        .expect(1) // Should NOT retry on 4xx
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        2,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = ShowRequest::new("invalid");
    let result = client.show_model(&request).await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[test]
fn test_show_model_sync_bad_request() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("POST", "/api/show")
        .with_status(400)
        .expect(1) // Should only be called once (no retry on 4xx)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        2,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = ShowRequest::new("invalid");
    let result = client.show_model_blocking(&request);

    assert!(result.is_err());
    mock.assert();
}

// ============================================================================
// ShowRequest Tests
// ============================================================================

#[test]
fn test_show_request_new_constructor() {
    let request = ShowRequest::new("llama3.1");
    assert_eq!(request.model, "llama3.1");
    assert!(request.verbose.is_none());
}

#[test]
fn test_show_request_verbose_constructor() {
    let request = ShowRequest::verbose("gemma3");
    assert_eq!(request.model, "gemma3");
    assert_eq!(request.verbose, Some(true));
}

#[test]
fn test_show_request_struct_init() {
    let request = ShowRequest {
        model: "my-model".to_string(),
        verbose: Some(false),
    };
    assert_eq!(request.model, "my-model");
    assert_eq!(request.verbose, Some(false));
}

#[test]
fn test_show_request_clone() {
    let request = ShowRequest::verbose("llama3.1");
    let cloned = request.clone();
    assert_eq!(request, cloned);
}

#[test]
fn test_show_request_debug() {
    let request = ShowRequest::new("llama3.1");
    let debug_str = format!("{:?}", request);
    assert!(debug_str.contains("llama3.1"));
}

// ============================================================================
// ShowResponse Tests
// ============================================================================

#[test]
fn test_show_response_has_capability() {
    let response = ShowResponse {
        capabilities: Some(vec![
            "completion".to_string(),
            "vision".to_string(),
            "tools".to_string(),
        ]),
        ..Default::default()
    };

    assert!(response.has_capability("completion"));
    assert!(response.has_capability("vision"));
    assert!(response.has_capability("tools"));
    assert!(!response.has_capability("embedding"));
}

#[test]
fn test_show_response_has_capability_none() {
    let response = ShowResponse::default();
    assert!(!response.has_capability("completion"));
}

#[test]
fn test_show_response_has_capability_empty() {
    let response = ShowResponse {
        capabilities: Some(vec![]),
        ..Default::default()
    };
    assert!(!response.has_capability("completion"));
}

#[test]
fn test_show_response_default() {
    let response = ShowResponse::default();
    assert!(response.parameters.is_none());
    assert!(response.license.is_none());
    assert!(response.modified_at.is_none());
    assert!(response.details.is_none());
    assert!(response.template.is_none());
    assert!(response.capabilities.is_none());
    assert!(response.model_info.is_none());
}

#[test]
fn test_show_response_clone() {
    let response = ShowResponse {
        license: Some("MIT".to_string()),
        capabilities: Some(vec!["completion".to_string()]),
        ..Default::default()
    };
    let cloned = response.clone();
    assert_eq!(response, cloned);
}

// ============================================================================
// ShowModelDetails Tests
// ============================================================================

#[test]
fn test_show_model_details_default() {
    let details = ShowModelDetails::default();
    assert!(details.parent_model.is_none());
    assert!(details.format.is_none());
    assert!(details.family.is_none());
    assert!(details.families.is_none());
    assert!(details.parameter_size.is_none());
    assert!(details.quantization_level.is_none());
}

#[test]
fn test_show_model_details_full() {
    let details = ShowModelDetails {
        parent_model: Some("base-model".to_string()),
        format: Some("gguf".to_string()),
        family: Some("llama".to_string()),
        families: Some(vec!["llama".to_string(), "transformer".to_string()]),
        parameter_size: Some("8B".to_string()),
        quantization_level: Some("Q4_K_M".to_string()),
    };

    assert_eq!(details.parent_model, Some("base-model".to_string()));
    assert_eq!(details.format, Some("gguf".to_string()));
    assert_eq!(details.family, Some("llama".to_string()));
    assert_eq!(details.parameter_size, Some("8B".to_string()));
    assert_eq!(details.quantization_level, Some("Q4_K_M".to_string()));
}

#[test]
fn test_show_model_details_clone() {
    let details = ShowModelDetails {
        format: Some("gguf".to_string()),
        family: Some("gemma".to_string()),
        ..Default::default()
    };
    let cloned = details.clone();
    assert_eq!(details, cloned);
}
