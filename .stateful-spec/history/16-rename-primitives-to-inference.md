# Rename Feature "primitives" to "inference"

**Document Version:** 1.1
**Date:** 2026-02-02
**Status:** ✅ Implemented

## Overview

Now that the base package contains only inference-purpose endpoints (chat, generate, embed), renaming the `primitives` feature to `inference` provides better semantic clarity.

### Rationale

| Current Name | Types Included | Better Name |
|-------------|----------------|-------------|
| `primitives` | ChatRequest, ChatResponse, GenerateRequest, GenerateResponse, EmbedRequest, EmbedResponse, etc. | `inference` |

The `primitives` name is generic and doesn't convey the purpose of the types. After the model consolidation, all types in `src/primitives/` are specifically for **inference operations**:
- Chat completions
- Text generation
- Embeddings

---

## Current State

### Feature Flags (Cargo.toml)
```toml
[features]
default = ["http", "primitives"]
conveniences = ["http", "primitives"]
http = []
primitives = []
tools = ["dep:schemars", "dep:futures"]
model = ["http", "primitives"]
```

### Module Structure
```
src/
├── primitives/           # Contains inference types (chat, generate, embed)
│   ├── mod.rs
│   ├── chat_message.rs
│   ├── chat_request.rs
│   ├── chat_response.rs
│   ├── chat_role.rs
│   ├── embed_input.rs
│   ├── embed_request.rs
│   ├── embed_response.rs
│   ├── format_setting.rs
│   ├── generate_request.rs
│   ├── generate_response.rs
│   ├── keep_alive_setting.rs
│   ├── logprob.rs
│   ├── model_options.rs
│   ├── response_message.rs
│   ├── stop_setting.rs
│   ├── think_setting.rs
│   ├── token_logprob.rs
│   ├── tool_call.rs         # tools feature
│   ├── tool_call_function.rs # tools feature
│   ├── tool_definition.rs    # tools feature
│   ├── tool_function.rs      # tools feature
│   └── version.rs
├── model/                # Model management types (model feature)
└── http/                 # HTTP client
```

---

## Target State

### Feature Flags (Cargo.toml)
```toml
[features]
default = ["http", "inference"]
conveniences = ["http", "inference"]
http = []
inference = []
tools = ["dep:schemars", "dep:futures"]
model = ["http", "inference"]
```

### Module Structure
```
src/
├── inference/            # Renamed from primitives
│   ├── mod.rs
│   └── ... (same files)
├── model/
└── http/
```

---

## Implementation Plan

### Phase 1: Rename Feature Flag in Cargo.toml

Update `Cargo.toml`:

```toml
# Before
default = ["http", "primitives"]
conveniences = ["http", "primitives"]
primitives = []
model = ["http", "primitives"]

# After
default = ["http", "inference"]
conveniences = ["http", "inference"]
inference = []
model = ["http", "inference"]
```

---

### Phase 2: Rename Folder

Use `git mv` to preserve history:

```bash
git mv src/primitives src/inference
```

---

### Phase 3: Update `src/lib.rs`

#### 3.1 Update Module Declaration

```rust
// Before
#[cfg(feature = "primitives")]
pub mod primitives;

// After
#[cfg(feature = "inference")]
pub mod inference;
```

#### 3.2 Update Re-exports

```rust
// Before
#[cfg(feature = "primitives")]
pub use primitives::{
    ChatMessage, ChatRequest, ChatResponse, ChatRole,
    // ...
};

// After
#[cfg(feature = "inference")]
pub use inference::{
    ChatMessage, ChatRequest, ChatResponse, ChatRole,
    // ...
};
```

#### 3.3 Update Tool Types Re-export

```rust
// Before
#[cfg(all(feature = "primitives", feature = "tools"))]
pub use primitives::{ToolCall, ToolCallFunction, ToolDefinition, ToolFunction};

// After
#[cfg(all(feature = "inference", feature = "tools"))]
pub use inference::{ToolCall, ToolCallFunction, ToolDefinition, ToolFunction};
```

#### 3.4 Update Prelude

```rust
// Before
#[cfg(feature = "primitives")]
pub use crate::{
    ChatMessage, ChatRequest, ...
};

#[cfg(all(feature = "primitives", feature = "tools"))]
pub use crate::{ToolCall, ...};

// After
#[cfg(feature = "inference")]
pub use crate::{
    ChatMessage, ChatRequest, ...
};

#[cfg(all(feature = "inference", feature = "tools"))]
pub use crate::{ToolCall, ...};
```

---

### Phase 4: Update `src/inference/mod.rs`

Update module documentation:

```rust
// Before
//! Primitive types for Ollama API responses and requests

// After
//! Inference types for Ollama API responses and requests
//!
//! This module contains all data types used for inference operations:
//! chat completions, text generation, and embeddings.
```

Update feature conditionals within the module:

```rust
// Before
#[cfg(feature = "tools")]
mod tool_call;

// After (no change needed - "tools" feature is independent)
#[cfg(feature = "tools")]
mod tool_call;
```

---

### Phase 5: Update Tests

#### 5.1 Rename Test File

The test file `primitives_list_tests.rs` tests `ListResponse` which is a **model** type (not inference), so it should be renamed to `model_list_tests.rs`:

```bash
git mv tests/primitives_list_tests.rs tests/model_list_tests.rs
```

#### 5.2 Update Cargo.toml Test Entry

```toml
# Before
[[test]]
name = "primitives_list_tests"
required-features = ["model"]

# After
[[test]]
name = "model_list_tests"
required-features = ["model"]
```

**Rationale:** The test file tests `ListResponse` and related model types, not inference types. Naming it `model_list_tests.rs` is semantically correct and consistent with the `model` feature requirement.

---

### Phase 6: Update Documentation

#### 6.1 Update README.md

Replace all references to `primitives` feature with `inference`.

#### 6.2 Update ARCHITECTURE.md

Update feature descriptions and module documentation.

#### 6.3 Update spec/definition.md

Update feature matrix and module descriptions.

#### 6.4 Update DEV_NOTES.md

Update feature documentation.

#### 6.5 Update CHANGELOG.md

Add entry under Unreleased:

```markdown
### Changed
- **Renamed feature**: `primitives` → `inference` for better semantic clarity
  - Feature flag `primitives` renamed to `inference` in Cargo.toml
  - Module `src/primitives/` renamed to `src/inference/`
  - All inference-related types (chat, generate, embed) now under `inference` feature
  - **Breaking change**: Users using `features = ["primitives"]` must change to `features = ["inference"]`
```

---

## File Change Summary

| File | Action |
|------|--------|
| `Cargo.toml` | Rename feature `primitives` → `inference` |
| `src/primitives/` | Rename folder to `src/inference/` via `git mv` |
| `src/lib.rs` | Update all `primitives` references to `inference` |
| `src/inference/mod.rs` | Update module documentation |
| `tests/primitives_list_tests.rs` | Rename to `tests/model_list_tests.rs` (tests model types) |
| `README.md` | Update feature documentation |
| `ARCHITECTURE.md` | Update module descriptions |
| `DEV_NOTES.md` | Update feature documentation |
| `spec/definition.md` | Update feature matrix |
| `CHANGELOG.md` | Add breaking change entry |
| `DECISIONS.md` | Add decision entry |

---

## Validation Checklist

After implementation, verify:

- [x] `cargo build` succeeds (default features)
- [x] `cargo build --features inference` succeeds
- [x] `cargo build --features model` succeeds
- [x] `cargo build --all-features` succeeds
- [x] `cargo test` succeeds (default features) - 76 passed
- [x] `cargo test --features inference` succeeds
- [x] `cargo test --features model` succeeds - 94 passed
- [x] `cargo test --all-features` succeeds - 130 passed
- [x] All examples compile with correct features

---

## Breaking Changes

| Change | Migration |
|--------|-----------|
| Feature `primitives` renamed to `inference` | Change `features = ["primitives"]` to `features = ["inference"]` |
| Module `primitives` renamed to `inference` | Change `use ollama_oxide::primitives::*` to `use ollama_oxide::inference::*` |

---

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Breaking existing users | Document in CHANGELOG with migration guide |
| Missing references | Comprehensive grep for "primitives" before finalizing |
| Test failures | Run full test suite with `--all-features` |

---

## Rollback Plan

If issues arise, revert by:
1. `git mv src/inference src/primitives`
2. Restore Cargo.toml feature names
3. Restore src/lib.rs references
4. This is a simple rename operation with no logic changes

---

## Approval

**Approved and implemented on 2026-02-02.**
