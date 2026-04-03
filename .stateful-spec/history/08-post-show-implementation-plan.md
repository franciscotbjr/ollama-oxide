# Implementation Plan: POST /api/show

**Endpoint:** POST /api/show
**Complexity:** Medium (POST with JSON response body)
**Phase:** Phase 1 - Foundation + All Endpoints (Non-Streaming Mode)
**Document Version:** 1.0
**Created:** 2026-01-17

## Overview

This document outlines the implementation plan for the `POST /api/show` endpoint, which retrieves detailed information about a specific model.

This is the **first POST endpoint with a JSON response body** in the library. Previous POST endpoints (`/api/copy`, `/api/delete`) returned empty responses. This requires:
- A new `post_with_retry<R, T>` helper method in `client.rs` that accepts a request body and returns a deserialized response
- A request type (`ShowRequest`)
- A response type (`ShowResponse`) with nested structures
- Handling of the `verbose` optional parameter

## API Specification Summary

**Endpoint:** `POST /api/show`
**Operation ID:** `show`
**Description:** Get detailed information about a specific model

**Request Body:**
```json
{
  "model": "gemma3",
  "verbose": true
}
```

**Response:**
```json
{
  "parameters": "temperature 0.7\nnum_ctx 2048",
  "license": "Gemma Terms of Use...",
  "modified_at": "2025-08-14T15:49:43.634137516-07:00",
  "details": {
    "parent_model": "",
    "format": "gguf",
    "family": "gemma3",
    "families": ["gemma3"],
    "parameter_size": "4.3B",
    "quantization_level": "Q4_K_M"
  },
  "template": "...",
  "capabilities": ["completion", "vision"],
  "model_info": {
    "gemma3.attention.head_count": 8,
    "general.architecture": "gemma3",
    ...
  }
}
```

## Schema Analysis

### ShowRequest (New Type)

```rust
/// Request body for POST /api/show endpoint
pub struct ShowRequest {
    /// Name of the model to show
    pub model: String,
    /// If true, includes verbose fields in the response
    pub verbose: Option<bool>,
}
```

This is a simple struct with one required field and one optional field.

### ShowResponse (New Type)

```rust
/// Response from POST /api/show endpoint
pub struct ShowResponse {
    /// Model parameter settings serialized as text
    pub parameters: Option<String>,
    /// The license of the model
    pub license: Option<String>,
    /// Last modified timestamp in ISO 8601 format
    pub modified_at: Option<String>,
    /// High-level model details
    pub details: Option<ShowModelDetails>,
    /// The template used by the model to render prompts
    pub template: Option<String>,
    /// List of supported features (e.g., "completion", "vision")
    pub capabilities: Option<Vec<String>>,
    /// Additional model metadata (flexible key-value)
    pub model_info: Option<serde_json::Value>,
}
```

**Note on `model_info`:** The OpenAPI spec defines this as a flexible object with arbitrary keys. Using `serde_json::Value` allows deserialization of any JSON structure without strict typing.

### ShowModelDetails (New Type)

This is different from the existing `ModelDetails` type used in `GET /api/tags`. The `/api/show` response has a more detailed structure:

```rust
/// Model details from POST /api/show endpoint
pub struct ShowModelDetails {
    /// Parent model name (empty if base model)
    pub parent_model: Option<String>,
    /// Model format (e.g., "gguf")
    pub format: Option<String>,
    /// Model family (e.g., "gemma3", "llama")
    pub family: Option<String>,
    /// List of model families
    pub families: Option<Vec<String>>,
    /// Parameter size (e.g., "4.3B", "7B")
    pub parameter_size: Option<String>,
    /// Quantization level (e.g., "Q4_K_M")
    pub quantization_level: Option<String>,
}
```

**Comparison with existing `ModelDetails`:**
- Existing `ModelDetails` (from `/api/tags`): `format`, `family`, `families`, `parameter_size`, `quantization_level`
- New `ShowModelDetails` (from `/api/show`): Same fields + `parent_model`

**Decision:** Create a new `ShowModelDetails` type to match the exact API response. The `parent_model` field is specific to `/api/show`.

## Implementation Strategy

### Step 1: Add POST with Response Helper Methods to client.rs

We need POST helpers that handle both request body and response deserialization.

**Location:** `src/http/client.rs`

#### 1.1 Async POST with Response

```rust
/// Execute async HTTP POST request with retry logic (with JSON response)
///
/// For endpoints that accept a request body and return a JSON response.
///
/// # Type Parameters
///
/// * `R` - Request type that implements `Serialize`
/// * `T` - Response type that implements `DeserializeOwned`
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
/// - Response cannot be deserialized
/// - Client errors (4xx) occur (no retry)
pub(super) async fn post_with_retry<R, T>(&self, url: &str, body: &R) -> Result<T>
where
    R: serde::Serialize,
    T: serde::de::DeserializeOwned,
{
    for attempt in 0..=self.config.max_retries {
        match self.client.post(url).json(body).send().await {
            Ok(response) => {
                // Retry on server errors (5xx)
                if response.status().is_server_error() && attempt < self.config.max_retries {
                    tokio::time::sleep(Duration::from_millis(100 * (attempt as u64 + 1))).await;
                    continue;
                }

                // Check for client errors (no retry)
                if response.status().is_client_error() {
                    return Err(Error::HttpStatusError(response.status().as_u16()));
                }

                // Deserialize and return
                let result = response.json::<T>().await?;
                return Ok(result);
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

#### 1.2 Blocking POST with Response

```rust
/// Execute blocking HTTP POST request with retry logic (with JSON response)
///
/// For endpoints that accept a request body and return a JSON response.
///
/// # Type Parameters
///
/// * `R` - Request type that implements `Serialize`
/// * `T` - Response type that implements `DeserializeOwned`
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
/// - Response cannot be deserialized
/// - Client errors (4xx) occur (no retry)
pub(super) fn post_blocking_with_retry<R, T>(&self, url: &str, body: &R) -> Result<T>
where
    R: serde::Serialize,
    T: serde::de::DeserializeOwned,
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

                // Check for client errors (no retry)
                if response.status().is_client_error() {
                    return Err(Error::HttpStatusError(response.status().as_u16()));
                }

                // Deserialize and return
                let result = response.json::<T>()?;
                return Ok(result);
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

### Step 2: Create ShowRequest Primitive Type

**Location:** `src/primitives/show_request.rs`

```rust
//! Show model request primitive type

use serde::{Deserialize, Serialize};

/// Request body for POST /api/show endpoint
///
/// Retrieves detailed information about a specific model.
///
/// # Example
///
/// ```
/// use ollama_oxide::ShowRequest;
///
/// // Basic request
/// let request = ShowRequest {
///     model: "llama3.1".to_string(),
///     verbose: None,
/// };
///
/// // Verbose request for more details
/// let verbose_request = ShowRequest {
///     model: "llama3.1".to_string(),
///     verbose: Some(true),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShowRequest {
    /// Name of the model to show information for
    pub model: String,

    /// If true, includes large verbose fields in the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbose: Option<bool>,
}

impl ShowRequest {
    /// Create a new ShowRequest for the specified model
    ///
    /// # Arguments
    ///
    /// * `model` - Name of the model to query
    ///
    /// # Example
    ///
    /// ```
    /// use ollama_oxide::ShowRequest;
    ///
    /// let request = ShowRequest::new("llama3.1");
    /// ```
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            verbose: None,
        }
    }

    /// Create a verbose ShowRequest for detailed model information
    ///
    /// # Arguments
    ///
    /// * `model` - Name of the model to query
    ///
    /// # Example
    ///
    /// ```
    /// use ollama_oxide::ShowRequest;
    ///
    /// let request = ShowRequest::verbose("llama3.1");
    /// assert_eq!(request.verbose, Some(true));
    /// ```
    pub fn verbose(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            verbose: Some(true),
        }
    }
}
```

### Step 3: Create ShowModelDetails Primitive Type

**Location:** `src/primitives/show_model_details.rs`

```rust
//! Show model details primitive type

use serde::{Deserialize, Serialize};

/// Model details returned by POST /api/show endpoint
///
/// Contains high-level information about the model's format,
/// family, and quantization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ShowModelDetails {
    /// Parent model name (empty string if this is a base model)
    #[serde(default)]
    pub parent_model: Option<String>,

    /// Model format (e.g., "gguf")
    #[serde(default)]
    pub format: Option<String>,

    /// Model family (e.g., "gemma3", "llama")
    #[serde(default)]
    pub family: Option<String>,

    /// List of model families this model belongs to
    #[serde(default)]
    pub families: Option<Vec<String>>,

    /// Parameter size (e.g., "4.3B", "7B", "13B")
    #[serde(default)]
    pub parameter_size: Option<String>,

    /// Quantization level (e.g., "Q4_K_M", "Q8_0")
    #[serde(default)]
    pub quantization_level: Option<String>,
}
```

### Step 4: Create ShowResponse Primitive Type

**Location:** `src/primitives/show_response.rs`

```rust
//! Show model response primitive type

use serde::{Deserialize, Serialize};

use super::ShowModelDetails;

/// Response from POST /api/show endpoint
///
/// Contains comprehensive information about a model including
/// parameters, license, capabilities, and detailed metadata.
///
/// # Example
///
/// ```no_run
/// use ollama_oxide::{OllamaClient, ShowRequest};
///
/// # async fn example() -> Result<(), ollama_oxide::Error> {
/// let client = OllamaClient::default()?;
/// let request = ShowRequest::new("llama3.1");
/// let response = client.show_model(&request).await?;
///
/// println!("Model capabilities: {:?}", response.capabilities);
/// println!("License: {:?}", response.license);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ShowResponse {
    /// Model parameter settings serialized as text
    ///
    /// Contains configuration like temperature, num_ctx, etc.
    #[serde(default)]
    pub parameters: Option<String>,

    /// The license of the model
    #[serde(default)]
    pub license: Option<String>,

    /// Last modified timestamp in ISO 8601 format
    #[serde(default)]
    pub modified_at: Option<String>,

    /// High-level model details
    #[serde(default)]
    pub details: Option<ShowModelDetails>,

    /// The template used by the model to render prompts
    #[serde(default)]
    pub template: Option<String>,

    /// List of supported features (e.g., "completion", "vision")
    #[serde(default)]
    pub capabilities: Option<Vec<String>>,

    /// Additional model metadata
    ///
    /// This is a flexible key-value structure that contains
    /// model-specific information like attention head counts,
    /// context length, embedding dimensions, etc.
    ///
    /// Use `serde_json::Value` to access nested properties.
    #[serde(default)]
    pub model_info: Option<serde_json::Value>,
}

impl ShowResponse {
    /// Check if the model supports a specific capability
    ///
    /// # Arguments
    ///
    /// * `capability` - The capability to check (e.g., "completion", "vision")
    ///
    /// # Example
    ///
    /// ```
    /// use ollama_oxide::ShowResponse;
    ///
    /// let response = ShowResponse {
    ///     capabilities: Some(vec!["completion".to_string(), "vision".to_string()]),
    ///     ..Default::default()
    /// };
    ///
    /// assert!(response.has_capability("vision"));
    /// assert!(!response.has_capability("tools"));
    /// ```
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities
            .as_ref()
            .is_some_and(|caps| caps.iter().any(|c| c == capability))
    }
}
```

### Step 5: Update primitives/mod.rs

**Location:** `src/primitives/mod.rs`

Add the new module declarations and re-exports:

```rust
mod show_model_details;
mod show_request;
mod show_response;

pub use show_model_details::ShowModelDetails;
pub use show_request::ShowRequest;
pub use show_response::ShowResponse;
```

### Step 6: Implement Async API Method

**Location:** `src/http/api_async.rs`

```rust
/// Show detailed information about a model
///
/// Retrieves comprehensive metadata including parameters,
/// license, capabilities, and model-specific configuration.
///
/// # Arguments
///
/// * `request` - ShowRequest containing the model name
///
/// # Example
///
/// ```no_run
/// use ollama_oxide::{OllamaClient, ShowRequest};
///
/// # async fn example() -> Result<(), ollama_oxide::Error> {
/// let client = OllamaClient::default()?;
///
/// // Basic request
/// let request = ShowRequest::new("llama3.1");
/// let response = client.show_model(&request).await?;
/// println!("Capabilities: {:?}", response.capabilities);
///
/// // Verbose request
/// let verbose_request = ShowRequest::verbose("llama3.1");
/// let verbose_response = client.show_model(&verbose_request).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The model does not exist (404)
/// - Network error occurs
/// - Response cannot be deserialized
pub async fn show_model(&self, request: &ShowRequest) -> Result<ShowResponse> {
    let url = self.config.url(Endpoints::SHOW);
    self.post_with_retry(&url, request).await
}
```

### Step 7: Implement Sync API Method

**Location:** `src/http/api_sync.rs`

```rust
/// Show detailed information about a model (blocking)
///
/// Retrieves comprehensive metadata including parameters,
/// license, capabilities, and model-specific configuration.
///
/// # Arguments
///
/// * `request` - ShowRequest containing the model name
///
/// # Example
///
/// ```no_run
/// use ollama_oxide::{OllamaClient, ShowRequest};
///
/// let client = OllamaClient::default()?;
///
/// let request = ShowRequest::new("llama3.1");
/// let response = client.show_model_blocking(&request)?;
/// println!("License: {:?}", response.license);
/// # Ok::<(), ollama_oxide::Error>(())
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The model does not exist (404)
/// - Network error occurs
/// - Response cannot be deserialized
pub fn show_model_blocking(&self, request: &ShowRequest) -> Result<ShowResponse> {
    let url = self.config.url(Endpoints::SHOW);
    self.post_blocking_with_retry(&url, request)
}
```

### Step 8: Update lib.rs Re-exports

**Location:** `src/lib.rs`

Add to the public API:

```rust
pub use primitives::{ShowModelDetails, ShowRequest, ShowResponse};
```

## Testing Strategy

### Unit Tests (tests/client_show_model_tests.rs)

Create comprehensive unit tests using mockito following the established project patterns:

```rust
//! Tests for show_model API methods (POST /api/show)

use ollama_oxide::{ClientConfig, OllamaApiAsync, OllamaApiSync, OllamaClient, ShowRequest, ShowResponse};
use std::time::Duration;

// ============================================================================
// Async API Tests
// ============================================================================

#[tokio::test]
async fn test_show_model_async_success() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/show")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "model": "llama3.1"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "parameters": "temperature 0.7",
            "license": "MIT",
            "modified_at": "2025-01-01T00:00:00Z",
            "capabilities": ["completion"]
        }"#)
        .create_async()
        .await;

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 0,
    };

    let client = OllamaClient::new(config).unwrap();
    let request = ShowRequest::new("llama3.1");
    let response = client.show_model(&request).await.unwrap();

    assert_eq!(response.parameters, Some("temperature 0.7".to_string()));
    assert_eq!(response.license, Some("MIT".to_string()));
    assert!(response.has_capability("completion"));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_show_model_async_verbose() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/show")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "model": "llama3.1",
            "verbose": true
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"parameters": "temperature 0.7"}"#)
        .create_async()
        .await;

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 0,
    };

    let client = OllamaClient::new(config).unwrap();
    let request = ShowRequest::verbose("llama3.1");
    let result = client.show_model(&request).await;

    assert!(result.is_ok());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_show_model_async_not_found() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/show")
        .with_status(404)
        .create_async()
        .await;

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 0,
    };

    let client = OllamaClient::new(config).unwrap();
    let request = ShowRequest::new("nonexistent");
    let result = client.show_model(&request).await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_show_model_async_retry_on_server_error() {
    let mut server = mockito::Server::new_async().await;

    let mock_fail = server
        .mock("POST", "/api/show")
        .with_status(500)
        .expect(1)
        .create_async()
        .await;

    let mock_success = server
        .mock("POST", "/api/show")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"parameters": ""}"#)
        .expect(1)
        .create_async()
        .await;

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 1,
    };

    let client = OllamaClient::new(config).unwrap();
    let request = ShowRequest::new("model");
    let result = client.show_model(&request).await;

    assert!(result.is_ok());
    mock_fail.assert_async().await;
    mock_success.assert_async().await;
}

// ============================================================================
// Sync API Tests
// ============================================================================

#[test]
fn test_show_model_sync_success() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("POST", "/api/show")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "model": "llama3.1"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"parameters": "temperature 0.7", "license": "MIT"}"#)
        .create();

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 0,
    };

    let client = OllamaClient::new(config).unwrap();
    let request = ShowRequest::new("llama3.1");
    let result = client.show_model_blocking(&request);

    assert!(result.is_ok());
    mock.assert();
}

#[test]
fn test_show_model_sync_not_found() {
    let mut server = mockito::Server::new();

    let mock = server.mock("POST", "/api/show").with_status(404).create();

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 0,
    };

    let client = OllamaClient::new(config).unwrap();
    let request = ShowRequest::new("missing");
    let result = client.show_model_blocking(&request);

    assert!(result.is_err());
    mock.assert();
}

// ============================================================================
// Primitive Type Tests
// ============================================================================

#[test]
fn test_show_request_new() {
    let request = ShowRequest::new("model-name");
    assert_eq!(request.model, "model-name");
    assert_eq!(request.verbose, None);
}

#[test]
fn test_show_request_verbose() {
    let request = ShowRequest::verbose("model-name");
    assert_eq!(request.model, "model-name");
    assert_eq!(request.verbose, Some(true));
}

#[test]
fn test_show_response_has_capability() {
    let response = ShowResponse {
        capabilities: Some(vec!["completion".to_string(), "vision".to_string()]),
        ..Default::default()
    };

    assert!(response.has_capability("completion"));
    assert!(response.has_capability("vision"));
    assert!(!response.has_capability("tools"));
}

#[test]
fn test_show_response_has_capability_none() {
    let response = ShowResponse::default();
    assert!(!response.has_capability("completion"));
}
```

### Integration Examples (examples/)

Create example files for real-world usage:

**examples/show_model_async.rs:**
```rust
//! Example: Show model information (async)

use ollama_oxide::{OllamaClient, ShowRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::default()?;

    // Basic model info
    let request = ShowRequest::new("llama3.1");
    let response = client.show_model(&request).await?;

    println!("Model: llama3.1");
    println!("Capabilities: {:?}", response.capabilities);
    println!("License: {:?}", response.license);

    if let Some(details) = &response.details {
        println!("Family: {:?}", details.family);
        println!("Parameter Size: {:?}", details.parameter_size);
        println!("Quantization: {:?}", details.quantization_level);
    }

    Ok(())
}
```

**examples/show_model_sync.rs:**
```rust
//! Example: Show model information (sync/blocking)

use ollama_oxide::{OllamaClient, ShowRequest};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::default()?;

    let request = ShowRequest::new("llama3.1");
    let response = client.show_model_blocking(&request)?;

    println!("Model: llama3.1");
    if response.has_capability("vision") {
        println!("This model supports vision!");
    }

    Ok(())
}
```

## File Changes Summary

| File | Action | Description |
|------|--------|-------------|
| `src/http/client.rs` | Modify | Add `post_with_retry` and `post_blocking_with_retry` helpers |
| `src/primitives/show_request.rs` | Create | ShowRequest struct |
| `src/primitives/show_model_details.rs` | Create | ShowModelDetails struct |
| `src/primitives/show_response.rs` | Create | ShowResponse struct |
| `src/primitives/mod.rs` | Modify | Add module declarations and re-exports |
| `src/http/api_async.rs` | Modify | Add `show_model()` method |
| `src/http/api_sync.rs` | Modify | Add `show_model_blocking()` method |
| `src/lib.rs` | Modify | Add public re-exports |
| `tests/client_show_model_tests.rs` | Create | Unit tests with mocking |
| `examples/show_model_async.rs` | Create | Async usage example |
| `examples/show_model_sync.rs` | Create | Sync usage example |

## Dependencies

**Existing (no changes needed):**
- `serde` with `derive` feature
- `serde_json` for `model_info` flexible structure
- `mockito` for testing

## Definition of Done

- [ ] `post_with_retry` and `post_blocking_with_retry` helper methods added to client.rs
- [ ] `ShowRequest` primitive type created with `new()` and `verbose()` constructors
- [ ] `ShowModelDetails` primitive type created
- [ ] `ShowResponse` primitive type created with `has_capability()` helper
- [ ] `show_model()` async method implemented
- [ ] `show_model_blocking()` sync method implemented
- [ ] Public re-exports added to lib.rs and primitives/mod.rs
- [ ] Unit tests created (10+ tests)
- [ ] Example files created (async and sync)
- [ ] All tests pass: `cargo test`
- [ ] Clippy clean: `cargo clippy`
- [ ] Formatted: `cargo fmt`

## Implementation Order

1. Add `post_with_retry` helpers to client.rs
2. Create `ShowRequest` primitive
3. Create `ShowModelDetails` primitive
4. Create `ShowResponse` primitive
5. Update `primitives/mod.rs` with new exports
6. Implement `show_model()` in api_async.rs
7. Implement `show_model_blocking()` in api_sync.rs
8. Update `lib.rs` re-exports
9. Create unit tests
10. Create examples
11. Run `cargo test`, `cargo clippy`, `cargo fmt`

## Notes

- The `model_info` field uses `serde_json::Value` for flexibility since the API returns arbitrary key-value pairs specific to each model architecture
- All response fields are `Option<T>` to handle partial responses gracefully
- The `ShowModelDetails` type is separate from `ModelDetails` (used in `/api/tags`) because they have different fields
- The `has_capability()` helper method provides a convenient way to check model capabilities

## Code Patterns

### URL Construction Pattern

**IMPORTANT:** Always use the `Endpoints` enum with `self.config.url()` for constructing API URLs:

```rust
// ✅ CORRECT - Use Endpoints enum
let url = self.config.url(Endpoints::SHOW);

// ❌ WRONG - Do not use format! with hardcoded paths
let url = format!("{}/api/show", self.config.url());
```

The `Endpoints` struct in `src/http/endpoints.rs` defines all API paths as constants, providing a single source of truth for API routes.
