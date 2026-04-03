# Move Implementations to "model" Feature

**Document Version:** 1.1
**Date:** 2026-02-02
**Status:** ✅ Implemented

## Overview

This plan describes the refactoring needed to consolidate model-related operations under the `model` feature flag.

### Features to Move

| Feature | Endpoint | Current Status | Target Status |
|---------|----------|----------------|---------------|
| List Models | `GET /api/tags` | No feature gate | `model` feature |
| List Running Models | `GET /api/ps` | No feature gate | `model` feature |
| Show Model Details | `POST /api/show` | No feature gate | `model` feature |
| Copy Model | `POST /api/copy` | No feature gate | `model` feature |

### Current State

Currently, these four methods are available without any feature gate (in default features). The `model` feature only gates:
- `create_model()` / `delete_model()` methods
- `CreateRequest`, `CreateResponse`, `DeleteRequest`, `LicenseSetting` types

### Target State

After refactoring, the `model` feature will gate ALL model-related operations:
- List, Show, Copy (read operations) - **NEW**
- Create, Delete (write operations) - existing

---

## Implementation Plan

### Phase 1: Update Trait Definitions

#### 1.1 Update `src/http/api_async.rs`

Add `#[cfg(feature = "model")]` to the following methods:

```rust
// Line ~423-426
#[cfg(feature = "model")]
async fn list_models(&self) -> Result<ListResponse>;

// Line ~433-436
#[cfg(feature = "model")]
async fn list_running_models(&self) -> Result<PsResponse>;

// Line ~428-431
#[cfg(feature = "model")]
async fn copy_model(&self, request: &CopyRequest) -> Result<()>;

// Line ~444-447
#[cfg(feature = "model")]
async fn show_model(&self, request: &ShowRequest) -> Result<ShowResponse>;
```

#### 1.2 Update `src/http/api_sync.rs`

Add `#[cfg(feature = "model")]` to the following methods:

```rust
// Line ~361-364
#[cfg(feature = "model")]
fn list_models(&self) -> Result<ListResponse>;

// Line ~371-374
#[cfg(feature = "model")]
fn list_running_models(&self) -> Result<PsResponse>;

// Line ~366-369
#[cfg(feature = "model")]
fn copy_model(&self, request: &CopyRequest) -> Result<()>;

// Line ~382-385
#[cfg(feature = "model")]
fn show_model(&self, request: &ShowRequest) -> Result<ShowResponse>;
```

---

### Phase 2: Gate Primitive Types

#### 2.1 Update `src/primitives/mod.rs`

Move the following re-exports under `#[cfg(feature = "model")]`:

```rust
#[cfg(feature = "model")]
mod copy_request;
#[cfg(feature = "model")]
mod list_response;
#[cfg(feature = "model")]
mod model_summary;
#[cfg(feature = "model")]
mod ps_response;
#[cfg(feature = "model")]
mod running_model;
#[cfg(feature = "model")]
mod show_request;
#[cfg(feature = "model")]
mod show_response;
#[cfg(feature = "model")]
mod show_model_details;

#[cfg(feature = "model")]
pub use copy_request::CopyRequest;
#[cfg(feature = "model")]
pub use list_response::ListResponse;
#[cfg(feature = "model")]
pub use model_summary::ModelSummary;
#[cfg(feature = "model")]
pub use ps_response::PsResponse;
#[cfg(feature = "model")]
pub use running_model::RunningModel;
#[cfg(feature = "model")]
pub use show_request::ShowRequest;
#[cfg(feature = "model")]
pub use show_response::ShowResponse;
#[cfg(feature = "model")]
pub use show_model_details::ShowModelDetails;
```

#### 2.2 Update `src/lib.rs`

Ensure public re-exports are also gated:

```rust
#[cfg(feature = "model")]
pub use primitives::{
    CopyRequest,
    ListResponse,
    ModelSummary,
    PsResponse,
    RunningModel,
    ShowModelDetails,
    ShowRequest,
    ShowResponse,
};
```

---

### Phase 3: Handle Dependencies

#### 3.1 Check `ModelDetails` Usage

`ModelDetails` is used in both:
- `ModelSummary` (list models)
- `RunningModel` (list running models)

**Decision:** `ModelDetails` should also be gated by `model` feature since it's only used by model-related types.

#### 3.2 Update `src/primitives/mod.rs` for `ModelDetails`

```rust
#[cfg(feature = "model")]
mod model_details;

#[cfg(feature = "model")]
pub use model_details::ModelDetails;
```

---

### Phase 4: Update Examples

#### 4.1 Update `Cargo.toml` - Example Configurations

Add `required-features = ["model"]` to the following examples:

```toml
[[example]]
name = "list_models_async"
required-features = ["model"]

[[example]]
name = "list_models_sync"
required-features = ["model"]

[[example]]
name = "list_running_models_async"
required-features = ["model"]

[[example]]
name = "list_running_models_sync"
required-features = ["model"]

[[example]]
name = "copy_model_async"
required-features = ["model"]

[[example]]
name = "copy_model_sync"
required-features = ["model"]

[[example]]
name = "show_model_async"
required-features = ["model"]

[[example]]
name = "show_model_sync"
required-features = ["model"]
```

---

### Phase 5: Update Tests

#### 5.1 Update `Cargo.toml` - Test Configurations

Add `required-features = ["model"]` to any integration tests that use these methods.

#### 5.2 Add Conditional Compilation to Unit Tests

For any unit tests within the primitive modules, add:

```rust
#[cfg(all(test, feature = "model"))]
mod tests {
    // existing tests
}
```

---

### Phase 6: Verify Feature Flag Inheritance

#### 6.1 Verify `Cargo.toml` Dependencies

Current configuration (already correct):
```toml
[features]
default = ["http", "primitives"]
model = ["http", "primitives"]
```

The `model` feature already depends on `http` and `primitives`, which is correct.

---

## File Change Summary

| File | Changes |
|------|---------|
| `src/http/api_async.rs` | Add `#[cfg(feature = "model")]` to 4 methods |
| `src/http/api_sync.rs` | Add `#[cfg(feature = "model")]` to 4 methods |
| `src/primitives/mod.rs` | Gate 9 modules/re-exports with `#[cfg(feature = "model")]` |
| `src/lib.rs` | Gate 8 public re-exports with `#[cfg(feature = "model")]` |
| `Cargo.toml` | Add `required-features` to 8 examples |
| Unit test files | Add conditional compilation where needed |

---

## Validation Checklist

After implementation, verify:

- [x] `cargo build` succeeds (default features - no model methods)
- [x] `cargo build --features model` succeeds (all model methods available)
- [x] `cargo test` succeeds (default features)
- [x] `cargo test --features model` succeeds (all tests pass)
- [x] `cargo test --all-features` succeeds
- [ ] Examples run correctly with `--features model`
- [ ] Documentation reflects all changes

---

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Breaking existing users | Users must add `model` feature to use these methods - document in CHANGELOG |
| Circular dependencies | ModelDetails dependency chain verified - no circular deps |
| Test failures | Run full test suite with `--all-features` before committing |

---

## Rollback Plan

If issues arise, revert by:
1. Removing all `#[cfg(feature = "model")]` annotations from methods and types
2. Removing `required-features` from examples in Cargo.toml
3. This is a simple revert as no logic changes are made

---

## Approval

**Awaiting user approval before proceeding with implementation.**
