# Development Notes

This document contains internal development notes, architectural decisions, and implementation details for ollama-oxide.

## Project Status

**Current Version:** 0.1.2
**Status:** Phase 1 Complete - All 12 endpoints implemented (non-streaming)
**Last Updated:** 2026-02-15

## Architecture Overview

### Single-Crate Structure

**Decision Date:** 2026-01-11

The project uses a **single-crate architecture** with modular organization:

```
ollama-oxide/
└── src/
    ├── lib.rs           # Main library entry point
    ├── inference/       # Inference types: chat, generate, embed (default feature)
    ├── http/            # HTTP client layer (default feature)
    ├── model/           # Model management types (optional feature)
    ├── tools/           # Tool types + ergonomic function calling (optional feature)
    └── conveniences/    # High-level API (optional feature)
```

**Rationale:**
- **Simpler build process**: No workspace complexity
- **Easier maintenance**: Single version, single release
- **Clearer dependencies**: Feature flags instead of crate dependencies
- **Better for library users**: One dependency instead of multiple
- **Faster compile times**: No cross-crate boundaries

**Changed From:** Previously planned as 5-crate workspace (ollama-oxide, primitives, http-core, conveniences, samples)

### Design Philosophy

1. **Modular Design**: Clear separation via modules and feature flags
2. **Type Safety**: Leverage Rust's type system for API correctness
3. **Async First**: Built on Tokio for async/await support
4. **Minimal Dependencies**: Keep dependency tree lean
5. **OpenAPI Driven**: Follow Ollama's official API specification
6. **Feature-Based**: Optional functionality via Cargo features
7. **Single-Concern Files**: One primary type per file with implementations
8. **Generic Abstractions**: Reusable helpers with trait bounds

### Feature Flags

```toml
[features]
default = ["http", "inference"]       # Standard usage (inference only)
conveniences = ["http", "inference"]  # High-level APIs
http = []                             # HTTP client layer
inference = []                        # Inference types (chat, generate, embed)
tools = ["dep:schemars", "dep:futures"] # Ergonomic function calling
model = ["http", "inference"]         # All model operations (opt-in)
```

**Feature Matrix:**

| Feature | Dependencies | Purpose |
|---------|-------------|---------|
| `default` | `http`, `inference` | Standard usage - inference APIs (generate, chat, embed, version) |
| `inference` | - | Standalone inference types for chat, generate, embed |
| `http` | - | HTTP client implementation (async/sync) |
| `tools` | `schemars`, `futures` | Tool types (ToolCall, ToolDefinition) + ergonomic function calling |
| `model` | `http`, `inference` | All model operations: list, show, copy, create, delete (opt-in) |
| `conveniences` | `http`, `inference` | High-level ergonomic APIs |

**Model Feature Contents:**
- Types: `ListResponse`, `ModelSummary`, `ModelDetails`, `PsResponse`, `RunningModel`, `ShowRequest`, `ShowResponse`, `ShowModelDetails`, `CopyRequest`, `CreateRequest`, `CreateResponse`, `DeleteRequest`, `LicenseSetting`, `PullRequest`, `PullResponse`, `PushRequest`, `PushResponse`
- Methods: `list_models()`, `list_running_models()`, `show_model()`, `copy_model()`, `create_model()`, `delete_model()`, `pull_model()`, `push_model()` (async and sync variants)

## Current State

### Implemented (Phase 1 Complete)
- Single-crate configuration with feature flags
- Module structure (inference, http, model, tools, conveniences)
- Dependency setup (tokio, serde, reqwest, async-trait)
- 12 OpenAPI specifications documented
- Comprehensive documentation foundation
- **All 12 API endpoints** (async + sync, non-streaming mode):
  - GET /api/version
  - GET /api/tags
  - GET /api/ps
  - POST /api/copy
  - DELETE /api/delete
  - POST /api/show
  - POST /api/embed
  - POST /api/generate
  - POST /api/chat
  - POST /api/create
  - POST /api/pull
  - POST /api/push
- Error handling with `thiserror`
- HTTP client with retry logic and exponential backoff
- POST helper methods (`post_empty_with_retry`, `post_empty_blocking_with_retry`, `post_with_retry`, `post_blocking_with_retry`)
- DELETE helper methods (`delete_empty_with_retry`, `delete_empty_blocking_with_retry`)
- Primitive types: `VersionResponse`, `ListResponse`, `ModelSummary`, `ModelDetails`, `PsResponse`, `RunningModel`, `CopyRequest`, `DeleteRequest`, `ShowRequest`, `ShowResponse`, `ShowModelDetails`, `EmbedRequest`, `EmbedResponse`, `EmbedInput`, `ModelOptions`, `GenerateRequest`, `GenerateResponse`, `ThinkSetting`, `FormatSetting`, `KeepAliveSetting`, `StopSetting`, `TokenLogprob`, `Logprob`, `ChatRequest`, `ChatResponse`, `ChatMessage`, `ChatRole`, `ResponseMessage`, `CreateRequest`, `CreateResponse`, `LicenseSetting`, `PullRequest`, `PullResponse`, `PushRequest`, `PushResponse`
- 330+ unit and integration tests
- Examples for all endpoints including tools
- `chat_with_tools_async` example: Complete tool calling flow with mock weather service

### In Progress
- v0.1.0 release preparation

### Completed (v0.1.0)
- [x] Implement POST /api/show endpoint
- [x] Implement POST /api/embed endpoint
- [x] Implement POST /api/generate endpoint (non-streaming only)
- [x] Implement POST /api/chat endpoint (non-streaming only)
- [x] Implement POST /api/create endpoint (non-streaming only)
- [x] Implement POST /api/pull endpoint (non-streaming only)
- [x] Implement POST /api/push endpoint (non-streaming only)

### TODO (v0.2.0)
- [ ] Implement streaming support for generate, chat, create, pull, push endpoints

### TODO (v0.3.0+)
- [ ] Create `conveniences` module
- [ ] Performance benchmarks

## Technical Decisions

### Single-Concern File Structure

**Decision Date:** 2026-01-14

**Implementation:**
- Each type defined in its own file with implementations
- `mod.rs` files serve as re-export facades
- Example: `src/error.rs` contains Error enum and Result type; `lib.rs` imports from error module

**Benefits:**
- Clear file boundaries matching type boundaries
- Easy navigation and maintenance
- Consistent pattern across codebase

### HTTP Retry Abstraction

**Decision Date:** 2026-01-14

**Implementation:**
- Added `get_with_retry<T>()` and `get_blocking_with_retry<T>()` to OllamaClient
- Added `post_empty_with_retry<R>()` and `post_empty_blocking_with_retry<R>()` for POST with empty response
- Added `post_with_retry<R, T>()` and `post_blocking_with_retry<R, T>()` for POST with JSON response
- Added `delete_empty_with_retry<R>()` and `delete_empty_blocking_with_retry<R>()` for DELETE with empty response
- Generic over response type with `serde::de::DeserializeOwned` bound (GET)
- Generic over request type with `serde::Serialize` bound (POST, DELETE)
- Automatic retry on network errors and 5xx server errors
- No retry on 4xx client errors (e.g., 404 model not found)
- Exponential backoff: 100ms × (attempt + 1)
- Marked `pub(super)` for http module internal use

**Code Reduction:**
- Endpoint implementations: 60 lines → 6 lines (90% reduction)
- Projected for 12 endpoints: 720 lines → 168 lines (78% reduction)

**Benefits:**
- Single source of truth for retry logic
- Type-safe with compiler guarantees
- Easy to extend for POST/streaming
- Consistent behavior across endpoints

### Type Erasure Pattern (ErasedTool)

**Decision Date:** 2026-01-26

The `ErasedTool` trait uses **type erasure** to enable storing heterogeneous tools in a single `ToolRegistry`.

**Problem:**
```rust
// Tool trait is NOT object-safe (has associated types)
trait Tool {
    type Params;   // Concrete type known at compile time
    type Output;   // Concrete type known at compile time
}
// Cannot create: Vec<Box<dyn Tool>>
```

**Solution:**
```rust
// ErasedTool is object-safe (types "erased" to JSON)
trait ErasedTool {
    fn execute_erased(&self, args: Value) -> ToolResult<Value>;
}
// Can create: Vec<Box<dyn ErasedTool>>
```

**Naming Convention:**
- "Erased" indicates type information is removed at compile time
- Common in Rust ecosystem (e.g., `erased-serde`, `type-erased` crates)
- `ToolWrapper<T>` bridges typed `Tool` → type-erased `ErasedTool`

**Location:** `src/tools/erased_tool.rs`

### OllamaClient Ergonomic Constructors

**Decision Date:** 2026-02-15

**Implementation:**
- Added `OllamaClient::with_base_url_and_timeout(base_url, timeout)` convenience constructor
- Provides ergonomic alternative to manual `ClientConfig` construction when only URL and timeout need customization

**Before:**
```rust
let client = OllamaClient::new(ClientConfig::with_base_url_and_timeout(
    "http://myserver:8080".to_string(),
    Duration::from_secs(60),
))?;
```

**After:**
```rust
let client = OllamaClient::with_base_url_and_timeout(
    "http://myserver:8080",
    Duration::from_secs(60),
)?;
```

### ClientConfig Private Fields and URL Validation Invariant

**Decision Date:** 2026-02-15

**Implementation:**
- `ClientConfig` fields (`base_url`, `timeout`, `max_retries`) are now private
- All constructors (`new()`, `with_base_url()`, `with_base_url_and_timeout()`) validate URLs and return `Result<Self>`
- Getter methods (`base_url()`, `timeout()`, `max_retries()`) provide read-only access
- `Default` trait impl provides the only non-validated construction (uses known-good localhost URL)

**Rationale:**
- With `pub` fields, users could bypass URL validation via struct literal construction
- Making fields private enforces that all construction goes through validated constructors
- Eliminates redundant URL validation in `OllamaClient::new()` (previously duplicated as a safety net)

**Impact:**
- `OllamaClient::with_base_url()` and `with_base_url_and_timeout()` now delegate to `ClientConfig` constructors
- All internal field access uses getter methods
- ~115 struct literal sites in tests migrated to `ClientConfig::new(...).unwrap()`

### Feature-Based Design Strategy

**Decision Date:** 2026-02-01

**Implementation:**
The library uses Cargo features to provide a modular, opt-in design where developers include only what they need.

**Key Design Decisions:**

1. **Opt-in for Destructive Operations**: The `model` feature isolates `CreateRequest`, `DeleteRequest`, and related API methods. Developers must explicitly enable model creation/deletion to prevent accidental misuse.

2. **Conditional Dependencies**: The `tools` feature brings in `schemars` and `futures` only when needed, keeping the default dependency tree lean.

3. **Three-Level Conditional Compilation**:
   - Module level: `#[cfg(feature = "tools")] pub mod tools;`
   - Struct field level: `#[cfg(feature = "tools")] pub tools: Option<Vec<ToolDefinition>>`
   - Method level: `#[cfg(feature = "tools")] pub fn with_tools(...)`

4. **Example/Test Gating**: Examples and tests requiring specific features use `required-features` in Cargo.toml.

**Benefits:**
- Minimal footprint by default
- Reduced compile times for users who don't need all features
- Clear separation of concerns
- Protection against accidental destructive operations
- Flexibility for different use cases (data types only, full client, with tools, etc.)

**Developer Experience Scenarios:**

| Use Case | Cargo.toml | What's Included |
|----------|------------|-----------------|
| Basic API client | (default) | HTTP client + all inference types |
| Data types only | `default-features = false, features = ["inference"]` | Just structs for JSON parsing |
| With function calling | `features = ["tools"]` | + ToolRegistry, auto-schema generation |
| Full with model management | `features = ["tools", "model"]` | Everything including model creation/deletion |

### Example Naming Convention

**Decision Date:** 2026-01-26

**Pattern:** `<feature>_<variant>_<mode>.rs`

**Examples:**
- `chat_with_tools_async.rs` - Chat with tool calling (weather example)
- `chat_with_tools_registry_async.rs` - Chat with ToolRegistry pattern
- `tools_async.rs` - Basic tool definition examples
- `chat_async.rs` - Basic chat without tools

**Rationale:**
- Clear feature identification in filename
- Consistent `_async` / `_sync` suffix for execution mode
- Grouped related examples by prefix

### HTTP Client: reqwest

**Rationale:**
- Mature, well-maintained async HTTP client
- Excellent tokio integration
- Built-in JSON support via serde
- HTTP/2 support
- Cookie handling

**Configuration:**
- Features: blocking, cookies, http2, json, native-tls
- Version: 0.13.1

### Async Runtime: Tokio

**Rationale:**
- Industry standard for async Rust
- Rich ecosystem
- Excellent performance
- Well-documented

**Configuration:**
- Features: macros, rt-multi-thread
- Version: 1.49.0

### Serialization: Serde

**Rationale:**
- De facto standard for Rust serialization
- Excellent derive macro support
- Comprehensive JSON support

**Configuration:**
- serde: 1.0.228
- serde_json: 1.0.149

## API Implementation Strategy

### Versioning Strategy Update (2026-01-17)
**Decision:** All 12 endpoints implemented in v0.1.0 (non-streaming mode), streaming deferred to v0.2.0

**Rationale:**
- Provides complete API coverage sooner
- Streaming is an enhancement, not a blocker for basic functionality
- Allows users to use all endpoints immediately with `stream: false`
- Clear separation between functionality (v0.1.0) and streaming (v0.2.0)

### Phase 1 (v0.1.0): Foundation + All Endpoints ✅ COMPLETE
All 12 endpoints implemented in non-streaming mode:
- 3 GET endpoints (version, tags, ps) ✅
- 2 Simple POST/DELETE endpoints (copy, delete) ✅
- 2 Medium complexity endpoints (show, embed) ✅
- 5 Complex endpoints in non-streaming mode (generate, chat, create, pull, push) ✅
- Full test coverage (330+ tests)

### Phase 2 (v0.2.0): Streaming Implementation
Add streaming support to applicable endpoints:
- POST /api/generate - streaming responses
- POST /api/chat - streaming responses
- POST /api/create - progress streaming
- POST /api/pull - progress streaming
- POST /api/push - progress streaming
- Stream helper utilities

### Phase 3 (v0.3.0): Conveniences Module
Build high-level APIs in `conveniences` module:
- Optional feature flag
- Simplified method signatures
- Builder patterns
- Common workflows

### Phase 4 (v0.4.0): Examples & Production
Polish and prepare for v1.0.0:
- Comprehensive examples in `/examples`
- API documentation complete
- Performance benchmarks
- Production guides

## OpenAPI Specification

Location: `spec/alloma_api.yaml`

Current endpoints to implement:
- `/api/generate` - Text generation (streaming & non-streaming)

Future endpoints:
- Model management
- Embeddings
- Additional completion modes

## Testing Strategy

### No Doc Tests Policy
**Decision Date:** 2026-01-26

**Rationale:**
- **Feature flag complexity**: Many types are gated behind feature flags (`tools`, `inference`, `http`, `model`), making doc tests hard to maintain
- **Maintenance burden**: Doc tests require keeping code in sync across documentation and actual tests
- **Coverage duplication**: All public interfaces are already covered by tests in `tests/` folder
- **Simpler workflow**: Run `cargo test` without doc test failures due to feature mismatches

**Testing locations:**
- **Unit tests**: Inside each component file (e.g., `src/inference/chat_request.rs` has `#[cfg(test)] mod tests`)
- **Public interface tests**: `tests/` folder for integration-style unit tests
- **Integration tests**: `examples/` folder for real Ollama server tests

### Unit Tests (inside source files)
**Location:** `src/**/*.rs` (in `#[cfg(test)] mod tests` blocks)
**Purpose:** Test internal/non-public behavior:
- Private helper functions
- Edge cases specific to the implementation
- Serialization/deserialization details
- Feature-gated code paths

### Public Interface Tests (`tests/` folder)
**Location:** `tests/*.rs`
**Purpose:** All tests in the `tests/` folder must be unit tests that:
- Do NOT require external services (Ollama server)
- Use mocking (mockito) for HTTP interactions
- Can run in CI/CD without additional setup
- Test public API contracts
- Validate request/response round-trips
- Cover error handling paths

### Integration Tests (`examples/` folder)
**Location:** `examples/*.rs`
**Purpose:** Integration tests are implemented as examples that:
- Require a running Ollama server
- Demonstrate real API interactions
- Serve as usage documentation
- Can be run manually: `cargo run --example <name>`

**Rationale:** This separation ensures:
- `cargo test` always succeeds without external dependencies
- Examples serve dual purpose: documentation + integration testing
- CI/CD pipelines run reliably
- Developers can test against real Ollama when needed

### Mocking Strategy
- Use `mockito` crate for HTTP mocking in unit tests
- All HTTP interactions in `tests/` folder are mocked
- Real Ollama instance used only via examples

## Performance Considerations

### Current Focus
- Correctness and usability over premature optimization
- Establish baseline performance metrics

### Future Optimization Areas
- Connection pooling
- Request batching
- Response streaming efficiency
- Memory allocations

## Known Issues / Limitations

- Project in early development stage
- API not yet stable
- Limited endpoint coverage
- No production usage yet

## Development Environment

### Recommended Tools
- rust-analyzer (LSP)
- cargo-watch (auto-rebuild)
- cargo-expand (macro debugging)
- cargo-audit (security)

### Testing with Ollama
Ensure Ollama is running locally:
```bash
ollama serve
```

Default endpoint: `http://localhost:11434`

## Code Style Guidelines

### Naming Conventions
- Types: `PascalCase`
- Functions: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

### Module Organization

**File Structure Pattern:**
- One primary type per file with implementations
- `mod.rs` as pure re-export facade
- Single-concern principle throughout

**Example:**
```rust
// src/http/mod.rs - Re-export facade
mod config;
mod client;
mod api_async;
mod api_sync;

pub use config::ClientConfig;
pub use client::OllamaClient;
pub use api_async::OllamaApiAsync;
pub use api_sync::OllamaApiSync;

// src/http/client.rs - Implementation
pub struct OllamaClient { ... }
impl OllamaClient {
    pub(super) async fn get_with_retry<T>(&self, url: &str) -> Result<T>
    where T: serde::de::DeserializeOwned { ... }
}
```

### Error Handling
Use `thiserror` for error types (to be added):
```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error: {message}")]
    Api { message: String },
}
```

## Future Considerations

### Potential Features
- CLI tool for Ollama interaction
- Server-side events for streaming
- Batch processing utilities
- Model management helpers
- Custom middleware/interceptors

### Breaking Changes Policy
- Document all breaking changes in CHANGELOG
- Follow semantic versioning strictly
- Provide migration guides for major versions

## Resources

- [Ollama GitHub](https://github.com/ollama)
- [Ollama API Docs](https://github.com/ollama/ollama/blob/main/docs/api.md)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Async Book](https://rust-lang.github.io/async-book/)

## POST /api/chat Implementation

### Messages API

`ChatRequest::new()` accepts `IntoIterator<Item = ChatMessage>`:

```rust
pub fn new<M, I>(model: M, messages: I) -> Self
where
    M: Into<String>,
    I: IntoIterator<Item = ChatMessage>,
```

### Custom Conversation Type

```rust
use ollama_oxide::{ChatMessage, ChatRequest};
use chrono::{DateTime, Utc};

struct TimestampedMessage {
    message: ChatMessage,
    created_at: DateTime<Utc>,
}

struct Conversation {
    id: uuid::Uuid,
    messages: Vec<TimestampedMessage>,
}

impl IntoIterator for Conversation {
    type Item = ChatMessage;
    type IntoIter = std::vec::IntoIter<ChatMessage>;

    fn into_iter(mut self) -> Self::IntoIter {
        self.messages.sort_by_key(|m| m.created_at);
        self.messages.into_iter().map(|m| m.message).collect::<Vec<_>>().into_iter()
    }
}

// Usage:
let conversation = load_from_database(id)?;
let request = ChatRequest::new(model, conversation);
```

### API Flow

```mermaid
sequenceDiagram
    participant User
    participant Client as OllamaClient
    participant Server as Ollama Server

    User->>Client: ChatRequest::new(model, messages)
    User->>Client: .with_tools(tools)
    User->>Client: client.chat(&request)
    Client->>Server: POST /api/chat
    Server-->>Client: ChatResponse
    Client-->>User: Result<ChatResponse>
```

### Tool Execution Flow (Client-Side)

```mermaid
sequenceDiagram
    autonumber
    participant App as Your Rust App
    participant Server as Ollama Server

    App->>Server: ChatRequest { messages, tools }
    Server-->>App: ChatResponse { tool_calls: [...] }

    rect rgb(255, 240, 220)
        Note over App: EXECUTION HAPPENS HERE
        App->>App: Parse tool_call.function_name()
        App->>App: Execute real Rust function
        App->>App: Get actual result
    end

    App->>Server: ChatRequest { messages: [..., tool_result] }
    Server-->>App: ChatResponse { content: "..." }
```

### Multi-turn Conversation Flow

```mermaid
stateDiagram-v2
    [*] --> AddUserMessage
    AddUserMessage --> SendRequest
    SendRequest --> ReceiveResponse

    state CheckToolCalls <<choice>>
    ReceiveResponse --> CheckToolCalls

    CheckToolCalls --> HasToolCalls : tool_calls present
    CheckToolCalls --> NoToolCalls : no tool_calls

    state HasToolCalls {
        [*] --> ExecuteLocally
        ExecuteLocally --> BuildToolMessage
        BuildToolMessage --> [*]
    }

    HasToolCalls --> SendRequest
    NoToolCalls --> Done
    Done --> [*]
```

### Response Helper Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| `content()` | `Option<&str>` | Assistant's text response |
| `tool_calls()` | `Option<&[ToolCall]>` | Tool calls array if present |
| `has_tool_calls()` | `bool` | Check if any tool calls exist |
| `thinking()` | `Option<&str>` | Thinking output if enabled |
| `is_done()` | `bool` | Check if generation completed |

---

## Questions & Decisions Log

### Q: Should we support sync and async APIs?
**Decision:** Start with async-first. Add sync wrapper if there's demand.
**Rationale:** Ollama API is inherently I/O bound, async is more natural.

### Q: How to handle streaming responses?
**Decision:** Deferred to v0.2.0. Will evaluate tokio-stream vs async-stream
**Status:** Deferred - v0.1.0 focuses on non-streaming endpoints only

### Q: Error handling strategy?
**Decision:** Use Result with custom Error enum using `thiserror`
**Status:** ✅ Implemented

Error variants:
- `HttpError` - HTTP request/response errors
- `HttpStatusError` - HTTP status code errors (e.g., 404, 400)
- `SerializationError` - JSON serialization/deserialization errors
- `ApiError` - Ollama API-specific errors
- `ConnectionError` - Connection/network errors
- `InvalidUrlError` - URL parsing errors
- `TimeoutError` - Request timeout errors
- `MaxRetriesExceededError` - Maximum retry attempts exceeded

---

**Note:** This document is for internal development reference. Update as the project evolves.
