// Client Configuration Tests - Phase 0 TDD
// These tests validate the ClientConfig struct

use ollama_oxide::ClientConfig;
use std::time::Duration;

#[test]
fn test_client_config_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ClientConfig>();
}

#[test]
fn test_client_config_default_values() {
    let config = ClientConfig::default();

    assert_eq!(config.base_url(), "http://localhost:11434");
    assert_eq!(config.timeout(), Duration::from_secs(30));
    assert_eq!(config.max_retries(), 3);
}

#[test]
fn test_client_config_custom_values() {
    let config = ClientConfig::new(
        "http://example.com:8080".to_string(),
        Duration::from_secs(60),
        5,
    ).unwrap();

    assert_eq!(config.base_url(), "http://example.com:8080");
    assert_eq!(config.timeout(), Duration::from_secs(60));
    assert_eq!(config.max_retries(), 5);
}

#[test]
fn test_client_config_clone() {
    let config = ClientConfig::new(
        "http://localhost:11434".to_string(),
        Duration::from_secs(30),
        3,
    ).unwrap();

    let cloned = config.clone();

    assert_eq!(config.base_url(), cloned.base_url());
    assert_eq!(config.timeout(), cloned.timeout());
    assert_eq!(config.max_retries(), cloned.max_retries());
}

#[test]
fn test_client_config_debug() {
    let config = ClientConfig::default();
    let debug = format!("{:?}", config);

    assert!(debug.contains("ClientConfig"));
    assert!(debug.contains("localhost"));
}

#[test]
fn test_client_config_with_zero_timeout() {
    let config = ClientConfig::new(
        "http://localhost:11434".to_string(),
        Duration::from_secs(0),
        3,
    ).unwrap();

    assert_eq!(config.timeout(), Duration::from_secs(0));
}

#[test]
fn test_client_config_with_zero_retries() {
    let config = ClientConfig::new(
        "http://localhost:11434".to_string(),
        Duration::from_secs(30),
        0,
    ).unwrap();

    assert_eq!(config.max_retries(), 0);
}

#[test]
fn test_client_config_with_https_url() {
    let config = ClientConfig::new(
        "https://secure.example.com".to_string(),
        Duration::from_secs(30),
        3,
    ).unwrap();

    assert_eq!(config.base_url(), "https://secure.example.com");
}

#[test]
fn test_client_config_with_custom_port() {
    let config = ClientConfig::new(
        "http://localhost:9999".to_string(),
        Duration::from_secs(30),
        3,
    ).unwrap();

    assert_eq!(config.base_url(), "http://localhost:9999");
}

#[test]
fn test_client_config_with_long_timeout() {
    let config = ClientConfig::new(
        "http://localhost:11434".to_string(),
        Duration::from_secs(300),
        3,
    ).unwrap();

    assert_eq!(config.timeout(), Duration::from_secs(300));
}

#[test]
fn test_client_config_with_many_retries() {
    let config = ClientConfig::new(
        "http://localhost:11434".to_string(),
        Duration::from_secs(30),
        10,
    ).unwrap();

    assert_eq!(config.max_retries(), 10);
}
