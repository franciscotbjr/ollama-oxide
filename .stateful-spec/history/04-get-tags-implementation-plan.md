# Implementation Plan: GET /api/tags

**Endpoint:** GET /api/tags
**Complexity:** Simple (2)
**Phase:** Phase 1 - Foundation + HTTP Core
**Document Version:** 1.0
**Created:** 2026-01-14

## Overview

This document outlines the implementation plan for the second endpoint: `GET /api/tags`. This endpoint retrieves a list of locally available models and their details. Since the foundational infrastructure (error handling, HTTP client, retry logic) was established with `GET /api/version`, this implementation focuses on:

1. New primitive types (`ListResponse`, `ModelSummary`, `ModelDetails`)
2. Extending the async and sync API traits
3. Adding comprehensive tests

## API Specification Summary

**Endpoint:** `GET /api/tags`
**Operation ID:** `list`
**Description:** Fetch a list of models and their details

**Response (200 OK):**
```json
{
  "models": [
    {
      "name": "gemma3",
      "modified_at": "2025-10-03T23:34:03.409490317-07:00",
      "size": 3338801804,
      "digest": "a2af6cc3eb7fa8be8504abaf9b04e88f17a119ec3f04a3addf55f92841195f5a",
      "details": {
        "format": "gguf",
        "family": "gemma",
        "families": ["gemma"],
        "parameter_size": "4.3B",
        "quantization_level": "Q4_K_M"
      }
    }
  ]
}
```

**Schemas:**

### ListResponse
```yaml
type: object
properties:
  models:
    type: array
    items:
      $ref: '#/components/schemas/ModelSummary'
```

### ModelSummary
```yaml
type: object
description: Summary information for a locally available model
properties:
  name:
    type: string
    description: Model name
  modified_at:
    type: string
    description: Last modified timestamp in ISO 8601 format
  size:
    type: integer
    description: Total size of the model on disk in bytes
  digest:
    type: string
    description: SHA256 digest identifier of the model contents
  details:
    $ref: '#/components/schemas/ModelDetails'
```

### ModelDetails
```yaml
type: object
description: Additional information about the model's format and family
properties:
  format:
    type: string
    description: Model file format (for example `gguf`)
  family:
    type: string
    description: Primary model family (for example `llama`)
  families:
    type: array
    items:
      type: string
    description: All families the model belongs to, when applicable
  parameter_size:
    type: string
    description: Approximate parameter count label (for example `7B`, `13B`)
  quantization_level:
    type: string
    description: Quantization level used (for example `Q4_0`)
```

## Implementation Strategy

### Phase 1: Primitives Module - New Types

**Location:** `src/primitives/`

#### Step 1.1: Create ModelDetails type

**File:** `src/primitives/model_details.rs`

```rust
//! Model details primitive type

use serde::{Deserialize, Serialize};

/// Additional information about a model's format and family
///
/// Contains metadata about the model's file format, family classification,
/// and quantization settings.
///
/// # Example
///
/// ```json
/// {
///   "format": "gguf",
///   "family": "gemma",
///   "families": ["gemma"],
///   "parameter_size": "4.3B",
///   "quantization_level": "Q4_K_M"
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelDetails {
    /// Model file format (e.g., "gguf")
    #[serde(default)]
    pub format: Option<String>,

    /// Primary model family (e.g., "llama", "gemma")
    #[serde(default)]
    pub family: Option<String>,

    /// All families the model belongs to
    #[serde(default)]
    pub families: Option<Vec<String>>,

    /// Approximate parameter count label (e.g., "7B", "13B")
    #[serde(default)]
    pub parameter_size: Option<String>,

    /// Quantization level used (e.g., "Q4_0", "Q4_K_M")
    #[serde(default)]
    pub quantization_level: Option<String>,
}
```

**Design Notes:**
- All fields are `Option<T>` because the API may not always return all fields
- Using `#[serde(default)]` ensures missing fields deserialize to `None`

#### Step 1.2: Create ModelSummary type

**File:** `src/primitives/model_summary.rs`

```rust
//! Model summary primitive type

use serde::{Deserialize, Serialize};

use super::ModelDetails;

/// Summary information for a locally available model
///
/// Contains basic information about a model including its name, size,
/// digest, and detailed metadata.
///
/// # Example
///
/// ```json
/// {
///   "name": "gemma3",
///   "modified_at": "2025-10-03T23:34:03.409490317-07:00",
///   "size": 3338801804,
///   "digest": "a2af6cc3eb7fa8be8504abaf9b04e88f17a119ec3f04a3addf55f92841195f5a",
///   "details": {
///     "format": "gguf",
///     "family": "gemma"
///   }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelSummary {
    /// Model name (e.g., "llama3.2", "gemma3")
    pub name: String,

    /// Last modified timestamp in ISO 8601 format
    #[serde(default)]
    pub modified_at: Option<String>,

    /// Total size of the model on disk in bytes
    #[serde(default)]
    pub size: Option<u64>,

    /// SHA256 digest identifier of the model contents
    #[serde(default)]
    pub digest: Option<String>,

    /// Additional information about the model
    #[serde(default)]
    pub details: Option<ModelDetails>,
}
```

**Design Notes:**
- `name` is required (always present in API response)
- Other fields are optional for forward compatibility
- `size` uses `u64` to handle large model sizes (multiple GB)

#### Step 1.3: Create ListResponse type

**File:** `src/primitives/list_response.rs`

```rust
//! List models response primitive type

use serde::{Deserialize, Serialize};

use super::ModelSummary;

/// Response from GET /api/tags endpoint
///
/// Contains a list of locally available models with their details.
///
/// # Example
///
/// ```json
/// {
///   "models": [
///     {
///       "name": "gemma3",
///       "modified_at": "2025-10-03T23:34:03.409490317-07:00",
///       "size": 3338801804,
///       "digest": "a2af6cc3..."
///     }
///   ]
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListResponse {
    /// List of available models
    #[serde(default)]
    pub models: Vec<ModelSummary>,
}
```

**Design Notes:**
- `models` defaults to empty vector if missing
- Using `Vec<ModelSummary>` for the list of models

#### Step 1.4: Update primitives mod.rs

**File:** `src/primitives/mod.rs`

```rust
//! Primitive types for Ollama API responses and requests
//!
//! This module contains all primitive data types used in the Ollama API,
//! including request and response structures.

mod version;
mod model_details;
mod model_summary;
mod list_response;

pub use version::VersionResponse;
pub use model_details::ModelDetails;
pub use model_summary::ModelSummary;
pub use list_response::ListResponse;
```

### Phase 2: HTTP Module - API Extension

#### Step 2.1: Update OllamaApiAsync trait

**File:** `src/http/api_async.rs`

Add new method to trait and implementation:

```rust
use crate::{Result, VersionResponse, ListResponse};

#[async_trait]
pub trait OllamaApiAsync: Send + Sync {
    /// Get Ollama server version (async)
    async fn version(&self) -> Result<VersionResponse>;

    /// List locally available models (async)
    ///
    /// Returns a list of models installed on the Ollama server with their details.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Network request fails
    /// - Maximum retry attempts exceeded
    /// - Response cannot be deserialized
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::{OllamaClient, OllamaApiAsync};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let response = client.list_models().await?;
    /// for model in &response.models {
    ///     println!("Model: {}", model.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn list_models(&self) -> Result<ListResponse>;
}

#[async_trait]
impl OllamaApiAsync for OllamaClient {
    async fn version(&self) -> Result<VersionResponse> {
        let url = self.config.url(Endpoints::VERSION);
        self.get_with_retry(&url).await
    }

    async fn list_models(&self) -> Result<ListResponse> {
        let url = self.config.url(Endpoints::TAGS);
        self.get_with_retry(&url).await
    }
}
```

#### Step 2.2: Update OllamaApiSync trait

**File:** `src/http/api_sync.rs`

Add new method to trait and implementation:

```rust
use crate::{Result, VersionResponse, ListResponse};

pub trait OllamaApiSync: Send + Sync {
    /// Get Ollama server version (blocking)
    fn version_blocking(&self) -> Result<VersionResponse>;

    /// List locally available models (blocking)
    ///
    /// Returns a list of models installed on the Ollama server with their details.
    /// This method blocks the current thread until the request completes.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Network request fails
    /// - Maximum retry attempts exceeded
    /// - Response cannot be deserialized
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::{OllamaClient, OllamaApiSync};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let response = client.list_models_blocking()?;
    /// for model in &response.models {
    ///     println!("Model: {}", model.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn list_models_blocking(&self) -> Result<ListResponse>;
}

impl OllamaApiSync for OllamaClient {
    fn version_blocking(&self) -> Result<VersionResponse> {
        let url = self.config.url(Endpoints::VERSION);
        self.get_blocking_with_retry(&url)
    }

    fn list_models_blocking(&self) -> Result<ListResponse> {
        let url = self.config.url(Endpoints::TAGS);
        self.get_blocking_with_retry(&url)
    }
}
```

### Phase 3: Library Entry Point Update

**File:** `src/lib.rs`

Add re-exports for new types:

```rust
#[cfg(feature = "primitives")]
pub use primitives::{
    VersionResponse,
    ModelDetails,
    ModelSummary,
    ListResponse,
};
```

### Phase 4: Testing

#### Step 4.1: Unit Tests for Primitives

**File:** `tests/primitives_list_tests.rs`

```rust
use ollama_oxide::{ModelDetails, ModelSummary, ListResponse};

#[test]
fn test_model_details_deserialization() {
    let json = r#"{
        "format": "gguf",
        "family": "gemma",
        "families": ["gemma"],
        "parameter_size": "4.3B",
        "quantization_level": "Q4_K_M"
    }"#;

    let details: ModelDetails = serde_json::from_str(json).unwrap();
    assert_eq!(details.format, Some("gguf".to_string()));
    assert_eq!(details.family, Some("gemma".to_string()));
    assert_eq!(details.families, Some(vec!["gemma".to_string()]));
    assert_eq!(details.parameter_size, Some("4.3B".to_string()));
    assert_eq!(details.quantization_level, Some("Q4_K_M".to_string()));
}

#[test]
fn test_model_details_partial_deserialization() {
    let json = r#"{"format": "gguf"}"#;

    let details: ModelDetails = serde_json::from_str(json).unwrap();
    assert_eq!(details.format, Some("gguf".to_string()));
    assert_eq!(details.family, None);
    assert_eq!(details.families, None);
}

#[test]
fn test_model_details_empty_deserialization() {
    let json = r#"{}"#;

    let details: ModelDetails = serde_json::from_str(json).unwrap();
    assert_eq!(details.format, None);
    assert_eq!(details.family, None);
}

#[test]
fn test_model_summary_deserialization() {
    let json = r#"{
        "name": "gemma3",
        "modified_at": "2025-10-03T23:34:03.409490317-07:00",
        "size": 3338801804,
        "digest": "a2af6cc3eb7fa8be8504abaf9b04e88f17a119ec3f04a3addf55f92841195f5a",
        "details": {
            "format": "gguf",
            "family": "gemma"
        }
    }"#;

    let summary: ModelSummary = serde_json::from_str(json).unwrap();
    assert_eq!(summary.name, "gemma3");
    assert_eq!(summary.modified_at, Some("2025-10-03T23:34:03.409490317-07:00".to_string()));
    assert_eq!(summary.size, Some(3338801804));
    assert!(summary.details.is_some());
}

#[test]
fn test_model_summary_minimal() {
    let json = r#"{"name": "llama3"}"#;

    let summary: ModelSummary = serde_json::from_str(json).unwrap();
    assert_eq!(summary.name, "llama3");
    assert_eq!(summary.modified_at, None);
    assert_eq!(summary.size, None);
    assert_eq!(summary.digest, None);
    assert_eq!(summary.details, None);
}

#[test]
fn test_list_response_deserialization() {
    let json = r#"{
        "models": [
            {
                "name": "gemma3",
                "size": 3338801804
            },
            {
                "name": "llama3",
                "size": 4000000000
            }
        ]
    }"#;

    let response: ListResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.models.len(), 2);
    assert_eq!(response.models[0].name, "gemma3");
    assert_eq!(response.models[1].name, "llama3");
}

#[test]
fn test_list_response_empty() {
    let json = r#"{"models": []}"#;

    let response: ListResponse = serde_json::from_str(json).unwrap();
    assert!(response.models.is_empty());
}

#[test]
fn test_list_response_missing_models() {
    let json = r#"{}"#;

    let response: ListResponse = serde_json::from_str(json).unwrap();
    assert!(response.models.is_empty());
}

// Thread safety tests
#[test]
fn test_types_are_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<ModelDetails>();
    assert_send_sync::<ModelSummary>();
    assert_send_sync::<ListResponse>();
}

// Serialization round-trip tests
#[test]
fn test_list_response_round_trip() {
    let original = ListResponse {
        models: vec![
            ModelSummary {
                name: "test-model".to_string(),
                modified_at: Some("2025-01-01T00:00:00Z".to_string()),
                size: Some(1000000),
                digest: Some("abc123".to_string()),
                details: Some(ModelDetails {
                    format: Some("gguf".to_string()),
                    family: Some("llama".to_string()),
                    families: Some(vec!["llama".to_string()]),
                    parameter_size: Some("7B".to_string()),
                    quantization_level: Some("Q4_0".to_string()),
                }),
            },
        ],
    };

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: ListResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(original, deserialized);
}
```

#### Step 4.2: API Tests

**File:** `tests/client_list_models_tests.rs`

```rust
use ollama_oxide::{OllamaClient, OllamaApiAsync, OllamaApiSync, ClientConfig};
use std::time::Duration;

#[tokio::test]
async fn test_list_models_async_with_mock() {
    let mut server = mockito::Server::new_async().await;

    let mock = server.mock("GET", "/api/tags")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "models": [
                {
                    "name": "test-model",
                    "size": 1000000
                }
            ]
        }"#)
        .create_async()
        .await;

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 0,
    };

    let client = OllamaClient::new(config).unwrap();
    let response = client.list_models().await.unwrap();

    assert_eq!(response.models.len(), 1);
    assert_eq!(response.models[0].name, "test-model");
    assert_eq!(response.models[0].size, Some(1000000));

    mock.assert_async().await;
}

#[test]
fn test_list_models_sync_with_mock() {
    let mut server = mockito::Server::new();

    let mock = server.mock("GET", "/api/tags")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "models": [
                {
                    "name": "sync-model",
                    "size": 2000000
                }
            ]
        }"#)
        .create();

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 0,
    };

    let client = OllamaClient::new(config).unwrap();
    let response = client.list_models_blocking().unwrap();

    assert_eq!(response.models.len(), 1);
    assert_eq!(response.models[0].name, "sync-model");

    mock.assert();
}

#[tokio::test]
async fn test_list_models_empty_response() {
    let mut server = mockito::Server::new_async().await;

    let mock = server.mock("GET", "/api/tags")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"models": []}"#)
        .create_async()
        .await;

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 0,
    };

    let client = OllamaClient::new(config).unwrap();
    let response = client.list_models().await.unwrap();

    assert!(response.models.is_empty());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_models_full_response() {
    let mut server = mockito::Server::new_async().await;

    let mock = server.mock("GET", "/api/tags")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "models": [
                {
                    "name": "gemma3",
                    "modified_at": "2025-10-03T23:34:03.409490317-07:00",
                    "size": 3338801804,
                    "digest": "a2af6cc3eb7fa8be8504abaf9b04e88f17a119ec3f04a3addf55f92841195f5a",
                    "details": {
                        "format": "gguf",
                        "family": "gemma",
                        "families": ["gemma"],
                        "parameter_size": "4.3B",
                        "quantization_level": "Q4_K_M"
                    }
                }
            ]
        }"#)
        .create_async()
        .await;

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 0,
    };

    let client = OllamaClient::new(config).unwrap();
    let response = client.list_models().await.unwrap();

    assert_eq!(response.models.len(), 1);
    let model = &response.models[0];
    assert_eq!(model.name, "gemma3");
    assert!(model.details.is_some());

    let details = model.details.as_ref().unwrap();
    assert_eq!(details.format, Some("gguf".to_string()));
    assert_eq!(details.family, Some("gemma".to_string()));

    mock.assert_async().await;
}
```

#### Step 4.3: Integration Tests

**File:** `tests/integration_list_tests.rs`

```rust
use ollama_oxide::{OllamaClient, OllamaApiAsync, OllamaApiSync};

/// Integration test that requires a running Ollama server.
/// Set OLLAMA_TEST_SERVER=1 to enable.
#[tokio::test]
async fn test_list_models_integration_async() {
    if std::env::var("OLLAMA_TEST_SERVER").is_err() {
        eprintln!("Skipping integration test: OLLAMA_TEST_SERVER not set");
        return;
    }

    let client = OllamaClient::default().expect("Failed to create client");
    let response = client.list_models().await;

    match response {
        Ok(list) => {
            println!("Found {} models", list.models.len());
            for model in &list.models {
                println!("  - {} ({:?} bytes)", model.name, model.size);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            // Don't fail - Ollama might not have any models
        }
    }
}

#[test]
fn test_list_models_integration_sync() {
    if std::env::var("OLLAMA_TEST_SERVER").is_err() {
        eprintln!("Skipping integration test: OLLAMA_TEST_SERVER not set");
        return;
    }

    let client = OllamaClient::default().expect("Failed to create client");
    let response = client.list_models_blocking();

    match response {
        Ok(list) => {
            println!("Found {} models (sync)", list.models.len());
            for model in &list.models {
                println!("  - {} ({:?} bytes)", model.name, model.size);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
```

### Phase 5: Examples

#### Step 5.1: Create list_models_async example

**File:** `examples/list_models_async.rs`

```rust
//! Example: List locally available models (async)
//!
//! This example demonstrates how to fetch and display all models
//! installed on the Ollama server using the async API.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example list_models_async
//! ```

use ollama_oxide::{OllamaClient, OllamaApiAsync, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Create client with default configuration
    let client = OllamaClient::default()?;

    // List all available models
    let response = client.list_models().await?;

    println!("Available models ({}):", response.models.len());
    println!("{:-<60}", "");

    for model in &response.models {
        println!("Name: {}", model.name);

        if let Some(size) = model.size {
            let size_gb = size as f64 / 1_073_741_824.0;
            println!("  Size: {:.2} GB", size_gb);
        }

        if let Some(modified_at) = &model.modified_at {
            println!("  Modified: {}", modified_at);
        }

        if let Some(digest) = &model.digest {
            println!("  Digest: {}...", &digest[..12.min(digest.len())]);
        }

        if let Some(details) = &model.details {
            if let Some(format) = &details.format {
                println!("  Format: {}", format);
            }
            if let Some(family) = &details.family {
                println!("  Family: {}", family);
            }
            if let Some(param_size) = &details.parameter_size {
                println!("  Parameters: {}", param_size);
            }
            if let Some(quant) = &details.quantization_level {
                println!("  Quantization: {}", quant);
            }
        }

        println!();
    }

    if response.models.is_empty() {
        println!("No models found. Try pulling a model first:");
        println!("  ollama pull llama3.2");
    }

    Ok(())
}
```

#### Step 5.2: Create list_models_sync example

**File:** `examples/list_models_sync.rs`

```rust
//! Example: List locally available models (sync)
//!
//! This example demonstrates how to fetch and display all models
//! installed on the Ollama server using the blocking API.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example list_models_sync
//! ```

use ollama_oxide::{OllamaClient, OllamaApiSync, Result};

fn main() -> Result<()> {
    // Create client with default configuration
    let client = OllamaClient::default()?;

    // List all available models (blocking)
    let response = client.list_models_blocking()?;

    println!("Available models ({}):", response.models.len());

    for model in &response.models {
        let size_str = model.size
            .map(|s| format!("{:.2} GB", s as f64 / 1_073_741_824.0))
            .unwrap_or_else(|| "unknown".to_string());

        println!("  - {} ({})", model.name, size_str);
    }

    Ok(())
}
```

## Implementation Order

### Step 1: Primitives (New Types)
1. Create `src/primitives/model_details.rs`
2. Create `src/primitives/model_summary.rs`
3. Create `src/primitives/list_response.rs`
4. Update `src/primitives/mod.rs` with re-exports

### Step 2: API Extension
1. Update `src/http/api_async.rs` with `list_models()` method
2. Update `src/http/api_sync.rs` with `list_models_blocking()` method

### Step 3: Library Entry Point
1. Update `src/lib.rs` to re-export new types

### Step 4: Tests
1. Create `tests/primitives_list_tests.rs`
2. Create `tests/client_list_models_tests.rs`
3. Create `tests/integration_list_tests.rs`

### Step 5: Examples
1. Create `examples/list_models_async.rs`
2. Create `examples/list_models_sync.rs`

### Step 6: Documentation
1. Update README.md with list_models example
2. Update CHANGELOG.md

## File Structure After Implementation

```
ollama-oxide/
├── src/
│   ├── lib.rs                    # Updated re-exports
│   ├── error.rs                  # (unchanged)
│   ├── primitives/
│   │   ├── mod.rs                # Updated with new re-exports
│   │   ├── version.rs            # (unchanged)
│   │   ├── model_details.rs      # NEW
│   │   ├── model_summary.rs      # NEW
│   │   └── list_response.rs      # NEW
│   └── http/
│       ├── mod.rs                # (unchanged)
│       ├── config.rs             # (unchanged)
│       ├── client.rs             # (unchanged)
│       ├── endpoints.rs          # (unchanged - TAGS already defined)
│       ├── api_async.rs          # Updated with list_models()
│       └── api_sync.rs           # Updated with list_models_blocking()
├── tests/
│   ├── primitives_list_tests.rs  # NEW
│   ├── client_list_models_tests.rs # NEW
│   └── integration_list_tests.rs # NEW
├── examples/
│   ├── list_models_async.rs      # NEW
│   └── list_models_sync.rs       # NEW
└── Cargo.toml                    # (unchanged)
```

## Success Criteria

The implementation will be considered successful when:

1. ✅ `cargo build` succeeds without warnings
2. ✅ `cargo test` passes all new tests
3. ✅ `cargo clippy` shows no warnings
4. ✅ `cargo doc --open` generates proper documentation
5. ✅ Examples run successfully against Ollama server
6. ✅ All primitive types implement `Send + Sync`
7. ✅ Deserialization handles partial/missing fields gracefully
8. ✅ Code follows established patterns from GET /api/version

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Optional fields | Use `Option<T>` | API may not return all fields |
| Size type | `u64` | Handle large model sizes (GBs) |
| Serde defaults | `#[serde(default)]` | Graceful handling of missing fields |
| Method naming | `list_models()` | Clear, action-oriented |
| File organization | Separate files per type | Follows ARCHITECTURE.md pattern |

## Notes

- This implementation leverages the existing HTTP retry infrastructure
- The `Endpoints::TAGS` constant is already defined
- No changes needed to `Cargo.toml` (all dependencies already available)
- Follows established TDD patterns from GET /api/version implementation

---

**Plan Status**: 📝 READY FOR REVIEW
**Created**: 2026-01-14
