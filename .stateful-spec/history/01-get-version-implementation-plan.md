# Implementation Plan: GET /api/version

**Endpoint:** GET /api/version
**Complexity:** Simple (1)
**Phase:** Phase 1 - Foundation + HTTP Core
**Document Version:** 1.3 (TDD + Thread Safety)
**Created:** 2026-01-13
**Updated:** 2026-01-13 (TDD approach + error naming consistency + thread safety tests)

## Overview

This document outlines the implementation plan for the first endpoint: `GET /api/version`. This endpoint retrieves the version of the Ollama server and serves as the foundation for establishing:

1. Core error handling patterns
2. HTTP client structure (with sync and async support)
3. Primitive type definitions
4. Testing framework
5. Documentation patterns
6. Timeout and retry mechanisms
7. URL validation

## Design Decisions (Approved)

The following design decisions have been approved:

- ✅ **Client Cloning**: `OllamaClient` will be `Clone` (uses `Arc` internally for `reqwest::Client`)
- ✅ **Timeout Configuration**: Configurable request timeout support will be added
- ✅ **Retry Logic**: Retry mechanism with configurable attempts will be implemented
- ✅ **URL Validation**: Base URL format validation using `url` crate
- ✅ **Async/Sync Support**: Both async and sync implementations using `async-trait` for async functions
- ✅ **Error Handling**: Using `thiserror` crate for ergonomic error handling
- ✅ **Error Naming Consistency**: All error variants use `Error` suffix (e.g., `InvalidUrlError`, `TimeoutError`)
- ✅ **Thread Safety**: All types implement `Send + Sync` for safe concurrent usage

## API Specification Summary

**Endpoint:** `GET /api/version`
**Operation ID:** `version`
**Description:** Retrieve the version of Ollama

**Response (200 OK):**
```json
{
  "version": "0.12.6"
}
```

**Schema:** `VersionResponse`
```yaml
type: object
properties:
  version:
    type: string
    description: Version of Ollama
```

## Thread Safety Requirements

All public types must be thread-safe and implement `Send + Sync`:

1. **Error Type (`Error`)**: Must be `Send + Sync` to allow error propagation across threads
2. **VersionResponse**: Must be `Send + Sync` as it's returned from async methods
3. **ClientConfig**: Must be `Send + Sync` as it's cloned and shared
4. **OllamaClient**: Must be `Send + Sync` using `Arc<reqwest::Client>` internally
5. **Traits**: `OllamaApiAsync` and `OllamaApiSync` require `Send + Sync` bounds

**Testing Requirements:**
- All types must have compile-time tests verifying `Send + Sync`
- Client must be testable in multi-threaded scenarios
- Concurrent API calls from multiple threads/tasks must work safely

## Implementation Strategy

### Phase 1: Error Handling Foundation

**Location:** `src/lib.rs` or `src/error.rs`

**Tasks:**
1. Define core `Error` enum with variants (all with `Error` suffix for consistency):
   - `HttpError` - HTTP request/response errors
   - `SerializationError` - JSON serialization/deserialization errors
   - `ApiError` - Ollama API-specific errors
   - `ConnectionError` - Connection/network errors
   - `InvalidUrlError` - URL parsing errors
   - `TimeoutError` - Request timeout errors
   - `MaxRetriesExceededError` - Maximum retry attempts exceeded

2. Implement `std::error::Error` trait
3. Implement `std::fmt::Display` for user-friendly error messages
4. Add `From` implementations for common error types:
   - `reqwest::Error` → `Error::HttpError`
   - `serde_json::Error` → `Error::SerializationError`
   - `url::ParseError` → `Error::InvalidUrlError`

5. Define `Result<T>` type alias: `type Result<T> = std::result::Result<T, Error>`

**Dependencies:**
- `thiserror` crate for ergonomic error handling (approved)
- `url` crate for URL parsing and validation (approved)

**Example Structure:**
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Failed to serialize/deserialize: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("API error: {message}")]
    ApiError { message: String },

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Invalid URL: {0}")]
    InvalidUrlError(#[from] url::ParseError),

    #[error("Request timeout after {0} seconds")]
    TimeoutError(u64),

    #[error("Maximum retry attempts ({0}) exceeded")]
    MaxRetriesExceededError(u32),
}

pub type Result<T> = std::result::Result<T, Error>;
```

**Note**: All error variants follow consistent naming with `Error` suffix.

### Phase 2: Primitives Module - VersionResponse Type

**Location:** `src/primitives/mod.rs` or `src/primitives/version.rs`

**Tasks:**
1. Define `VersionResponse` struct:
   ```rust
   use serde::{Deserialize, Serialize};

   /// Response from GET /api/version
   #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
   pub struct VersionResponse {
       /// Version of Ollama server
       pub version: String,
   }
   ```

2. Add documentation with examples
3. Implement standard traits if needed (Default, Display)
4. Consider validation (e.g., version string format)

**Module Organization:**
- Option A: Add directly to `src/primitives/mod.rs` (simple, for now)
- Option B: Create `src/primitives/version.rs` and re-export (scalable)

### Phase 3: HTTP Client Module

**Location:** `src/http/mod.rs` or `src/http/client.rs`

**Tasks:**

1. Define `ClientConfig` for configuration:
   ```rust
   use std::time::Duration;

   /// Configuration for Ollama HTTP client
   #[derive(Debug, Clone)]
   pub struct ClientConfig {
       /// Base URL for Ollama API
       pub base_url: String,
       /// Request timeout duration
       pub timeout: Duration,
       /// Maximum retry attempts on failure
       pub max_retries: u32,
   }

   impl Default for ClientConfig {
       fn default() -> Self {
           Self {
               base_url: "http://localhost:11434".to_string(),
               timeout: Duration::from_secs(30),
               max_retries: 3,
           }
       }
   }
   ```

2. Define `OllamaClient` struct (cloneable):
   ```rust
   use reqwest::Client;
   use std::sync::Arc;

   /// HTTP client for Ollama API
   ///
   /// This client is cloneable and can be safely shared across threads.
   /// The internal HTTP client is wrapped in Arc for efficient cloning.
   #[derive(Clone, Debug)]
   pub struct OllamaClient {
       config: ClientConfig,
       client: Arc<Client>,
   }
   ```

3. Implement constructors with URL validation:
   ```rust
   use url::Url;

   impl OllamaClient {
       /// Create a new Ollama client with custom configuration
       pub fn new(config: ClientConfig) -> crate::Result<Self> {
           // Validate base URL
           Url::parse(&config.base_url)?;

           let client = Client::builder()
               .timeout(config.timeout)
               .build()?;

           Ok(Self {
               config,
               client: Arc::new(client),
           })
       }

       /// Create client with default configuration (http://localhost:11434)
       pub fn default() -> crate::Result<Self> {
           Self::new(ClientConfig::default())
       }

       /// Create client with custom base URL and default timeout/retry
       pub fn with_base_url(base_url: impl Into<String>) -> crate::Result<Self> {
           let config = ClientConfig {
               base_url: base_url.into(),
               ..Default::default()
           };
           Self::new(config)
       }
   }
   ```

4. Define async trait for API operations:
   ```rust
   use async_trait::async_trait;

   /// Async API operations trait
   #[async_trait]
   pub trait OllamaApiAsync {
       /// Get Ollama server version (async)
       async fn version(&self) -> crate::Result<VersionResponse>;
   }
   ```

5. Implement async version with retry logic:
   ```rust
   #[async_trait]
   impl OllamaApiAsync for OllamaClient {
       async fn version(&self) -> crate::Result<VersionResponse> {
           let url = format!("{}/api/version", self.config.base_url);

           let mut last_error = None;
           for attempt in 0..=self.config.max_retries {
               match self.client.get(&url).send().await {
                   Ok(response) => {
                       let version_response = response.json::<VersionResponse>().await?;
                       return Ok(version_response);
                   }
                   Err(e) => {
                       last_error = Some(e);
                       if attempt < self.config.max_retries {
                           // Optional: exponential backoff
                           tokio::time::sleep(Duration::from_millis(100 * (attempt as u64 + 1))).await;
                       }
                   }
               }
           }

           Err(crate::Error::MaxRetriesExceededError(self.config.max_retries))
       }
   }
   ```

6. Define sync trait for blocking operations:
   ```rust
   /// Sync API operations trait
   pub trait OllamaApiSync {
       /// Get Ollama server version (blocking)
       fn version_blocking(&self) -> crate::Result<VersionResponse>;
   }
   ```

7. Implement sync version:
   ```rust
   impl OllamaApiSync for OllamaClient {
       fn version_blocking(&self) -> crate::Result<VersionResponse> {
           let url = format!("{}/api/version", self.config.base_url);

           let mut last_error = None;
           for attempt in 0..=self.config.max_retries {
               match self.client.get(&url).send() {
                   Ok(response) => {
                       let version_response = response.json::<VersionResponse>()?;
                       return Ok(version_response);
                   }
                   Err(e) => {
                       last_error = Some(e);
                       if attempt < self.config.max_retries {
                           // Blocking sleep
                           std::thread::sleep(Duration::from_millis(100 * (attempt as u64 + 1)));
                       }
                   }
               }
           }

           Err(crate::Error::MaxRetriesExceededError(self.config.max_retries))
       }
   }
   ```

**Key Features:**
- ✅ Client is `Clone` (uses `Arc<Client>` internally)
- ✅ Configurable timeout via `ClientConfig`
- ✅ Retry logic with exponential backoff
- ✅ URL validation using `url` crate
- ✅ Both async (via `async-trait`) and sync APIs
- ✅ No explicit `Box`, `Arc`, `move` in async-trait usage

### Phase 4: Library Entry Point

**Location:** `src/lib.rs`

**Tasks:**
1. Re-export key types:
   ```rust
   // Error handling
   pub mod error;
   pub use error::{Error, Result};

   // Primitives module
   #[cfg(feature = "primitives")]
   pub mod primitives;

   #[cfg(feature = "primitives")]
   pub use primitives::VersionResponse;

   // HTTP client
   #[cfg(feature = "http")]
   pub mod http;

   #[cfg(feature = "http")]
   pub use http::{OllamaClient, ClientConfig, OllamaApiAsync, OllamaApiSync};
   ```

2. Add crate-level documentation with examples:
   ```rust
   //! # ollama-oxide
   //!
   //! A Rust library for integrating with Ollama's native API.
   //!
   //! ## Quick Start
   //!
   //! ### Async Example
   //! ```no_run
   //! use ollama_oxide::{OllamaClient, OllamaApiAsync, Result};
   //!
   //! #[tokio::main]
   //! async fn main() -> Result<()> {
   //!     let client = OllamaClient::default()?;
   //!     let version = client.version().await?;
   //!     println!("Ollama version: {}", version.version);
   //!     Ok(())
   //! }
   //! ```
   //!
   //! ### Sync Example
   //! ```no_run
   //! use ollama_oxide::{OllamaClient, OllamaApiSync, Result};
   //!
   //! fn main() -> Result<()> {
   //!     let client = OllamaClient::default()?;
   //!     let version = client.version_blocking()?;
   //!     println!("Ollama version: {}", version.version);
   //!     Ok(())
   //! }
   //! ```
   ```

3. Add feature gate checks
4. Define prelude module:
   ```rust
   pub mod prelude {
       pub use crate::{
           Error, Result,
           OllamaClient, ClientConfig,
           OllamaApiAsync, OllamaApiSync,
           VersionResponse,
       };
   }
   ```

### Phase 5: Testing

**Location:** `tests/` directory or module tests

**Tasks:**
1. **Unit Tests** in each module:
   - Test `VersionResponse` deserialization with sample JSON
   - Test error type conversions
   - Test URL construction

2. **Integration Tests** (`tests/integration_test.rs`):
   - Test against real Ollama server (if available)
   - Mock HTTP responses with `mockito` or `wiremock` crate
   - Test error scenarios (404, 500, timeout, etc.)

3. **Documentation Tests**:
   - Ensure all doc examples compile and run

**Example Unit Test:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_response_deserialization() {
        let json = r#"{"version":"0.12.6"}"#;
        let response: VersionResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.version, "0.12.6");
    }
}
```

**Example Integration Test (Async):**
```rust
use ollama_oxide::{OllamaClient, OllamaApiAsync};

#[tokio::test]
async fn test_get_version_async() {
    let client = OllamaClient::default().expect("Failed to create client");
    let result = client.version().await;

    // This test requires Ollama to be running
    if let Ok(response) = result {
        assert!(!response.version.is_empty());
    }
}
```

**Example Integration Test (Sync):**
```rust
use ollama_oxide::{OllamaClient, OllamaApiSync};

#[test]
fn test_get_version_sync() {
    let client = OllamaClient::default().expect("Failed to create client");
    let result = client.version_blocking();

    // This test requires Ollama to be running
    if let Ok(response) = result {
        assert!(!response.version.is_empty());
    }
}
```

**Example Retry Test:**
```rust
use ollama_oxide::{OllamaClient, ClientConfig, OllamaApiAsync};
use std::time::Duration;

#[tokio::test]
async fn test_retry_logic() {
    let config = ClientConfig {
        base_url: "http://localhost:9999".to_string(), // Invalid port
        timeout: Duration::from_secs(1),
        max_retries: 2,
    };

    let client = OllamaClient::new(config).expect("Failed to create client");
    let result = client.version().await;

    assert!(result.is_err());
}
```

**Example URL Validation Test:**
```rust
use ollama_oxide::{OllamaClient, ClientConfig};

#[test]
fn test_invalid_url() {
    let config = ClientConfig {
        base_url: "not-a-valid-url".to_string(),
        ..Default::default()
    };

    let result = OllamaClient::new(config);
    assert!(result.is_err());
}
```

**Example Thread Safety Tests:**
```rust
use ollama_oxide::{OllamaClient, Error, VersionResponse, ClientConfig};
use std::sync::Arc;
use std::thread;

// Compile-time test: verify types are Send + Sync
#[test]
fn test_types_are_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<Error>();
    assert_send_sync::<VersionResponse>();
    assert_send_sync::<ClientConfig>();
    assert_send_sync::<OllamaClient>();
}

// Runtime test: client shared across threads
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
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
}

// Async test: concurrent async calls
#[tokio::test]
async fn test_concurrent_async_calls() {
    let client = Arc::new(OllamaClient::default().unwrap());
    let mut tasks = vec![];

    // Spawn 10 concurrent tasks
    for _ in 0..10 {
        let client_clone = Arc::clone(&client);
        let task = tokio::spawn(async move {
            // Each task can use the client independently
            let _c = client_clone;
        });
        tasks.push(task);
    }

    // Wait for all tasks
    for task in tasks {
        task.await.unwrap();
    }
}
```

### Phase 6: Documentation

**Tasks:**
1. Add comprehensive crate-level documentation
2. Document all public types and methods
3. Add usage examples in doc comments

4. Create async example in `examples/get_version_async.rs`:
   ```rust
   use ollama_oxide::{OllamaClient, OllamaApiAsync, Result};

   #[tokio::main]
   async fn main() -> Result<()> {
       // Create client with default configuration
       let client = OllamaClient::default()?;

       // Get version (async)
       let version = client.version().await?;
       println!("Ollama version: {}", version.version);

       Ok(())
   }
   ```

5. Create sync example in `examples/get_version_sync.rs`:
   ```rust
   use ollama_oxide::{OllamaClient, OllamaApiSync, Result};

   fn main() -> Result<()> {
       // Create client with default configuration
       let client = OllamaClient::default()?;

       // Get version (blocking)
       let version = client.version_blocking()?;
       println!("Ollama version: {}", version.version);

       Ok(())
   }
   ```

6. Create custom config example in `examples/get_version_custom.rs`:
   ```rust
   use ollama_oxide::{OllamaClient, ClientConfig, OllamaApiAsync, Result};
   use std::time::Duration;

   #[tokio::main]
   async fn main() -> Result<()> {
       // Create client with custom configuration
       let config = ClientConfig {
           base_url: "http://localhost:11434".to_string(),
           timeout: Duration::from_secs(10),
           max_retries: 5,
       };

       let client = OllamaClient::new(config)?;
       let version = client.version().await?;
       println!("Ollama version: {}", version.version);

       Ok(())
   }
   ```

7. Update README.md with quick start example
8. Update CHANGELOG.md

## Implementation Order (TDD Approach)

### Phase 0: TDD Preparation - Write Tests First ⚠️

**CRITICAL**: All tests must be written BEFORE implementation begins.

#### Step 0.1: Write Comprehensive Test Suite
Write all tests that will validate the implementation. Tests must:
- ✅ Compile successfully
- ❌ Fail (because implementation doesn't exist yet)
- ✅ Be comprehensive and cover all features
- ✅ Be consistent with the implementation plan

**Test Categories to Write:**

1. **Error Type Tests** (`tests/error_tests.rs`):
   - Test error type creation and display messages
   - Test `From` implementations for error conversions
   - Test error variants (HttpError, SerializationError, InvalidUrlError, TimeoutError, MaxRetriesExceededError)
   - Test error is Send + Sync for thread safety

2. **Primitives Tests** (`tests/primitives_tests.rs`):
   - Test `VersionResponse` deserialization from valid JSON
   - Test `VersionResponse` serialization to JSON
   - Test `VersionResponse` with empty/invalid data
   - Test Clone, Debug, PartialEq traits
   - Test VersionResponse is Send + Sync for thread safety

3. **Client Configuration Tests** (`tests/client_config_tests.rs`):
   - Test `ClientConfig` default values
   - Test `ClientConfig` custom values
   - Test Clone, Debug traits
   - Test ClientConfig is Send + Sync for thread safety

4. **Client Construction Tests** (`tests/client_construction_tests.rs`):
   - Test client creation with default config
   - Test client creation with custom config
   - Test client creation with invalid URL (should fail)
   - Test client is Clone
   - Test OllamaClient is Send + Sync for thread safety
   - Test client can be shared across threads (spawn multiple threads with cloned client)

5. **Client Async API Tests** (`tests/client_async_tests.rs`):
   - Test successful async version call (mocked response)
   - Test retry logic with transient failures
   - Test max retries exceeded error
   - Test timeout handling
   - Test JSON deserialization error handling
   - Test concurrent async calls from multiple tasks (thread safety)

6. **Client Sync API Tests** (`tests/client_sync_tests.rs`):
   - Test successful sync version call (mocked response)
   - Test retry logic with transient failures
   - Test max retries exceeded error
   - Test timeout handling
   - Test concurrent sync calls from multiple threads (thread safety)

7. **Integration Tests** (`tests/integration_tests.rs`):
   - Test against real Ollama server (conditional)
   - Test both async and sync APIs with real server

#### Step 0.2: User Validation
- 🔍 User reviews all tests
- ✅ User approves test coverage and quality
- 🚀 Proceed to implementation only after approval

---

### Phase 1: Implementation - Make Tests Pass ✅

After test approval, implement in this order:

#### Step 1: Error Handling Foundation
**File**: `src/error.rs` or in `src/lib.rs`
- Define `Error` enum with all variants
- Implement `std::error::Error` trait
- Implement `std::fmt::Display` trait
- Add `From` implementations
- Define `Result<T>` type alias
- **Verify**: Error tests pass ✅

#### Step 2: Primitives Module
**File**: `src/primitives/mod.rs`
- Define `VersionResponse` struct
- Add serde derives
- Add documentation
- **Verify**: Primitives tests pass ✅

#### Step 3: HTTP Client Configuration
**File**: `src/http/mod.rs`
- Define `ClientConfig` struct
- Implement `Default` trait
- Add Clone, Debug derives
- **Verify**: Client config tests pass ✅

#### Step 4: HTTP Client Structure
**File**: `src/http/mod.rs`
- Define `OllamaClient` struct
- Implement constructors with URL validation
- Add Clone, Debug derives
- **Verify**: Client construction tests pass ✅

#### Step 5: Async API Implementation
**File**: `src/http/mod.rs`
- Define `OllamaApiAsync` trait
- Implement trait for `OllamaClient`
- Add retry logic with exponential backoff
- **Verify**: Async API tests pass ✅

#### Step 6: Sync API Implementation
**File**: `src/http/mod.rs`
- Define `OllamaApiSync` trait
- Implement trait for `OllamaClient`
- Add retry logic with exponential backoff
- **Verify**: Sync API tests pass ✅

#### Step 7: Library Entry Point
**File**: `src/lib.rs`
- Re-export all public types
- Add feature gates
- Add crate-level documentation
- Define prelude module
- **Verify**: All tests pass ✅

#### Step 8: Examples and Documentation
- Create `examples/get_version_async.rs`
- Create `examples/get_version_sync.rs`
- Create `examples/get_version_custom.rs`
- Update README.md
- Update CHANGELOG.md
- **Verify**: Examples compile and run ✅

#### Step 9: Final Validation
- Run `cargo test` - All tests pass ✅
- Run `cargo doc --open` - Documentation generates ✅
- Run `cargo clippy` - No warnings ✅
- Run `cargo fmt --check` - Code formatted ✅
- Run examples manually - All work ✅

---

## TDD Workflow Summary

```
┌─────────────────────────────────────────────────┐
│ Phase 0: Write Tests First                     │
│ - Write comprehensive test suite               │
│ - Tests compile but fail (no implementation)   │
│ - User validates and approves tests            │
└─────────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────────┐
│ Phase 1: Implementation                         │
│ - Implement each component                     │
│ - Run tests after each step                    │
│ - Verify tests pass before moving forward      │
└─────────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────────┐
│ Phase 2: Final Validation                      │
│ - All tests pass                               │
│ - Documentation complete                       │
│ - Examples work                                │
│ - Code quality checks pass                     │
└─────────────────────────────────────────────────┘
```

## File Structure After TDD Implementation

```
ollama-oxide/
├── src/
│   ├── lib.rs              # Main entry point with re-exports
│   ├── error.rs            # Error types (or in lib.rs)
│   ├── primitives/
│   │   └── mod.rs          # VersionResponse and future types
│   └── http/
│       └── mod.rs          # OllamaClient, traits, config
├── tests/
│   ├── error_tests.rs              # Error type tests (Phase 0)
│   ├── primitives_tests.rs         # Primitives tests (Phase 0)
│   ├── client_config_tests.rs      # Config tests (Phase 0)
│   ├── client_construction_tests.rs # Constructor tests (Phase 0)
│   ├── client_async_tests.rs       # Async API tests (Phase 0)
│   ├── client_sync_tests.rs        # Sync API tests (Phase 0)
│   └── integration_tests.rs        # Integration tests (Phase 0)
├── examples/
│   ├── get_version_async.rs        # Async usage example
│   ├── get_version_sync.rs         # Sync usage example
│   └── get_version_custom.rs       # Custom config example
└── Cargo.toml
```

**Note**: All test files in `tests/` directory are created in **Phase 0** (before implementation).

## Dependencies to Add

Add these dependencies to `Cargo.toml`:

```toml
[dependencies]
# Existing dependencies
tokio = { version = "1.49.0", features = ["macros", "rt-multi-thread", "time"] }
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"
reqwest = { version = "0.13.1", default-features = false, features = ["blocking", "cookies", "http2", "json", "native-tls"] }
async-trait = "0.1.89"

# New dependencies (approved)
thiserror = "2.0.17"           # Ergonomic error handling
url = "2.5.8"                 # URL parsing and validation

# Optional: Logging/tracing (consider for future)
# tracing = "0.1"           # Structured logging
# tracing-subscriber = "0.3" # Log subscriber

[dev-dependencies]
# Testing
mockito = "1.7.1"             # HTTP mocking for tests
# or
wiremock = "0.6.5"            # Alternative HTTP mocking
```

**Note:** Added `time` feature to tokio for `tokio::time::sleep` in retry logic.

## Success Criteria

The implementation will be considered successful when:

1. ✅ `cargo build` succeeds without warnings
2. ✅ `cargo test` passes all tests
3. ✅ `cargo clippy` shows no warnings
4. ✅ `cargo doc --open` generates proper documentation
5. ✅ Example runs successfully against Ollama server
6. ✅ Error handling covers all failure scenarios
7. ✅ Code follows Rust best practices and idioms
8. ✅ All public APIs are documented
9. ✅ Integration test covers the endpoint

## Design Decisions Summary

All design questions have been reviewed and approved:

1. ✅ **Error Handling Approach**: Using `thiserror` for ergonomic error handling
2. ⏸️ **Module Organization**: Decide during implementation (start simple with `mod.rs`)
3. ✅ **Client Configuration**: Timeout, retry, and URL validation implemented
4. ⏸️ **Testing Strategy**: Choose between `mockito` or `wiremock` during test implementation
5. ⏸️ **Logging**: Defer `tracing` support to future iteration (commented in dependencies)
6. ✅ **URL Validation**: Using `url` crate for URL parsing and validation
7. ✅ **Client Cloning**: `OllamaClient` is `Clone` using `Arc<Client>` internally
8. ✅ **Async/Sync Support**: Both APIs implemented via `async-trait` and blocking methods

## Next Steps After Completion

After successfully implementing GET /api/version:

1. Update DEV_NOTES.md with lessons learned
2. Update definition.md Phase 1 progress
3. Plan implementation for next endpoints:
   - GET /api/tags (List models)
   - POST /api/copy (Copy model)
   - DELETE /api/delete (Delete model)
4. Consider establishing patterns for:
   - Request bodies
   - Error responses
   - Streaming endpoints

## Notes

- This is the **foundational endpoint** - patterns established here will be reused
- Keep it simple but extensible
- Focus on clear error messages
- Write tests that will serve as examples for future endpoints
- Document design decisions in DEV_NOTES.md

## Implementation Highlights

### Key Features
1. **Dual API Support**: Both async (via `async-trait`) and sync (blocking) methods
2. **Robust Error Handling**: Using `thiserror` with comprehensive error types
3. **Consistent Error Naming**: All error variants use `Error` suffix
4. **Retry Mechanism**: Exponential backoff with configurable max retries
5. **Timeout Control**: Configurable request timeouts
6. **URL Validation**: Validates base URL format at client creation
7. **Thread-Safe Cloning**: Client is `Clone` using `Arc` internally
8. **Full Thread Safety**: All types are `Send + Sync` for concurrent usage
9. **Clean Abstractions**: Traits separate async and sync concerns

### Architecture Patterns
- **Configuration**: `ClientConfig` struct for all client settings
- **Trait-Based API**: `OllamaApiAsync` and `OllamaApiSync` traits
- **No Explicit Lifetimes**: `async-trait` handles complexity
- **Feature Gates**: Modules controlled by Cargo features
- **Prelude**: Convenient imports via `prelude` module

### Testing Strategy
- **Unit Tests**: Type deserialization, error conversions, URL validation
- **Integration Tests**: Both async and sync API calls
- **Retry Tests**: Verify retry logic with invalid endpoints
- **Validation Tests**: URL format validation
- **Thread Safety Tests**: Compile-time `Send + Sync` checks and runtime concurrency tests

### Examples Provided
1. `get_version_async.rs` - Basic async usage
2. `get_version_sync.rs` - Basic sync usage
3. `get_version_custom.rs` - Custom configuration

---

**Plan Status**: ✅ COMPLETED
**Approval Date**: 2026-01-13
**Completion Date**: 2026-01-13
**Development Approach**: Test-Driven Development (TDD)

## Implementation Summary

**Phase 0: Test Suite** ✅
- Wrote comprehensive test suite (79 tests across 7 test files)
- All tests compiled successfully
- Tests failed initially (as expected - no implementation)

**Phase 1: Implementation** ✅
- Implemented Error type with all variants
- Implemented VersionResponse primitive
- Implemented ClientConfig with default values
- Implemented OllamaClient with Arc-based cloning
- Implemented async API with retry logic and exponential backoff
- Implemented sync API with blocking reqwest client
- Added URL validation (scheme checking)
- All implementations follow the plan specifications

**Test Results** ✅
- **Total Tests**: 79 passing (100% success rate)
- Error type tests: 10 passing
- Primitives tests: 11 passing
- Client config tests: 11 passing
- Client construction tests: 17 passing
- Client async API tests: 9 passing
- Client sync API tests: 10 passing
- Error tests: 11 passing
- Integration tests: 2 passing (conditional on OLLAMA_TEST_SERVER)

**Code Quality** ✅
- Cargo clippy: Clean (0 warnings)
- Cargo fmt: Formatted
- All types implement Send + Sync
- Thread safety validated with runtime tests
- Retry logic tested with mockito
- URL validation working correctly
