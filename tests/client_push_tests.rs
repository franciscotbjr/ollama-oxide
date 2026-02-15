//! Tests for POST /api/push endpoint (push_model, push_model_blocking)

use mockito::{Matcher, Server};
use ollama_oxide::{ClientConfig, OllamaApiAsync, OllamaApiSync, OllamaClient, PushRequest};
use std::time::Duration;

fn make_config(base_url: String) -> ClientConfig {
    ClientConfig::new(
        base_url,
        Duration::from_secs(30),
        3,
    ).unwrap()
}

// ============================================================================
// Async Tests
// ============================================================================

#[tokio::test]
async fn test_push_model_success() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/api/push")
        .match_body(Matcher::Json(serde_json::json!({
            "model": "myuser/mymodel:latest",
            "stream": false
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"status": "success"}"#)
        .create_async()
        .await;

    let config = make_config(server.url());
    let client = OllamaClient::new(config).unwrap();

    let request = PushRequest::new("myuser/mymodel:latest");
    let response = client.push_model(&request).await.unwrap();

    assert!(response.is_success());
    assert_eq!(response.status(), Some("success"));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_push_model_with_insecure() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/api/push")
        .match_body(Matcher::Json(serde_json::json!({
            "model": "private/model:latest",
            "insecure": true,
            "stream": false
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"status": "success"}"#)
        .create_async()
        .await;

    let config = make_config(server.url());
    let client = OllamaClient::new(config).unwrap();

    let request = PushRequest::new("private/model:latest").with_insecure(true);
    let response = client.push_model(&request).await.unwrap();

    assert!(response.is_success());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_push_model_not_found() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/api/push")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "model not found"}"#)
        .create_async()
        .await;

    let config = make_config(server.url());
    let client = OllamaClient::new(config).unwrap();

    let request = PushRequest::new("nonexistent:latest");
    let result = client.push_model(&request).await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_push_model_unauthorized() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/api/push")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "unauthorized"}"#)
        .create_async()
        .await;

    let config = make_config(server.url());
    let client = OllamaClient::new(config).unwrap();

    let request = PushRequest::new("myuser/mymodel:latest");
    let result = client.push_model(&request).await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_push_model_response_status_methods() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/api/push")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"status": "uploading"}"#)
        .create_async()
        .await;

    let config = make_config(server.url());
    let client = OllamaClient::new(config).unwrap();

    let request = PushRequest::new("model:tag");
    let response = client.push_model(&request).await.unwrap();

    assert!(!response.is_success());
    assert_eq!(response.status(), Some("uploading"));
    mock.assert_async().await;
}

// ============================================================================
// Sync Tests
// ============================================================================

#[test]
fn test_push_model_blocking_success() {
    let mut server = Server::new();
    let mock = server
        .mock("POST", "/api/push")
        .match_body(Matcher::Json(serde_json::json!({
            "model": "user/model:v1",
            "stream": false
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"status": "success"}"#)
        .create();

    let config = make_config(server.url());
    let client = OllamaClient::new(config).unwrap();

    let request = PushRequest::new("user/model:v1");
    let response = client.push_model_blocking(&request).unwrap();

    assert!(response.is_success());
    mock.assert();
}

#[test]
fn test_push_model_blocking_with_insecure() {
    let mut server = Server::new();
    let mock = server
        .mock("POST", "/api/push")
        .match_body(Matcher::Json(serde_json::json!({
            "model": "custom:model",
            "insecure": true,
            "stream": false
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"status": "success"}"#)
        .create();

    let config = make_config(server.url());
    let client = OllamaClient::new(config).unwrap();

    let request = PushRequest::new("custom:model").with_insecure(true);
    let response = client.push_model_blocking(&request).unwrap();

    assert!(response.is_success());
    mock.assert();
}

#[test]
fn test_push_model_blocking_not_found() {
    let mut server = Server::new();
    let mock = server.mock("POST", "/api/push").with_status(404).create();

    let config = make_config(server.url());
    let client = OllamaClient::new(config).unwrap();

    let request = PushRequest::new("nonexistent:latest");
    let result = client.push_model_blocking(&request);

    assert!(result.is_err());
    mock.assert();
}

#[test]
fn test_push_model_blocking_empty_response() {
    let mut server = Server::new();
    let mock = server
        .mock("POST", "/api/push")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{}"#)
        .create();

    let config = make_config(server.url());
    let client = OllamaClient::new(config).unwrap();

    let request = PushRequest::new("model:tag");
    let response = client.push_model_blocking(&request).unwrap();

    assert!(!response.is_success());
    assert_eq!(response.status(), None);
    mock.assert();
}
