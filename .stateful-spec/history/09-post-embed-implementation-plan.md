# Implementation Plan: POST /api/embed

**Endpoint:** POST /api/embed
**Complexity:** Medium (input can be string or array, response with nested arrays)
**Phase:** Phase 1 - Foundation + Non-Streaming Endpoints
**Document Version:** 1.0
**Created:** 2026-01-23

## Overview

This document outlines the implementation plan for the `POST /api/embed` endpoint, which generates vector embeddings for input text.

This endpoint is similar to `POST /api/show`:
- Requires a request body with model name and input
- Returns a JSON response with embeddings and timing metrics
- Uses existing `post_with_retry` helper methods

**Key Characteristics:**
- Input can be a single string or array of strings
- Response contains nested arrays (array of embedding vectors)
- Supports optional parameters: `truncate`, `dimensions`, `keep_alive`, `options`
- Returns timing metrics: `total_duration`, `load_duration`, `prompt_eval_count`

## API Specification Summary

**Endpoint:** `POST /api/embed`
**Operation ID:** `embed`
**Description:** Creates vector embeddings representing the input text

**Request Body:**
```json
{
  "model": "nomic-embed-text",
  "input": "Generate embeddings for this text"
}
```

Or with array input:
```json
{
  "model": "nomic-embed-text",
  "input": ["First text", "Second text", "Third text"]
}
```

**Full Request with Optional Parameters:**
```json
{
  "model": "nomic-embed-text",
  "input": "Generate embeddings for this text",
  "truncate": true,
  "dimensions": 768,
  "keep_alive": "5m",
  "options": {
    "temperature": 0.7,
    "num_ctx": 4096
  }
}
```

**Response:**
```json
{
  "model": "nomic-embed-text",
  "embeddings": [
    [0.010071029, -0.0017594862, 0.05007221, 0.04692972, ...]
  ],
  "total_duration": 14143917,
  "load_duration": 1019500,
  "prompt_eval_count": 8
}
```

**Error Responses:**
- `404 Not Found` - Model does not exist

## Schema Analysis

### EmbedInput (New Type)

To handle the `oneOf` input type (string or array of strings), we need a custom type:

```rust
/// Input for embedding generation
///
/// Can be a single text string or an array of text strings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbedInput {
    /// Single text input
    Single(String),
    /// Multiple text inputs
    Multiple(Vec<String>),
}
```

The `#[serde(untagged)]` attribute allows serde to serialize/deserialize without a type discriminator.

### ModelOptions (New Type)

The `options` field uses a flexible `ModelOptions` struct:

```rust
/// Runtime options that control embedding generation
///
/// All fields are optional and will use model defaults if not specified.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ModelOptions {
    /// Random seed for reproducible outputs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,

    /// Controls randomness in generation (higher = more random)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Limits next token selection to the K most likely
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,

    /// Cumulative probability threshold for nucleus sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Minimum probability threshold for token selection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_p: Option<f32>,

    /// Context length size (number of tokens)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_ctx: Option<i32>,

    /// Maximum number of tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<i32>,

    /// Additional options not covered by explicit fields
    #[serde(flatten)]
    pub extra: Option<serde_json::Value>,
}
```

### EmbedRequest (New Type)

```rust
/// Request body for POST /api/embed endpoint
///
/// Generates vector embeddings for the provided input text(s).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbedRequest {
    /// Name of the embedding model to use
    pub model: String,

    /// Text or array of texts to generate embeddings for
    pub input: EmbedInput,

    /// If true, truncate inputs that exceed context window (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncate: Option<bool>,

    /// Number of dimensions for the embedding (model-specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<i32>,

    /// How long to keep the model loaded (e.g., "5m", "1h")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<String>,

    /// Runtime options for embedding generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ModelOptions>,
}
```

### EmbedResponse (New Type)

```rust
/// Response from POST /api/embed endpoint
///
/// Contains the generated embeddings and timing information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EmbedResponse {
    /// Model that produced the embeddings
    #[serde(default)]
    pub model: Option<String>,

    /// Array of embedding vectors (one per input text)
    #[serde(default)]
    pub embeddings: Vec<Vec<f64>>,

    /// Total time spent generating embeddings in nanoseconds
    #[serde(default)]
    pub total_duration: Option<i64>,

    /// Time spent loading the model in nanoseconds
    #[serde(default)]
    pub load_duration: Option<i64>,

    /// Number of input tokens processed
    #[serde(default)]
    pub prompt_eval_count: Option<i32>,
}
```

## Implementation Strategy

### Step 1: Create EmbedInput Type

**Location:** `src/primitives/embed_input.rs`

```rust
//! Embed input primitive type

use serde::{Deserialize, Serialize};

/// Input for embedding generation
///
/// Can be a single text string or an array of text strings.
/// Uses untagged serde deserialization to accept either format.
///
/// # Examples
///
/// ```
/// use ollama_oxide::EmbedInput;
///
/// // Single text input
/// let single = EmbedInput::Single("Hello, world!".to_string());
///
/// // Multiple text inputs
/// let multiple = EmbedInput::Multiple(vec![
///     "First text".to_string(),
///     "Second text".to_string(),
/// ]);
/// ```
///
/// # JSON Serialization
///
/// Single input serializes as a string:
/// ```json
/// "Hello, world!"
/// ```
///
/// Multiple inputs serialize as an array:
/// ```json
/// ["First text", "Second text"]
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbedInput {
    /// Single text input
    Single(String),
    /// Multiple text inputs
    Multiple(Vec<String>),
}

impl EmbedInput {
    /// Create a single text input
    ///
    /// # Arguments
    ///
    /// * `text` - The text to embed
    ///
    /// # Example
    ///
    /// ```
    /// use ollama_oxide::EmbedInput;
    ///
    /// let input = EmbedInput::single("Hello, world!");
    /// ```
    pub fn single(text: impl Into<String>) -> Self {
        Self::Single(text.into())
    }

    /// Create a multiple text input
    ///
    /// # Arguments
    ///
    /// * `texts` - Iterator of texts to embed
    ///
    /// # Example
    ///
    /// ```
    /// use ollama_oxide::EmbedInput;
    ///
    /// let input = EmbedInput::multiple(["First", "Second", "Third"]);
    /// ```
    pub fn multiple<I, S>(texts: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::Multiple(texts.into_iter().map(|s| s.into()).collect())
    }

    /// Get the number of texts in the input
    ///
    /// # Example
    ///
    /// ```
    /// use ollama_oxide::EmbedInput;
    ///
    /// let single = EmbedInput::single("Hello");
    /// assert_eq!(single.len(), 1);
    ///
    /// let multiple = EmbedInput::multiple(["A", "B", "C"]);
    /// assert_eq!(multiple.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        match self {
            Self::Single(_) => 1,
            Self::Multiple(v) => v.len(),
        }
    }

    /// Check if the input is empty
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Single(s) => s.is_empty(),
            Self::Multiple(v) => v.is_empty(),
        }
    }
}

impl From<String> for EmbedInput {
    fn from(s: String) -> Self {
        Self::Single(s)
    }
}

impl From<&str> for EmbedInput {
    fn from(s: &str) -> Self {
        Self::Single(s.to_string())
    }
}

impl From<Vec<String>> for EmbedInput {
    fn from(v: Vec<String>) -> Self {
        Self::Multiple(v)
    }
}

impl<const N: usize> From<[&str; N]> for EmbedInput {
    fn from(arr: [&str; N]) -> Self {
        Self::Multiple(arr.iter().map(|s| s.to_string()).collect())
    }
}
```

### Step 2: Create ModelOptions Type

**Location:** `src/primitives/model_options.rs`

```rust
//! Model options primitive type

use serde::{Deserialize, Serialize};

/// Runtime options that control model behavior
///
/// These options can be used to customize embedding generation.
/// All fields are optional and will use model defaults if not specified.
///
/// # Example
///
/// ```
/// use ollama_oxide::ModelOptions;
///
/// let options = ModelOptions::default()
///     .with_temperature(0.7)
///     .with_num_ctx(4096);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ModelOptions {
    /// Random seed for reproducible outputs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,

    /// Controls randomness in generation (higher = more random)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Limits next token selection to the K most likely
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,

    /// Cumulative probability threshold for nucleus sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Minimum probability threshold for token selection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_p: Option<f32>,

    /// Context length size (number of tokens)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_ctx: Option<i32>,

    /// Maximum number of tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<i32>,
}

impl ModelOptions {
    /// Create empty options (all defaults)
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the random seed
    pub fn with_seed(mut self, seed: i64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set the temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set the top_k value
    pub fn with_top_k(mut self, top_k: i32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    /// Set the top_p value
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Set the min_p value
    pub fn with_min_p(mut self, min_p: f32) -> Self {
        self.min_p = Some(min_p);
        self
    }

    /// Set the context length
    pub fn with_num_ctx(mut self, num_ctx: i32) -> Self {
        self.num_ctx = Some(num_ctx);
        self
    }

    /// Set the max tokens to generate
    pub fn with_num_predict(mut self, num_predict: i32) -> Self {
        self.num_predict = Some(num_predict);
        self
    }

    /// Check if any options are set
    pub fn is_empty(&self) -> bool {
        self.seed.is_none()
            && self.temperature.is_none()
            && self.top_k.is_none()
            && self.top_p.is_none()
            && self.min_p.is_none()
            && self.num_ctx.is_none()
            && self.num_predict.is_none()
    }
}
```

### Step 3: Create EmbedRequest Type

**Location:** `src/primitives/embed_request.rs`

```rust
//! Embed request primitive type

use serde::{Deserialize, Serialize};

use super::{EmbedInput, ModelOptions};

/// Request body for POST /api/embed endpoint
///
/// Generates vector embeddings for the provided input text(s).
///
/// # Examples
///
/// Basic single text request:
/// ```
/// use ollama_oxide::EmbedRequest;
///
/// let request = EmbedRequest::new("nomic-embed-text", "Hello, world!");
/// ```
///
/// Multiple texts:
/// ```
/// use ollama_oxide::{EmbedRequest, EmbedInput};
///
/// let request = EmbedRequest::new(
///     "nomic-embed-text",
///     EmbedInput::multiple(["First text", "Second text"])
/// );
/// ```
///
/// With options:
/// ```
/// use ollama_oxide::{EmbedRequest, ModelOptions};
///
/// let request = EmbedRequest::new("nomic-embed-text", "Hello")
///     .with_truncate(true)
///     .with_dimensions(768)
///     .with_keep_alive("5m");
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbedRequest {
    /// Name of the embedding model to use
    pub model: String,

    /// Text or array of texts to generate embeddings for
    pub input: EmbedInput,

    /// If true, truncate inputs that exceed context window (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncate: Option<bool>,

    /// Number of dimensions for the embedding (model-specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<i32>,

    /// How long to keep the model loaded (e.g., "5m", "1h")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<String>,

    /// Runtime options for embedding generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ModelOptions>,
}

impl EmbedRequest {
    /// Create a new embed request
    ///
    /// # Arguments
    ///
    /// * `model` - Name of the embedding model to use
    /// * `input` - Text or array of texts to embed
    ///
    /// # Example
    ///
    /// ```
    /// use ollama_oxide::EmbedRequest;
    ///
    /// let request = EmbedRequest::new("nomic-embed-text", "Hello, world!");
    /// ```
    pub fn new(model: impl Into<String>, input: impl Into<EmbedInput>) -> Self {
        Self {
            model: model.into(),
            input: input.into(),
            truncate: None,
            dimensions: None,
            keep_alive: None,
            options: None,
        }
    }

    /// Set the truncate option
    ///
    /// If true, truncate inputs that exceed the model's context window.
    /// If false, returns an error for inputs that are too long.
    pub fn with_truncate(mut self, truncate: bool) -> Self {
        self.truncate = Some(truncate);
        self
    }

    /// Set the embedding dimensions
    ///
    /// Some models support generating embeddings with different dimensions.
    pub fn with_dimensions(mut self, dimensions: i32) -> Self {
        self.dimensions = Some(dimensions);
        self
    }

    /// Set the keep_alive duration
    ///
    /// Controls how long the model stays loaded in memory (e.g., "5m", "1h").
    pub fn with_keep_alive(mut self, keep_alive: impl Into<String>) -> Self {
        self.keep_alive = Some(keep_alive.into());
        self
    }

    /// Set the model options
    pub fn with_options(mut self, options: ModelOptions) -> Self {
        self.options = Some(options);
        self
    }
}
```

### Step 4: Create EmbedResponse Type

**Location:** `src/primitives/embed_response.rs`

```rust
//! Embed response primitive type

use serde::{Deserialize, Serialize};

/// Response from POST /api/embed endpoint
///
/// Contains the generated embeddings and timing information.
///
/// # Example Response
///
/// ```json
/// {
///   "model": "nomic-embed-text",
///   "embeddings": [[0.010071, -0.001759, 0.050072, ...]],
///   "total_duration": 14143917,
///   "load_duration": 1019500,
///   "prompt_eval_count": 8
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EmbedResponse {
    /// Model that produced the embeddings
    #[serde(default)]
    pub model: Option<String>,

    /// Array of embedding vectors (one per input text)
    ///
    /// Each inner vector contains the embedding dimensions (e.g., 768 or 1024 floats).
    #[serde(default)]
    pub embeddings: Vec<Vec<f64>>,

    /// Total time spent generating embeddings in nanoseconds
    #[serde(default)]
    pub total_duration: Option<i64>,

    /// Time spent loading the model in nanoseconds
    #[serde(default)]
    pub load_duration: Option<i64>,

    /// Number of input tokens processed
    #[serde(default)]
    pub prompt_eval_count: Option<i32>,
}

impl EmbedResponse {
    /// Get the number of embeddings returned
    ///
    /// This corresponds to the number of input texts provided.
    pub fn len(&self) -> usize {
        self.embeddings.len()
    }

    /// Check if there are no embeddings
    pub fn is_empty(&self) -> bool {
        self.embeddings.is_empty()
    }

    /// Get the dimension of the embeddings
    ///
    /// Returns None if there are no embeddings.
    pub fn dimensions(&self) -> Option<usize> {
        self.embeddings.first().map(|e| e.len())
    }

    /// Get the first embedding (convenience for single-input requests)
    ///
    /// Returns None if there are no embeddings.
    pub fn first_embedding(&self) -> Option<&Vec<f64>> {
        self.embeddings.first()
    }

    /// Get total duration in milliseconds (convenience method)
    ///
    /// Converts from nanoseconds to milliseconds.
    pub fn total_duration_ms(&self) -> Option<f64> {
        self.total_duration.map(|ns| ns as f64 / 1_000_000.0)
    }

    /// Get load duration in milliseconds (convenience method)
    ///
    /// Converts from nanoseconds to milliseconds.
    pub fn load_duration_ms(&self) -> Option<f64> {
        self.load_duration.map(|ns| ns as f64 / 1_000_000.0)
    }
}
```

### Step 5: Update primitives/mod.rs

**Location:** `src/primitives/mod.rs`

Add module declarations and re-exports:

```rust
mod embed_input;
mod embed_request;
mod embed_response;
mod model_options;

pub use embed_input::EmbedInput;
pub use embed_request::EmbedRequest;
pub use embed_response::EmbedResponse;
pub use model_options::ModelOptions;
```

### Step 6: Update lib.rs Re-exports

**Location:** `src/lib.rs`

Add new types to public re-exports:

```rust
#[cfg(feature = "primitives")]
pub use primitives::{
    CopyRequest, DeleteRequest, EmbedInput, EmbedRequest, EmbedResponse, ListResponse,
    ModelDetails, ModelOptions, ModelSummary, PsResponse, RunningModel, ShowModelDetails,
    ShowRequest, ShowResponse, VersionResponse,
};
```

And update the prelude:

```rust
pub mod prelude {
    // ... existing exports ...

    #[cfg(feature = "primitives")]
    pub use crate::{
        CopyRequest, DeleteRequest, EmbedInput, EmbedRequest, EmbedResponse, ListResponse,
        ModelDetails, ModelOptions, ModelSummary, PsResponse, RunningModel, ShowModelDetails,
        ShowRequest, ShowResponse, VersionResponse,
    };
}
```

### Step 7: Add API Methods

#### 7.1 Async API

**Location:** `src/http/api_async.rs`

Add import:
```rust
use crate::{
    CopyRequest, DeleteRequest, EmbedRequest, EmbedResponse, ListResponse, PsResponse, Result,
    ShowRequest, ShowResponse, VersionResponse,
};
```

Add to `OllamaApiAsync` trait:

```rust
/// Generate embeddings for text (async)
///
/// Creates vector embeddings representing the input text(s).
/// Embeddings are useful for semantic search, similarity comparison,
/// and machine learning tasks.
///
/// # Arguments
///
/// * `request` - Embed request containing model name and input text(s)
///
/// # Errors
///
/// Returns an error if:
/// - Model doesn't exist (404)
/// - Input exceeds context window and truncate is false
/// - Network request fails
/// - Maximum retry attempts exceeded
///
/// # Examples
///
/// Single text embedding:
/// ```no_run
/// use ollama_oxide::{OllamaClient, OllamaApiAsync, EmbedRequest};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = OllamaClient::default()?;
/// let request = EmbedRequest::new("nomic-embed-text", "Hello, world!");
/// let response = client.embed(&request).await?;
/// println!("Embedding dimensions: {:?}", response.dimensions());
/// # Ok(())
/// # }
/// ```
///
/// Multiple text embeddings:
/// ```no_run
/// use ollama_oxide::{OllamaClient, OllamaApiAsync, EmbedRequest, EmbedInput};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = OllamaClient::default()?;
/// let request = EmbedRequest::new(
///     "nomic-embed-text",
///     EmbedInput::multiple(["First text", "Second text"])
/// );
/// let response = client.embed(&request).await?;
/// println!("Got {} embeddings", response.len());
/// # Ok(())
/// # }
/// ```
async fn embed(&self, request: &EmbedRequest) -> Result<EmbedResponse>;
```

Add implementation:

```rust
async fn embed(&self, request: &EmbedRequest) -> Result<EmbedResponse> {
    let url = self.config.url(Endpoints::EMBED);
    self.post_with_retry(&url, request).await
}
```

#### 7.2 Sync API

**Location:** `src/http/api_sync.rs`

Add import:
```rust
use crate::{
    CopyRequest, DeleteRequest, EmbedRequest, EmbedResponse, ListResponse, PsResponse, Result,
    ShowRequest, ShowResponse, VersionResponse,
};
```

Add to `OllamaApiSync` trait:

```rust
/// Generate embeddings for text (blocking)
///
/// Creates vector embeddings representing the input text(s).
/// This method blocks the current thread until the request completes.
///
/// # Arguments
///
/// * `request` - Embed request containing model name and input text(s)
///
/// # Errors
///
/// Returns an error if:
/// - Model doesn't exist (404)
/// - Input exceeds context window and truncate is false
/// - Network request fails
/// - Maximum retry attempts exceeded
///
/// # Examples
///
/// ```no_run
/// use ollama_oxide::{OllamaClient, OllamaApiSync, EmbedRequest};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = OllamaClient::default()?;
/// let request = EmbedRequest::new("nomic-embed-text", "Hello, world!");
/// let response = client.embed_blocking(&request)?;
/// println!("Embedding dimensions: {:?}", response.dimensions());
/// # Ok(())
/// # }
/// ```
fn embed_blocking(&self, request: &EmbedRequest) -> Result<EmbedResponse>;
```

Add implementation:

```rust
fn embed_blocking(&self, request: &EmbedRequest) -> Result<EmbedResponse> {
    let url = self.config.url(Endpoints::EMBED);
    self.post_blocking_with_retry(&url, request)
}
```

### Step 8: Add Unit Tests

**Location:** `tests/client_embed_tests.rs`

```rust
//! Tests for embed API methods (POST /api/embed)

use ollama_oxide::{
    ClientConfig, EmbedInput, EmbedRequest, EmbedResponse, ModelOptions, OllamaApiAsync,
    OllamaApiSync, OllamaClient,
};
use std::time::Duration;

// ============================================================================
// EmbedInput Type Tests
// ============================================================================

#[test]
fn test_embed_input_single_serialization() {
    let input = EmbedInput::single("Hello, world!");
    let json = serde_json::to_string(&input).unwrap();
    assert_eq!(json, r#""Hello, world!""#);
}

#[test]
fn test_embed_input_multiple_serialization() {
    let input = EmbedInput::multiple(["First", "Second"]);
    let json = serde_json::to_string(&input).unwrap();
    assert_eq!(json, r#"["First","Second"]"#);
}

#[test]
fn test_embed_input_single_deserialization() {
    let json = r#""Hello, world!""#;
    let input: EmbedInput = serde_json::from_str(json).unwrap();
    assert_eq!(input, EmbedInput::Single("Hello, world!".to_string()));
}

#[test]
fn test_embed_input_multiple_deserialization() {
    let json = r#"["First","Second"]"#;
    let input: EmbedInput = serde_json::from_str(json).unwrap();
    assert_eq!(
        input,
        EmbedInput::Multiple(vec!["First".to_string(), "Second".to_string()])
    );
}

#[test]
fn test_embed_input_len() {
    let single = EmbedInput::single("Hello");
    assert_eq!(single.len(), 1);

    let multiple = EmbedInput::multiple(["A", "B", "C"]);
    assert_eq!(multiple.len(), 3);
}

#[test]
fn test_embed_input_from_string() {
    let input: EmbedInput = "Hello".into();
    assert_eq!(input, EmbedInput::Single("Hello".to_string()));
}

#[test]
fn test_embed_input_from_vec() {
    let input: EmbedInput = vec!["A".to_string(), "B".to_string()].into();
    assert_eq!(
        input,
        EmbedInput::Multiple(vec!["A".to_string(), "B".to_string()])
    );
}

// ============================================================================
// ModelOptions Type Tests
// ============================================================================

#[test]
fn test_model_options_builder() {
    let options = ModelOptions::new()
        .with_temperature(0.7)
        .with_num_ctx(4096)
        .with_seed(42);

    assert_eq!(options.temperature, Some(0.7));
    assert_eq!(options.num_ctx, Some(4096));
    assert_eq!(options.seed, Some(42));
}

#[test]
fn test_model_options_serialization() {
    let options = ModelOptions::new().with_temperature(0.7);
    let json = serde_json::to_string(&options).unwrap();
    assert!(json.contains("\"temperature\":0.7"));
    // Should not include unset fields
    assert!(!json.contains("seed"));
}

#[test]
fn test_model_options_is_empty() {
    let empty = ModelOptions::new();
    assert!(empty.is_empty());

    let with_temp = ModelOptions::new().with_temperature(0.5);
    assert!(!with_temp.is_empty());
}

// ============================================================================
// EmbedRequest Type Tests
// ============================================================================

#[test]
fn test_embed_request_new() {
    let request = EmbedRequest::new("nomic-embed-text", "Hello, world!");
    assert_eq!(request.model, "nomic-embed-text");
    assert_eq!(request.input, EmbedInput::Single("Hello, world!".to_string()));
}

#[test]
fn test_embed_request_with_options() {
    let request = EmbedRequest::new("nomic-embed-text", "Hello")
        .with_truncate(true)
        .with_dimensions(768)
        .with_keep_alive("5m");

    assert_eq!(request.truncate, Some(true));
    assert_eq!(request.dimensions, Some(768));
    assert_eq!(request.keep_alive, Some("5m".to_string()));
}

#[test]
fn test_embed_request_serialization_minimal() {
    let request = EmbedRequest::new("nomic-embed-text", "Hello");
    let json = serde_json::to_string(&request).unwrap();

    assert!(json.contains(r#""model":"nomic-embed-text""#));
    assert!(json.contains(r#""input":"Hello""#));
    // Should not include optional fields
    assert!(!json.contains("truncate"));
    assert!(!json.contains("dimensions"));
}

#[test]
fn test_embed_request_serialization_full() {
    let request = EmbedRequest::new("nomic-embed-text", "Hello")
        .with_truncate(true)
        .with_dimensions(512)
        .with_keep_alive("10m")
        .with_options(ModelOptions::new().with_temperature(0.5));

    let json = serde_json::to_string(&request).unwrap();

    assert!(json.contains(r#""truncate":true"#));
    assert!(json.contains(r#""dimensions":512"#));
    assert!(json.contains(r#""keep_alive":"10m""#));
    assert!(json.contains(r#""temperature":0.5"#));
}

#[test]
fn test_embed_request_multiple_inputs() {
    let request = EmbedRequest::new(
        "nomic-embed-text",
        EmbedInput::multiple(["First", "Second"]),
    );
    let json = serde_json::to_string(&request).unwrap();

    assert!(json.contains(r#""input":["First","Second"]"#));
}

// ============================================================================
// EmbedResponse Type Tests
// ============================================================================

#[test]
fn test_embed_response_deserialization() {
    let json = r#"{
        "model": "nomic-embed-text",
        "embeddings": [[0.1, 0.2, 0.3], [0.4, 0.5, 0.6]],
        "total_duration": 14143917,
        "load_duration": 1019500,
        "prompt_eval_count": 8
    }"#;

    let response: EmbedResponse = serde_json::from_str(json).unwrap();

    assert_eq!(response.model, Some("nomic-embed-text".to_string()));
    assert_eq!(response.embeddings.len(), 2);
    assert_eq!(response.embeddings[0], vec![0.1, 0.2, 0.3]);
    assert_eq!(response.total_duration, Some(14143917));
    assert_eq!(response.prompt_eval_count, Some(8));
}

#[test]
fn test_embed_response_dimensions() {
    let response = EmbedResponse {
        embeddings: vec![vec![0.1, 0.2, 0.3, 0.4]],
        ..Default::default()
    };

    assert_eq!(response.dimensions(), Some(4));
}

#[test]
fn test_embed_response_first_embedding() {
    let response = EmbedResponse {
        embeddings: vec![vec![0.1, 0.2], vec![0.3, 0.4]],
        ..Default::default()
    };

    assert_eq!(response.first_embedding(), Some(&vec![0.1, 0.2]));
}

#[test]
fn test_embed_response_duration_conversion() {
    let response = EmbedResponse {
        total_duration: Some(1_000_000), // 1ms in nanoseconds
        load_duration: Some(500_000),    // 0.5ms in nanoseconds
        ..Default::default()
    };

    assert!((response.total_duration_ms().unwrap() - 1.0).abs() < 0.001);
    assert!((response.load_duration_ms().unwrap() - 0.5).abs() < 0.001);
}

// ============================================================================
// Async API Tests
// ============================================================================

#[tokio::test]
async fn test_embed_async_success() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/embed")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "model": "nomic-embed-text",
            "input": "Hello, world!"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "model": "nomic-embed-text",
            "embeddings": [[0.1, 0.2, 0.3]],
            "total_duration": 1000000,
            "prompt_eval_count": 3
        }"#)
        .create_async()
        .await;

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 0,
    };

    let client = OllamaClient::new(config).unwrap();
    let request = EmbedRequest::new("nomic-embed-text", "Hello, world!");
    let response = client.embed(&request).await.unwrap();

    assert_eq!(response.model, Some("nomic-embed-text".to_string()));
    assert_eq!(response.embeddings.len(), 1);
    assert_eq!(response.embeddings[0], vec![0.1, 0.2, 0.3]);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_embed_async_multiple_inputs() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/embed")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "model": "nomic-embed-text",
            "input": ["First", "Second"]
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "model": "nomic-embed-text",
            "embeddings": [[0.1, 0.2], [0.3, 0.4]]
        }"#)
        .create_async()
        .await;

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 0,
    };

    let client = OllamaClient::new(config).unwrap();
    let request = EmbedRequest::new("nomic-embed-text", EmbedInput::multiple(["First", "Second"]));
    let response = client.embed(&request).await.unwrap();

    assert_eq!(response.embeddings.len(), 2);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_embed_async_model_not_found() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/embed")
        .with_status(404)
        .create_async()
        .await;

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 0,
    };

    let client = OllamaClient::new(config).unwrap();
    let request = EmbedRequest::new("nonexistent-model", "Hello");
    let result = client.embed(&request).await;

    assert!(result.is_err());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_embed_async_retry_on_server_error() {
    let mut server = mockito::Server::new_async().await;

    let mock_fail = server
        .mock("POST", "/api/embed")
        .with_status(500)
        .expect(1)
        .create_async()
        .await;

    let mock_success = server
        .mock("POST", "/api/embed")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"embeddings": [[0.1]]}"#)
        .expect(1)
        .create_async()
        .await;

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 1,
    };

    let client = OllamaClient::new(config).unwrap();
    let request = EmbedRequest::new("model", "Hello");
    let result = client.embed(&request).await;

    assert!(result.is_ok());

    mock_fail.assert_async().await;
    mock_success.assert_async().await;
}

// ============================================================================
// Sync API Tests
// ============================================================================

#[test]
fn test_embed_sync_success() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("POST", "/api/embed")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "model": "nomic-embed-text",
            "embeddings": [[0.1, 0.2, 0.3]]
        }"#)
        .create();

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 0,
    };

    let client = OllamaClient::new(config).unwrap();
    let request = EmbedRequest::new("nomic-embed-text", "Hello");
    let response = client.embed_blocking(&request).unwrap();

    assert_eq!(response.embeddings.len(), 1);

    mock.assert();
}

#[test]
fn test_embed_sync_model_not_found() {
    let mut server = mockito::Server::new();

    let mock = server.mock("POST", "/api/embed").with_status(404).create();

    let config = ClientConfig {
        base_url: server.url(),
        timeout: Duration::from_secs(5),
        max_retries: 0,
    };

    let client = OllamaClient::new(config).unwrap();
    let request = EmbedRequest::new("nonexistent", "Hello");
    let result = client.embed_blocking(&request);

    assert!(result.is_err());

    mock.assert();
}

// ============================================================================
// Type Safety Tests
// ============================================================================

#[test]
fn test_embed_input_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<EmbedInput>();
}

#[test]
fn test_model_options_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ModelOptions>();
}

#[test]
fn test_embed_request_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<EmbedRequest>();
}

#[test]
fn test_embed_response_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<EmbedResponse>();
}
```

### Step 9: Add Examples

**Location:** `examples/embed_async.rs`

```rust
//! Example: Generate embeddings (async)
//!
//! This example demonstrates how to generate text embeddings using
//! an embedding model.
//!
//! Run with: cargo run --example embed_async
//!
//! Note: Requires a running Ollama server with an embedding model
//! (e.g., nomic-embed-text, all-minilm, mxbai-embed-large)

use ollama_oxide::{EmbedInput, EmbedRequest, OllamaApiAsync, OllamaClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with default configuration
    let client = OllamaClient::default()?;

    // Model to use (change to your installed embedding model)
    let model = "nomic-embed-text";

    println!("Generating embeddings with model: {}", model);

    // Example 1: Single text embedding
    println!("\n--- Single Text Embedding ---");
    let request = EmbedRequest::new(model, "The quick brown fox jumps over the lazy dog.");
    let response = client.embed(&request).await?;

    println!("Model: {:?}", response.model);
    println!("Embedding dimensions: {:?}", response.dimensions());
    println!("Total duration: {:?} ms", response.total_duration_ms());
    println!("Tokens processed: {:?}", response.prompt_eval_count);

    if let Some(embedding) = response.first_embedding() {
        println!("First 5 values: {:?}", &embedding[..5.min(embedding.len())]);
    }

    // Example 2: Multiple text embeddings
    println!("\n--- Multiple Text Embeddings ---");
    let texts = vec![
        "Artificial intelligence is transforming industries.",
        "Machine learning models require training data.",
        "Neural networks can learn complex patterns.",
    ];

    let request = EmbedRequest::new(model, EmbedInput::multiple(texts.clone()));
    let response = client.embed(&request).await?;

    println!("Generated {} embeddings", response.len());
    for (i, embedding) in response.embeddings.iter().enumerate() {
        println!(
            "  Text {}: '{}...' -> {} dimensions",
            i + 1,
            &texts[i][..30.min(texts[i].len())],
            embedding.len()
        );
    }

    // Example 3: With options
    println!("\n--- With Options ---");
    let request = EmbedRequest::new(model, "Hello, world!")
        .with_truncate(true)
        .with_keep_alive("5m");

    let response = client.embed(&request).await?;
    println!("Embedding with options: {:?} dimensions", response.dimensions());

    println!("\nDone!");

    Ok(())
}
```

**Location:** `examples/embed_sync.rs`

```rust
//! Example: Generate embeddings (sync)
//!
//! This example demonstrates how to generate text embeddings using
//! the blocking API.
//!
//! Run with: cargo run --example embed_sync
//!
//! Note: Requires a running Ollama server with an embedding model

use ollama_oxide::{EmbedRequest, OllamaApiSync, OllamaClient};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with default configuration
    let client = OllamaClient::default()?;

    // Model to use (change to your installed embedding model)
    let model = "nomic-embed-text";

    println!("Generating embeddings with model: {}", model);

    // Generate embedding
    let request = EmbedRequest::new(model, "Hello, world!");
    let response = client.embed_blocking(&request)?;

    println!("Model: {:?}", response.model);
    println!("Embedding dimensions: {:?}", response.dimensions());
    println!("Total duration: {:?} ms", response.total_duration_ms());

    if let Some(embedding) = response.first_embedding() {
        println!("First 5 values: {:?}", &embedding[..5.min(embedding.len())]);
    }

    println!("\nDone!");

    Ok(())
}
```

## File Changes Summary

### New Files
| File | Description |
|------|-------------|
| `src/primitives/embed_input.rs` | EmbedInput enum for single/multiple text input |
| `src/primitives/model_options.rs` | ModelOptions struct for runtime options |
| `src/primitives/embed_request.rs` | EmbedRequest struct with builder methods |
| `src/primitives/embed_response.rs` | EmbedResponse struct with helper methods |
| `tests/client_embed_tests.rs` | Unit tests with mocking |
| `examples/embed_async.rs` | Async usage example |
| `examples/embed_sync.rs` | Sync usage example |

### Modified Files
| File | Changes |
|------|---------|
| `src/primitives/mod.rs` | Add module declarations and re-exports |
| `src/lib.rs` | Add new types to public re-exports and prelude |
| `src/http/api_async.rs` | Add `embed()` method to trait and implementation |
| `src/http/api_sync.rs` | Add `embed_blocking()` method to trait and implementation |

## Testing Checklist

- [ ] EmbedInput serialization/deserialization tests pass
- [ ] ModelOptions builder pattern tests pass
- [ ] EmbedRequest serialization tests pass
- [ ] EmbedResponse deserialization tests pass
- [ ] All types implement Send + Sync
- [ ] Async embed tests pass
- [ ] Sync embed_blocking tests pass
- [ ] Retry logic tests pass
- [ ] Error handling tests pass (404 model not found)
- [ ] Examples build successfully
- [ ] `cargo build` succeeds
- [ ] `cargo test` passes
- [ ] `cargo clippy` has no warnings
- [ ] `cargo fmt` applied

## Implementation Order

1. Create `src/primitives/embed_input.rs` with EmbedInput enum
2. Create `src/primitives/model_options.rs` with ModelOptions struct
3. Create `src/primitives/embed_request.rs` with EmbedRequest struct
4. Create `src/primitives/embed_response.rs` with EmbedResponse struct
5. Update `src/primitives/mod.rs` with new exports
6. Update `src/lib.rs` with public re-exports and prelude
7. Add async method to `src/http/api_async.rs` (trait + impl)
8. Add sync method to `src/http/api_sync.rs` (trait + impl)
9. Create unit tests in `tests/client_embed_tests.rs`
10. Create async example
11. Create sync example
12. Run full test suite (`cargo test`)
13. Run linter (`cargo clippy`)
14. Apply formatting (`cargo fmt`)
15. Update DEV_NOTES.md
16. Update definition.md checklist

## Comparison with POST /api/show

| Aspect | POST /api/show | POST /api/embed |
|--------|----------------|-----------------|
| HTTP Method | POST | POST |
| Request Type | ShowRequest (model, verbose) | EmbedRequest (model, input, options) |
| Response Type | ShowResponse (metadata) | EmbedResponse (embeddings, metrics) |
| Input Flexibility | Single model name | Single string or array of strings |
| Response Complexity | Optional nested fields | Nested arrays (Vec<Vec<f64>>) |
| Helper Method | post_with_retry | post_with_retry |
| New Types Needed | 2 (ShowRequest, ShowResponse) | 4 (EmbedInput, ModelOptions, EmbedRequest, EmbedResponse) |

## Notes

- The `Endpoints::EMBED` constant already exists in `endpoints.rs`
- Uses existing `post_with_retry` helper method (no new client helpers needed)
- `EmbedInput` uses `#[serde(untagged)]` for flexible JSON input format
- `ModelOptions` is reusable for future endpoints (generate, chat, etc.)
- All types must implement `Send + Sync` for thread safety
- Response embedding vectors use `f64` for precision
- Examples use "nomic-embed-text" model (popular open embedding model)
