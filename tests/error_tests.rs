// Error Type Tests - Phase 0 TDD
// These tests validate the Error enum and its implementations

use ollama_oxide::{Error, Result};

#[test]
fn test_error_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Error>();
}

#[test]
fn test_http_error_display() {
    let error = Error::HttpError("connection refused".to_string());
    let display = format!("{}", error);
    assert!(display.contains("HTTP request failed"));
    assert!(display.contains("connection refused"));
}

#[test]
fn test_serialization_error_display() {
    let error = Error::SerializationError("invalid JSON".to_string());
    let display = format!("{}", error);
    assert!(display.contains("Serialization error"));
    assert!(display.contains("invalid JSON"));
}

#[test]
fn test_api_error_display() {
    let error = Error::ApiError {
        message: "model not found".to_string(),
    };
    let display = format!("{}", error);
    assert!(display.contains("API error"));
    assert!(display.contains("model not found"));
}

#[test]
fn test_connection_error_display() {
    let error = Error::ConnectionError("timeout".to_string());
    let display = format!("{}", error);
    assert!(display.contains("Connection error"));
    assert!(display.contains("timeout"));
}

#[test]
fn test_invalid_url_error_from_parse_error() {
    use url::Url;

    let parse_result = Url::parse("not-a-valid-url");
    assert!(parse_result.is_err());

    let parse_error = parse_result.unwrap_err();
    let error: Error = parse_error.into();

    let display = format!("{}", error);
    assert!(display.contains("Invalid URL"));
}

#[test]
fn test_timeout_error_display() {
    let error = Error::TimeoutError(30);
    let display = format!("{}", error);
    assert!(display.contains("Request timeout"));
    assert!(display.contains("30"));
    assert!(display.contains("seconds"));
}

#[test]
fn test_stream_error_display() {
    let error = Error::StreamError("invalid NDJSON line".to_string());
    let display = format!("{}", error);
    assert!(display.contains("Stream error"));
    assert!(display.contains("invalid NDJSON"));
}

#[test]
fn test_max_retries_exceeded_error_display() {
    let error = Error::MaxRetriesExceededError(3);
    let display = format!("{}", error);
    assert!(display.contains("Maximum retry attempts"));
    assert!(display.contains("3"));
    assert!(display.contains("exceeded"));
}

#[test]
fn test_error_is_std_error() {
    let error = Error::HttpError("test".to_string());
    let _std_error: &dyn std::error::Error = &error;
}

#[test]
fn test_result_type_alias() {
    fn returns_result() -> Result<String> {
        Ok("success".to_string())
    }

    let result = returns_result();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
}

#[test]
fn test_error_debug_format() {
    let error = Error::ApiError {
        message: "test error".to_string(),
    };
    let debug = format!("{:?}", error);
    assert!(debug.contains("ApiError"));
}
