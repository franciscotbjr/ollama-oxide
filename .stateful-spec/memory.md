# Project Memory — ollama-oxide

## Project

- **Name:** ollama-oxide
- **Description:** Rust client library for Ollama's native HTTP API, providing typed async and sync interfaces
- **Status:** Active development
- **Current Version:** 0.1.2
- **Branch:** release/0_1_3

## Active Work

None — no active iteration.

## Recent Completions

All Phase 1 work (v0.1.x) is complete: 12 API endpoints implemented in non-streaming mode, HTTP client with retry/backoff, feature flag architecture (`tools`, `model`), ergonomic constructors, and private `ClientConfig` fields. See History Index below for the full list of prior iterations imported from `impl/`.

## Key Decisions

| Decision | Date | Rationale |
|----------|------|-----------|
| Single crate with feature flags (not workspace) | Pre-Stateful Spec | Simpler dependency management for a library; features provide opt-in modularity |
| Inference types as pure data (no HTTP knowledge) | Pre-Stateful Spec | Testability, reusability, separation of concerns |
| mockito for all tests/ (no external services) | Pre-Stateful Spec | CI reliability; real integration tests live in examples/ |
| Private ClientConfig fields with validated constructors | 2026-02-15 | URL validation at construction time prevents invalid state |
| Manual From impls (no #[from] for external types) | Pre-Stateful Spec | Avoid exposing external error types through the public API |
| Adopted Stateful Spec methodology with Cursor agent | 2026-04-03 | AI memory persistence across sessions and agents; operation prompts as `.cursor/rules/*.mdc` |

## Constraints & Reminders

- All public items must have rustdoc documentation
- All types must be Send + Sync
- No unsafe without justification
- Inference types must not depend on HTTP module (strict dependency hierarchy)
- `mod.rs` files are facades only — no logic
- `tests/` must pass without external services
- Feature flags: `default = ["http", "inference"]`, optional `tools`, `model`, `conveniences`
- `.stateful-spec/` is version-controlled — commit changes to memory and history files

## History Index

Prior iterations imported from `impl/` (all pre-Stateful Spec):

| # | Name | Type | Status | Summary |
|---|------|------|--------|---------|
| 01 | get-version-implementation-plan | feature | done | `GET /api/version` — first endpoint, error model, config, retry, URL validation, sync/async traits |
| 02 | api-endpoint-abstraction-analysis | analysis | done | Const endpoint paths and URL builder (`Endpoints` + `build_url`) |
| 03 | http-retry-abstraction-analysis | analysis | done | Shared client helpers for GET/POST with retry and exponential backoff |
| 04 | get-tags-implementation-plan | feature | done | `GET /api/tags` — `ListResponse`, `ModelSummary`, `ModelDetails` |
| 05 | get-ps-implementation-plan | feature | done | `GET /api/ps` — `PsResponse`, `RunningModel` |
| 06 | post-copy-implementation-plan | feature | done | `POST /api/copy` — `CopyRequest`, empty-body POST helpers |
| 07 | delete-model-implementation-plan | feature | done | `DELETE /api/delete` — `DeleteRequest`, DELETE helpers |
| 08 | post-show-implementation-plan | feature | done | `POST /api/show` — `ShowRequest`/`ShowResponse`, `post_with_retry` |
| 09 | post-embed-implementation-plan | feature | done | `POST /api/embed` — `EmbedRequest`/`EmbedResponse`, string/array input |
| 10 | post-generate-implementation-plan | feature | done | `POST /api/generate` — non-streaming, rich options, logprobs, builders |
| 10b | post-generate-with-stop-case | analysis | done | Educational note on using `stop` sequences to limit output |
| 11 | post-chat-implementation-plan | feature | done | `POST /api/chat` — non-streaming, messages, tools support |
| 12 | ergonomic-tools-api-proposal | feature | done | `Tool` trait, schema generation, `ToolRegistry`, typed dispatch |
| 13 | post-create-implementation-plan | feature | done | `POST /api/create` — custom model creation, status response |
| 14 | move-to-model-feature-plan | refactor | done | Gate model types behind `model` feature flag |
| 15 | move-model-primitives-to-model-folder | refactor | done | Move model types into `src/model/` with facade `mod.rs` |
| 16 | rename-primitives-to-inference | refactor | done | Rename primitives module/feature to `inference` |
| 17 | post-pull-implementation-plan | feature | done | `POST /api/pull` — model download, non-streaming |
| 18 | post-push-implementation-plan | feature | done | `POST /api/push` — model upload to registry |
| 19 | ollama-client-ergonomic-constructors | feature | done | `OllamaClient::with_base_url()`, `with_base_url_and_timeout()` |
| 20 | client-config-private-fields | refactor | done | `ClientConfig` fields private with getters, URL validation at construction |
