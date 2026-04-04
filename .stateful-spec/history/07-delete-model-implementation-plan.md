# Implementation Plan: DELETE /api/delete

**Endpoint:** DELETE /api/delete
**Complexity:** Simple (second POST-like endpoint with request body)
**Phase:** Phase 1 - Foundation + Non-Streaming Endpoints
**Document Version:** 1.0
**Created:** 2026-01-16

## Overview

This document outlines the implementation plan for the `DELETE /api/delete` endpoint, which deletes a model from the Ollama server.

This endpoint is very similar to `POST /api/copy`:
- Requires a request body with model name
- Returns empty response on success (200 OK)
- Uses DELETE HTTP method instead of POST

The implementation can reuse the existing `post_empty_with_retry` helper methods pattern but needs new DELETE-specific helpers.

## API Specification Summary

**Endpoint:** `DELETE /api/delete`
**Operation ID:** `delete`
**Description:** Delete a model from the Ollama server

**Request Body:**
```json
{
  "model": "gemma3"
}
```

**Response:**
- `200 OK` - Model successfully deleted (empty body)
- `404 Not Found` - Model does not exist

## Schema Analysis

### DeleteRequest (New Type)

```rust
pub struct DeleteRequest {
    /// Name of the model to delete
    pub model: String,
}
```

This is a simple struct with a single required field matching the OpenAPI specification.

### Response Handling

The `/api/delete` endpoint returns `200 OK` with an empty body on success, exactly like `/api/copy`. We'll use `Result<()>` as the return type.

**Method signatures:**
- Async: `fn delete_model(&self, request: &DeleteRequest) -> Result<()>`
- Sync: `fn delete_model_blocking(&self, request: &DeleteRequest) -> Result<()>`

## Implementation Strategy

### Step 1: Add DELETE Helper Methods to client.rs

Following the pattern of `post_empty_with_retry`, we need DELETE equivalents.

**Location:** `src/http/client.rs`

#### 1.1 Async DELETE with Empty Response

```rust
/// Execute async HTTP DELETE request with retry logic (no response body)
///
/// For endpoints that return 200 OK with empty body.
///
/// # Type Parameters
///
/// * `R` - Request type that implements `Serialize`
///
/// # Arguments
///
/// * `url` - Full URL to request
/// * `body` - Request body to serialize as JSON
///
/// # Errors
///
/// Returns an error if:
/// - Maximum retry attempts exceeded
/// - Client errors (4xx) occur (no retry)
pub(super) async fn delete_empty_with_retry<R>(&self, url: &str, body: &R) -> Result<()>
where
    R: serde::Serialize,
{
    for attempt in 0..=self.config.max_retries {
        match self.client.delete(url).json(body).send().await {
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
                return Err(Error::HttpStatusError(response.status().as_u16()));
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

#### 1.2 Blocking DELETE with Empty Response

```rust
/// Execute blocking HTTP DELETE request with retry logic (no response body)
///
/// For endpoints that return 200 OK with empty body.
///
/// # Type Parameters
///
/// * `R` - Request type that implements `Serialize`
///
/// # Arguments
///
/// * `url` - Full URL to request
/// * `body` - Request body to serialize as JSON
///
/// # Errors
///
/// Returns an error if:
/// - Maximum retry attempts exceeded
/// - Client errors (4xx) occur (no retry)
pub(super) fn delete_empty_blocking_with_retry<R>(&self, url: &str, body: &R) -> Result<()>
where
    R: serde::Serialize,
{
    let blocking_client = reqwest::blocking::Client::builder()
        .timeout(self.config.timeout)
        .build()?;

    for attempt in 0..=self.config.max_retries {
        match blocking_client.delete(url).json(body).send() {
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
                return Err(Error::HttpStatusError(response.status().as_u16()));
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

### Step 2: Create DeleteRequest Primitive Type

**Location:** `src/primitives/delete_request.rs`

```rust
//! Delete model request primitive type

use serde::{Deserialize, Serialize};

/// Request body for DELETE /api/delete endpoint
///
/// Deletes an existing model from the Ollama server.
///
/// # Example
///
/// ```
/// use ollama_oxide::DeleteRequest;
///
/// let request = DeleteRequest {
///     model: "llama3.1".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteRequest {
    /// Name of the model to delete
    pub model: String,
}

impl DeleteRequest {
    /// Create a new delete request
    ///
    /// # Arguments
    ///
    /// * `model` - Name of the model to delete
    ///
    /// # Example
    ///
    /// ```
    /// use ollama_oxide::DeleteRequest;
    ///
    /// let request = DeleteRequest::new("llama3.1");
    /// ```
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_request_serialization() {
        let request = DeleteRequest {
            model: "gemma3".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"model\":\"gemma3\""));
    }

    #[test]
    fn test_delete_request_deserialization() {
        let json = r#"{"model": "llama3"}"#;
        let request: DeleteRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.model, "llama3");
    }

    #[test]
    fn test_delete_request_new() {
        let request = DeleteRequest::new("model-a");
        assert_eq!(request.model, "model-a");
    }

    #[test]
    fn test_delete_request_roundtrip() {
        let request = DeleteRequest::new("source-model");
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: DeleteRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request, deserialized);
    }

    #[test]
    fn test_delete_request_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DeleteRequest>();
    }
}
```

### Step 3: Update primitives/mod.rs

**Location:** `src/primitives/mod.rs`

Add module declaration and re-export:

```rust
mod delete_request;

pub use delete_request::DeleteRequest;
```

### Step 4: Update lib.rs Re-exports

**Location:** `src/lib.rs`

Add `DeleteRequest` to public re-exports:

```rust
pub use primitives::{
    CopyRequest, DeleteRequest, ListResponse, ModelDetails, ModelSummary, PsResponse, RunningModel, VersionResponse,
};
```

### Step 5: Add API Methods

#### 5.1 Async API

**Location:** `src/http/api_async.rs`

Add to `OllamaApiAsync` trait:

```rust
/// Delete a model (async)
///
/// Permanently removes a model from the Ollama server. This operation
/// cannot be undone.
///
/// # Arguments
///
/// * `request` - Delete request containing the model name to delete
///
/// # Errors
///
/// Returns an error if:
/// - Model doesn't exist (404)
/// - Network request fails
/// - Maximum retry attempts exceeded
///
/// # Examples
///
/// ```no_run
/// use ollama_oxide::{OllamaClient, OllamaApiAsync, DeleteRequest};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = OllamaClient::default()?;
/// let request = DeleteRequest::new("llama3.1-backup");
/// client.delete_model(&request).await?;
/// println!("Model deleted successfully!");
/// # Ok(())
/// # }
/// ```
async fn delete_model(&self, request: &DeleteRequest) -> Result<()>;
```

Add implementation:

```rust
async fn delete_model(&self, request: &DeleteRequest) -> Result<()> {
    let url = self.config.url(Endpoints::DELETE);
    self.delete_empty_with_retry(&url, request).await
}
```

#### 5.2 Sync API

**Location:** `src/http/api_sync.rs`

Add to `OllamaApiSync` trait:

```rust
/// Delete a model (blocking)
///
/// Permanently removes a model from the Ollama server. This operation
/// cannot be undone.
/// This method blocks the current thread until the request completes.
///
/// # Arguments
///
/// * `request` - Delete request containing the model name to delete
///
/// # Errors
///
/// Returns an error if:
/// - Model doesn't exist (404)
/// - Network request fails
/// - Maximum retry attempts exceeded
///
/// # Examples
///
/// ```no_run
/// use ollama_oxide::{OllamaClient, OllamaApiSync, DeleteRequest};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = OllamaClient::default()?;
/// let request = DeleteRequest::new("llama3.1-backup");
/// client.delete_model_blocking(&request)?;
/// println!("Model deleted successfully!");
/// # Ok(())
/// # }
/// ```
fn delete_model_blocking(&self, request: &DeleteRequest) -> Result<()>;
```

Add implementation:

```rust
fn delete_model_blocking(&self, request: &DeleteRequest) -> Result<()> {
    let url = self.config.url(Endpoints::DELETE);
    self.delete_empty_blocking_with_retry(&url, request)
}
```

### Step 6: Add Unit Tests

**Location:** `tests/client_delete_model_tests.rs`

```rust
//! Tests for delete_model API methods (DELETE /api/delete)

use ollama_oxide::{ClientConfig, DeleteRequest, OllamaApiAsync, OllamaApiSync, OllamaClient};
use std::time::Duration;

// ============================================================================
// Async API Tests
// ============================================================================

#[tokio::test]
async fn test_delete_model_async_success() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("DELETE", "/api/delete")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "model": "llama3.1-backup"
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
    let request = DeleteRequest::new("llama3.1-backup");
    let result = client.delete_model(&request).await;

    assert!(result.is_ok());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_delete_model_async_model_not_found() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("DELETE", "/api/delete")
        .with_status(404)
        .create_async()
        .await;

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 0,
    };

    let client = OllamaClient::new(config).unwrap();
    let request = DeleteRequest::new("nonexistent");
    let result = client.delete_model(&request).await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_delete_model_async_retry_on_server_error() {
    let mut server = mockito::Server::new_async().await;

    let mock_fail = server
        .mock("DELETE", "/api/delete")
        .with_status(500)
        .expect(1)
        .create_async()
        .await;

    let mock_success = server
        .mock("DELETE", "/api/delete")
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
    let request = DeleteRequest::new("model");
    let result = client.delete_model(&request).await;

    assert!(result.is_ok());
    mock_fail.assert_async().await;
    mock_success.assert_async().await;
}

// ============================================================================
// Sync API Tests
// ============================================================================

#[test]
fn test_delete_model_sync_success() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("DELETE", "/api/delete")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "model": "gemma3-backup"
        })))
        .with_status(200)
        .create();

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 0,
    };

    let client = OllamaClient::new(config).unwrap();
    let request = DeleteRequest::new("gemma3-backup");
    let result = client.delete_model_blocking(&request);

    assert!(result.is_ok());
    mock.assert();
}

#[test]
fn test_delete_model_sync_model_not_found() {
    let mut server = mockito::Server::new();

    let mock = server.mock("DELETE", "/api/delete").with_status(404).create();

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 0,
    };

    let client = OllamaClient::new(config).unwrap();
    let request = DeleteRequest::new("missing");
    let result = client.delete_model_blocking(&request);

    assert!(result.is_err());
    mock.assert();
}

#[test]
fn test_delete_model_sync_retry_on_server_error() {
    let mut server = mockito::Server::new();

    let mock_fail = server
        .mock("DELETE", "/api/delete")
        .with_status(500)
        .expect(1)
        .create();

    let mock_success = server
        .mock("DELETE", "/api/delete")
        .with_status(200)
        .expect(1)
        .create();

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 1,
    };

    let client = OllamaClient::new(config).unwrap();
    let request = DeleteRequest::new("model");
    let result = client.delete_model_blocking(&request);

    assert!(result.is_ok());
    mock_fail.assert();
    mock_success.assert();
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_delete_model_async_max_retries_exceeded() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("DELETE", "/api/delete")
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
    let request = DeleteRequest::new("model");
    let result = client.delete_model(&request).await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[test]
fn test_delete_model_sync_max_retries_exceeded() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("DELETE", "/api/delete")
        .with_status(500)
        .expect(3)
        .create();

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(1),
        max_retries: 2,
    };

    let client = OllamaClient::new(config).unwrap();
    let request = DeleteRequest::new("model");
    let result = client.delete_model_blocking(&request);

    assert!(result.is_err());
    mock.assert();
}

// ============================================================================
// DeleteRequest Type Tests
// ============================================================================

#[test]
fn test_delete_request_debug_impl() {
    let request = DeleteRequest::new("test-model");
    let debug_str = format!("{:?}", request);
    assert!(debug_str.contains("test-model"));
}

#[test]
fn test_delete_request_clone_impl() {
    let request = DeleteRequest::new("original");
    let cloned = request.clone();
    assert_eq!(request, cloned);
}
```

### Step 7: Add Examples

**Location:** `examples/delete_model_async.rs`

```rust
//! Example: Delete a model (async)
//!
//! This example demonstrates how to delete a model from the Ollama server.
//!
//! Run with: cargo run --example delete_model_async
//!
//! Note: Requires a running Ollama server.
//! WARNING: This will permanently delete the specified model!

use ollama_oxide::{DeleteRequest, OllamaApiAsync, OllamaClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with default configuration
    let client = OllamaClient::default()?;

    // Model to delete (change this to a model you want to delete)
    let model_name = "llama3.1-backup";

    println!("WARNING: This will permanently delete '{}'", model_name);
    println!("Deleting model...");

    // Create delete request
    let request = DeleteRequest::new(model_name);

    // Execute delete
    match client.delete_model(&request).await {
        Ok(()) => {
            println!("Model '{}' deleted successfully!", model_name);
        }
        Err(e) => {
            eprintln!("Failed to delete model: {}", e);
            eprintln!("The model '{}' may not exist.", model_name);
        }
    }

    Ok(())
}
```

**Location:** `examples/delete_model_sync.rs`

```rust
//! Example: Delete a model (sync)
//!
//! This example demonstrates how to delete a model from the Ollama server
//! using the blocking API.
//!
//! Run with: cargo run --example delete_model_sync
//!
//! Note: Requires a running Ollama server.
//! WARNING: This will permanently delete the specified model!

use ollama_oxide::{DeleteRequest, OllamaApiSync, OllamaClient};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with default configuration
    let client = OllamaClient::default()?;

    // Model to delete (change this to a model you want to delete)
    let model_name = "llama3.1-backup";

    println!("WARNING: This will permanently delete '{}'", model_name);
    println!("Deleting model...");

    // Create delete request
    let request = DeleteRequest::new(model_name);

    // Execute delete
    match client.delete_model_blocking(&request) {
        Ok(()) => {
            println!("Model '{}' deleted successfully!", model_name);
        }
        Err(e) => {
            eprintln!("Failed to delete model: {}", e);
            eprintln!("The model '{}' may not exist.", model_name);
        }
    }

    Ok(())
}
```

## File Changes Summary

### New Files
| File | Description |
|------|-------------|
| `src/primitives/delete_request.rs` | DeleteRequest struct |
| `tests/client_delete_model_tests.rs` | Unit tests with mocking |
| `examples/delete_model_async.rs` | Async usage example |
| `examples/delete_model_sync.rs` | Sync usage example |

### Modified Files
| File | Changes |
|------|---------|
| `src/http/client.rs` | Add `delete_empty_with_retry` and `delete_empty_blocking_with_retry` methods |
| `src/primitives/mod.rs` | Add module declaration and re-export |
| `src/lib.rs` | Add `DeleteRequest` to public re-exports |
| `src/http/api_async.rs` | Add `delete_model()` method to trait and implementation |
| `src/http/api_sync.rs` | Add `delete_model_blocking()` method to trait and implementation |

## Testing Checklist

- [ ] Unit tests for DeleteRequest pass (in `src/primitives/delete_request.rs`)
- [ ] DeleteRequest Send + Sync test passes
- [ ] DELETE helper methods work correctly
- [ ] Async delete_model tests pass
- [ ] Sync delete_model_blocking tests pass
- [ ] Retry logic tests pass
- [ ] Error handling tests pass (404 not found, max retries exceeded)
- [ ] Examples build successfully
- [ ] `cargo build` succeeds
- [ ] `cargo test` passes
- [ ] `cargo clippy` has no warnings
- [ ] `cargo fmt` applied

## Implementation Order

1. Add DELETE helper methods to `src/http/client.rs`
2. Create `src/primitives/delete_request.rs` with struct and tests
3. Update `src/primitives/mod.rs` with new export
4. Update `src/lib.rs` with public re-export
5. Add async method to `src/http/api_async.rs` (trait + impl)
6. Add sync method to `src/http/api_sync.rs` (trait + impl)
7. Create unit tests in `tests/client_delete_model_tests.rs`
8. Create async example
9. Create sync example
10. Run full test suite (`cargo test`)
11. Run linter (`cargo clippy`)
12. Apply formatting (`cargo fmt`)
13. Update DEV_NOTES.md if needed
14. Update definition.md checklist

## Comparison with POST /api/copy

| Aspect | POST /api/copy | DELETE /api/delete |
|--------|----------------|-------------------|
| HTTP Method | POST | DELETE |
| Request Type | CopyRequest (source, destination) | DeleteRequest (model) |
| Response | 200 OK (empty) | 200 OK (empty) |
| Helper Method | post_empty_with_retry | delete_empty_with_retry |
| Error on 404 | Source not found | Model not found |

## Notes

- The `Endpoints::DELETE` constant already exists in `endpoints.rs`
- This follows the same pattern as POST /api/copy
- The DELETE HTTP method with a JSON body is unusual but matches Ollama's API spec
- All types must implement `Send + Sync` for thread safety
- Error handling for 4xx errors (like 404 model not found) should not retry
- This is a destructive operation - examples include warnings
- **Examples use "llama3.1-backup"** - This model name is consistent with the copy_model examples, which create a backup copy. This allows testing delete without downloading a new model (users can first run copy_model_async/sync to create the backup, then run delete_model_async/sync to remove it)
