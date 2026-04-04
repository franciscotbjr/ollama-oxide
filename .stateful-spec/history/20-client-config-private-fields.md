# Make ClientConfig Fields Private

**Document Version:** 1.0
**Date:** 2026-02-15
**Status:** Pending

## Overview

Make `ClientConfig` fields private to enforce URL validation invariant. Since `ClientConfig` constructors now validate URLs, struct literal construction bypasses this validation. Making fields private eliminates this gap and removes redundant validation from `OllamaClient::new()`.

### Rationale

With `pub` fields, users can bypass validation:
```rust
// No validation happens here — invalid URL accepted silently
let config = ClientConfig {
    base_url: "garbage".to_string(),
    timeout: Duration::from_secs(30),
    max_retries: 3,
};
```

After this change, all construction goes through validated constructors:
```rust
// Fails immediately with InvalidUrlError
let config = ClientConfig::new("garbage".to_string(), Duration::from_secs(30), 3)?;
```

---

## Implementation Plan

### Step 1: Make fields private and add getters

**File:** `src/http/client_config.rs`

Change fields from `pub` to private. Add getter methods:

```rust
pub struct ClientConfig {
    base_url: String,
    timeout: Duration,
    max_retries: u32,
}

impl ClientConfig {
    pub fn base_url(&self) -> &str { &self.base_url }
    pub fn timeout(&self) -> Duration { self.timeout }
    pub fn max_retries(&self) -> u32 { self.max_retries }
}
```

Remove struct literal doc examples from `ClientConfig` docs.

### Step 2: Remove redundant validation from `OllamaClient::new()`

**File:** `src/http/client.rs`

Remove `Url::parse` and scheme check from `new()`. Remove `use url::Url` import. Simplify to:

```rust
pub fn new(config: ClientConfig) -> Result<Self> {
    let client = Client::builder().timeout(config.timeout()).build()?;
    Ok(Self {
        config,
        client: Arc::new(client),
    })
}
```

### Step 3: Update `OllamaClient` `with_*` methods to use `ClientConfig` constructors

**File:** `src/http/client.rs`

Replace struct literals with validated constructors:

```rust
pub fn with_base_url(base_url: impl Into<String>) -> Result<Self> {
    let config = ClientConfig::with_base_url(base_url.into())?;
    Self::new(config)
}

pub fn with_base_url_and_timeout(base_url: impl Into<String>, timeout: Duration) -> Result<Self> {
    let config = ClientConfig::with_base_url_and_timeout(base_url.into(), timeout)?;
    Self::new(config)
}
```

Update doc examples to use constructors instead of struct literals.

### Step 4: Update internal access via `config` field

**File:** `src/http/client.rs`

The `config` field is `pub(super)`, so internal code accesses fields directly. Update to use getters:

- `config.base_url` → `config.base_url()`
- `config.timeout` → `config.timeout()`
- `config.max_retries` → `config.max_retries()`

Search in `src/http/` for all direct field access patterns.

### Step 5: Update `src/main.rs`

Replace struct literal with constructor call.

### Step 6: Update all test files (~110 struct literal sites)

All `ClientConfig { base_url: url, timeout: dur, max_retries: n }` become `ClientConfig::new(url, dur, n).unwrap()`.

**Test files to update:**
- `tests/client_construction_tests.rs`
- `tests/client_config_tests.rs` (also field access → getters)
- `tests/client_async_tests.rs`
- `tests/client_sync_tests.rs`
- `tests/client_chat_tests.rs`
- `tests/client_generate_tests.rs`
- `tests/client_embed_tests.rs`
- `tests/client_copy_model_tests.rs`
- `tests/client_delete_model_tests.rs`
- `tests/client_show_model_tests.rs`
- `tests/client_list_models_tests.rs`
- `tests/client_list_running_models_tests.rs`
- `tests/client_create_model_tests.rs`
- `tests/client_pull_tests.rs`
- `tests/client_push_tests.rs`

**Pattern:** Most tests use mockito server URL which is always valid, so `.unwrap()` is safe.

### Step 7: Update example files

- `examples/get_version_custom.rs` — struct literal → constructor

---

## Verification

1. `cargo check --all-features` — compilation
2. `cargo test --all-features` — all tests pass
3. Verify `ClientConfig { ... }` struct literal no longer compiles outside the module
