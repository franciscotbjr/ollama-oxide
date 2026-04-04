# HTTP Retry Logic Abstraction Analysis

**Document Version:** 1.0
**Analysis Date:** 2026-01-14
**Status:** ✅ IMPLEMENTED

---

## Problem Statement

The current `version()` and `version_blocking()` implementations contain duplicated HTTP retry logic:

**Duplicated Pattern (async):**
```rust
for attempt in 0..=self.config.max_retries {
    match self.client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_server_error() && attempt < self.config.max_retries {
                tokio::time::sleep(Duration::from_millis(100 * (attempt as u64 + 1))).await;
                continue;
            }
            let result = response.json::<T>().await?;
            return Ok(result);
        }
        Err(_e) => {
            if attempt < self.config.max_retries {
                tokio::time::sleep(Duration::from_millis(100 * (attempt as u64 + 1))).await;
            }
        }
    }
}
Err(Error::MaxRetriesExceededError(self.config.max_retries))
```

**Duplicated Pattern (sync):**
```rust
// Nearly identical, just replace:
// - tokio::time::sleep → std::thread::sleep
// - .await → (removed)
// - Arc<reqwest::Client> → reqwest::blocking::Client
```

### Issues with Current Approach

1. **Code Duplication**: 30+ lines of retry logic will be duplicated across 12 endpoints × 2 (async/sync) = **~720 lines of boilerplate**
2. **Maintenance Burden**: Changes to retry strategy require updating 24 locations
3. **Inconsistency Risk**: Easy to introduce bugs by updating one implementation but not others
4. **Testing Complexity**: Retry logic must be tested 24 times instead of once
5. **No Single Source of Truth**: Retry policy scattered across codebase

### Scale of the Problem

**Current (1 endpoint):**
- Lines of retry logic: ~60 (30 async + 30 sync)

**Phase 2 (12 endpoints):**
- Lines of retry logic: ~720 (12 × 30 × 2)
- Without abstraction: **12x code duplication**

---

## Proposed Solution: Helper Methods on OllamaClient

### Design Principles

1. **Keep it simple**: Helper methods, not complex frameworks
2. **Type-safe**: Generic over response types
3. **Non-invasive**: Existing API surface unchanged
4. **Testable**: Retry logic isolated and unit-testable
5. **Single responsibility**: One method does one thing

### Implementation

#### 1. Add Helper Methods to OllamaClient

```rust
// src/http/client.rs

impl OllamaClient {
    // ... existing constructors ...

    /// Execute async HTTP GET with retry logic
    ///
    /// Handles exponential backoff and server error retries automatically.
    pub(super) async fn get_with_retry<T>(&self, url: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        for attempt in 0..=self.config.max_retries {
            match self.client.get(url).send().await {
                Ok(response) => {
                    // Retry on server errors (5xx)
                    if response.status().is_server_error() && attempt < self.config.max_retries {
                        tokio::time::sleep(Duration::from_millis(100 * (attempt as u64 + 1))).await;
                        continue;
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

    /// Execute blocking HTTP GET with retry logic
    ///
    /// Handles exponential backoff and server error retries automatically.
    pub(super) fn get_blocking_with_retry<T>(&self, url: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        // Create blocking client (cached would be better, but keep simple for now)
        let blocking_client = reqwest::blocking::Client::builder()
            .timeout(self.config.timeout)
            .build()?;

        for attempt in 0..=self.config.max_retries {
            match blocking_client.get(url).send() {
                Ok(response) => {
                    // Retry on server errors (5xx)
                    if response.status().is_server_error() && attempt < self.config.max_retries {
                        std::thread::sleep(Duration::from_millis(100 * (attempt as u64 + 1)));
                        continue;
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
}
```

#### 2. Refactor Async Implementation

**Before:**
```rust
// api_async.rs - 28 lines
async fn version(&self) -> Result<VersionResponse> {
    let url = self.config.url(Endpoints::VERSION);

    for attempt in 0..=self.config.max_retries {
        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_server_error() && attempt < self.config.max_retries {
                    tokio::time::sleep(Duration::from_millis(100 * (attempt as u64 + 1))).await;
                    continue;
                }
                let version_response = response.json::<VersionResponse>().await?;
                return Ok(version_response);
            }
            Err(_e) => {
                if attempt < self.config.max_retries {
                    tokio::time::sleep(Duration::from_millis(100 * (attempt as u64 + 1))).await;
                }
            }
        }
    }

    Err(Error::MaxRetriesExceededError(self.config.max_retries))
}
```

**After:**
```rust
// api_async.rs - 3 lines
async fn version(&self) -> Result<VersionResponse> {
    let url = self.config.url(Endpoints::VERSION);
    self.get_with_retry(&url).await
}
```

#### 3. Refactor Sync Implementation

**Before:**
```rust
// api_sync.rs - 32 lines
fn version_blocking(&self) -> Result<VersionResponse> {
    let url = self.config.url(Endpoints::VERSION);

    let blocking_client = reqwest::blocking::Client::builder()
        .timeout(self.config.timeout)
        .build()?;

    for attempt in 0..=self.config.max_retries {
        match blocking_client.get(&url).send() {
            Ok(response) => {
                if response.status().is_server_error() && attempt < self.config.max_retries {
                    std::thread::sleep(Duration::from_millis(100 * (attempt as u64 + 1)));
                    continue;
                }
                let version_response = response.json::<VersionResponse>()?;
                return Ok(version_response);
            }
            Err(_e) => {
                if attempt < self.config.max_retries {
                    std::thread::sleep(Duration::from_millis(100 * (attempt as u64 + 1)));
                }
            }
        }
    }

    Err(Error::MaxRetriesExceededError(self.config.max_retries))
}
```

**After:**
```rust
// api_sync.rs - 3 lines
fn version_blocking(&self) -> Result<VersionResponse> {
    let url = self.config.url(Endpoints::VERSION);
    self.get_blocking_with_retry(&url)
}
```

---

## Benefits

### Code Reduction

| Metric | Before | After | Savings |
|--------|--------|-------|---------|
| Lines per async endpoint | ~28 | ~3 | 89% reduction |
| Lines per sync endpoint | ~32 | ~3 | 91% reduction |
| Total for 12 endpoints | ~720 | ~36 + ~120 helpers | **78% reduction** |

### Maintenance

- ✅ **Single source of truth** for retry logic
- ✅ **One place to update** retry strategy
- ✅ **Consistent behavior** across all endpoints
- ✅ **Easier to test** - test helpers once, not 24 times
- ✅ **Type-safe** - compiler ensures correct usage

### Scalability

Adding new endpoints in Phase 2:

**Without abstraction:**
```rust
// Must write 60 lines (30 async + 30 sync)
async fn tags(&self) -> Result<Vec<Model>> {
    // 28 lines of retry logic
}
fn tags_blocking(&self) -> Result<Vec<Model>> {
    // 32 lines of retry logic
}
```

**With abstraction:**
```rust
// Just 6 lines (3 async + 3 sync)
async fn tags(&self) -> Result<Vec<Model>> {
    let url = self.config.url(Endpoints::TAGS);
    self.get_with_retry(&url).await
}

fn tags_blocking(&self) -> Result<Vec<Model>> {
    let url = self.config.url(Endpoints::TAGS);
    self.get_blocking_with_retry(&url)
}
```

---

## Alternative Approaches Considered

### Alternative 1: Macro-Based

```rust
macro_rules! retry_get {
    ($client:expr, $url:expr, $response_type:ty) => {
        // macro expansion with retry logic
    };
}
```

**Rejected because:**
- ❌ Harder to debug (macro expansion)
- ❌ Poor IDE support
- ❌ Not discoverable (hidden behind macro)
- ❌ More complex than needed

### Alternative 2: Trait-Based Middleware

```rust
trait HttpMiddleware {
    fn execute<T>(&self, request: Request) -> Result<T>;
}
```

**Rejected because:**
- ❌ Over-engineering for current needs
- ❌ Adds abstraction complexity
- ❌ Not idiomatic Rust for this use case
- ❌ Would require significant refactoring

### Alternative 3: Function-Based Helper (External)

```rust
// http/retry.rs
pub(crate) async fn get_with_retry<T>(
    client: &Client,
    url: &str,
    config: &ClientConfig
) -> Result<T> { ... }
```

**Rejected because:**
- ❌ More verbose at call site
- ❌ Exposes internal client/config details
- ❌ Doesn't leverage OOP encapsulation

---

## Implementation Plan

### File Structure

```
src/http/
├── client.rs           # ADD: get_with_retry(), get_blocking_with_retry()
├── api_async.rs        # REFACTOR: use helper methods
└── api_sync.rs         # REFACTOR: use helper methods
```

### Implementation Steps

1. **Add helper methods to `client.rs`**
   - Implement `get_with_retry<T>(&self, url: &str) -> Result<T>`
   - Implement `get_blocking_with_retry<T>(&self, url: &str) -> Result<T>`
   - Mark as `pub(super)` - visible only in http module

2. **Refactor `api_async.rs`**
   - Replace retry loop in `version()` with `self.get_with_retry(&url).await`
   - Reduce from 28 lines to 3 lines

3. **Refactor `api_sync.rs`**
   - Replace retry loop in `version_blocking()` with `self.get_blocking_with_retry(&url)`
   - Reduce from 32 lines to 3 lines

4. **Testing**
   - Existing tests should pass unchanged (no public API changes)
   - Consider adding unit tests for helper methods

5. **Verification**
   - Run `cargo test` - all 79 tests should pass
   - Run `cargo clippy` - no warnings
   - Verify behavior unchanged with integration tests

---

## Future Enhancements

### POST Request Support

When implementing POST endpoints (generate, chat, etc.):

```rust
impl OllamaClient {
    pub(super) async fn post_with_retry<T, B>(
        &self,
        url: &str,
        body: &B
    ) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        // Similar retry logic for POST
    }
}
```

### Streaming Support

For streaming endpoints (generate, chat with streaming):

```rust
impl OllamaClient {
    pub(super) async fn post_stream_with_retry<B>(
        &self,
        url: &str,
        body: &B
    ) -> Result<impl Stream<Item = Result<Event>>>
    where
        B: serde::Serialize,
    {
        // Streaming with retry logic
        // Note: Retry on stream interruption is complex
    }
}
```

### Retry Policy Customization

Could add retry policy to `ClientConfig`:

```rust
pub struct RetryPolicy {
    pub base_delay_ms: u64,  // Default: 100
    pub backoff_multiplier: f64,  // Default: 1.0 (linear)
    pub retry_on_5xx: bool,  // Default: true
}
```

But for now, **simple is better** - fixed exponential backoff works well.

---

## Testing Strategy

### Unit Tests for Helpers

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_with_retry_success() {
        // Mock server that succeeds
        // Verify single attempt
    }

    #[tokio::test]
    async fn test_get_with_retry_server_error_then_success() {
        // Mock server: 500, 500, 200
        // Verify 3 attempts with backoff
    }

    #[tokio::test]
    async fn test_get_with_retry_max_retries_exceeded() {
        // Mock server always returns 500
        // Verify max retries error
    }

    // Similar tests for get_blocking_with_retry
}
```

### Integration Tests

Existing integration tests should continue to pass without modification, proving that the refactoring maintains identical behavior.

---

## Implementation Checklist

- [ ] Add `get_with_retry<T>()` method to `OllamaClient` in `client.rs`
- [ ] Add `get_blocking_with_retry<T>()` method to `OllamaClient` in `client.rs`
- [ ] Refactor `api_async.rs::version()` to use helper
- [ ] Refactor `api_sync.rs::version_blocking()` to use helper
- [ ] Run tests: `cargo test` - verify all pass
- [ ] Run clippy: `cargo clippy` - verify no warnings
- [ ] Update this analysis with "IMPLEMENTED" status when complete

---

## Recommendation

**Implement the helper methods approach immediately** before adding more endpoints.

**Reasoning:**
1. Solves the stated problem (eliminate boilerplate)
2. Simple and maintainable
3. Type-safe with generics
4. No public API changes
5. Easy to extend for POST/streaming
6. Scales perfectly to 12 endpoints

**Next Step:** Implement helper methods in [src/http/client.rs](../src/http/client.rs) and refactor existing code.
