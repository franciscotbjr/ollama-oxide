# Implementation Plan: POST /api/copy

**Endpoint:** POST /api/copy
**Complexity:** Simple (first POST endpoint)
**Phase:** Phase 1 - Foundation + Non-Streaming Endpoints
**Document Version:** 1.0
**Created:** 2026-01-15

## Overview

This document outlines the implementation plan for the `POST /api/copy` endpoint, which duplicates an existing model with a new name.

This is the **first POST endpoint** in the library, requiring:
- New `post_with_retry` and `post_blocking_with_retry` helper methods in `client.rs`
- A request type (`CopyRequest`)
- Handling of empty response body (200 OK with no content)

## API Specification Summary

**Endpoint:** `POST /api/copy`
**Operation ID:** `copy`
**Description:** Copy a model - creates a new model with a different name from an existing model

**Request Body:**
```json
{
  "source": "gemma3",
  "destination": "gemma3-backup"
}
```

**Response:**
- `200 OK` - Model successfully copied (empty body)

## Schema Analysis

### CopyRequest (New Type)

```rust
pub struct CopyRequest {
    /// Existing model name to copy from
    pub source: String,
    /// New model name to create
    pub destination: String,
}
```

### Response Handling

The `/api/copy` endpoint returns `200 OK` with an empty body on success. We have two options:

**Option A: Return `Result<()>`**
- Simple, matches semantics (no response data)
- Method signature: `fn copy_model(&self, request: CopyRequest) -> Result<()>`

**Option B: Return `Result<CopyResponse>` (empty struct)**
- Consistent with other endpoints
- Allows future API changes without breaking interface
- Method signature: `fn copy_model(&self, request: CopyRequest) -> Result<CopyResponse>`

**Decision:** Use Option A (`Result<()>`) since the API explicitly returns no data. This is idiomatic Rust for operations that succeed or fail without returning data.

## Implementation Strategy

### Step 1: Add POST Helper Methods to client.rs

The client currently only has `get_with_retry` and `get_blocking_with_retry`. We need POST equivalents that:
- Accept a serializable request body
- Handle empty response (for /api/copy)
- Support response deserialization (for future POST endpoints)

**Location:** `src/http/client.rs`

#### 1.1 Async POST with Empty Response

```rust
/// Execute async HTTP POST request with retry logic (no response body)
///
/// For endpoints that return 200 OK with empty body.
pub(super) async fn post_empty_with_retry<R>(&self, url: &str, body: &R) -> Result<()>
where
    R: serde::Serialize,
{
    for attempt in 0..=self.config.max_retries {
        match self.client.post(url).json(body).send().await {
            Ok(response) => {
                // Retry on server errors (5xx)
                if response.status().is_server_error() && attempt < self.config.max_retries {
                    tokio::time::sleep(Duration::from_millis(100 * (attempt as u64 + 1))).await;
                    continue;
                }

                // Check for success status
                if response.status().is_success() {
                    return Ok(());
                }

                // Client error - no retry
                return Err(Error::HttpError(response.status().as_u16()));
            }
            Err(_e) => {
                // Retry on network errors
                if attempt < self.config.max_retries {
                    tokio::time::sleep(Duration::from_millis(100 * (attempt as u64 + 1))).await;
                }
            }
        }
    }

    Err(Error::MaxRetriesExceededError(self.config.max_retries))
}
```

#### 1.2 Blocking POST with Empty Response

```rust
/// Execute blocking HTTP POST request with retry logic (no response body)
///
/// For endpoints that return 200 OK with empty body.
pub(super) fn post_empty_blocking_with_retry<R>(&self, url: &str, body: &R) -> Result<()>
where
    R: serde::Serialize,
{
    let blocking_client = reqwest::blocking::Client::builder()
        .timeout(self.config.timeout)
        .build()?;

    for attempt in 0..=self.config.max_retries {
        match blocking_client.post(url).json(body).send() {
            Ok(response) => {
                // Retry on server errors (5xx)
                if response.status().is_server_error() && attempt < self.config.max_retries {
                    std::thread::sleep(Duration::from_millis(100 * (attempt as u64 + 1)));
                    continue;
                }

                // Check for success status
                if response.status().is_success() {
                    return Ok(());
                }

                // Client error - no retry
                return Err(Error::HttpError(response.status().as_u16()));
            }
            Err(_e) => {
                // Retry on network errors
                if attempt < self.config.max_retries {
                    std::thread::sleep(Duration::from_millis(100 * (attempt as u64 + 1)));
                }
            }
        }
    }

    Err(Error::MaxRetriesExceededError(self.config.max_retries))
}
```

### Step 2: Add HttpError Variant to Error Type

**Location:** `src/error.rs`

Add a new error variant for HTTP status code errors:

```rust
/// HTTP error with status code
#[error("HTTP error: {0}")]
HttpError(u16),
```

### Step 3: Create CopyRequest Primitive Type

**Location:** `src/primitives/copy_request.rs`

```rust
//! Copy model request primitive type

use serde::{Deserialize, Serialize};

/// Request body for POST /api/copy endpoint
///
/// Creates a copy of an existing model with a new name.
///
/// # Example
///
/// ```
/// use ollama_oxide::CopyRequest;
///
/// let request = CopyRequest {
///     source: "llama3.1".to_string(),
///     destination: "llama3.1-backup".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CopyRequest {
    /// Existing model name to copy from
    pub source: String,
    /// New model name to create
    pub destination: String,
}

impl CopyRequest {
    /// Create a new copy request
    ///
    /// # Arguments
    ///
    /// * `source` - Name of the existing model to copy
    /// * `destination` - Name for the new model copy
    ///
    /// # Example
    ///
    /// ```
    /// use ollama_oxide::CopyRequest;
    ///
    /// let request = CopyRequest::new("llama3.1", "llama3.1-backup");
    /// ```
    pub fn new(source: impl Into<String>, destination: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            destination: destination.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_request_serialization() {
        let request = CopyRequest {
            source: "gemma3".to_string(),
            destination: "gemma3-backup".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"source\":\"gemma3\""));
        assert!(json.contains("\"destination\":\"gemma3-backup\""));
    }

    #[test]
    fn test_copy_request_deserialization() {
        let json = r#"{"source": "llama3", "destination": "llama3-copy"}"#;
        let request: CopyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.source, "llama3");
        assert_eq!(request.destination, "llama3-copy");
    }

    #[test]
    fn test_copy_request_new() {
        let request = CopyRequest::new("model-a", "model-b");
        assert_eq!(request.source, "model-a");
        assert_eq!(request.destination, "model-b");
    }

    #[test]
    fn test_copy_request_roundtrip() {
        let request = CopyRequest::new("source-model", "dest-model");
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: CopyRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request, deserialized);
    }

    #[test]
    fn test_copy_request_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<CopyRequest>();
    }
}
```

### Step 4: Update primitives/mod.rs

**Location:** `src/primitives/mod.rs`

Add module declaration and re-export:

```rust
mod copy_request;

pub use copy_request::CopyRequest;
```

### Step 5: Update lib.rs Re-exports

**Location:** `src/lib.rs`

Add `CopyRequest` to public re-exports:

```rust
pub use primitives::{
    CopyRequest, ListResponse, ModelDetails, ModelSummary, PsResponse, RunningModel, VersionResponse,
};
```

### Step 6: Add API Methods

#### 6.1 Async API

**Location:** `src/http/api_async.rs`

Add to `OllamaApiAsync` trait:

```rust
/// Copy a model (async)
///
/// Creates a copy of an existing model with a new name.
///
/// # Arguments
///
/// * `request` - Copy request containing source and destination model names
///
/// # Errors
///
/// Returns an error if:
/// - Source model doesn't exist
/// - Destination model name is invalid
/// - Network request fails
/// - Maximum retry attempts exceeded
///
/// # Examples
///
/// ```no_run
/// use ollama_oxide::{OllamaClient, OllamaApiAsync, CopyRequest};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = OllamaClient::default()?;
/// let request = CopyRequest::new("llama3.1", "llama3.1-backup");
/// client.copy_model(&request).await?;
/// println!("Model copied successfully!");
/// # Ok(())
/// # }
/// ```
async fn copy_model(&self, request: &CopyRequest) -> Result<()>;
```

Add implementation:

```rust
async fn copy_model(&self, request: &CopyRequest) -> Result<()> {
    let url = self.config.url(Endpoints::COPY);
    self.post_empty_with_retry(&url, request).await
}
```

#### 6.2 Sync API

**Location:** `src/http/api_sync.rs`

Add to `OllamaApiSync` trait:

```rust
/// Copy a model (blocking)
///
/// Creates a copy of an existing model with a new name.
/// This method blocks the current thread until the request completes.
///
/// # Arguments
///
/// * `request` - Copy request containing source and destination model names
///
/// # Errors
///
/// Returns an error if:
/// - Source model doesn't exist
/// - Destination model name is invalid
/// - Network request fails
/// - Maximum retry attempts exceeded
///
/// # Examples
///
/// ```no_run
/// use ollama_oxide::{OllamaClient, OllamaApiSync, CopyRequest};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = OllamaClient::default()?;
/// let request = CopyRequest::new("llama3.1", "llama3.1-backup");
/// client.copy_model_blocking(&request)?;
/// println!("Model copied successfully!");
/// # Ok(())
/// # }
/// ```
fn copy_model_blocking(&self, request: &CopyRequest) -> Result<()>;
```

Add implementation:

```rust
fn copy_model_blocking(&self, request: &CopyRequest) -> Result<()> {
    let url = self.config.url(Endpoints::COPY);
    self.post_empty_blocking_with_retry(&url, request)
}
```

### Step 7: Add Unit Tests

**Location:** `tests/client_copy_model_tests.rs`

```rust
//! Tests for copy_model API methods (POST /api/copy)

use ollama_oxide::{ClientConfig, CopyRequest, OllamaApiAsync, OllamaApiSync, OllamaClient};
use std::time::Duration;

// ============================================================================
// Async API Tests
// ============================================================================

#[tokio::test]
async fn test_copy_model_async_success() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/copy")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "source": "llama3.1",
            "destination": "llama3.1-backup"
        })))
        .with_status(200)
        .create_async()
        .await;

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 0,
    };

    let client = OllamaClient::new(config).unwrap();
    let request = CopyRequest::new("llama3.1", "llama3.1-backup");
    let result = client.copy_model(&request).await;

    assert!(result.is_ok());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_copy_model_async_model_not_found() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/copy")
        .with_status(404)
        .create_async()
        .await;

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 0,
    };

    let client = OllamaClient::new(config).unwrap();
    let request = CopyRequest::new("nonexistent", "backup");
    let result = client.copy_model(&request).await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_copy_model_async_retry_on_server_error() {
    let mut server = mockito::Server::new_async().await;

    let mock_fail = server
        .mock("POST", "/api/copy")
        .with_status(500)
        .expect(1)
        .create_async()
        .await;

    let mock_success = server
        .mock("POST", "/api/copy")
        .with_status(200)
        .expect(1)
        .create_async()
        .await;

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 1,
    };

    let client = OllamaClient::new(config).unwrap();
    let request = CopyRequest::new("model", "model-copy");
    let result = client.copy_model(&request).await;

    assert!(result.is_ok());
    mock_fail.assert_async().await;
    mock_success.assert_async().await;
}

// ============================================================================
// Sync API Tests
// ============================================================================

#[test]
fn test_copy_model_sync_success() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("POST", "/api/copy")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "source": "gemma3",
            "destination": "gemma3-backup"
        })))
        .with_status(200)
        .create();

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 0,
    };

    let client = OllamaClient::new(config).unwrap();
    let request = CopyRequest::new("gemma3", "gemma3-backup");
    let result = client.copy_model_blocking(&request);

    assert!(result.is_ok());
    mock.assert();
}

#[test]
fn test_copy_model_sync_model_not_found() {
    let mut server = mockito::Server::new();

    let mock = server.mock("POST", "/api/copy").with_status(404).create();

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 0,
    };

    let client = OllamaClient::new(config).unwrap();
    let request = CopyRequest::new("missing", "copy");
    let result = client.copy_model_blocking(&request);

    assert!(result.is_err());
    mock.assert();
}

#[test]
fn test_copy_model_sync_retry_on_server_error() {
    let mut server = mockito::Server::new();

    let mock_fail = server
        .mock("POST", "/api/copy")
        .with_status(500)
        .expect(1)
        .create();

    let mock_success = server
        .mock("POST", "/api/copy")
        .with_status(200)
        .expect(1)
        .create();

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 1,
    };

    let client = OllamaClient::new(config).unwrap();
    let request = CopyRequest::new("model", "backup");
    let result = client.copy_model_blocking(&request);

    assert!(result.is_ok());
    mock_fail.assert();
    mock_success.assert();
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_copy_model_async_max_retries_exceeded() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/copy")
        .with_status(500)
        .expect(3)
        .create_async()
        .await;

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(1),
        max_retries: 2,
    };

    let client = OllamaClient::new(config).unwrap();
    let request = CopyRequest::new("model", "copy");
    let result = client.copy_model(&request).await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[test]
fn test_copy_model_sync_max_retries_exceeded() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("POST", "/api/copy")
        .with_status(500)
        .expect(3)
        .create();

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(1),
        max_retries: 2,
    };

    let client = OllamaClient::new(config).unwrap();
    let request = CopyRequest::new("model", "copy");
    let result = client.copy_model_blocking(&request);

    assert!(result.is_err());
    mock.assert();
}
```

### Step 8: Add Examples

**Location:** `examples/copy_model_async.rs`

```rust
//! Example: Copy a model (async)
//!
//! This example demonstrates how to create a copy of an existing model.
//!
//! Run with: cargo run --example copy_model_async
//!
//! Note: Requires a running Ollama server with at least one model installed.

use ollama_oxide::{CopyRequest, OllamaApiAsync, OllamaClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with default configuration
    let client = OllamaClient::default()?;

    // Define source and destination
    let source = "llama3.1";
    let destination = "llama3.1-backup";

    println!("Copying model '{}' to '{}'...", source, destination);

    // Create copy request
    let request = CopyRequest::new(source, destination);

    // Execute copy
    match client.copy_model(&request).await {
        Ok(()) => {
            println!("Model copied successfully!");
            println!("You can now use '{}' as a separate model.", destination);
        }
        Err(e) => {
            eprintln!("Failed to copy model: {}", e);
            eprintln!("Make sure the source model '{}' exists.", source);
        }
    }

    Ok(())
}
```

**Location:** `examples/copy_model_sync.rs`

```rust
//! Example: Copy a model (sync)
//!
//! This example demonstrates how to create a copy of an existing model
//! using the blocking API.
//!
//! Run with: cargo run --example copy_model_sync
//!
//! Note: Requires a running Ollama server with at least one model installed.

use ollama_oxide::{CopyRequest, OllamaApiSync, OllamaClient};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with default configuration
    let client = OllamaClient::default()?;

    // Define source and destination
    let source = "llama3.1";
    let destination = "llama3.1-backup";

    println!("Copying model '{}' to '{}'...", source, destination);

    // Create copy request
    let request = CopyRequest::new(source, destination);

    // Execute copy
    match client.copy_model_blocking(&request) {
        Ok(()) => {
            println!("Model copied successfully!");
            println!("You can now use '{}' as a separate model.", destination);
        }
        Err(e) => {
            eprintln!("Failed to copy model: {}", e);
            eprintln!("Make sure the source model '{}' exists.", source);
        }
    }

    Ok(())
}
```

## File Changes Summary

### New Files
| File | Description |
|------|-------------|
| `src/primitives/copy_request.rs` | CopyRequest struct |
| `tests/client_copy_model_tests.rs` | Unit tests with mocking |
| `examples/copy_model_async.rs` | Async usage example |
| `examples/copy_model_sync.rs` | Sync usage example |

### Modified Files
| File | Changes |
|------|---------|
| `src/error.rs` | Add `HttpError(u16)` variant |
| `src/http/client.rs` | Add `post_empty_with_retry` and `post_empty_blocking_with_retry` methods |
| `src/primitives/mod.rs` | Add module declaration and re-export |
| `src/lib.rs` | Add `CopyRequest` to public re-exports |
| `src/http/api_async.rs` | Add `copy_model()` method |
| `src/http/api_sync.rs` | Add `copy_model_blocking()` method |

## Testing Checklist

- [ ] Unit tests for CopyRequest pass
- [ ] CopyRequest Send + Sync test passes
- [ ] POST helper methods work correctly
- [ ] Async copy_model tests pass
- [ ] Sync copy_model_blocking tests pass
- [ ] Retry logic tests pass
- [ ] Error handling tests pass
- [ ] Examples build successfully
- [ ] `cargo build` succeeds
- [ ] `cargo test` passes
- [ ] `cargo clippy` has no warnings
- [ ] `cargo fmt` applied

## Implementation Order

1. Add `HttpError` variant to `src/error.rs`
2. Add POST helper methods to `src/http/client.rs`
3. Create `src/primitives/copy_request.rs` with struct and tests
4. Update `src/primitives/mod.rs` with new export
5. Update `src/lib.rs` with public re-export
6. Add async method to `src/http/api_async.rs`
7. Add sync method to `src/http/api_sync.rs`
8. Create unit tests in `tests/client_copy_model_tests.rs`
9. Create async example
10. Create sync example
11. Run full test suite
12. Update DEV_NOTES.md if needed

## Notes

- This is the **first POST endpoint** - establishes pattern for future POST implementations
- The `Endpoints::COPY` constant already exists in `endpoints.rs`
- Empty response handling (`Result<()>`) is specific to this endpoint
- Future POST endpoints like `/api/show` will need `post_with_retry` that returns deserialized response
- All types must implement `Send + Sync` for thread safety
- Error handling for 4xx errors (like 404 model not found) should not retry
