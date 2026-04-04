# Move Model Primitives to "model" Folder

**Document Version:** 1.1
**Date:** 2026-02-02
**Status:** ✅ Implemented

## Overview

The model-related primitive types are currently located in `src/primitives/` but are gated behind the `model` feature. These files should be moved to `src/model/` to:

1. Keep feature-gated code organized together
2. Make it clear these types are not part of the default library
3. Follow the same pattern used for `CreateRequest`, `CreateResponse`, `DeleteRequest`, and `LicenseSetting`

---

## Files to Move

| Current Location | New Location |
|------------------|--------------|
| `src/primitives/copy_request.rs` | `src/model/copy_request.rs` |
| `src/primitives/list_response.rs` | `src/model/list_response.rs` |
| `src/primitives/model_details.rs` | `src/model/model_details.rs` |
| `src/primitives/model_summary.rs` | `src/model/model_summary.rs` |
| `src/primitives/ps_response.rs` | `src/model/ps_response.rs` |
| `src/primitives/running_model.rs` | `src/model/running_model.rs` |
| `src/primitives/show_model_details.rs` | `src/model/show_model_details.rs` |
| `src/primitives/show_request.rs` | `src/model/show_request.rs` |
| `src/primitives/show_response.rs` | `src/model/show_response.rs` |

**Total:** 9 files to move

---

## Implementation Plan

### Phase 1: Move Files

Move each file from `src/primitives/` to `src/model/`:

```
src/primitives/copy_request.rs      → src/model/copy_request.rs
src/primitives/list_response.rs     → src/model/list_response.rs
src/primitives/model_details.rs     → src/model/model_details.rs
src/primitives/model_summary.rs     → src/model/model_summary.rs
src/primitives/ps_response.rs       → src/model/ps_response.rs
src/primitives/running_model.rs     → src/model/running_model.rs
src/primitives/show_model_details.rs → src/model/show_model_details.rs
src/primitives/show_request.rs      → src/model/show_request.rs
src/primitives/show_response.rs     → src/model/show_response.rs
```

---

### Phase 2: Update `src/model/mod.rs`

Add module declarations and re-exports for the moved files:

```rust
// Existing modules
mod create_request;
mod create_response;
mod delete_request;
mod license_setting;

// Moved from primitives
mod copy_request;
mod list_response;
mod model_details;
mod model_summary;
mod ps_response;
mod running_model;
mod show_model_details;
mod show_request;
mod show_response;

// Existing re-exports
pub use create_request::CreateRequest;
pub use create_response::CreateResponse;
pub use delete_request::DeleteRequest;
pub use license_setting::LicenseSetting;

// New re-exports (moved from primitives)
pub use copy_request::CopyRequest;
pub use list_response::ListResponse;
pub use model_details::ModelDetails;
pub use model_summary::ModelSummary;
pub use ps_response::PsResponse;
pub use running_model::RunningModel;
pub use show_model_details::ShowModelDetails;
pub use show_request::ShowRequest;
pub use show_response::ShowResponse;
```

---

### Phase 3: Update `src/primitives/mod.rs`

Remove the module declarations and re-exports for the moved files:

**Remove these lines:**
```rust
// Remove module declarations
#[cfg(feature = "model")]
mod copy_request;
#[cfg(feature = "model")]
mod list_response;
#[cfg(feature = "model")]
mod model_details;
#[cfg(feature = "model")]
mod model_summary;
#[cfg(feature = "model")]
mod ps_response;
#[cfg(feature = "model")]
mod running_model;
#[cfg(feature = "model")]
mod show_model_details;
#[cfg(feature = "model")]
mod show_request;
#[cfg(feature = "model")]
mod show_response;

// Remove re-exports
#[cfg(feature = "model")]
pub use copy_request::CopyRequest;
// ... etc
```

---

### Phase 4: Update `src/lib.rs`

Change re-exports from `primitives` to `model`:

**Before:**
```rust
#[cfg(all(feature = "primitives", feature = "model"))]
pub use primitives::{
    CopyRequest,
    ListResponse,
    ModelDetails,
    ModelSummary,
    PsResponse,
    RunningModel,
    ShowModelDetails,
    ShowRequest,
    ShowResponse,
};
```

**After:**
```rust
#[cfg(feature = "model")]
pub use model::{
    CopyRequest,
    CreateRequest,
    CreateResponse,
    DeleteRequest,
    LicenseSetting,
    ListResponse,
    ModelDetails,
    ModelSummary,
    PsResponse,
    RunningModel,
    ShowModelDetails,
    ShowRequest,
    ShowResponse,
};
```

---

### Phase 5: Update Prelude in `src/lib.rs`

Update the prelude to use `model` instead of `primitives`:

**Before:**
```rust
#[cfg(all(feature = "primitives", feature = "model"))]
pub use crate::{
    CopyRequest,
    CreateRequest,
    // ...
};
```

**After:**
```rust
#[cfg(feature = "model")]
pub use crate::{
    CopyRequest,
    CreateRequest,
    // ...
};
```

---

### Phase 6: Update Internal Imports

Update any internal imports in moved files that reference other model types:

- `model_summary.rs` imports `ModelDetails`
- `running_model.rs` imports `ModelDetails`
- `list_response.rs` imports `ModelSummary`
- `ps_response.rs` imports `RunningModel`
- `show_response.rs` imports `ShowModelDetails`

Change from:
```rust
use crate::primitives::ModelDetails;
```

To:
```rust
use super::ModelDetails;
// or
use crate::model::ModelDetails;
```

---

## File Change Summary

| File | Action |
|------|--------|
| `src/primitives/*.rs` (9 files) | Move to `src/model/` |
| `src/model/mod.rs` | Add 9 module declarations + re-exports |
| `src/primitives/mod.rs` | Remove 9 module declarations + re-exports |
| `src/lib.rs` | Update re-exports source |
| Moved files | Update internal imports if needed |

---

## Validation Checklist

After implementation, verify:

- [x] `cargo build` succeeds (default features)
- [x] `cargo build --features model` succeeds
- [x] `cargo test` succeeds (default features)
- [x] `cargo test --features model` succeeds
- [x] `cargo test --all-features` succeeds

---

## Benefits

1. **Cleaner organization:** All model-related code in one place
2. **Clear feature boundary:** `src/model/` = `model` feature
3. **Simplified conditionals:** No need for `#[cfg(all(feature = "primitives", feature = "model"))]`
4. **Consistent pattern:** Matches existing structure with `CreateRequest`, etc.

---

## Approval

**Awaiting user approval before proceeding with implementation.**
