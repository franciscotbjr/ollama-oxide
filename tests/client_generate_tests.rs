//! Tests for generate API methods (POST /api/generate)

use ollama_oxide::{
    ClientConfig, FormatSetting, GenerateRequest, GenerateResponse, KeepAliveSetting, Logprob,
    ModelOptions, OllamaApiAsync, OllamaApiSync, OllamaClient, StopSetting, ThinkSetting,
    TokenLogprob,
};
use std::time::Duration;

// ============================================================================
// ThinkSetting Type Tests
// ============================================================================

#[test]
fn test_think_setting_enabled_serialization() {
    let setting = ThinkSetting::enabled();
    let json = serde_json::to_string(&setting).unwrap();
    assert_eq!(json, "true");
}

#[test]
fn test_think_setting_disabled_serialization() {
    let setting = ThinkSetting::disabled();
    let json = serde_json::to_string(&setting).unwrap();
    assert_eq!(json, "false");
}

#[test]
fn test_think_setting_level_serialization() {
    let setting = ThinkSetting::high();
    let json = serde_json::to_string(&setting).unwrap();
    assert_eq!(json, r#""high""#);
}

#[test]
fn test_think_setting_bool_deserialization() {
    let json = "true";
    let setting: ThinkSetting = serde_json::from_str(json).unwrap();
    assert_eq!(setting, ThinkSetting::Enabled(true));
}

#[test]
fn test_think_setting_string_deserialization() {
    let json = r#""medium""#;
    let setting: ThinkSetting = serde_json::from_str(json).unwrap();
    assert_eq!(setting, ThinkSetting::Level("medium".to_string()));
}

#[test]
fn test_think_setting_from_bool() {
    let setting: ThinkSetting = true.into();
    assert_eq!(setting, ThinkSetting::Enabled(true));
}

#[test]
fn test_think_setting_from_str() {
    let setting: ThinkSetting = "low".into();
    assert_eq!(setting, ThinkSetting::Level("low".to_string()));
}

#[test]
fn test_think_setting_levels() {
    assert_eq!(
        ThinkSetting::high(),
        ThinkSetting::Level("high".to_string())
    );
    assert_eq!(
        ThinkSetting::medium(),
        ThinkSetting::Level("medium".to_string())
    );
    assert_eq!(ThinkSetting::low(), ThinkSetting::Level("low".to_string()));
}

// ============================================================================
// FormatSetting Type Tests
// ============================================================================

#[test]
fn test_format_setting_json_serialization() {
    let setting = FormatSetting::json();
    let json = serde_json::to_string(&setting).unwrap();
    assert_eq!(json, r#""json""#);
}

#[test]
fn test_format_setting_schema_serialization() {
    let schema = serde_json::json!({"type": "object"});
    let setting = FormatSetting::schema(schema);
    let json = serde_json::to_string(&setting).unwrap();
    assert_eq!(json, r#"{"type":"object"}"#);
}

#[test]
fn test_format_setting_string_deserialization() {
    let json = r#""json""#;
    let setting: FormatSetting = serde_json::from_str(json).unwrap();
    assert_eq!(setting, FormatSetting::String("json".to_string()));
}

#[test]
fn test_format_setting_schema_deserialization() {
    let json = r#"{"type":"object"}"#;
    let setting: FormatSetting = serde_json::from_str(json).unwrap();
    assert_eq!(
        setting,
        FormatSetting::Schema(serde_json::json!({"type": "object"}))
    );
}

#[test]
fn test_format_setting_from_str() {
    let setting: FormatSetting = "json".into();
    assert_eq!(setting, FormatSetting::String("json".to_string()));
}

#[test]
fn test_format_setting_from_value() {
    let schema = serde_json::json!({"type": "string"});
    let setting: FormatSetting = schema.clone().into();
    assert_eq!(setting, FormatSetting::Schema(schema));
}

// ============================================================================
// KeepAliveSetting Type Tests
// ============================================================================

#[test]
fn test_keep_alive_duration_serialization() {
    let setting = KeepAliveSetting::duration("5m");
    let json = serde_json::to_string(&setting).unwrap();
    assert_eq!(json, r#""5m""#);
}

#[test]
fn test_keep_alive_seconds_serialization() {
    let setting = KeepAliveSetting::seconds(300);
    let json = serde_json::to_string(&setting).unwrap();
    assert_eq!(json, "300");
}

#[test]
fn test_keep_alive_unload_serialization() {
    let setting = KeepAliveSetting::unload_immediately();
    let json = serde_json::to_string(&setting).unwrap();
    assert_eq!(json, "0");
}

#[test]
fn test_keep_alive_string_deserialization() {
    let json = r#""10m""#;
    let setting: KeepAliveSetting = serde_json::from_str(json).unwrap();
    assert_eq!(setting, KeepAliveSetting::Duration("10m".to_string()));
}

#[test]
fn test_keep_alive_number_deserialization() {
    let json = "600";
    let setting: KeepAliveSetting = serde_json::from_str(json).unwrap();
    assert_eq!(setting, KeepAliveSetting::Seconds(600));
}

#[test]
fn test_keep_alive_from_str() {
    let setting: KeepAliveSetting = "1h".into();
    assert_eq!(setting, KeepAliveSetting::Duration("1h".to_string()));
}

#[test]
fn test_keep_alive_from_i64() {
    let setting: KeepAliveSetting = 120i64.into();
    assert_eq!(setting, KeepAliveSetting::Seconds(120));
}

// ============================================================================
// StopSetting Type Tests
// ============================================================================

#[test]
fn test_stop_setting_single_serialization() {
    let setting = StopSetting::single("\n");
    let json = serde_json::to_string(&setting).unwrap();
    assert_eq!(json, r#""\n""#);
}

#[test]
fn test_stop_setting_multiple_serialization() {
    let setting = StopSetting::multiple([".", "!", "?"]);
    let json = serde_json::to_string(&setting).unwrap();
    assert_eq!(json, r#"[".","!","?"]"#);
}

#[test]
fn test_stop_setting_single_deserialization() {
    let json = r#""stop""#;
    let setting: StopSetting = serde_json::from_str(json).unwrap();
    assert_eq!(setting, StopSetting::Single("stop".to_string()));
}

#[test]
fn test_stop_setting_multiple_deserialization() {
    let json = r#"["a","b"]"#;
    let setting: StopSetting = serde_json::from_str(json).unwrap();
    assert_eq!(
        setting,
        StopSetting::Multiple(vec!["a".to_string(), "b".to_string()])
    );
}

#[test]
fn test_stop_setting_from_str() {
    let setting: StopSetting = "stop".into();
    assert_eq!(setting, StopSetting::Single("stop".to_string()));
}

#[test]
fn test_stop_setting_from_vec() {
    let setting: StopSetting = vec!["a".to_string(), "b".to_string()].into();
    assert_eq!(
        setting,
        StopSetting::Multiple(vec!["a".to_string(), "b".to_string()])
    );
}

// ============================================================================
// TokenLogprob Type Tests
// ============================================================================

#[test]
fn test_token_logprob_deserialization() {
    let json = r#"{"token":"The","logprob":-0.5,"bytes":[84,104,101]}"#;
    let logprob: TokenLogprob = serde_json::from_str(json).unwrap();

    assert_eq!(logprob.token, Some("The".to_string()));
    assert_eq!(logprob.logprob, Some(-0.5));
    assert_eq!(logprob.bytes, Some(vec![84, 104, 101]));
}

#[test]
fn test_token_logprob_partial_deserialization() {
    let json = r#"{"token":"A"}"#;
    let logprob: TokenLogprob = serde_json::from_str(json).unwrap();

    assert_eq!(logprob.token, Some("A".to_string()));
    assert!(logprob.logprob.is_none());
    assert!(logprob.bytes.is_none());
}

// ============================================================================
// Logprob Type Tests
// ============================================================================

#[test]
fn test_logprob_deserialization() {
    let json = r#"{
        "token": "The",
        "logprob": -0.5,
        "bytes": [84, 104, 101],
        "top_logprobs": [
            {"token": "The", "logprob": -0.5},
            {"token": "A", "logprob": -1.2}
        ]
    }"#;
    let logprob: Logprob = serde_json::from_str(json).unwrap();

    assert_eq!(logprob.token, Some("The".to_string()));
    assert_eq!(logprob.logprob, Some(-0.5));
    assert!(logprob.top_logprobs.is_some());
    assert_eq!(logprob.top_logprobs.as_ref().unwrap().len(), 2);
}

#[test]
fn test_logprob_minimal_deserialization() {
    let json = r#"{"token":"x"}"#;
    let logprob: Logprob = serde_json::from_str(json).unwrap();

    assert_eq!(logprob.token, Some("x".to_string()));
    assert!(logprob.top_logprobs.is_none());
}

// ============================================================================
// GenerateRequest Type Tests
// ============================================================================

#[test]
fn test_generate_request_new() {
    let request = GenerateRequest::new("qwen3:0.6b", "Why is the sky blue?");

    assert_eq!(request.model, "qwen3:0.6b");
    assert_eq!(request.prompt, Some("Why is the sky blue?".to_string()));
    assert_eq!(request.stream, Some(false)); // Default for v0.1.0
}

#[test]
fn test_generate_request_with_system() {
    let request =
        GenerateRequest::new("model", "Hello").with_system("You are a helpful assistant.");

    assert_eq!(
        request.system,
        Some("You are a helpful assistant.".to_string())
    );
}

#[test]
fn test_generate_request_with_format() {
    let request = GenerateRequest::new("model", "Hello").with_format(FormatSetting::json());

    assert_eq!(request.format, Some(FormatSetting::json()));
}

#[test]
fn test_generate_request_with_think() {
    let request = GenerateRequest::new("model", "Hello").with_think(ThinkSetting::enabled());

    assert_eq!(request.think, Some(ThinkSetting::Enabled(true)));
}

#[test]
fn test_generate_request_with_options() {
    let options = ModelOptions::new().with_temperature(0.7);
    let request = GenerateRequest::new("model", "Hello").with_options(options);

    assert!(request.options.is_some());
    assert_eq!(request.options.unwrap().temperature, Some(0.7));
}

#[test]
fn test_generate_request_with_keep_alive() {
    let request =
        GenerateRequest::new("model", "Hello").with_keep_alive(KeepAliveSetting::seconds(300));

    assert_eq!(request.keep_alive, Some(KeepAliveSetting::Seconds(300)));
}

#[test]
fn test_generate_request_with_logprobs() {
    let request = GenerateRequest::new("model", "Hello")
        .with_logprobs(true)
        .with_top_logprobs(5);

    assert_eq!(request.logprobs, Some(true));
    assert_eq!(request.top_logprobs, Some(5));
}

#[test]
fn test_generate_request_with_raw() {
    let request = GenerateRequest::new("model", "Hello").with_raw(true);

    assert_eq!(request.raw, Some(true));
}

#[test]
fn test_generate_request_with_suffix() {
    let request = GenerateRequest::new("model", "Hello").with_suffix("world");

    assert_eq!(request.suffix, Some("world".to_string()));
}

#[test]
fn test_generate_request_with_image() {
    let request = GenerateRequest::new("model", "Describe this image")
        .with_image("base64data1")
        .with_image("base64data2");

    assert_eq!(
        request.images,
        Some(vec!["base64data1".to_string(), "base64data2".to_string()])
    );
}

#[test]
fn test_generate_request_with_images() {
    let request = GenerateRequest::new("model", "Describe").with_images(["img1", "img2", "img3"]);

    assert_eq!(
        request.images,
        Some(vec![
            "img1".to_string(),
            "img2".to_string(),
            "img3".to_string()
        ])
    );
}

#[test]
fn test_generate_request_serialization_minimal() {
    let request = GenerateRequest::new("qwen3:0.6b", "Hello");
    let json = serde_json::to_string(&request).unwrap();

    assert!(json.contains(r#""model":"qwen3:0.6b""#));
    assert!(json.contains(r#""prompt":"Hello""#));
    assert!(json.contains(r#""stream":false"#));
    // Should not include unset optional fields
    assert!(!json.contains("system"));
    assert!(!json.contains("format"));
}

#[test]
fn test_generate_request_serialization_full() {
    let request = GenerateRequest::new("model", "Hello")
        .with_system("Be helpful")
        .with_format(FormatSetting::json())
        .with_think(ThinkSetting::high())
        .with_keep_alive(KeepAliveSetting::duration("5m"))
        .with_options(ModelOptions::new().with_temperature(0.8));

    let json = serde_json::to_string(&request).unwrap();

    assert!(json.contains(r#""system":"Be helpful""#));
    assert!(json.contains(r#""format":"json""#));
    assert!(json.contains(r#""think":"high""#));
    assert!(json.contains(r#""keep_alive":"5m""#));
    assert!(json.contains(r#""temperature":0.8"#));
}

// ============================================================================
// GenerateResponse Type Tests
// ============================================================================

#[test]
fn test_generate_response_deserialization() {
    let json = r#"{
        "model": "qwen3:0.6b",
        "created_at": "2025-10-17T23:14:07.414671Z",
        "response": "The sky is blue because...",
        "done": true,
        "done_reason": "stop",
        "total_duration": 174560334,
        "load_duration": 101397084,
        "prompt_eval_count": 11,
        "prompt_eval_duration": 13074791,
        "eval_count": 18,
        "eval_duration": 52479709
    }"#;

    let response: GenerateResponse = serde_json::from_str(json).unwrap();

    assert_eq!(response.model, Some("qwen3:0.6b".to_string()));
    assert_eq!(
        response.response,
        Some("The sky is blue because...".to_string())
    );
    assert_eq!(response.done, Some(true));
    assert_eq!(response.done_reason, Some("stop".to_string()));
    assert_eq!(response.eval_count, Some(18));
}

#[test]
fn test_generate_response_with_thinking() {
    let json = r#"{
        "response": "The answer is 42.",
        "thinking": "Let me think about this...",
        "done": true
    }"#;

    let response: GenerateResponse = serde_json::from_str(json).unwrap();

    assert_eq!(response.thinking_text(), Some("Let me think about this..."));
}

#[test]
fn test_generate_response_with_logprobs() {
    let json = r#"{
        "response": "Hello",
        "done": true,
        "logprobs": [{"token": "Hello", "logprob": -0.1}]
    }"#;

    let response: GenerateResponse = serde_json::from_str(json).unwrap();

    assert!(response.logprobs.is_some());
    assert_eq!(response.logprobs.as_ref().unwrap().len(), 1);
}

#[test]
fn test_generate_response_default() {
    let response = GenerateResponse::default();

    assert!(response.model.is_none());
    assert!(response.response.is_none());
    assert!(!response.is_done());
}

#[test]
fn test_generate_response_text() {
    let response = GenerateResponse {
        response: Some("Hello, world!".to_string()),
        ..Default::default()
    };

    assert_eq!(response.text(), Some("Hello, world!"));
}

#[test]
fn test_generate_response_is_done() {
    let incomplete = GenerateResponse::default();
    assert!(!incomplete.is_done());

    let complete = GenerateResponse {
        done: Some(true),
        ..Default::default()
    };
    assert!(complete.is_done());
}

#[test]
fn test_generate_response_duration_conversions() {
    let response = GenerateResponse {
        total_duration: Some(1_000_000_000), // 1 second in nanoseconds
        load_duration: Some(500_000_000),    // 0.5 seconds
        prompt_eval_duration: Some(100_000_000), // 0.1 seconds
        eval_duration: Some(400_000_000),    // 0.4 seconds
        ..Default::default()
    };

    assert!((response.total_duration_ms().unwrap() - 1000.0).abs() < 0.001);
    assert!((response.load_duration_ms().unwrap() - 500.0).abs() < 0.001);
    assert!((response.prompt_eval_duration_ms().unwrap() - 100.0).abs() < 0.001);
    assert!((response.eval_duration_ms().unwrap() - 400.0).abs() < 0.001);
}

#[test]
fn test_generate_response_tokens_per_second() {
    let response = GenerateResponse {
        eval_count: Some(100),
        eval_duration: Some(1_000_000_000), // 1 second
        ..Default::default()
    };

    let tps = response.tokens_per_second().unwrap();
    assert!((tps - 100.0).abs() < 0.001);
}

#[test]
fn test_generate_response_tokens_per_second_zero_duration() {
    let response = GenerateResponse {
        eval_count: Some(100),
        eval_duration: Some(0),
        ..Default::default()
    };

    assert!(response.tokens_per_second().is_none());
}

#[test]
fn test_generate_response_tokens_per_second_missing() {
    let response = GenerateResponse::default();
    assert!(response.tokens_per_second().is_none());
}

// ============================================================================
// ModelOptions with StopSetting Tests
// ============================================================================

#[test]
fn test_model_options_with_stop() {
    let options = ModelOptions::new().with_stop(StopSetting::single("\n"));

    assert!(options.stop.is_some());
    assert_eq!(options.stop, Some(StopSetting::Single("\n".to_string())));
}

#[test]
fn test_model_options_with_stop_serialization() {
    let options = ModelOptions::new()
        .with_temperature(0.7)
        .with_stop(StopSetting::multiple([".", "!"]));

    let json = serde_json::to_string(&options).unwrap();

    assert!(json.contains(r#""stop":[".","!"]"#));
}

#[test]
fn test_model_options_is_empty_with_stop() {
    let empty = ModelOptions::new();
    assert!(empty.is_empty());

    let with_stop = ModelOptions::new().with_stop("stop");
    assert!(!with_stop.is_empty());
}

// ============================================================================
// Async API Tests
// ============================================================================

#[tokio::test]
async fn test_generate_async_success() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/generate")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "model": "qwen3:0.6b",
            "prompt": "Hello",
            "stream": false
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "model": "qwen3:0.6b",
            "response": "Hello! How can I help you?",
            "done": true,
            "done_reason": "stop",
            "eval_count": 10,
            "eval_duration": 500000000
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
    let request = GenerateRequest::new("qwen3:0.6b", "Hello");
    let response = client.generate(&request).await.unwrap();

    assert_eq!(response.model, Some("qwen3:0.6b".to_string()));
    assert_eq!(response.text(), Some("Hello! How can I help you?"));
    assert!(response.is_done());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_generate_async_with_options() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/generate")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "model": "model",
            "prompt": "Hello",
            "stream": false,
            "system": "Be helpful",
            "options": {
                "temperature": 0.7
            }
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"response": "Hi!", "done": true}"#)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = GenerateRequest::new("model", "Hello")
        .with_system("Be helpful")
        .with_options(ModelOptions::new().with_temperature(0.7));
    let response = client.generate(&request).await.unwrap();

    assert_eq!(response.text(), Some("Hi!"));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_generate_async_model_not_found() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/generate")
        .with_status(404)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = GenerateRequest::new("nonexistent", "Hello");
    let result = client.generate(&request).await;

    assert!(result.is_err());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_generate_async_retry_on_server_error() {
    let mut server = mockito::Server::new_async().await;

    let mock_fail = server
        .mock("POST", "/api/generate")
        .with_status(500)
        .expect(1)
        .create_async()
        .await;

    let mock_success = server
        .mock("POST", "/api/generate")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"response": "Ok", "done": true}"#)
        .expect(1)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        1,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = GenerateRequest::new("model", "Hello");
    let result = client.generate(&request).await;

    assert!(result.is_ok());

    mock_fail.assert_async().await;
    mock_success.assert_async().await;
}

#[tokio::test]
async fn test_generate_async_max_retries_exceeded() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/generate")
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
    let request = GenerateRequest::new("model", "Hello");
    let result = client.generate(&request).await;

    assert!(result.is_err());

    mock.assert_async().await;
}

// ============================================================================
// Sync API Tests
// ============================================================================

#[test]
fn test_generate_sync_success() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("POST", "/api/generate")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "model": "qwen3:0.6b",
            "response": "Hello there!",
            "done": true
        }"#,
        )
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = GenerateRequest::new("qwen3:0.6b", "Hello");
    let response = client.generate_blocking(&request).unwrap();

    assert_eq!(response.text(), Some("Hello there!"));

    mock.assert();
}

#[test]
fn test_generate_sync_model_not_found() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("POST", "/api/generate")
        .with_status(404)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = GenerateRequest::new("nonexistent", "Hello");
    let result = client.generate_blocking(&request);

    assert!(result.is_err());

    mock.assert();
}

#[test]
fn test_generate_sync_retry_on_server_error() {
    let mut server = mockito::Server::new();

    let mock_fail = server
        .mock("POST", "/api/generate")
        .with_status(500)
        .expect(1)
        .create();

    let mock_success = server
        .mock("POST", "/api/generate")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"response": "Ok", "done": true}"#)
        .expect(1)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        1,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = GenerateRequest::new("model", "Hello");
    let result = client.generate_blocking(&request);

    assert!(result.is_ok());

    mock_fail.assert();
    mock_success.assert();
}

// ============================================================================
// Type Safety Tests
// ============================================================================

#[test]
fn test_think_setting_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ThinkSetting>();
}

#[test]
fn test_format_setting_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<FormatSetting>();
}

#[test]
fn test_keep_alive_setting_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<KeepAliveSetting>();
}

#[test]
fn test_stop_setting_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<StopSetting>();
}

#[test]
fn test_generate_request_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<GenerateRequest>();
}

#[test]
fn test_generate_response_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<GenerateResponse>();
}

#[test]
fn test_logprob_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Logprob>();
}

#[test]
fn test_token_logprob_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<TokenLogprob>();
}

// ============================================================================
// Clone and Debug Tests
// ============================================================================

#[test]
fn test_think_setting_clone() {
    let setting = ThinkSetting::high();
    let cloned = setting.clone();
    assert_eq!(setting, cloned);
}

#[test]
fn test_format_setting_clone() {
    let setting = FormatSetting::json();
    let cloned = setting.clone();
    assert_eq!(setting, cloned);
}

#[test]
fn test_generate_request_clone() {
    let request = GenerateRequest::new("model", "Hello").with_system("Be helpful");
    let cloned = request.clone();
    assert_eq!(request, cloned);
}

#[test]
fn test_generate_response_clone() {
    let response = GenerateResponse {
        response: Some("Hello".to_string()),
        done: Some(true),
        ..Default::default()
    };
    let cloned = response.clone();
    assert_eq!(response, cloned);
}

#[test]
fn test_think_setting_debug() {
    let setting = ThinkSetting::high();
    let debug_str = format!("{:?}", setting);
    assert!(debug_str.contains("high"));
}

#[test]
fn test_generate_request_debug() {
    let request = GenerateRequest::new("model", "Hello");
    let debug_str = format!("{:?}", request);
    assert!(debug_str.contains("model"));
    assert!(debug_str.contains("Hello"));
}
