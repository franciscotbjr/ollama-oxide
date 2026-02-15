//! Unit tests for POST /api/create endpoint
//!
//! All tests use mockito for HTTP mocking - no real Ollama server required.

use mockito::{Matcher, Server};
use ollama_oxide::{ChatMessage, ClientConfig, OllamaApiAsync, OllamaApiSync, OllamaClient};

use ollama_oxide::{CreateRequest, CreateResponse, LicenseSetting};
use serde_json::json;
use std::time::Duration;

fn make_config(base_url: String) -> ClientConfig {
    ClientConfig::new(
        base_url,
        Duration::from_secs(30),
        3,
    ).unwrap()
}

// ============================================================================
// Async Client Tests (with mocking)
// ============================================================================

#[tokio::test]
async fn test_create_model_async_success() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/api/create")
        .match_body(Matcher::PartialJson(json!({
            "model": "mario",
            "from": "qwen3:0.6b",
            "stream": false
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"status": "success"}"#)
        .create_async()
        .await;

    let config = make_config(server.url());
    let client = OllamaClient::new(config).unwrap();

    let request = CreateRequest::from_model("mario", "qwen3:0.6b");
    let response = client.create_model(&request).await.unwrap();

    assert!(response.is_success());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_create_model_async_with_system() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/api/create")
        .match_body(Matcher::PartialJson(json!({
            "model": "mario",
            "from": "qwen3:0.6b",
            "system": "You are Mario from Super Mario Bros.",
            "stream": false
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"status": "success"}"#)
        .create_async()
        .await;

    let config = make_config(server.url());
    let client = OllamaClient::new(config).unwrap();

    let request = CreateRequest::from_model("mario", "qwen3:0.6b")
        .with_system("You are Mario from Super Mario Bros.");
    let response = client.create_model(&request).await.unwrap();

    assert!(response.is_success());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_create_model_async_with_all_options() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/api/create")
        .match_body(Matcher::PartialJson(json!({
            "model": "mario",
            "from": "qwen3:0.6b",
            "system": "You are Mario",
            "template": "{{ .System }}\n{{ .Prompt }}",
            "license": "MIT",
            "quantize": "q4_K_M",
            "stream": false
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"status": "success"}"#)
        .create_async()
        .await;

    let config = make_config(server.url());
    let client = OllamaClient::new(config).unwrap();

    let request = CreateRequest::from_model("mario", "qwen3:0.6b")
        .with_system("You are Mario")
        .with_template("{{ .System }}\n{{ .Prompt }}")
        .with_license("MIT")
        .with_quantize("q4_K_M");

    let response = client.create_model(&request).await.unwrap();
    assert!(response.is_success());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_create_model_async_with_messages() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/api/create")
        .match_body(Matcher::PartialJson(json!({
            "model": "mario",
            "from": "qwen3:0.6b",
            "messages": [
                {"role": "user", "content": "Who are you?"},
                {"role": "assistant", "content": "It's-a me, Mario!"}
            ],
            "stream": false
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"status": "success"}"#)
        .create_async()
        .await;

    let config = make_config(server.url());
    let client = OllamaClient::new(config).unwrap();

    let request = CreateRequest::from_model("mario", "qwen3:0.6b").with_messages([
        ChatMessage::user("Who are you?"),
        ChatMessage::assistant("It's-a me, Mario!"),
    ]);

    let response = client.create_model(&request).await.unwrap();
    assert!(response.is_success());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_create_model_async_with_parameters() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/api/create")
        .match_body(Matcher::PartialJson(json!({
            "model": "mario",
            "from": "qwen3:0.6b",
            "parameters": {
                "temperature": 0.8,
                "top_k": 40
            },
            "stream": false
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"status": "success"}"#)
        .create_async()
        .await;

    let config = make_config(server.url());
    let client = OllamaClient::new(config).unwrap();

    let request = CreateRequest::from_model("mario", "qwen3:0.6b").with_parameters(json!({
        "temperature": 0.8,
        "top_k": 40
    }));

    let response = client.create_model(&request).await.unwrap();
    assert!(response.is_success());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_create_model_async_with_multiple_licenses() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/api/create")
        .match_body(Matcher::PartialJson(json!({
            "model": "mario",
            "from": "qwen3:0.6b",
            "license": ["MIT", "Apache-2.0"],
            "stream": false
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"status": "success"}"#)
        .create_async()
        .await;

    let config = make_config(server.url());
    let client = OllamaClient::new(config).unwrap();

    let request = CreateRequest::from_model("mario", "qwen3:0.6b")
        .with_license(LicenseSetting::multiple(["MIT", "Apache-2.0"]));

    let response = client.create_model(&request).await.unwrap();
    assert!(response.is_success());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_create_model_async_not_found() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/api/create")
        .with_status(404)
        .with_body(r#"{"error": "model not found"}"#)
        .create_async()
        .await;

    let config = make_config(server.url());
    let client = OllamaClient::new(config).unwrap();

    let request = CreateRequest::from_model("mario", "nonexistent:latest");
    let result = client.create_model(&request).await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_create_model_async_response_status() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/api/create")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"status": "success"}"#)
        .create_async()
        .await;

    let config = make_config(server.url());
    let client = OllamaClient::new(config).unwrap();

    let request = CreateRequest::new("test-model");
    let response = client.create_model(&request).await.unwrap();

    assert_eq!(response.status(), Some("success"));
    assert!(response.is_success());
    mock.assert_async().await;
}

// ============================================================================
// Sync Client Tests (with mocking)
// ============================================================================

#[test]
fn test_create_model_sync_success() {
    let mut server = Server::new();

    let mock = server
        .mock("POST", "/api/create")
        .match_body(Matcher::PartialJson(json!({
            "model": "mario",
            "from": "qwen3:0.6b",
            "stream": false
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"status": "success"}"#)
        .create();

    let config = make_config(server.url());
    let client = OllamaClient::new(config).unwrap();

    let request = CreateRequest::from_model("mario", "qwen3:0.6b");
    let response = client.create_model_blocking(&request).unwrap();

    assert!(response.is_success());
    mock.assert();
}

#[test]
fn test_create_model_sync_with_system() {
    let mut server = Server::new();

    let mock = server
        .mock("POST", "/api/create")
        .match_body(Matcher::PartialJson(json!({
            "model": "mario",
            "from": "qwen3:0.6b",
            "system": "You are Mario from Super Mario Bros.",
            "stream": false
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"status": "success"}"#)
        .create();

    let config = make_config(server.url());
    let client = OllamaClient::new(config).unwrap();

    let request = CreateRequest::from_model("mario", "qwen3:0.6b")
        .with_system("You are Mario from Super Mario Bros.");
    let response = client.create_model_blocking(&request).unwrap();

    assert!(response.is_success());
    mock.assert();
}

#[test]
fn test_create_model_sync_not_found() {
    let mut server = Server::new();

    let mock = server
        .mock("POST", "/api/create")
        .with_status(404)
        .with_body(r#"{"error": "model not found"}"#)
        .create();

    let config = make_config(server.url());
    let client = OllamaClient::new(config).unwrap();

    let request = CreateRequest::from_model("mario", "nonexistent:latest");
    let result = client.create_model_blocking(&request);

    assert!(result.is_err());
    mock.assert();
}

// ============================================================================
// CreateResponse Tests
// ============================================================================

#[test]
fn test_create_response_deserialization() {
    let json = r#"{"status": "success"}"#;
    let response: CreateResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.status(), Some("success"));
    assert!(response.is_success());
}

#[test]
fn test_create_response_is_success_true() {
    let response: CreateResponse = serde_json::from_str(r#"{"status": "success"}"#).unwrap();
    assert!(response.is_success());
}

#[test]
fn test_create_response_is_success_false() {
    let response: CreateResponse = serde_json::from_str(r#"{"status": "creating"}"#).unwrap();
    assert!(!response.is_success());
}

#[test]
fn test_create_response_empty() {
    let response: CreateResponse = serde_json::from_str("{}").unwrap();
    assert!(response.status().is_none());
    assert!(!response.is_success());
}
