# ollama-oxide Project Definition

**Document Version:** 1.8
**Last Updated:** 2026-02-15
**Project Version:** 0.1.2

## Executive Summary

**ollama-oxide** is a Rust library that provides comprehensive integration with Ollama's native API through a layered architecture. The library offers both low-level primitives for direct API control and high-level conveniences for common use cases, enabling Rust developers to seamlessly integrate Ollama's AI capabilities into their applications.

## Project Identity

### Basic Information

- **Project Name:** ollama-oxide
- **Repository:** https://github.com/franciscotbjr/ollama-oxide
- **License:** MIT
- **Current Version:** 0.1.0
- **Status:** Early Development / Foundation Phase
- **Primary Language:** Rust (Edition 2024)
- **Author:** Francisco

### Project Purpose

To provide the easiest and most idiomatic way for Rust developers to integrate with Ollama, offering:
- Type-safe API bindings
- Ergonomic async/await interfaces
- Clear separation between low-level and high-level abstractions
- Production-ready error handling
- Comprehensive documentation

## Technical Architecture

### Single-Crate Structure

The project is organized as a **single crate** with modular structure and feature flags:

```
ollama-oxide/
└── src/
    ├── lib.rs           # Main library entry point
    ├── inference/       # Inference types module (default)
    │   └── mod.rs
    ├── http/            # HTTP client module (default)
    │   └── mod.rs
    ├── tools/           # Ergonomic function calling (optional, "tools" feature)
    │   └── mod.rs
    ├── model/           # Model creation/deletion (optional, "model" feature)
    │   └── mod.rs
    └── conveniences/    # High-level convenience APIs module (optional)
        └── mod.rs
```

### Module Organization

Each module follows single-concern file structure:
- One primary type per file with its implementations
- `mod.rs` serves as re-export facade
- Example: `error.rs` contains Error enum + implementations; `lib.rs` imports from error module

#### 1. inference (Module)
**Purpose:** Data structures for inference operations (chat, generate, embed).

**Key Responsibilities:**
- Request/response type definitions for inference APIs
- Serialization/deserialization implementations
- API model validation
- Type-safe enum representations

**Feature:** `inference` (default)
**Status:** Implementation complete

#### 2. http (Module)
**Purpose:** HTTP client layer for API communication.

**Key Responsibilities:**
- Connection management
- Request/response handling
- Error mapping and propagation
- Generic retry helpers (`get_with_retry`, `get_blocking_with_retry`)
- Stream handling

**HTTP Client Design:**
- Type-safe generic methods with `serde::de::DeserializeOwned` bounds
- Automatic retry logic with exponential backoff
- Separate async (`tokio::time::sleep`) and sync (`std::thread::sleep`) helpers
- Marked `pub(super)` for module-internal use
- `ClientConfig` with private fields and validated constructors (URL validation at construction time)
- Ergonomic constructors: `OllamaClient::with_base_url()`, `with_base_url_and_timeout()`, `default()`

**Feature:** `http` (default)
**Status:** Implementation complete

#### 3. tools (Module)
**Purpose:** Tool types and ergonomic function calling with auto-generated JSON schemas.

**Key Responsibilities:**
- `ToolCall`, `ToolCallFunction` - API response types for tool invocations
- `ToolDefinition`, `ToolFunction` - API request types for tool definitions
- `Tool` trait for type-safe tool definitions
- `ToolRegistry` for automatic dispatch
- Auto-generated JSON schemas from Rust types via `schemars`
- Type-erased tool storage for heterogeneous collections

**Feature:** `tools` (optional, requires `schemars` and `futures`)
**Status:** Implementation complete
**Note:** All tool-related types consolidated here. Simplifies feature gating (only requires `tools` feature).

#### 4. model (Module)
**Purpose:** Model management operations and types.

**Key Responsibilities:**
- `CreateRequest` / `CreateResponse` for model creation
- `DeleteRequest` for model deletion
- `LicenseSetting` for license configuration
- `CopyRequest` for model copying
- `ListResponse` / `ModelSummary` / `ModelDetails` for listing models
- `PsResponse` / `RunningModel` for running model info
- `ShowRequest` / `ShowResponse` / `ShowModelDetails` for model details
- `PullRequest` / `PullResponse` for model downloading

**Feature:** `model` (optional, requires `http` and `inference`)
**Status:** Implementation complete
**Note:** Opt-in feature consolidating all model-related operations and types. Includes destructive operations (create/delete).

#### 5. conveniences (Module)
**Purpose:** High-level, ergonomic APIs for common workflows.

**Key Responsibilities:**
- Simplified method signatures
- Builder patterns
- Common operation helpers
- Streaming abstractions
- Response post-processing

**Feature:** `conveniences` (optional, requires `http` and `inference`)
**Status:** Implementation pending

### Feature Flags

```toml
[features]
default = ["http", "inference"]       # Standard usage
conveniences = ["http", "inference"]  # High-level APIs
http = []                             # HTTP client layer
inference = []                        # Inference types
tools = ["dep:schemars", "dep:futures"] # Ergonomic function calling
model = ["http", "inference"]         # Model management (opt-in)
```

**Feature Matrix:**

| Feature | Dependencies | Purpose |
|---------|-------------|---------|
| `default` | `http`, `inference` | Standard usage - HTTP client + all inference types |
| `inference` | - | Standalone inference types (chat, generate, embed) |
| `http` | - | HTTP client implementation (async/sync) |
| `tools` | `schemars`, `futures` | Tool types (ToolCall, ToolDefinition) + ergonomic function calling |
| `model` | `http`, `inference` | Model management API - all model-related types and operations (list, show, copy, create, delete) |
| `conveniences` | `http`, `inference` | High-level ergonomic APIs |

**Usage Examples:**
```toml
# Default features (inference + http)
ollama-oxide = "0.1.0"

# With function calling support
ollama-oxide = { version = "0.1.0", features = ["tools"] }

# With model management
ollama-oxide = { version = "0.1.0", features = ["model"] }

# Full featured
ollama-oxide = { version = "0.1.0", features = ["tools", "model"] }

# Inference types only (no HTTP client)
ollama-oxide = { version = "0.1.0", default-features = false, features = ["inference"] }
```

## Technical Stack

### Core Dependencies

| Dependency | Version | Purpose | Features |
|------------|---------|---------|----------|
| tokio | 1.49.0 | Async runtime | macros, rt-multi-thread |
| reqwest | 0.13.1 | HTTP client | blocking, cookies, http2, json, native-tls |
| serde | 1.0.228 | Serialization | derive |
| serde_json | 1.0.149 | JSON handling | - |
| async-trait | 0.1.89 | Trait async methods | - |

**Build Configuration:**
- Single crate architecture
- Feature-based module organization
- Default features: `http` + `inference`
- Optional features: `tools`, `model`, `conveniences`
- Optional dependencies: `schemars`, `futures` (for `tools` feature)

### Dependency Rationale

**Tokio:**
- Industry-standard async runtime
- Excellent ecosystem support
- High performance
- Well-documented

**reqwest:**
- Mature HTTP client with tokio integration
- Built-in JSON support via serde
- HTTP/2 capability
- Cookie and TLS handling

**serde/serde_json:**
- De facto Rust serialization standard
- Excellent derive macro support
- Comprehensive JSON handling
- Zero-cost abstractions

**async-trait:**
- Enables async methods in traits
- Simplifies trait-based design
- Required for extensible async APIs

## API Specification

### OpenAPI Integration

**Location:** [spec/alloma_api.yaml](alloma_api.yaml)
**Version:** 0.1.0
**Format:** OpenAPI 3.1.0

The library's implementation is driven by Ollama's official OpenAPI specification, ensuring accuracy and compatibility.

### Supported Endpoints (Planned)

**Complete API Analysis:** See [api-analysis.md](api-analysis.md) for comprehensive endpoint documentation.

**Total Endpoints:** 12 (across all phases)

#### Phase 1 (v0.1.0): Foundation + All Endpoints (Non-Streaming Mode)
**Focus:** Primitives module structure, HTTP module implementation, and all 12 endpoints in non-streaming mode

**Scope:**
- Set up `inference` module with shared types (ModelOptions, Logprob, etc.)
- Implement `http` module with connection management
- Build error handling infrastructure
- Create serialization/deserialization framework
- Establish testing foundation
- Implement all 12 endpoints (streaming endpoints work in non-streaming mode only)

**GET Endpoints (3):**
1. `GET /api/version` - Get Ollama version
2. `GET /api/tags` - List available models
3. `GET /api/ps` - List running models

**Simple POST/DELETE Endpoints (2):**
4. `POST /api/copy` - Copy a model
5. `DELETE /api/delete` - Delete a model

**Medium Complexity POST Endpoints (2):**
6. `POST /api/show` - Show detailed model information ✅
7. `POST /api/embed` - Generate text embeddings ✅

**Complex POST Endpoints (5) - Non-Streaming Mode:**
8. `POST /api/generate` - Generate text completions (non-streaming only)
9. `POST /api/chat` - Chat completions (non-streaming only)
10. `POST /api/create` - Create custom models (non-streaming only)
11. `POST /api/pull` - Download models (non-streaming only)
12. `POST /api/push` - Upload models (non-streaming only)

**Deliverables:**
- All 12 endpoints fully implemented (streaming endpoints in non-streaming mode)
- Functional HTTP client in `http` module (GET, POST, DELETE)
- Error types and handling in error module
- Basic integration test framework
- Module structure with feature flags working

#### Phase 2 (v0.2.0): Streaming Implementation
**Focus:** Add streaming support to endpoints that support it

**Streaming Endpoints (5):**
1. `POST /api/generate` - Generate text completions with streaming
2. `POST /api/chat` - Chat completions with streaming
3. `POST /api/create` - Create custom models with progress streaming
4. `POST /api/pull` - Download models with progress streaming
5. `POST /api/push` - Upload models with progress streaming

**Scope:**
- Streaming infrastructure and abstractions
- Stream helper utilities and iterators
- Progress callback system for model operations
- Async stream implementations

**Deliverables:**
- Streaming support for all 5 streaming-capable endpoints
- Stream abstraction utilities
- Progress tracking for long-running operations
- Comprehensive streaming tests
- Documentation for streaming usage

#### Phase 3 (v0.3.0): Conveniences Module
**Focus:** High-level ergonomic APIs built on inference module

**Scope:**
- Implement `conveniences` module as optional feature
- Client builder pattern for easy initialization
- Simplified method signatures for common operations
- Response post-processing and formatting
- Error recovery patterns
- Convenience methods for chaining operations

**Deliverables:**
- Complete `conveniences` module implementation
- `conveniences` feature flag working correctly
- Builder patterns for complex requests
- High-level client interface
- Comprehensive documentation

#### Phase 4 (v0.4.0): Examples & Production Readiness
**Focus:** Examples, documentation, and polish

**Scope:**
- Comprehensive usage examples in `/examples` directory
- Real-world integration patterns
- Performance benchmarking
- API stability review
- Production deployment guides
- Migration documentation

**Example Programs:**
- Basic text generation
- Chat conversation with history
- Model management (pull, create, delete)
- Embedding generation and similarity search
- Streaming responses with progress
- Error handling patterns
- Batch processing
- Custom tool/function calling (✅ `chat_with_tools_async` with mock weather API)
- Type-safe tool registry (✅ `chat_with_tools_registry_async`)

**Deliverables:**
- 10+ comprehensive examples in `/examples`
- Performance benchmarks and optimization
- Production-ready documentation
- Stable API (v1.0.0 target)
- Migration guide (inference → conveniences)

## Design Philosophy

### Core Principles

1. **Layered Architecture**
   - Clear separation between inference types and conveniences
   - Users can choose their abstraction level
   - Internal flexibility for future changes

2. **Type Safety**
   - Leverage Rust's type system for correctness
   - Compile-time guarantees where possible
   - Minimize runtime errors

3. **Async First**
   - Built on tokio for non-blocking I/O
   - Native async/await support
   - Optional sync wrapper (future consideration)

4. **Minimal Dependencies**
   - Keep dependency tree lean
   - Avoid unnecessary transitive dependencies
   - Prefer well-maintained, stable crates

5. **OpenAPI Driven**
   - Follow Ollama's official specification
   - Maintain API compatibility
   - Automated validation against spec (future)

### API Design Guidelines

**Consistency:**
- Follow Rust API guidelines
- Maintain naming conventions
- Consistent error handling patterns

**Ergonomics:**
- Make common tasks easy
- Provide sensible defaults
- Use builder patterns for complex types

**Safety:**
- Prevent invalid states at compile time
- Clear error messages
- No unsafe code unless absolutely necessary

**Performance:**
- Avoid unnecessary allocations
- Efficient serialization
- Connection reuse
- Streaming where appropriate

**Clarity:**
- Explicit over implicit
- Self-documenting code
- Comprehensive documentation

**Success Criteria:**
- All shared types compile and serialize correctly
- HTTP client can make basic requests
- Error handling propagates properly
- Test infrastructure operational

---

## Testing Strategy

### Unit Tests (`tests/` folder)

**Location:** `tests/*.rs`

**Scope:**
- Individual function validation
- Data structure serialization/deserialization
- Error handling paths
- Edge cases
- HTTP interactions via mocking

**Requirements:**
- Must NOT require external services (Ollama server)
- Use `mockito` crate for HTTP mocking
- Must pass with `cargo test` without additional setup

**Tools:**
- Standard Rust test framework
- serde_json for JSON validation
- mockito for HTTP mocking

### Integration Tests (`examples/` folder)

**Location:** `examples/*.rs`

**Scope:**
- Full API interactions with real Ollama server
- End-to-end workflows
- Error scenarios
- Streaming behavior
- Serve as usage documentation

**Requirements:**
- Running Ollama instance
- Network connectivity
- Run manually: `cargo run --example <name>`

**Rationale:**
- `cargo test` always succeeds without external dependencies
- Examples serve dual purpose: documentation + integration testing
- CI/CD pipelines run reliably

### Mocking Strategy

**Approach:**
- All tests in `tests/` folder use HTTP mocking
- `mockito` crate for simulating Ollama API responses
- Real Ollama instance only used via examples

**Benefits:**
- Fast unit test execution
- No external dependencies for `cargo test`
- CI/CD always passes
- Integration testing available when needed via examples

## Quality Standards

### Code Quality

**Formatting:**
- Use `cargo fmt` (rustfmt)
- Enforce in CI/CD

**Linting:**
- Use `cargo clippy`
- Treat warnings as errors in CI
- Custom lints for project-specific patterns

**Documentation:**
- All public APIs must have doc comments
- Include examples in documentation
- Generate docs with `cargo doc`

### Testing Coverage

**Targets:**
- 80%+ code coverage (aspirational)
- 100% coverage for critical paths
- All error paths tested

**Tools:**
- cargo-tarpaulin for coverage reports
- Integration with CI/CD

### Performance

**Benchmarks:**
- Establish baseline metrics
- Regular performance regression testing
- Profile critical paths

**Tools:**
- criterion.rs for benchmarking
- flamegraph for profiling

## Development Workflow

### Version Control

**Branch Strategy:**
- `main` - stable releases
- `release` - integration branch
- `feature/*` - new features
- `fix/*` - bug fixes
- `docs/*` - documentation

**Commit Convention:**
- Follow Conventional Commits
- Clear, descriptive messages
- Reference issues when applicable

### CI/CD Pipeline

**Automated Checks:**
- Build verification
- Test execution
- Linting and formatting
- Documentation generation
- Security audits

**Workflows:**
- Build on push/PR
- Release on tag creation
- Automated binary builds

**Location:** [.github/workflows/](.github/workflows/)

### Release Process

**Versioning:**
- Follow Semantic Versioning 2.0.0
- MAJOR: Breaking changes
- MINOR: New features, backward compatible
- PATCH: Bug fixes, backward compatible

**Release Checklist:**
1. Update CHANGELOG.md
2. Update version in Cargo.toml
3. Run full test suite
4. Generate documentation
5. Create git tag
6. Publish to crates.io
7. Create GitHub release

## Documentation Structure

### Project Documentation

1. **[README.md](../README.md)**
   - Project overview
   - Quick start guide
   - Installation instructions
   - Basic usage examples

2. **[CONTRIBUTING.md](../CONTRIBUTING.md)**
   - Contribution guidelines
   - Development setup
   - Code standards
   - PR process

3. **[CHANGELOG.md](../CHANGELOG.md)**
   - Version history
   - Breaking changes
   - New features
   - Bug fixes

4. **[LICENSE](../LICENSE)**
   - MIT License
   - Copyright information

5. **[DEV_NOTES.md](../DEV_NOTES.md)**
   - Internal development notes
   - Architectural decisions
   - Implementation details
   - Technical rationale

6. **[spec/definition.md](definition.md)** (This Document)
   - Comprehensive project analysis
   - Architecture documentation
   - Implementation strategy
   - Quality standards

### API Documentation

**Generated Documentation:**
- Rust doc comments (`///`)
- Generated with `cargo doc`
- Hosted on docs.rs (future)

**Content Requirements:**
- Brief description
- Detailed explanation
- Usage examples
- Error conditions
- Related types/functions

## Known Limitations & Considerations

### Current Limitations

1. **Early Development Stage**
   - API not yet stable
   - Breaking changes expected
   - Limited functionality

2. **Endpoint Coverage**
   - Only `/api/generate` planned for initial release
   - Other endpoints coming in future versions

3. **Testing**
   - Requires running Ollama instance
   - No comprehensive test suite yet
   - Integration tests pending

4. **Performance**
   - No optimization yet
   - Baseline metrics not established
   - Profiling needed

### Future Considerations

**Potential Features:**
- CLI tool for Ollama interaction
- Server-sent events for streaming
- Batch processing utilities
- Model management helpers
- Custom middleware/interceptors
- WebAssembly support
- Sync API wrapper

**Technical Debt:**
- Consider using `thiserror` for errors
- Evaluate tracing/logging strategy
- Connection pooling optimization
- Request retry logic

## Resource Requirements

### Development Environment

**Minimum Requirements:**
- Rust 1.75+
- Ollama installed and running
- Git
- 4GB RAM
- Network connectivity

**Recommended Tools:**
- rust-analyzer (IDE support)
- cargo-watch (auto-rebuild)
- cargo-expand (macro debugging)
- cargo-audit (security scanning)
- cargo-outdated (dependency updates)

### Runtime Requirements

**For Library Users:**
- Rust 1.75+ (edition 2024)
- Ollama instance (local or remote)
- Network connectivity to Ollama server
- Tokio runtime

**System Resources:**
- Minimal CPU overhead
- Memory depends on model size
- Network bandwidth for API calls

## Success Criteria

### Version 0.1.0: Foundation + All Endpoints (Non-Streaming Mode) (Current)
**Status:** In Progress

**Completed:**
- [x] Project structure established
- [x] Documentation foundation (README, CONTRIBUTING, CHANGELOG, DEV_NOTES, definition.md, api-analysis.md)
- [x] Dependency configuration
- [x] OpenAPI specifications (12 endpoints documented)
- [x] Git repository initialized
- [x] Workspace configuration
- [x] `inference` module structure with shared types
- [x] `http` module implementation
- [x] Error type hierarchy in error module
- [x] Testing infrastructure
- [x] Feature flags configuration

**GET Endpoints (3):**
- [x] `GET /api/version` - Get Ollama version
- [x] `GET /api/tags` - List available models
- [x] `GET /api/ps` - List running models

**Simple POST/DELETE Endpoints (2):**
- [x] `POST /api/copy` - Copy a model
- [x] `DELETE /api/delete` - Delete a model

**Medium Complexity POST Endpoints (2):**
- [x] `POST /api/show` - Show detailed model information
- [x] `POST /api/embed` - Generate text embeddings

**Complex POST Endpoints (5) - Non-Streaming Mode:**
- [x] `POST /api/generate` - Text generation (non-streaming only)
- [x] `POST /api/chat` - Chat completions (non-streaming only)
- [x] `POST /api/create` - Model creation (non-streaming only)
- [x] `POST /api/pull` - Model download (non-streaming only)
- [x] `POST /api/push` - Model upload (non-streaming only)

**Examples:**
- [x] `chat_with_tools_async` - Complete tool call flow with mock weather service
- [x] `chat_with_tools_registry_async` - ToolRegistry pattern with type erasure (renamed from `tool_registry_async`)

**Definition of Done:**
- All shared types (ModelOptions, Logprob, enums) compile
- All 12 endpoints working (streaming endpoints in non-streaming mode)
- HTTP client in `http` module can make GET, POST, DELETE requests
- POST helper methods with retry logic implemented (`post_empty_with_retry`, `post_with_retry`)
- DELETE helper methods with retry logic implemented
- Error handling system in place (including HttpStatusError)
- Feature flags (`http`, `inference`) working correctly
- Unit test framework operational
- Integration test setup complete

---

### Version 0.2.0: Streaming Implementation (Planned)
**Status:** Not Started

**Checklist:**
- [ ] `POST /api/generate` - Streaming support
- [ ] `POST /api/chat` - Streaming support
- [ ] `POST /api/create` - Progress streaming support
- [ ] `POST /api/pull` - Progress streaming support
- [ ] `POST /api/push` - Progress streaming support
- [ ] Streaming infrastructure and abstractions
- [ ] Stream helper utilities and async iterators
- [ ] Progress callback system
- [ ] Comprehensive streaming tests
- [ ] Streaming documentation

**Definition of Done:**
- All 5 streaming endpoints support streaming mode
- Stream utilities work with async iterators
- Progress tracking functional for long operations
- Documentation includes streaming examples
- Tests cover both streaming and non-streaming modes

---

### Version 0.3.0: Conveniences Layer (Planned)
**Status:** Not Started

**Checklist:**
- [ ] OllamaClient with builder pattern
- [ ] Convenience methods for all endpoints
- [ ] Builder patterns for complex requests
- [ ] Response formatters
- [ ] Error recovery patterns
- [ ] High-level documentation with examples

**Definition of Done:**
- All convenience APIs ergonomic and intuitive
- Complex operations reduced to 3-5 calls
- Documentation includes real-world examples
- User feedback incorporated

---

### Version 0.4.0: Samples & Production Readiness (Planned)
**Status:** Not Started

**Checklist:**
- [ ] 10+ comprehensive examples in samples crate
- [ ] Performance benchmarks established
- [ ] Memory profiling completed
- [ ] API stability review
- [ ] Breaking change assessment
- [ ] Migration guide (primitives → conveniences)
- [ ] Production deployment guide
- [ ] Published to crates.io
- [ ] API declared stable (semver commitment)

**Definition of Done:**
- All examples run successfully
- Performance meets benchmarks
- Documentation covers 95%+ of use cases
- API frozen and stable
- Production-ready
- Community feedback positive

## Risk Analysis

### Technical Risks

1. **API Changes**
   - **Risk:** Ollama API may change
   - **Mitigation:** Follow OpenAPI spec, version pinning

2. **Performance**
   - **Risk:** Inefficient implementation
   - **Mitigation:** Benchmarking, profiling, optimization

3. **Error Handling**
   - **Risk:** Incomplete error coverage
   - **Mitigation:** Comprehensive testing, user feedback

### Project Risks

1. **Maintenance**
   - **Risk:** Single maintainer
   - **Mitigation:** Good documentation, community building

2. **Adoption**
   - **Risk:** Low user adoption
   - **Mitigation:** Examples, documentation, community engagement

3. **Compatibility**
   - **Risk:** Breaking changes in dependencies
   - **Mitigation:** Conservative updates, testing

## Community & Support

### Communication Channels

- **Issues:** GitHub issue tracker
- **Discussions:** GitHub discussions (future)
- **Documentation:** docs.rs (future)

## References

### External Resources

- [Ollama GitHub Repository](https://github.com/ollama)
- [Ollama API Documentation](https://github.com/ollama/ollama/blob/main/docs/api.md)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [The Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Documentation](https://tokio.rs/)
- [reqwest Documentation](https://docs.rs/reqwest/)

### Internal Documents

- [README.md](../README.md) - Project overview
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Contribution guidelines
- [CHANGELOG.md](../CHANGELOG.md) - Version history
- [DEV_NOTES.md](../DEV_NOTES.md) - Development notes

---

**Document Status:** Living document - updated as project evolves
**Next Review:** After Phase 1 completion
**Maintainer:** Francisco (@franciscotbjr)
