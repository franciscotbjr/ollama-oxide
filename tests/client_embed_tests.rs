//! Tests for embed API methods (POST /api/embed)

use ollama_oxide::{
    ClientConfig, EmbedInput, EmbedRequest, EmbedResponse, ModelOptions, OllamaApiAsync,
    OllamaApiSync, OllamaClient,
};
use std::time::Duration;

// ============================================================================
// EmbedInput Type Tests
// ============================================================================

#[test]
fn test_embed_input_single_serialization() {
    let input = EmbedInput::single("Hello, world!");
    let json = serde_json::to_string(&input).unwrap();
    assert_eq!(json, r#""Hello, world!""#);
}

#[test]
fn test_embed_input_multiple_serialization() {
    let input = EmbedInput::multiple(["First", "Second"]);
    let json = serde_json::to_string(&input).unwrap();
    assert_eq!(json, r#"["First","Second"]"#);
}

#[test]
fn test_embed_input_single_deserialization() {
    let json = r#""Hello, world!""#;
    let input: EmbedInput = serde_json::from_str(json).unwrap();
    assert_eq!(input, EmbedInput::Single("Hello, world!".to_string()));
}

#[test]
fn test_embed_input_multiple_deserialization() {
    let json = r#"["First","Second"]"#;
    let input: EmbedInput = serde_json::from_str(json).unwrap();
    assert_eq!(
        input,
        EmbedInput::Multiple(vec!["First".to_string(), "Second".to_string()])
    );
}

#[test]
fn test_embed_input_len() {
    let single = EmbedInput::single("Hello");
    assert_eq!(single.len(), 1);

    let multiple = EmbedInput::multiple(["A", "B", "C"]);
    assert_eq!(multiple.len(), 3);
}

#[test]
fn test_embed_input_is_empty() {
    let empty_single = EmbedInput::single("");
    assert!(empty_single.is_empty());

    let non_empty = EmbedInput::single("Hello");
    assert!(!non_empty.is_empty());

    let empty_multiple = EmbedInput::Multiple(vec![]);
    assert!(empty_multiple.is_empty());
}

#[test]
fn test_embed_input_from_string() {
    let input: EmbedInput = "Hello".into();
    assert_eq!(input, EmbedInput::Single("Hello".to_string()));
}

#[test]
fn test_embed_input_from_str() {
    let input: EmbedInput = "Hello".into();
    assert_eq!(input, EmbedInput::Single("Hello".to_string()));
}

#[test]
fn test_embed_input_from_vec() {
    let input: EmbedInput = vec!["A".to_string(), "B".to_string()].into();
    assert_eq!(
        input,
        EmbedInput::Multiple(vec!["A".to_string(), "B".to_string()])
    );
}

#[test]
fn test_embed_input_from_array() {
    let input: EmbedInput = ["A", "B", "C"].into();
    assert_eq!(
        input,
        EmbedInput::Multiple(vec!["A".to_string(), "B".to_string(), "C".to_string()])
    );
}

// ============================================================================
// ModelOptions Type Tests
// ============================================================================

#[test]
fn test_model_options_default() {
    let options = ModelOptions::default();
    assert!(options.seed.is_none());
    assert!(options.temperature.is_none());
    assert!(options.is_empty());
}

#[test]
fn test_model_options_builder() {
    let options = ModelOptions::new()
        .with_temperature(0.7)
        .with_num_ctx(4096)
        .with_seed(42);

    assert_eq!(options.temperature, Some(0.7));
    assert_eq!(options.num_ctx, Some(4096));
    assert_eq!(options.seed, Some(42));
}

#[test]
fn test_model_options_all_builders() {
    let options = ModelOptions::new()
        .with_seed(123)
        .with_temperature(0.8)
        .with_top_k(40)
        .with_top_p(0.9)
        .with_min_p(0.05)
        .with_num_ctx(2048)
        .with_num_predict(100);

    assert_eq!(options.seed, Some(123));
    assert_eq!(options.temperature, Some(0.8));
    assert_eq!(options.top_k, Some(40));
    assert_eq!(options.top_p, Some(0.9));
    assert_eq!(options.min_p, Some(0.05));
    assert_eq!(options.num_ctx, Some(2048));
    assert_eq!(options.num_predict, Some(100));
}

#[test]
fn test_model_options_serialization() {
    let options = ModelOptions::new().with_temperature(0.7);
    let json = serde_json::to_string(&options).unwrap();
    assert!(json.contains("\"temperature\":0.7"));
    // Should not include unset fields
    assert!(!json.contains("seed"));
}

#[test]
fn test_model_options_serialization_empty() {
    let options = ModelOptions::new();
    let json = serde_json::to_string(&options).unwrap();
    assert_eq!(json, "{}");
}

#[test]
fn test_model_options_is_empty() {
    let empty = ModelOptions::new();
    assert!(empty.is_empty());

    let with_temp = ModelOptions::new().with_temperature(0.5);
    assert!(!with_temp.is_empty());
}

// ============================================================================
// EmbedRequest Type Tests
// ============================================================================

#[test]
fn test_embed_request_new() {
    let request = EmbedRequest::new("nomic-embed-text", "Hello, world!");
    assert_eq!(request.model, "nomic-embed-text");
    assert_eq!(
        request.input,
        EmbedInput::Single("Hello, world!".to_string())
    );
}

#[test]
fn test_embed_request_with_options() {
    let request = EmbedRequest::new("nomic-embed-text", "Hello")
        .with_truncate(true)
        .with_dimensions(768)
        .with_keep_alive("5m");

    assert_eq!(request.truncate, Some(true));
    assert_eq!(request.dimensions, Some(768));
    assert_eq!(request.keep_alive, Some("5m".to_string()));
}

#[test]
fn test_embed_request_with_model_options() {
    let options = ModelOptions::new().with_temperature(0.5);
    let request = EmbedRequest::new("nomic-embed-text", "Hello").with_options(options);

    assert!(request.options.is_some());
    assert_eq!(request.options.unwrap().temperature, Some(0.5));
}

#[test]
fn test_embed_request_serialization_minimal() {
    let request = EmbedRequest::new("nomic-embed-text", "Hello");
    let json = serde_json::to_string(&request).unwrap();

    assert!(json.contains(r#""model":"nomic-embed-text""#));
    assert!(json.contains(r#""input":"Hello""#));
    // Should not include optional fields
    assert!(!json.contains("truncate"));
    assert!(!json.contains("dimensions"));
}

#[test]
fn test_embed_request_serialization_full() {
    let request = EmbedRequest::new("nomic-embed-text", "Hello")
        .with_truncate(true)
        .with_dimensions(512)
        .with_keep_alive("10m")
        .with_options(ModelOptions::new().with_temperature(0.5));

    let json = serde_json::to_string(&request).unwrap();

    assert!(json.contains(r#""truncate":true"#));
    assert!(json.contains(r#""dimensions":512"#));
    assert!(json.contains(r#""keep_alive":"10m""#));
    assert!(json.contains(r#""temperature":0.5"#));
}

#[test]
fn test_embed_request_multiple_inputs() {
    let request = EmbedRequest::new(
        "nomic-embed-text",
        EmbedInput::multiple(["First", "Second"]),
    );
    let json = serde_json::to_string(&request).unwrap();

    assert!(json.contains(r#""input":["First","Second"]"#));
}

#[test]
fn test_embed_request_deserialization() {
    let json = r#"{"model":"nomic-embed-text","input":"Hello"}"#;
    let request: EmbedRequest = serde_json::from_str(json).unwrap();

    assert_eq!(request.model, "nomic-embed-text");
    assert_eq!(request.input, EmbedInput::Single("Hello".to_string()));
}

// ============================================================================
// EmbedResponse Type Tests
// ============================================================================

#[test]
fn test_embed_response_deserialization() {
    let json = r#"{
        "model": "nomic-embed-text",
        "embeddings": [[0.1, 0.2, 0.3], [0.4, 0.5, 0.6]],
        "total_duration": 14143917,
        "load_duration": 1019500,
        "prompt_eval_count": 8
    }"#;

    let response: EmbedResponse = serde_json::from_str(json).unwrap();

    assert_eq!(response.model, Some("nomic-embed-text".to_string()));
    assert_eq!(response.embeddings.len(), 2);
    assert_eq!(response.embeddings[0], vec![0.1, 0.2, 0.3]);
    assert_eq!(response.total_duration, Some(14143917));
    assert_eq!(response.prompt_eval_count, Some(8));
}

#[test]
fn test_embed_response_default() {
    let response = EmbedResponse::default();
    assert!(response.model.is_none());
    assert!(response.embeddings.is_empty());
    assert!(response.total_duration.is_none());
}

#[test]
fn test_embed_response_len() {
    let response = EmbedResponse {
        embeddings: vec![vec![0.1], vec![0.2], vec![0.3]],
        ..Default::default()
    };
    assert_eq!(response.len(), 3);
}

#[test]
fn test_embed_response_is_empty() {
    let empty = EmbedResponse::default();
    assert!(empty.is_empty());

    let non_empty = EmbedResponse {
        embeddings: vec![vec![0.1]],
        ..Default::default()
    };
    assert!(!non_empty.is_empty());
}

#[test]
fn test_embed_response_dimensions() {
    let response = EmbedResponse {
        embeddings: vec![vec![0.1, 0.2, 0.3, 0.4]],
        ..Default::default()
    };

    assert_eq!(response.dimensions(), Some(4));
}

#[test]
fn test_embed_response_dimensions_empty() {
    let response = EmbedResponse::default();
    assert_eq!(response.dimensions(), None);
}

#[test]
fn test_embed_response_first_embedding() {
    let response = EmbedResponse {
        embeddings: vec![vec![0.1, 0.2], vec![0.3, 0.4]],
        ..Default::default()
    };

    assert_eq!(response.first_embedding(), Some(&vec![0.1, 0.2]));
}

#[test]
fn test_embed_response_first_embedding_empty() {
    let response = EmbedResponse::default();
    assert_eq!(response.first_embedding(), None);
}

#[test]
fn test_embed_response_duration_conversion() {
    let response = EmbedResponse {
        total_duration: Some(1_000_000), // 1ms in nanoseconds
        load_duration: Some(500_000),    // 0.5ms in nanoseconds
        ..Default::default()
    };

    assert!((response.total_duration_ms().unwrap() - 1.0).abs() < 0.001);
    assert!((response.load_duration_ms().unwrap() - 0.5).abs() < 0.001);
}

#[test]
fn test_embed_response_duration_conversion_none() {
    let response = EmbedResponse::default();
    assert!(response.total_duration_ms().is_none());
    assert!(response.load_duration_ms().is_none());
}

// ============================================================================
// Async API Tests
// ============================================================================

#[tokio::test]
async fn test_embed_async_success() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/embed")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "model": "nomic-embed-text",
            "input": "Hello, world!"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "model": "nomic-embed-text",
            "embeddings": [[0.1, 0.2, 0.3]],
            "total_duration": 1000000,
            "prompt_eval_count": 3
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
    let request = EmbedRequest::new("nomic-embed-text", "Hello, world!");
    let response = client.embed(&request).await.unwrap();

    assert_eq!(response.model, Some("nomic-embed-text".to_string()));
    assert_eq!(response.embeddings.len(), 1);
    assert_eq!(response.embeddings[0], vec![0.1, 0.2, 0.3]);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_embed_async_multiple_inputs() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/embed")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "model": "nomic-embed-text",
            "input": ["First", "Second"]
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "model": "nomic-embed-text",
            "embeddings": [[0.1, 0.2], [0.3, 0.4]]
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
    let request = EmbedRequest::new(
        "nomic-embed-text",
        EmbedInput::multiple(["First", "Second"]),
    );
    let response = client.embed(&request).await.unwrap();

    assert_eq!(response.embeddings.len(), 2);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_embed_async_model_not_found() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/embed")
        .with_status(404)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = EmbedRequest::new("nonexistent-model", "Hello");
    let result = client.embed(&request).await;

    assert!(result.is_err());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_embed_async_retry_on_server_error() {
    let mut server = mockito::Server::new_async().await;

    let mock_fail = server
        .mock("POST", "/api/embed")
        .with_status(500)
        .expect(1)
        .create_async()
        .await;

    let mock_success = server
        .mock("POST", "/api/embed")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"embeddings": [[0.1]]}"#)
        .expect(1)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        1,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = EmbedRequest::new("model", "Hello");
    let result = client.embed(&request).await;

    assert!(result.is_ok());

    mock_fail.assert_async().await;
    mock_success.assert_async().await;
}

#[tokio::test]
async fn test_embed_async_max_retries_exceeded() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/embed")
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
    let request = EmbedRequest::new("model", "Hello");
    let result = client.embed(&request).await;

    assert!(result.is_err());

    mock.assert_async().await;
}

// ============================================================================
// Sync API Tests
// ============================================================================

#[test]
fn test_embed_sync_success() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("POST", "/api/embed")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "model": "nomic-embed-text",
            "embeddings": [[0.1, 0.2, 0.3]]
        }"#,
        )
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = EmbedRequest::new("nomic-embed-text", "Hello");
    let response = client.embed_blocking(&request).unwrap();

    assert_eq!(response.embeddings.len(), 1);

    mock.assert();
}

#[test]
fn test_embed_sync_model_not_found() {
    let mut server = mockito::Server::new();

    let mock = server.mock("POST", "/api/embed").with_status(404).create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = EmbedRequest::new("nonexistent", "Hello");
    let result = client.embed_blocking(&request);

    assert!(result.is_err());

    mock.assert();
}

#[test]
fn test_embed_sync_retry_on_server_error() {
    let mut server = mockito::Server::new();

    let mock_fail = server
        .mock("POST", "/api/embed")
        .with_status(500)
        .expect(1)
        .create();

    let mock_success = server
        .mock("POST", "/api/embed")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"embeddings": [[0.1]]}"#)
        .expect(1)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        1,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = EmbedRequest::new("model", "Hello");
    let result = client.embed_blocking(&request);

    assert!(result.is_ok());

    mock_fail.assert();
    mock_success.assert();
}

// ============================================================================
// Type Safety Tests
// ============================================================================

#[test]
fn test_embed_input_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<EmbedInput>();
}

#[test]
fn test_model_options_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ModelOptions>();
}

#[test]
fn test_embed_request_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<EmbedRequest>();
}

#[test]
fn test_embed_response_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<EmbedResponse>();
}

// ============================================================================
// Clone and Debug Tests
// ============================================================================

#[test]
fn test_embed_input_clone() {
    let input = EmbedInput::single("Hello");
    let cloned = input.clone();
    assert_eq!(input, cloned);
}

#[test]
fn test_embed_input_debug() {
    let input = EmbedInput::single("Hello");
    let debug_str = format!("{:?}", input);
    assert!(debug_str.contains("Hello"));
}

#[test]
fn test_embed_request_clone() {
    let request = EmbedRequest::new("model", "Hello");
    let cloned = request.clone();
    assert_eq!(request, cloned);
}

#[test]
fn test_embed_response_clone() {
    let response = EmbedResponse {
        embeddings: vec![vec![0.1, 0.2]],
        ..Default::default()
    };
    let cloned = response.clone();
    assert_eq!(response, cloned);
}
