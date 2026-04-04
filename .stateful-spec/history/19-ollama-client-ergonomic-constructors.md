# OllamaClient Ergonomic Constructors

**Document Version:** 1.1
**Date:** 2026-02-15
**Status:** Pending

## Overview

Two improvements to `OllamaClient` and `ClientConfig`:

1. Add `with_base_url_and_timeout` convenience constructor to `OllamaClient`
2. Add early URL validation to `ClientConfig` constructors that accept a `base_url`

### Rationale

**Goal 1 — Ergonomic constructor:**

Currently, to use a custom URL + timeout, developers must:
```rust
let client = OllamaClient::new(ClientConfig::with_base_url_and_timeout(
    "http://myserver:8080".to_string(),
    Duration::from_secs(60),
));
```

After this change:
```rust
let client = OllamaClient::with_base_url_and_timeout(
    "http://myserver:8080",
    Duration::from_secs(60),
)?;
```

**Goal 2 — Early URL validation in ClientConfig:**

Currently, `ClientConfig::new()`, `with_base_url()`, and `with_base_url_and_timeout()` all return `Self` with no validation. Invalid URLs are only caught later when `OllamaClient::new()` is called. This means a developer can create a `ClientConfig` with `base_url: "not-a-url"` and only discover the error much later.

After this change, `ClientConfig` constructors that accept a `base_url` will validate the URL immediately and return `Result<Self>`.

---

## Current State

### OllamaClient constructors (`src/http/client.rs`)

| Method | Signature | Validates URL? |
|--------|-----------|----------------|
| `new` | `(config: ClientConfig) -> Result<Self>` | Yes (lines 77-85) |
| `with_base_url` | `(base_url: impl Into<String>) -> Result<Self>` | Via `new()` |
| `default` | `() -> Result<Self>` | Via `new()` |
| `with_base_url_and_timeout` | — | **Missing** |

### ClientConfig constructors (`src/http/client_config.rs`)

| Method | Signature | Validates URL? |
|--------|-----------|----------------|
| `new` | `(base_url, timeout, max_retries) -> Self` | **No** |
| `with_base_url` | `(base_url) -> Self` | **No** |
| `with_base_url_and_timeout` | `(base_url, timeout) -> Self` | **No** |

---

## Implementation Plan

### Step 1: Add URL validation to `ClientConfig`

**File:** `src/http/client_config.rs`

Add `use crate::{Error, Result}` and `use url::Url` imports.

Add a private validation function:

```rust
/// Validates that a URL is well-formed and uses http or https scheme
fn validate_base_url(base_url: &str) -> Result<()> {
    let url = Url::parse(base_url)?;
    if url.scheme() != "http" && url.scheme() != "https" {
        return Err(Error::InvalidUrlError(
            url::ParseError::RelativeUrlWithoutBase,
        ));
    }
    Ok(())
}
```

### Step 2: Update `ClientConfig` constructors to validate and return `Result`

**File:** `src/http/client_config.rs`

Change signatures from `-> Self` to `-> Result<Self>`:

```rust
pub fn new(base_url: String, timeout: Duration, max_retries: u32) -> Result<Self> {
    validate_base_url(&base_url)?;
    Ok(Self { base_url, timeout, max_retries })
}

pub fn with_base_url(base_url: String) -> Result<Self> {
    validate_base_url(&base_url)?;
    Ok(Self { base_url, ..Self::default() })
}

pub fn with_base_url_and_timeout(base_url: String, timeout: Duration) -> Result<Self> {
    validate_base_url(&base_url)?;
    Ok(Self { base_url, timeout, ..Self::default() })
}
```

**Note:** `OllamaClient::new()` still accepts raw `ClientConfig` (which can be constructed via struct literal without validation). Keep the existing URL validation in `OllamaClient::new()` as a safety net for struct-literal usage.

### Step 3: Add `with_base_url_and_timeout` to OllamaClient

**File:** `src/http/client.rs`
**Location:** After `with_base_url` method (after line 119), before `default` method

```rust
pub fn with_base_url_and_timeout(
    base_url: impl Into<String>,
    timeout: Duration,
) -> Result<Self> {
    let config = ClientConfig {
        base_url: base_url.into(),
        timeout,
        ..Default::default()
    };
    Self::new(config)
}
```

### Step 4: Update all call sites for `ClientConfig` `Result` change

Search all files that call `ClientConfig::new()`, `ClientConfig::with_base_url()`, or `ClientConfig::with_base_url_and_timeout()` and add `?` or `.unwrap()` to handle the new `Result` return type.

Files to check:
- `src/http/client.rs` — `OllamaClient::with_base_url` and `default` (use struct literal, not affected)
- `tests/client_construction_tests.rs`
- `examples/get_version_custom.rs`
- Any other test/example files

### Step 5: Add tests

**File:** `tests/client_construction_tests.rs`

```rust
#[test]
fn test_with_base_url_and_timeout() {
    let client = OllamaClient::with_base_url_and_timeout(
        "http://localhost:8080",
        Duration::from_secs(60),
    )
    .unwrap();
    assert_eq!(client.config.base_url, "http://localhost:8080");
    assert_eq!(client.config.timeout, Duration::from_secs(60));
    assert_eq!(client.config.max_retries, 3); // default
}

#[test]
fn test_client_config_validates_url() {
    assert!(ClientConfig::with_base_url("not-a-url".to_string()).is_err());
    assert!(ClientConfig::with_base_url("ftp://invalid.scheme".to_string()).is_err());
    assert!(ClientConfig::with_base_url("http://valid.url".to_string()).is_ok());
}

#[test]
fn test_client_config_new_validates_url() {
    assert!(ClientConfig::new(
        "not-a-url".to_string(),
        Duration::from_secs(30),
        3,
    ).is_err());
}
```

---

## Verification

1. `cargo check` — compilation
2. `cargo test --all-features` — all tests pass
3. Verify invalid URLs are rejected at `ClientConfig` construction time