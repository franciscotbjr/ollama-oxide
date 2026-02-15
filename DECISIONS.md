# Decisions Log

This document tracks architectural and design decisions made during development.

| Date | Decision | Rationale | Reference |
|------|----------|-----------|-----------|
| 2026-01-11 | Single-crate architecture | Simpler build, easier maintenance, clearer dependencies | DEV_NOTES.md |
| 2026-01-13 | Use `thiserror` for error handling | Ergonomic error handling with derive macros | impl/01-get-version-implementation-plan.md |
| 2026-01-13 | Use `url` crate for URL validation | Proper URL parsing and validation at client creation | impl/01-get-version-implementation-plan.md |
| 2026-01-13 | Client uses `Arc<Client>` internally | Thread-safe cloning without overhead | impl/01-get-version-implementation-plan.md |
| 2026-01-13 | All error variants use `Error` suffix | Naming consistency (HttpError, TimeoutError, etc.) | impl/01-get-version-implementation-plan.md |
| 2026-01-13 | TDD approach for all features | Tests written before implementation | impl/01-get-version-implementation-plan.md |
| 2026-01-14 | Single-concern file structure | One primary type per file, mod.rs as facade | ARCHITECTURE.md |
| 2026-01-14 | Generic retry helpers | `get_with_retry<T>()` reduces 90% code duplication | impl/03-http-retry-abstraction-analysis.md |
| 2026-01-14 | `ClientConfig.url()` method | More ergonomic than standalone function | impl/02-api-endpoint-abstraction-analysis.md |
| 2026-01-14 | Unit tests in `tests/`, integration in `examples/` | `cargo test` always succeeds without external dependencies | ARCHITECTURE.md |
| 2026-01-15 | No retry on 4xx errors | Client errors should fail immediately | DEV_NOTES.md |
| 2026-01-16 | Exponential backoff: 100ms × (attempt + 1) | Balance between retry speed and server load | DEV_NOTES.md |
| 2026-01-24 | Use `#[serde(untagged)]` for oneOf types | ThinkSetting, FormatSetting, KeepAliveSetting, StopSetting need flexible JSON representation | impl/10-post-generate-implementation-plan.md |
| 2026-01-24 | StopSetting integrated into ModelOptions | Stop sequences are model options, not request-level settings; consistent with Ollama API | impl/10-post-generate-implementation-plan.md |
| 2026-01-24 | GenerateRequest always sets `stream: false` | v0.1.0 is non-streaming only; streaming deferred to v0.2.0 | impl/10-post-generate-implementation-plan.md |
| 2026-01-24 | Helper methods on GenerateResponse | `text()`, `is_done()`, `tokens_per_second()` provide ergonomic access to common data | impl/10-post-generate-implementation-plan.md |
| 2026-01-24 | Ergonomic tools API behind `tools` feature flag | Keeps core library lightweight; `schemars`/`futures` deps are optional | impl/12-ergonomic-tools-api-proposal.md |
| 2026-01-25 | No `Chat` struct - use `Vec<ChatMessage>` | Library provides primitives, not app-level abstractions; users define their own entities | impl/11-post-chat-implementation-plan.md |
| 2026-01-25 | `ChatRequest::new()` accepts `IntoIterator<Item=ChatMessage>` | More flexible than `Into<Vec<...>>`; enables custom types via `IntoIterator` impl | impl/11-post-chat-implementation-plan.md |
| 2026-01-26 | ~~Tool types stay in `src/inference/`, not `src/tools/`~~ | ~~Semantic separation: `inference/` = API data structures (ToolCall, ToolDefinition), `tools/` = ergonomic Rust abstractions (Tool trait, ToolRegistry).~~ **SUPERSEDED by 2026-02-03 decision** | DECISIONS.md |
| 2026-01-26 | No doc tests - all tests in source files or `tests/` folder | Feature flag complexity makes doc tests hard to maintain; coverage already exists in unit tests; simpler workflow without doc test failures | DEV_NOTES.md, ARCHITECTURE.md |
| 2026-01-26 | Example naming: `<feature>_<variant>_<mode>.rs` | Clear feature identification; consistent async/sync suffix; grouped by prefix | DEV_NOTES.md |
| 2026-01-26 | `chat_with_tools_async` example with mock weather service | Demonstrates complete tool call flow; uses Open-Meteo API format as reference; shows type-safe Tool trait usage | examples/chat_with_tools_async.rs |
| 2026-02-02 | Model primitives consolidated in `src/model/`, not `src/primitives/` | Clear feature boundary: `src/model/` = `model` feature. All model-related types (CopyRequest, ListResponse, ShowRequest, etc.) moved together. Simplifies conditional compilation from `#[cfg(all(feature = "primitives", feature = "model"))]` to `#[cfg(feature = "model")]`. | impl/15-move-model-primitives-to-model-folder.md |
| 2026-02-02 | Gate list/show/copy/running model methods behind `model` feature | Model operations are opt-in for libraries that only need generation/chat/embed. Reduces default binary size. Clear separation: default features = inference, `model` feature = model management. | impl/14-move-to-model-feature-plan.md |
| 2026-02-02 | Rename feature `primitives` → `inference` | Better semantic clarity: types are specifically for inference operations (chat, generate, embed). "Primitives" was generic; "inference" conveys purpose. Module renamed `src/primitives/` → `src/inference/`. | impl/16-rename-primitives-to-inference.md |
| 2026-02-03 | Tool types moved to `src/tools/`, not `src/inference/` | Consolidates all tool-related code in one module. Simplifies feature gating: `tools` feature now independent of `inference`. ToolCall, ToolCallFunction, ToolDefinition, ToolFunction moved from inference to tools. Supersedes 2026-01-26 decision. | DECISIONS.md |
| 2026-02-04 | Phase 1 (v0.1.0) complete with all 12 endpoints | All endpoints implemented in non-streaming mode: version, tags, ps, copy, delete, show, embed, generate, chat, create, pull, push. 330+ tests. Ready for v0.1.0 release. | DEV_NOTES.md, definition.md |
| 2026-02-15 | `OllamaClient::with_base_url_and_timeout()` convenience constructor | Ergonomic shorthand for custom URL + timeout; avoids manual `ClientConfig` construction for common use case | impl/19-ollama-client-ergonomic-constructors.md |
| 2026-02-15 | Early URL validation in `ClientConfig` constructors | `new()`, `with_base_url()`, `with_base_url_and_timeout()` return `Result<Self>` with immediate URL validation; catches invalid URLs at config creation instead of at client construction | impl/19-ollama-client-ergonomic-constructors.md |
| 2026-02-15 | `ClientConfig` fields made private | Enforces URL validation invariant — struct literal construction no longer possible outside the module. Getter methods (`base_url()`, `timeout()`, `max_retries()`) provide read-only access. Eliminates redundant validation in `OllamaClient::new()`. | impl/20-client-config-private-fields.md |
