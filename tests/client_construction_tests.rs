// Client Construction Tests - Phase 0 TDD
// These tests validate OllamaClient construction and basic properties

use ollama_oxide::{ClientConfig, OllamaClient};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_ollama_client_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<OllamaClient>();
}

#[test]
fn test_client_creation_with_default_config() {
    let client = OllamaClient::default();
    assert!(client.is_ok());
}

#[test]
fn test_client_creation_with_custom_config() {
    let config = ClientConfig::new(
        "http://localhost:11434".to_string(),
        Duration::from_secs(30),
        3,
    ).unwrap();

    let client = OllamaClient::new(config);
    assert!(client.is_ok());
}

#[test]
fn test_client_creation_with_invalid_url() {
    let result = ClientConfig::new(
        "not-a-valid-url".to_string(),
        Duration::from_secs(30),
        3,
    );
    assert!(result.is_err());
}

#[test]
fn test_client_creation_with_invalid_url_missing_scheme() {
    let result = ClientConfig::new(
        "localhost:11434".to_string(),
        Duration::from_secs(30),
        3,
    );
    assert!(result.is_err());
}

#[test]
fn test_client_creation_with_empty_url() {
    let result = ClientConfig::new(
        "".to_string(),
        Duration::from_secs(30),
        3,
    );
    assert!(result.is_err());
}

#[test]
fn test_client_with_base_url_helper() {
    let client = OllamaClient::with_base_url("http://localhost:11434");
    assert!(client.is_ok());
}

#[test]
fn test_client_with_base_url_helper_invalid() {
    let result = OllamaClient::with_base_url("invalid-url");
    assert!(result.is_err());
}

#[test]
fn test_client_with_base_url_custom_port() {
    let client = OllamaClient::with_base_url("http://localhost:9999");
    assert!(client.is_ok());
}

#[test]
fn test_client_with_base_url_https() {
    let client = OllamaClient::with_base_url("https://secure.example.com");
    assert!(client.is_ok());
}

#[test]
fn test_client_is_clone() {
    let client = OllamaClient::default().unwrap();
    let cloned = client.clone();

    // Both should be valid clients
    drop(client);
    drop(cloned);
}

#[test]
fn test_client_debug_format() {
    let client = OllamaClient::default().unwrap();
    let debug = format!("{:?}", client);
    assert!(debug.contains("OllamaClient"));
}

#[test]
fn test_client_shared_across_threads() {
    let client = Arc::new(OllamaClient::default().unwrap());
    let mut handles = vec![];

    // Spawn 10 threads, each cloning the client
    for i in 0..10 {
        let client_clone = Arc::clone(&client);
        let handle = thread::spawn(move || {
            // Just verify the client can be used in the thread
            let _id = i;
            let _c = client_clone;
            // Thread completes successfully
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_client_clone_multiple_times() {
    let client1 = OllamaClient::default().unwrap();
    let client2 = client1.clone();
    let client3 = client2.clone();
    let client4 = client3.clone();

    // All should be valid
    drop(client1);
    drop(client2);
    drop(client3);
    drop(client4);
}

#[test]
fn test_client_creation_with_ipv4_address() {
    let client = OllamaClient::with_base_url("http://127.0.0.1:11434");
    assert!(client.is_ok());
}

#[test]
fn test_client_creation_with_ipv6_address() {
    let client = OllamaClient::with_base_url("http://[::1]:11434");
    assert!(client.is_ok());
}

#[test]
fn test_client_creation_with_domain_name() {
    let client = OllamaClient::with_base_url("http://example.com");
    assert!(client.is_ok());
}

#[test]
fn test_client_with_base_url_and_timeout() {
    let client = OllamaClient::with_base_url_and_timeout(
        "http://localhost:8080",
        Duration::from_secs(60),
    );
    assert!(client.is_ok());
}

#[test]
fn test_client_with_base_url_and_timeout_invalid_url() {
    let result = OllamaClient::with_base_url_and_timeout(
        "not-a-url",
        Duration::from_secs(60),
    );
    assert!(result.is_err());
}

#[test]
fn test_client_config_new_validates_url() {
    let valid = ClientConfig::new(
        "http://localhost:11434".to_string(),
        Duration::from_secs(30),
        3,
    );
    assert!(valid.is_ok());

    let invalid = ClientConfig::new(
        "not-a-url".to_string(),
        Duration::from_secs(30),
        3,
    );
    assert!(invalid.is_err());
}

#[test]
fn test_client_config_with_base_url_validates_url() {
    assert!(ClientConfig::with_base_url("http://valid.url".to_string()).is_ok());
    assert!(ClientConfig::with_base_url("not-a-url".to_string()).is_err());
    assert!(ClientConfig::with_base_url("ftp://invalid.scheme".to_string()).is_err());
}

#[test]
fn test_client_config_with_base_url_and_timeout_validates_url() {
    assert!(ClientConfig::with_base_url_and_timeout(
        "http://valid.url".to_string(),
        Duration::from_secs(60),
    ).is_ok());

    assert!(ClientConfig::with_base_url_and_timeout(
        "not-a-url".to_string(),
        Duration::from_secs(60),
    ).is_err());
}
