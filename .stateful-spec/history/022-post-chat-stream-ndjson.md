# Iteration 022 — POST /api/chat NDJSON streaming

| Field | Value |
|-------|-------|
| **Type** | feature |
| **Status** | done |
| **Branch / context** | `feature/chat_stream` (merge target: release line as per project workflow) |
| **Date** | 2026-04-04 |

## Description

Streaming chat over `POST /api/chat` using newline-delimited JSON: `OllamaApiAsync::chat_stream` → `ChatStream`, `OllamaApiSync::chat_stream_blocking` → `ChatStreamBlocking`, NDJSON framing in `src/http/streaming.rs` and client helpers. `ChatRequest::with_stream(true)` requests streamed responses. Integration tests in `tests/client_chat_stream_tests.rs` (mockito). Examples: `chat_stream_async`, `chat_stream_sync`, `chat_stream_think_async`, `chat_stream_think_sync` (Cargo.toml `[[example]]` entries).

Think-stream examples skip **empty** `thinking()` / `content()` strings so `[thinking]` / `[response]` labels are not flipped when Ollama sends placeholder `""` during reasoning.

Documentation updated: `CHANGELOG.md` (v0.2.0), `README.md` (features + run commands), `ARCHITECTURE.md` (HTTP streaming note, trait diagram).

## Acceptance criteria

- [x] Async and sync streaming APIs on `OllamaClient` with typed chunk iteration
- [x] NDJSON parsing robust for streamed lines; errors mapped consistently
- [x] Tests in `tests/` without live Ollama
- [x] Examples build; think examples document empty-chunk behavior
- [x] Quality gates: `cargo fmt`, `cargo clippy --all-features -- -D warnings`, `cargo test --all-features`

## Tasks completed

- [x] Implement `chat_stream` / `chat_stream_blocking`, `ChatStream` / `ChatStreamBlocking`
- [x] Wire `ChatRequest` stream flag and inference↔HTTP boundaries (no HTTP in inference-only types beyond request JSON)
- [x] Add `client_chat_stream_tests.rs`
- [x] Add examples (including think variants) and fix empty-string display bug in think examples
- [x] Update README, CHANGELOG, ARCHITECTURE

## Decisions

| Decision | Rationale |
|----------|-------------|
| Filter empty strings in **examples** only for labeling | `ChatResponse::content()` / `thinking()` may return `Some("")`; consumers that split UI by field should treat empty strings like absent for section headers |

## References

- `src/http/streaming.rs`, `src/http/client.rs`, `src/http/api_async.rs`, `src/http/api_sync.rs`
- `examples/chat_stream_*.rs`, `examples/chat_stream_think_*.rs`

## Follow-up

- [x] Release notes: `CHANGELOG.md` **v0.2.0** (2026-04-04)
- [ ] Optional: streaming for generate, pull, push, create (see CHANGELOG Planned v0.2.0)
