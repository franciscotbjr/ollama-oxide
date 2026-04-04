# Implementation Plan: POST /api/push (Model Upload)

**Document Version:** 1.0
**Created:** 2026-02-04
**Endpoint:** POST /api/push
**Feature:** `model`
**Mode:** Non-streaming only (v0.1.0)

## Overview

This document describes the implementation plan for the `POST /api/push` endpoint, which uploads/pushes models to a remote Ollama registry. This is the final endpoint needed to complete Phase 1 (v0.1.0).

The implementation follows the established patterns from `POST /api/pull`, as both endpoints share nearly identical request/response structures.

## API Specification

### Request Schema

```rust
struct PushRequest {
    model: String,              // Required: model name (e.g., "namespace/model:tag")
    insecure: Option<bool>,     // Optional: allow insecure connections
    stream: Option<bool>,       // Optional: stream progress (default: true in Ollama)
}
```

### Response Schema (Non-Streaming)

```rust
struct PushResponse {
    status: Option<String>,     // Status message (e.g., "success")
}
```

### HTTP Details

- **Method:** POST
- **Path:** `/api/push`
- **Content-Type:** application/json
- **Success Response:** 200 OK
- **Error Responses:** 404 (model not found), 401 (unauthorized), 500 (server error)

## Implementation Tasks

### 1. Types (src/model/)

#### 1.1 Create `src/model/push_request.rs`

```rust
use serde::{Deserialize, Serialize};

/// Request body for POST /api/push endpoint.
///
/// Uploads a model to a remote Ollama registry.
///
/// # JSON Examples
///
/// Minimal request:
/// ```json
/// {
///   "model": "namespace/model:tag"
/// }
/// ```
///
/// Full request with options:
/// ```json
/// {
///   "model": "namespace/model:tag",
///   "insecure": false,
///   "stream": false
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PushRequest {
    /// Name of the model to push (e.g., "namespace/model:tag")
    pub model: String,

    /// Allow uploading over insecure connections (without TLS verification)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insecure: Option<bool>,

    /// Stream progress updates. Default: true in Ollama, but we set false for v0.1.0
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

impl PushRequest {
    /// Create a new push request for the specified model.
    ///
    /// The request is configured with `stream: false` for non-streaming mode.
    ///
    /// # Arguments
    ///
    /// * `model` - Name of the model to push (e.g., "namespace/model:tag")
    ///
    /// # Example
    ///
    /// ```
    /// use ollama_oxide::PushRequest;
    ///
    /// let request = PushRequest::new("myuser/mymodel:latest");
    /// ```
    pub fn new<M: Into<String>>(model: M) -> Self {
        Self {
            model: model.into(),
            insecure: None,
            stream: Some(false), // v0.1.0: non-streaming only
        }
    }

    /// Allow uploading over insecure connections.
    ///
    /// When set to `true`, the upload will proceed without TLS verification.
    /// Use with caution, only in trusted network environments.
    ///
    /// # Arguments
    ///
    /// * `insecure` - Whether to allow insecure connections
    ///
    /// # Example
    ///
    /// ```
    /// use ollama_oxide::PushRequest;
    ///
    /// let request = PushRequest::new("myuser/mymodel:latest")
    ///     .with_insecure(true);
    /// ```
    pub fn with_insecure(mut self, insecure: bool) -> Self {
        self.insecure = Some(insecure);
        self
    }
}
```

#### 1.2 Create `src/model/push_response.rs`

```rust
use serde::{Deserialize, Serialize};

/// Response from POST /api/push endpoint.
///
/// Contains the status of the push operation.
///
/// # JSON Example
///
/// ```json
/// {
///   "status": "success"
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PushResponse {
    /// Status message indicating the result of the operation
    #[serde(default)]
    pub status: Option<String>,
}

impl PushResponse {
    /// Get the status message.
    ///
    /// # Returns
    ///
    /// The status string if present, or None.
    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    /// Check if the push operation was successful.
    ///
    /// # Returns
    ///
    /// `true` if status is "success", `false` otherwise.
    pub fn is_success(&self) -> bool {
        self.status.as_deref() == Some("success")
    }
}
```

### 2. Module Exports

#### 2.1 Update `src/model/mod.rs`

Add the following lines:

```rust
mod push_request;
mod push_response;

pub use push_request::PushRequest;
pub use push_response::PushResponse;
```

#### 2.2 Update `src/lib.rs`

Add `PushRequest` and `PushResponse` to the `model` feature exports:

```rust
#[cfg(feature = "model")]
pub use model::{
    // ... existing exports ...
    PushRequest, PushResponse,
};
```

### 3. API Trait Methods

#### 3.1 Update `src/http/api_async.rs`

Add import:
```rust
#[cfg(feature = "model")]
use crate::{
    // ... existing imports ...
    PushRequest, PushResponse,
};
```

Add trait method:
```rust
/// Push (upload) a model to the Ollama registry.
///
/// Uploads the specified model to a remote registry. Requires proper
/// authentication and namespace permissions.
///
/// # Arguments
///
/// * `request` - The push request containing the model name and options
///
/// # Returns
///
/// A `PushResponse` indicating the success or failure of the operation.
///
/// # Errors
///
/// * `HttpStatusError(404)` - Model not found locally
/// * `HttpStatusError(401)` - Unauthorized (invalid credentials)
/// * `HttpError` - Network or HTTP errors
/// * `MaxRetriesExceededError` - Server errors after all retries
///
/// # Examples
///
/// ```no_run
/// use ollama_oxide::{OllamaClient, OllamaApiAsync, PushRequest};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = OllamaClient::default()?;
/// let request = PushRequest::new("myuser/mymodel:latest");
/// let response = client.push_model(&request).await?;
/// println!("Status: {:?}", response.status());
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "model")]
async fn push_model(&self, request: &PushRequest) -> Result<PushResponse>;
```

Add implementation:
```rust
#[cfg(feature = "model")]
async fn push_model(&self, request: &PushRequest) -> Result<PushResponse> {
    let url = self.config.url(Endpoints::PUSH);
    self.post_with_retry(&url, request).await
}
```

#### 3.2 Update `src/http/api_sync.rs`

Add import:
```rust
#[cfg(feature = "model")]
use crate::{
    // ... existing imports ...
    PushRequest, PushResponse,
};
```

Add trait method:
```rust
/// Push (upload) a model to the Ollama registry (blocking).
///
/// Uploads the specified model to a remote registry. Requires proper
/// authentication and namespace permissions.
///
/// # Arguments
///
/// * `request` - The push request containing the model name and options
///
/// # Returns
///
/// A `PushResponse` indicating the success or failure of the operation.
///
/// # Errors
///
/// * `HttpStatusError(404)` - Model not found locally
/// * `HttpStatusError(401)` - Unauthorized (invalid credentials)
/// * `HttpError` - Network or HTTP errors
/// * `MaxRetriesExceededError` - Server errors after all retries
///
/// # Examples
///
/// ```no_run
/// use ollama_oxide::{OllamaClient, OllamaApiSync, PushRequest};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = OllamaClient::default()?;
/// let request = PushRequest::new("myuser/mymodel:latest");
/// let response = client.push_model_blocking(&request)?;
/// println!("Status: {:?}", response.status());
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "model")]
fn push_model_blocking(&self, request: &PushRequest) -> Result<PushResponse>;
```

Add implementation:
```rust
#[cfg(feature = "model")]
fn push_model_blocking(&self, request: &PushRequest) -> Result<PushResponse> {
    let url = self.config.url(Endpoints::PUSH);
    self.post_blocking_with_retry(&url, request)
}
```

### 4. Unit Tests

#### 4.1 Create `tests/client_push_tests.rs`

```rust
//! Tests for POST /api/push endpoint (push_model, push_model_blocking)

use mockito::{Matcher, Server};
use ollama_oxide::{ClientConfig, OllamaApiAsync, OllamaApiSync, OllamaClient, PushRequest};
use std::time::Duration;

fn make_config(base_url: String) -> ClientConfig {
    ClientConfig {
        base_url,
        timeout: Duration::from_secs(30),
        max_retries: 3,
    }
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
```

### 5. Examples

#### 5.1 Create `examples/push_model_async.rs`

```rust
//! Example: Push (upload) a model to Ollama registry (async)
//!
//! This example demonstrates how to push a local model to a remote
//! Ollama registry using the async API.
//!
//! # Prerequisites
//!
//! - Ollama server running at http://localhost:11434
//! - A local model that you want to push
//! - Proper authentication configured for the registry
//!
//! # Running
//!
//! ```bash
//! cargo run --example push_model_async --features model
//! ```
//!
//! # Note
//!
//! Pushing models requires:
//! 1. The model to exist locally
//! 2. Proper namespace permissions (e.g., "myuser/mymodel")
//! 3. Registry authentication (typically via `ollama login`)

use ollama_oxide::{OllamaApiAsync, OllamaClient, PushRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with default configuration
    let client = OllamaClient::default()?;

    // Specify the model to push
    // Format: "namespace/model:tag" (e.g., "myuser/mymodel:latest")
    let model_name = "myuser/mymodel:latest";

    println!("Pushing model: {}", model_name);

    // Create the push request
    let request = PushRequest::new(model_name);

    // Push the model (non-streaming in v0.1.0)
    match client.push_model(&request).await {
        Ok(response) => {
            println!("Push completed!");
            println!("Status: {:?}", response.status());
            if response.is_success() {
                println!("Model successfully pushed to registry!");
            }
        }
        Err(e) => {
            eprintln!("Failed to push model: {}", e);
            eprintln!("Make sure:");
            eprintln!("  1. The model exists locally");
            eprintln!("  2. You have proper namespace permissions");
            eprintln!("  3. You are authenticated (run 'ollama login' if needed)");
        }
    }

    Ok(())
}
```

#### 5.2 Create `examples/push_model_sync.rs`

```rust
//! Example: Push (upload) a model to Ollama registry (sync/blocking)
//!
//! This example demonstrates how to push a local model to a remote
//! Ollama registry using the blocking API.
//!
//! # Prerequisites
//!
//! - Ollama server running at http://localhost:11434
//! - A local model that you want to push
//! - Proper authentication configured for the registry
//!
//! # Running
//!
//! ```bash
//! cargo run --example push_model_sync --features model
//! ```
//!
//! # Note
//!
//! Pushing models requires:
//! 1. The model to exist locally
//! 2. Proper namespace permissions (e.g., "myuser/mymodel")
//! 3. Registry authentication (typically via `ollama login`)

use ollama_oxide::{OllamaApiSync, OllamaClient, PushRequest};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with default configuration
    let client = OllamaClient::default()?;

    // Specify the model to push
    // Format: "namespace/model:tag" (e.g., "myuser/mymodel:latest")
    let model_name = "myuser/mymodel:latest";

    println!("Pushing model: {}", model_name);

    // Create the push request
    let request = PushRequest::new(model_name);

    // Push the model (non-streaming in v0.1.0)
    match client.push_model_blocking(&request) {
        Ok(response) => {
            println!("Push completed!");
            println!("Status: {:?}", response.status());
            if response.is_success() {
                println!("Model successfully pushed to registry!");
            }
        }
        Err(e) => {
            eprintln!("Failed to push model: {}", e);
            eprintln!("Make sure:");
            eprintln!("  1. The model exists locally");
            eprintln!("  2. You have proper namespace permissions");
            eprintln!("  3. You are authenticated (run 'ollama login' if needed)");
        }
    }

    Ok(())
}
```

### 6. Cargo.toml Updates

Add the following entries:

```toml
[[example]]
name = "push_model_async"
required-features = ["model"]

[[example]]
name = "push_model_sync"
required-features = ["model"]

[[test]]
name = "client_push_tests"
required-features = ["model"]
```

### 7. Documentation Updates

#### 7.1 Update `spec/definition.md`

Mark the endpoint as complete:
```markdown
- [x] `POST /api/push` - Model upload (non-streaming only)
```

#### 7.2 Update `CHANGELOG.md`

Add to the `[Unreleased]` section:
```markdown
- **POST /api/push endpoint**: Upload models to registry (non-streaming)
  - `PushRequest` type with builder pattern (model, insecure option)
  - `PushResponse` type with helper methods (`status()`, `is_success()`)
  - `push_model()` async method
  - `push_model_blocking()` sync method
  - 10 new unit tests with mocking
  - Examples: `push_model_async.rs`, `push_model_sync.rs`
```

Update the "Planned for v0.1.0" section to remove the push entry.

## File Summary

| File | Action | Description |
|------|--------|-------------|
| `src/model/push_request.rs` | Create | PushRequest struct with builder |
| `src/model/push_response.rs` | Create | PushResponse struct with helpers |
| `src/model/mod.rs` | Update | Add module exports |
| `src/lib.rs` | Update | Add public exports |
| `src/http/api_async.rs` | Update | Add push_model() method |
| `src/http/api_sync.rs` | Update | Add push_model_blocking() method |
| `tests/client_push_tests.rs` | Create | 10 unit tests |
| `examples/push_model_async.rs` | Create | Async example |
| `examples/push_model_sync.rs` | Create | Sync example |
| `Cargo.toml` | Update | Add example and test entries |
| `spec/definition.md` | Update | Mark endpoint complete |
| `CHANGELOG.md` | Update | Document changes |

## Implementation Order

1. Create type files (`push_request.rs`, `push_response.rs`)
2. Update module exports (`mod.rs`, `lib.rs`)
3. Add API trait methods (`api_async.rs`, `api_sync.rs`)
4. Create unit tests (`client_push_tests.rs`)
5. Update `Cargo.toml`
6. Run `cargo test --all-features` to verify
7. Create examples (`push_model_async.rs`, `push_model_sync.rs`)
8. Update documentation (`definition.md`, `CHANGELOG.md`)
9. Run `cargo clippy --all-features` and `cargo fmt`
10. Final verification with `cargo build --all-features`

## Verification Checklist

- [ ] `cargo check --all-features` passes
- [ ] `cargo test --all-features` passes (all tests including new ones)
- [ ] `cargo clippy --all-features` has no warnings
- [ ] `cargo fmt -- --check` passes
- [ ] `cargo doc --all-features` generates documentation
- [ ] Examples compile: `cargo build --example push_model_async --features model`
- [ ] Examples compile: `cargo build --example push_model_sync --features model`

## Notes

1. **Similar to Pull**: This implementation mirrors `/api/pull` closely, with the main difference being the upload direction.

2. **Authentication**: Pushing models typically requires authentication. Users need to run `ollama login` before pushing. Error handling should provide clear guidance.

3. **Non-Streaming**: v0.1.0 only supports non-streaming mode. The `stream` field is hardcoded to `false`. Streaming support will be added in v0.2.0.

4. **Model Naming**: Models pushed to a registry require a namespace prefix (e.g., `username/modelname:tag`).

## Completion Criteria

This implementation completes Phase 1 (v0.1.0) of the ollama-oxide project:

- All 12 API endpoints implemented (non-streaming mode)
- GET endpoints: `/api/version`, `/api/tags`, `/api/ps`
- POST endpoints: `/api/copy`, `/api/show`, `/api/embed`, `/api/generate`, `/api/chat`, `/api/create`, `/api/pull`, `/api/push`
- DELETE endpoint: `/api/delete`

After this implementation, the project will be ready for:
- v0.1.0 release preparation
- Phase 2 (v0.2.0): Streaming implementation
