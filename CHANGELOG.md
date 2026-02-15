# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Feature-based library design**: Modular opt-in architecture via Cargo features
  - `tools` feature: Ergonomic function calling with auto-generated JSON schemas
    - `Tool` trait for type-safe tool definitions
    - `ToolRegistry` for automatic dispatch
    - Optional dependencies: `schemars`, `futures`
    - Examples: `tools_async`, `chat_with_tools_async`, `chat_with_tools_registry_async`
  - `model` feature: Model creation/deletion (opt-in for destructive operations)
    - `CreateRequest`, `CreateResponse`, `DeleteRequest`, `LicenseSetting` types
    - Gated API methods: `create_model()`, `delete_model()`
    - Examples: `model_create_async`, `model_delete_async`
  - Three-level conditional compilation: module, struct field, and method level
  - Example and test gating via `required-features` in Cargo.toml
- **New example: `chat_with_tools_async`**: Complete chat with tools flow using weather API
  - Demonstrates full tool call cycle (request → tool calls → execution → response)
  - Mock weather service simulating HTTP calls to Open-Meteo API
  - Type-safe tool implementation using `Tool` trait
  - Multi-turn conversation with tool results
- **Renamed example**: `tool_registry_async` → `chat_with_tools_registry_async` for consistent naming
- **POST /api/generate endpoint**: Text generation (non-streaming)
  - `GenerateRequest` primitive type with full builder pattern
  - `GenerateResponse` primitive type with helper methods (text(), is_done(), tokens_per_second(), duration conversions)
  - `ThinkSetting` enum for controlling thinking output (bool or "high"/"medium"/"low")
  - `FormatSetting` enum for output format (string "json" or JSON schema)
  - `KeepAliveSetting` enum for model caching duration (string or seconds)
  - `StopSetting` enum for stop sequences (single or multiple)
  - `TokenLogprob` and `Logprob` primitive types for log probability support
  - Updated `ModelOptions` with `stop` field and `with_stop()` builder
  - `generate()` async method
  - `generate_blocking()` sync method
  - 79 new unit tests with mocking
  - Examples: `generate_async.rs`, `generate_sync.rs`, `generate_concise.rs`
- **POST /api/embed endpoint**: Generate text embeddings
  - `EmbedRequest` primitive type with builder pattern support
  - `EmbedResponse` primitive type with embeddings vector
  - `EmbedInput` enum for single text or batch input
  - `ModelOptions` primitive type for model parameters (temperature, top_k, top_p, etc.)
  - `embed()` async method
  - `embed_blocking()` sync method
  - 28 new unit tests with mocking
  - Examples: `embed_async.rs`, `embed_sync.rs`
- **POST /api/show endpoint**: Show detailed model information
  - `ShowRequest` primitive type with `new()` and `verbose()` constructors
  - `ShowResponse` primitive type with `has_capability()` helper method
  - `ShowModelDetails` primitive type for model metadata
  - `show_model()` async method
  - `show_model_blocking()` sync method
  - POST helper methods: `post_with_retry()`, `post_blocking_with_retry()`
  - 26 new unit tests with mocking
  - Examples: `show_model_async.rs`, `show_model_sync.rs`
- **DELETE /api/delete endpoint**: Delete a model from the server
  - `DeleteRequest` primitive type with `model` field
  - `delete_model()` async method
  - `delete_model_blocking()` sync method
  - DELETE helper methods: `delete_empty_with_retry()`, `delete_empty_blocking_with_retry()`
  - 10 new unit tests with mocking
  - Examples: `delete_model_async.rs`, `delete_model_sync.rs`
- **POST /api/copy endpoint**: Copy/duplicate a model
  - `CopyRequest` primitive type with `source` and `destination` fields
  - `copy_model()` async method
  - `copy_model_blocking()` sync method
  - `HttpStatusError(u16)` error variant for HTTP status code errors
  - POST helper methods: `post_empty_with_retry()`, `post_empty_blocking_with_retry()`
  - 16 new unit tests with mocking
  - Examples: `copy_model_async.rs`, `copy_model_sync.rs`
- **POST /api/pull endpoint**: Download models from registry (non-streaming)
  - `PullRequest` type with builder pattern (model, insecure option)
  - `PullResponse` type with helper methods (`status()`, `is_success()`)
  - `pull_model()` async method
  - `pull_model_blocking()` sync method
  - 8 new unit tests with mocking
  - Examples: `pull_model_async.rs`, `pull_model_sync.rs`
- **POST /api/push endpoint**: Upload models to registry (non-streaming)
  - `PushRequest` type with builder pattern (model, insecure option)
  - `PushResponse` type with helper methods (`status()`, `is_success()`)
  - `push_model()` async method
  - `push_model_blocking()` sync method
  - 10 new unit tests with mocking
  - Examples: `push_model_async.rs`, `push_model_sync.rs`
- **GET /api/ps endpoint**: List currently running models
  - `PsResponse`, `RunningModel` primitive types
  - `list_running_models()` async method
  - `list_running_models_blocking()` sync method
  - 13 new unit tests with mocking
  - Examples: `list_running_models_async.rs`, `list_running_models_sync.rs`
- **GET /api/tags endpoint**: List locally available models
  - `ListResponse`, `ModelSummary`, `ModelDetails` primitive types
  - `list_models()` async method
  - `list_models_blocking()` sync method
  - Comprehensive test coverage (33 new tests)
- **Examples for GET /api/version**:
  - `get_version_async.rs` - Async usage example
  - `get_version_sync.rs` - Sync/blocking usage example
  - `get_version_custom.rs` - Custom configuration example
- **Examples for GET /api/tags**:
  - `list_models_async.rs` - Async list models example
  - `list_models_sync.rs` - Sync list models example

- **`OllamaClient::with_base_url_and_timeout()` convenience constructor**: Create client with custom URL and timeout in a single call
- **Early URL validation in `ClientConfig` constructors**: `ClientConfig::new()`, `with_base_url()`, and `with_base_url_and_timeout()` now validate URLs immediately, returning `Result<Self>` instead of `Self`
- **`ClientConfig` getter methods**: `base_url()`, `timeout()`, `max_retries()` for accessing private fields

### Changed
- **`ClientConfig` fields are now private**: Fields `base_url`, `timeout`, and `max_retries` are no longer `pub`. All construction must go through validated constructors (`new()`, `with_base_url()`, `with_base_url_and_timeout()`, or `Default`). This enforces URL validation invariant and prevents bypassing validation via struct literals.
- **`ClientConfig` constructors return `Result<Self>`**: `new()`, `with_base_url()`, and `with_base_url_and_timeout()` now return `Result<Self>` instead of `Self`, performing URL validation at construction time.
- **Removed redundant URL validation from `OllamaClient::new()`**: Since `ClientConfig` now enforces URL validation at construction, the duplicate validation in `OllamaClient::new()` was removed.
- **`OllamaClient::with_base_url()` and `with_base_url_and_timeout()` now use `ClientConfig` constructors**: Instead of struct literals, these methods delegate to validated `ClientConfig` constructors.
- **Tool types consolidated in `src/tools/` module**: Moved `ToolCall`, `ToolCallFunction`, `ToolDefinition`, `ToolFunction` from `src/inference/` to `src/tools/`
  - Simplified feature gating: Tool types now require only `tools` feature, not `all(feature = "inference", feature = "tools")`
  - Updated imports in chat types to use `crate::tools::` instead of `super::`
  - Cleaner module separation: API data structures (inference) vs Rust abstractions (tools)
- **Renamed feature**: `primitives` → `inference` for better semantic clarity
  - Feature flag `primitives` renamed to `inference` in Cargo.toml
  - Module `src/primitives/` renamed to `src/inference/`
  - All inference-related types (chat, generate, embed) now under `inference` feature
  - Test file `primitives_list_tests.rs` renamed to `model_list_tests.rs`
  - **Breaking change**: Users using `features = ["primitives"]` must change to `features = ["inference"]`
- **Model feature consolidation**: All model-related types and methods now behind `model` feature
  - Moved 9 types from `src/inference/` to `src/model/`: `CopyRequest`, `ListResponse`, `ModelDetails`, `ModelSummary`, `PsResponse`, `RunningModel`, `ShowModelDetails`, `ShowRequest`, `ShowResponse`
  - Gated methods: `list_models()`, `list_running_models()`, `show_model()`, `copy_model()` now require `model` feature
  - Updated Cargo.toml with `required-features = ["model"]` for 8 examples and 5 tests
  - Simplified feature conditionals: `#[cfg(feature = "model")]` instead of `#[cfg(all(feature = "inference", feature = "model"))]`
  - **Breaking change**: Users must add `model` feature to use model-related APIs
- **Feature flag architecture**: Expanded from 4 features to 6
  - New: `tools` feature for ergonomic function calling
  - New: `model` feature for model creation/deletion (opt-in)
  - Updated dependency configuration for optional deps
- **Documentation updates**: All critical documents reflect feature-based design
  - README.md: Feature flags table and usage examples
  - ARCHITECTURE.md: Feature dependency graph and conditional compilation patterns
  - CONTRIBUTING.md: Feature development guidelines
  - DEV_NOTES.md: Feature-based design decisions
  - spec/definition.md: Updated module descriptions
- **Versioning strategy update**: All 12 endpoints will be implemented in v0.1.0 (non-streaming mode)
  - Streaming features moved to v0.2.0
  - v0.3.0 will focus on conveniences module
  - v0.4.0 will focus on examples and production readiness

### Planned for v0.2.0
- Streaming support for generate, chat, create, pull, push endpoints

## [0.1.0] - 2025-01-10

### Added
- Initial project structure and Cargo workspace setup
- Core crate structure (primitives, http-core, conveniences, samples)
- Basic project documentation (README, LICENSE, CONTRIBUTING)
- OpenAPI specification for Ollama API (v0.1.0)
- Project build configuration and CI/CD workflows

### Notes
- This is the initial release establishing the project foundation
- API is not yet stable and may change significantly

[Unreleased]: https://github.com/franciscotbjr/ollama-oxide/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/franciscotbjr/ollama-oxide/releases/tag/v0.1.0
