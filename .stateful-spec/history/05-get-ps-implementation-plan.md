# Implementation Plan: GET /api/ps

**Endpoint:** GET /api/ps
**Complexity:** Simple
**Phase:** Phase 1 - Foundation + Non-Streaming Endpoints
**Document Version:** 1.0
**Created:** 2026-01-15

## Overview

This document outlines the implementation plan for the `GET /api/ps` endpoint, which retrieves a list of models currently loaded into memory and running on the Ollama server.

This endpoint follows the established patterns from:
- `GET /api/version` - Base implementation pattern
- `GET /api/tags` - Similar response structure (list of models)

## API Specification Summary

**Endpoint:** `GET /api/ps`
**Operation ID:** `ps`
**Description:** Retrieve a list of models that are currently running

**Response (200 OK):**
```json
{
  "models": [
    {
      "model": "gemma3",
      "size": 6591830464,
      "digest": "a2af6cc3eb7fa8be8504abaf9b04e88f17a119ec3f04a3addf55f92841195f5a",
      "details": {
        "parent_model": "",
        "format": "gguf",
        "family": "gemma3",
        "families": ["gemma3"],
        "parameter_size": "4.3B",
        "quantization_level": "Q4_K_M"
      },
      "expires_at": "2025-10-17T16:47:07.93355-07:00",
      "size_vram": 5333539264,
      "context_length": 4096
    }
  ]
}
```

## Schema Analysis

### PsResponse (New Type)
```rust
pub struct PsResponse {
    pub models: Vec<RunningModel>,
}
```

### RunningModel (New Type)
```rust
pub struct RunningModel {
    pub model: String,              // Name of the running model
    pub size: Option<u64>,          // Size of the model in bytes
    pub digest: Option<String>,     // SHA256 digest of the model
    pub details: Option<ModelDetails>,  // Reuse existing ModelDetails
    pub expires_at: Option<String>, // Time when the model will be unloaded
    pub size_vram: Option<u64>,     // VRAM usage in bytes
    pub context_length: Option<u32>, // Context length for the running model
}
```

### Comparison with ModelSummary (GET /api/tags)

| Field | ModelSummary | RunningModel | Notes |
|-------|--------------|--------------|-------|
| name/model | `name: String` | `model: String` | Different field name |
| modified_at | `modified_at: Option<String>` | - | Only in ModelSummary |
| size | `size: Option<u64>` | `size: Option<u64>` | Same |
| digest | `digest: Option<String>` | `digest: Option<String>` | Same |
| details | `details: Option<ModelDetails>` | `details: Option<ModelDetails>` | Same (reuse) |
| expires_at | - | `expires_at: Option<String>` | Only in RunningModel |
| size_vram | - | `size_vram: Option<u64>` | Only in RunningModel |
| context_length | - | `context_length: Option<u32>` | Only in RunningModel |

**Decision:** Create a new `RunningModel` type rather than extending `ModelSummary`, as the fields differ significantly and represent different concepts (available vs running models).

## Implementation Strategy

### Step 1: Create Primitive Types

**Files to create:**
1. `src/primitives/running_model.rs` - RunningModel struct
2. `src/primitives/ps_response.rs` - PsResponse struct

**Update:**
- `src/primitives/mod.rs` - Add module declarations and re-exports

#### 1.1 RunningModel Type

**Location:** `src/primitives/running_model.rs`

```rust
//! Running model primitive type

use serde::{Deserialize, Serialize};

use super::ModelDetails;

/// Information about a model currently loaded in memory
///
/// Contains runtime information about a model including VRAM usage,
/// context length, and expiration time.
///
/// # Example
///
/// ```json
/// {
///   "model": "gemma3",
///   "size": 6591830464,
///   "digest": "a2af6cc3...",
///   "details": { "format": "gguf", "family": "gemma3" },
///   "expires_at": "2025-10-17T16:47:07.93355-07:00",
///   "size_vram": 5333539264,
///   "context_length": 4096
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunningModel {
    /// Model name (e.g., "llama3.2", "gemma3")
    pub model: String,

    /// Total size of the model in bytes
    #[serde(default)]
    pub size: Option<u64>,

    /// SHA256 digest identifier of the model contents
    #[serde(default)]
    pub digest: Option<String>,

    /// Additional information about the model
    #[serde(default)]
    pub details: Option<ModelDetails>,

    /// Time when the model will be unloaded from memory (ISO 8601)
    #[serde(default)]
    pub expires_at: Option<String>,

    /// VRAM (GPU memory) usage in bytes
    #[serde(default)]
    pub size_vram: Option<u64>,

    /// Context length for the running model
    #[serde(default)]
    pub context_length: Option<u32>,
}
```

#### 1.2 PsResponse Type

**Location:** `src/primitives/ps_response.rs`

```rust
//! List running models response primitive type

use serde::{Deserialize, Serialize};

use super::RunningModel;

/// Response from GET /api/ps endpoint
///
/// Contains a list of models currently loaded into memory.
///
/// # Example
///
/// ```json
/// {
///   "models": [
///     {
///       "model": "gemma3",
///       "size": 6591830464,
///       "expires_at": "2025-10-17T16:47:07.93355-07:00"
///     }
///   ]
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PsResponse {
    /// List of currently running models
    #[serde(default)]
    pub models: Vec<RunningModel>,
}
```

#### 1.3 Update primitives/mod.rs

```rust
//! Primitive types for Ollama API responses and requests

mod version;
mod model_details;
mod model_summary;
mod list_response;
mod running_model;
mod ps_response;

pub use version::VersionResponse;
pub use model_details::ModelDetails;
pub use model_summary::ModelSummary;
pub use list_response::ListResponse;
pub use running_model::RunningModel;
pub use ps_response::PsResponse;
```

### Step 2: Update lib.rs Re-exports

**Location:** `src/lib.rs`

Add to re-exports:
```rust
pub use primitives::{
    ListResponse, ModelDetails, ModelSummary, PsResponse, RunningModel, VersionResponse,
};
```

### Step 3: Add API Methods

#### 3.1 Async API

**Location:** `src/http/api_async.rs`

Add to `OllamaApiAsync` trait:
```rust
/// List currently running models (async)
///
/// Returns a list of models currently loaded into memory.
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
/// let response = client.list_running_models().await?;
/// for model in &response.models {
///     println!("Running: {} (VRAM: {:?} bytes)", model.model, model.size_vram);
/// }
/// # Ok(())
/// # }
/// ```
async fn list_running_models(&self) -> Result<PsResponse>;
```

Add implementation:
```rust
async fn list_running_models(&self) -> Result<PsResponse> {
    let url = self.config.url(Endpoints::PS);
    self.get_with_retry(&url).await
}
```

#### 3.2 Sync API

**Location:** `src/http/api_sync.rs`

Add to `OllamaApiSync` trait:
```rust
/// List currently running models (blocking)
///
/// Returns a list of models currently loaded into memory.
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
/// let response = client.list_running_models_blocking()?;
/// for model in &response.models {
///     println!("Running: {} (VRAM: {:?} bytes)", model.model, model.size_vram);
/// }
/// # Ok(())
/// # }
/// ```
fn list_running_models_blocking(&self) -> Result<PsResponse>;
```

Add implementation:
```rust
fn list_running_models_blocking(&self) -> Result<PsResponse> {
    let url = self.config.url(Endpoints::PS);
    self.get_blocking_with_retry(&url)
}
```

### Step 4: Add Tests

#### 4.1 Unit Tests for RunningModel

**Location:** `src/primitives/running_model.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_running_model_deserialization() {
        let json = r#"{
            "model": "gemma3",
            "size": 6591830464,
            "digest": "a2af6cc3eb7fa8be8504abaf9b04e88f17a119ec3f04a3addf55f92841195f5a",
            "expires_at": "2025-10-17T16:47:07.93355-07:00",
            "size_vram": 5333539264,
            "context_length": 4096
        }"#;

        let model: RunningModel = serde_json::from_str(json).unwrap();
        assert_eq!(model.model, "gemma3");
        assert_eq!(model.size, Some(6591830464));
        assert_eq!(model.size_vram, Some(5333539264));
        assert_eq!(model.context_length, Some(4096));
    }

    #[test]
    fn test_running_model_minimal() {
        let json = r#"{"model": "llama3.2"}"#;
        let model: RunningModel = serde_json::from_str(json).unwrap();
        assert_eq!(model.model, "llama3.2");
        assert!(model.size.is_none());
        assert!(model.expires_at.is_none());
    }

    #[test]
    fn test_running_model_serialization_roundtrip() {
        let model = RunningModel {
            model: "test-model".to_string(),
            size: Some(1000),
            digest: Some("abc123".to_string()),
            details: None,
            expires_at: Some("2025-01-01T00:00:00Z".to_string()),
            size_vram: Some(500),
            context_length: Some(2048),
        };

        let json = serde_json::to_string(&model).unwrap();
        let deserialized: RunningModel = serde_json::from_str(&json).unwrap();
        assert_eq!(model, deserialized);
    }

    #[test]
    fn test_running_model_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RunningModel>();
    }
}
```

#### 4.2 Unit Tests for PsResponse

**Location:** `src/primitives/ps_response.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ps_response_deserialization() {
        let json = r#"{
            "models": [
                {"model": "gemma3", "size": 6591830464},
                {"model": "llama3.2", "size": 3338801804}
            ]
        }"#;

        let response: PsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.models.len(), 2);
        assert_eq!(response.models[0].model, "gemma3");
        assert_eq!(response.models[1].model, "llama3.2");
    }

    #[test]
    fn test_ps_response_empty() {
        let json = r#"{"models": []}"#;
        let response: PsResponse = serde_json::from_str(json).unwrap();
        assert!(response.models.is_empty());
    }

    #[test]
    fn test_ps_response_missing_models_defaults_empty() {
        let json = r#"{}"#;
        let response: PsResponse = serde_json::from_str(json).unwrap();
        assert!(response.models.is_empty());
    }

    #[test]
    fn test_ps_response_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PsResponse>();
    }
}
```

#### 4.3 Integration Tests

**Location:** `tests/ps_integration.rs`

```rust
//! Integration tests for GET /api/ps endpoint
//!
//! These tests require a running Ollama server at localhost:11434

use ollama_oxide::{OllamaApiAsync, OllamaApiSync, OllamaClient};

#[tokio::test]
async fn test_list_running_models_async() {
    let client = OllamaClient::default().expect("Failed to create client");
    let result = client.list_running_models().await;

    // Should succeed even if no models are running
    assert!(result.is_ok(), "Failed to list running models: {:?}", result);

    let response = result.unwrap();
    // Verify structure (may be empty if no models loaded)
    for model in &response.models {
        assert!(!model.model.is_empty(), "Model name should not be empty");
    }
}

#[test]
fn test_list_running_models_blocking() {
    let client = OllamaClient::default().expect("Failed to create client");
    let result = client.list_running_models_blocking();

    assert!(result.is_ok(), "Failed to list running models: {:?}", result);
}

#[tokio::test]
async fn test_list_running_models_concurrent() {
    let client = OllamaClient::default().expect("Failed to create client");

    let handles: Vec<_> = (0..5)
        .map(|_| {
            let c = client.clone();
            tokio::spawn(async move { c.list_running_models().await })
        })
        .collect();

    for handle in handles {
        let result = handle.await.expect("Task panicked");
        assert!(result.is_ok(), "Concurrent request failed");
    }
}
```

### Step 5: Add Example

**Location:** `examples/list_running_models.rs`

```rust
//! Example: List running models
//!
//! This example demonstrates how to retrieve the list of models
//! currently loaded into memory on the Ollama server.
//!
//! Run with: cargo run --example list_running_models

use ollama_oxide::{OllamaApiAsync, OllamaClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with default configuration
    let client = OllamaClient::default()?;

    // List running models
    let response = client.list_running_models().await?;

    if response.models.is_empty() {
        println!("No models currently running.");
        println!("Tip: Load a model with 'ollama run <model>' to see it here.");
    } else {
        println!("Currently running models:");
        println!("{:-<60}", "");

        for model in &response.models {
            println!("Model: {}", model.model);

            if let Some(size) = model.size {
                println!("  Size: {:.2} GB", size as f64 / 1_073_741_824.0);
            }

            if let Some(vram) = model.size_vram {
                println!("  VRAM: {:.2} GB", vram as f64 / 1_073_741_824.0);
            }

            if let Some(ctx) = model.context_length {
                println!("  Context: {} tokens", ctx);
            }

            if let Some(expires) = &model.expires_at {
                println!("  Expires: {}", expires);
            }

            if let Some(details) = &model.details {
                if let Some(family) = &details.family {
                    println!("  Family: {}", family);
                }
                if let Some(quant) = &details.quantization_level {
                    println!("  Quantization: {}", quant);
                }
            }

            println!();
        }
    }

    Ok(())
}
```

## File Changes Summary

### New Files
| File | Description |
|------|-------------|
| `src/primitives/running_model.rs` | RunningModel struct |
| `src/primitives/ps_response.rs` | PsResponse struct |
| `tests/ps_integration.rs` | Integration tests |
| `examples/list_running_models.rs` | Usage example |

### Modified Files
| File | Changes |
|------|---------|
| `src/primitives/mod.rs` | Add module declarations and re-exports |
| `src/lib.rs` | Add PsResponse, RunningModel to public re-exports |
| `src/http/api_async.rs` | Add `list_running_models()` method |
| `src/http/api_sync.rs` | Add `list_running_models_blocking()` method |

## Testing Checklist

- [ ] Unit tests for RunningModel pass
- [ ] Unit tests for PsResponse pass
- [ ] Send + Sync tests pass for new types
- [ ] Integration tests pass (requires Ollama running)
- [ ] Example runs successfully
- [ ] `cargo build` succeeds
- [ ] `cargo test` passes
- [ ] `cargo clippy` has no warnings
- [ ] `cargo fmt` applied

## Implementation Order

1. Create `running_model.rs` with struct and tests
2. Create `ps_response.rs` with struct and tests
3. Update `primitives/mod.rs` with new exports
4. Update `lib.rs` with public re-exports
5. Add async method to `api_async.rs`
6. Add sync method to `api_sync.rs`
7. Create integration tests
8. Create example
9. Run full test suite
10. Update DEV_NOTES.md and definition.md

## Notes

- The `Endpoints::PS` constant already exists in `endpoints.rs`
- Reuses existing `ModelDetails` type from `/api/tags` implementation
- Follows established patterns for consistency
- All types must implement `Send + Sync` for thread safety
