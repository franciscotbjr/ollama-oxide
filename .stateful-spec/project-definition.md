# Project Definition — ollama-oxide

> Rust client library for Ollama's native HTTP API. Generated from the Rust Library preset, customized with discovered project conventions.

---

## Project Identity

- **Project Name:** ollama-oxide
- **Description:** A Rust library providing comprehensive, type-safe integration with Ollama's native API through async and sync interfaces
- **Project Type:** library
- **Repository URL:** https://github.com/franciscotbjr/ollama-oxide
- **License:** MIT
- **Current Version:** 0.2.0
- **Status:** Active development — Phase 1 complete (all 12 endpoints); **v0.2.0** adds **POST /api/chat** NDJSON streaming (`chat_stream` / `chat_stream_blocking`)
- **Author:** Francisco (@franciscotbjr)

## Technology Stack

### Language(s)

| Language | Version | Role |
|----------|---------|------|
| Rust | Edition 2024 | Primary |

### Framework(s)

| Framework | Version | Purpose |
|-----------|---------|---------|
| tokio | 1.49.0 | Async runtime (macros, rt-multi-thread, time) |

### Key Dependencies

| Dependency | Version | Purpose | Optional |
|------------|---------|---------|----------|
| serde | 1.0.228 | Serialization / deserialization (derive) | No |
| serde_json | 1.0.149 | JSON support | No |
| reqwest | 0.13.1 | HTTP client (blocking, cookies, http2, json, native-tls) | No |
| async-trait | 0.1.89 | Async trait support | No |
| thiserror | 2.0.18 | Error derive macros | No |
| url | 2.5.8 | URL parsing and validation | No |
| schemars | 1.2.0 | JSON schema generation for tools | Yes (`tools` feature) |
| futures | 0.3.31 | Async utilities for tools | Yes (`tools` feature) |

### Dev Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| mockito | 1.7.1 | HTTP mocking for tests |

### Build System & Package Manager

- **Package Manager:** cargo
- **Build Tool:** cargo
- **Task Runner:** cargo

## Repository Structure

```
ollama-oxide/
├── .claude/                # Claude Code commands and scripts
│   ├── commands/           # Slash commands for Claude Code
│   └── scripts/            # Helper Rust scripts
├── .github/
│   └── workflows/          # CI/CD (build.yml, binaries.yml, publish.yml)
├── .stateful-spec/         # Stateful Spec methodology and memory
│   ├── memory.md
│   ├── project-definition.md
│   ├── methodology/
│   └── history/
├── assets/                 # SVG artwork (logo, illustrations)
├── examples/               # 30 example programs (integration tests against real Ollama)
├── impl/                   # Numbered implementation plan markdown files
├── spec/
│   ├── apis/               # YAML API specs per endpoint
│   ├── api-analysis.md
│   └── definition.md       # Full project definition (legacy, pre-Stateful Spec)
├── src/
│   ├── lib.rs              # Module declarations + re-exports + prelude
│   ├── main.rs             # Demo binary
│   ├── error.rs            # Error enum + Result alias
│   ├── inference/          # Inference types (chat, generate, embed, version, options)
│   ├── http/               # Client, config, async/sync API traits, endpoints
│   ├── model/              # Model lifecycle types (list, show, copy, create, delete, pull, push, ps)
│   ├── tools/              # Tool trait, registry, definitions, type-erased dispatch
│   └── conveniences/       # (Future) high-level convenience APIs
├── tests/                  # 18 integration test files (mockito-based, no external deps)
├── ARCHITECTURE.md
├── BLOCKERS.md
├── CHANGELOG.md
├── CONTRIBUTING.md
├── Cargo.toml
├── DECISIONS.md
├── DEV_NOTES.md
├── LICENSE
└── README.md
```

### Key Directories

| Directory | Purpose |
|-----------|---------|
| src/ | Library source code (single crate) |
| src/inference/ | Request/response types for inference APIs (chat, generate, embed, version) |
| src/http/ | HTTP client layer (OllamaClient, ClientConfig, OllamaApiAsync/Sync traits) |
| src/model/ | Model management types (feature-gated under `model`) |
| src/tools/ | Tool trait + registry for function calling (feature-gated under `tools`) |
| tests/ | Integration tests using mockito (no external service required) |
| examples/ | Usage examples / integration tests against real Ollama server |
| spec/ | API specifications (YAML) and analysis docs |
| impl/ | Implementation plan documents |

## Feature Flags

```toml
[features]
default = ["http", "inference"]       # Standard usage
conveniences = ["http", "inference"]  # High-level APIs (future)
http = []                             # HTTP client layer
inference = []                        # Inference types (chat, generate, embed)
tools = ["dep:schemars", "dep:futures"] # Ergonomic function calling
model = ["http", "inference"]         # Model management (opt-in)
```

| Feature | Dependencies | Purpose |
|---------|-------------|---------|
| `default` | `http`, `inference` | Standard usage — HTTP client + inference types |
| `inference` | — | Standalone inference types (chat, generate, embed) |
| `http` | — | HTTP client implementation (async/sync) |
| `tools` | `schemars`, `futures` | Tool types + ergonomic function calling |
| `model` | `http`, `inference` | Model management API (list, show, copy, create, delete, pull, push) |
| `conveniences` | `http`, `inference` | High-level ergonomic APIs (future) |

## Code Conventions

### Naming

| Item | Convention | Example |
|------|-----------|---------|
| Files | snake_case | client_config.rs |
| Functions/Methods | snake_case | get_with_retry |
| Types/Structs/Enums | PascalCase | OllamaClient |
| Constants | SCREAMING_SNAKE_CASE | MAX_RETRIES |
| Modules | snake_case | http, inference |
| Lifetimes | lowercase, short | 'a, 'de |
| Async methods | no suffix | version() |
| Blocking methods | `_blocking` suffix | version_blocking() |

### Code Style

- **Formatter:** rustfmt (default settings, no `rustfmt.toml`)
- **Max Line Length:** 100 (rustfmt default)
- **Indentation:** 4 spaces
- **Import Order:** std → external crates → crate internal → super/self

### Module Patterns

- **Single concern per file:** Each file contains one primary type with its implementations
- **mod.rs as facade:** Module declarations (`mod foo;`) and re-exports (`pub use foo::Foo;`) only — no logic
- **File names match types:** `client_config.rs` → `struct ClientConfig`
- **Visibility:** `pub(super)` for module-internal sharing, `pub(crate)` for cross-module, fully private fields with getter methods where validation is needed

### API Design Patterns

- **Error Handling:** Custom `Error` enum with `thiserror`, `Result<T>` type alias, manual `From` impls for external types (no `#[from]` exposing external types)
- **Constructors:** `::new()` with required fields, validated (returns `Result`)
- **Builder pattern:** `.with_*()` method chain for optional fields (returns `Self`)
- **Async/Sync parity:** `OllamaApiAsync` trait + `OllamaApiSync` trait on the same `OllamaClient`
- **Feature gating:** `#[cfg(feature = "...")]` at module, struct field, and method levels
- **Serde:** `#[serde(skip_serializing_if = "Option::is_none")]` on optional fields, `#[serde(default)]` on response fields

### Dependency Hierarchy

```
lib.rs (re-exports) ← Top: Facade only
    tools/          ← High: Uses inference (optional)
    http/           ← High: Uses inference, model
    model/          ← Mid: Independent types (optional)
    inference/      ← Low: Independent types
    error.rs        ← Base: Used by all
```

Inference types must remain pure data types with no knowledge of how they are transported.

## Testing

### Strategy

- **Unit Tests:** Co-located with source in `#[cfg(test)] mod tests` blocks
- **Integration Tests:** In `tests/` directory (18 files), one file per feature area, using mockito for HTTP mocking
- **Real Integration Tests:** In `examples/` directory, require running Ollama server
- **Test Framework:** cargo test + `#[tokio::test]` for async
- **Mocking:** mockito for HTTP mocking — all `tests/` must pass without external services
- **Coverage Target:** No formal target; focus on behavior coverage
- **Feature gating:** Model tests use `required-features = ["model"]` in `Cargo.toml`

### Test Naming Convention

`test_{what_is_being_tested}_{scenario}` — e.g., `test_version_async_successful`, `test_retry_on_server_error`

### Test Rules

- Tests in `tests/` must NEVER require external services (Ollama server)
- Tests in `tests/` use mockito for HTTP mocking
- Real Ollama integration tests live in `examples/` and run via `cargo run --example <name>`
- `cargo test --all-features` must always pass without additional setup

## Quality Gates

```bash
# Linter
cargo clippy --all-features -- -D warnings

# Formatter check
cargo fmt --check

# Tests
cargo test --all-features

# Build
cargo build --all-features

# Doc build
cargo doc --all-features --no-deps
```

**Note:** CI workflows currently run `cargo test --all-features` and `cargo build --all-features` but do not enforce `cargo fmt --check` or `cargo clippy`. These are local quality gates.

## Documentation

### Required Documentation Files

| File | Purpose |
|------|---------|
| README.md | Project overview, features, install, quick start |
| CHANGELOG.md | Version history (Keep a Changelog format) |
| ARCHITECTURE.md | Module organization, design patterns, dependency rules |
| CONTRIBUTING.md | Contribution guidelines, dev setup |
| DECISIONS.md | Architectural decision records |
| DEV_NOTES.md | Internal development notes |
| LICENSE | MIT License |

### Documentation Style

- **Code Comments:** Rustdoc (`///` for public items, `//` for internal)
- **Doc Examples:** `no_run` attribute on doc examples
- **All public items** must have rustdoc documentation

## Deployment

- **Target Environment:** crates.io
- **CI/CD:** GitHub Actions (3 workflows)
  - `build.yml` — Build + test on push/PR (ubuntu, macOS, windows × stable + nightly)
  - `binaries.yml` — Release binary builds (workflow_dispatch)
  - `publish.yml` — Audit + test + publish to crates.io on version tags (`v*`)
- **Branch Strategy:** `main` (stable) + `release/*` (integration) + `feature/*`, `bug/*`, `issue/*` (work branches)
- **Versioning:** Semantic Versioning 2.0.0

## Constraints & Non-Negotiables

- No unsafe code without justification and documentation
- All public items must have rustdoc documentation
- All types must be Send + Sync (verified by compile-time test)
- No `#[from]` on error variants that expose external types — use manual `From` impls
- Feature flags for optional functionality
- Strict dependency hierarchy: inference types must remain pure data types with no HTTP knowledge
- `mod.rs` files contain only module declarations and re-exports — no logic
- Single concern per file: one primary type per `.rs` file
- `tests/` must pass without external services (Ollama server)

## Roadmap

| Version | Focus | Status |
|---------|-------|--------|
| 0.1.x | All 12 endpoints (non-streaming) | Complete |
| 0.2.0 | Streaming support for 5 endpoints | Planned |
| 0.3.0 | Conveniences module | Planned |
| 0.4.0 | Examples, benchmarks, production readiness | Planned |
