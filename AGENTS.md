# AGENTS.md — ollama-oxide

This project uses the **Stateful Spec** methodology for AI-assisted development.

## Getting Started

1. Read **`.stateful-spec/memory.md`** first — it contains the current project state, active work, and constraints.
2. Read **`.stateful-spec/project-definition.md`** — it defines the technology stack, conventions, and quality gates.
3. Read **`.stateful-spec/methodology/`** — the full methodology (phases, roles, decision framework).

## Methodology

Every unit of work follows the iteration cycle: **Analyze → Plan → Specify → Implement → Verify**

- Phase guides live in `.stateful-spec/methodology/phases/`
- Roles and expectations: `.stateful-spec/methodology/roles.md`
- Decision framework: `.stateful-spec/methodology/decision-framework.md`

## Key Constraints

- Follow the Project Definition's conventions for all code
- All quality gates must pass: `cargo clippy --all-features -- -D warnings`, `cargo fmt --check`, `cargo test --all-features`, `cargo build --all-features`
- No unsafe code without justification
- All public items must have rustdoc documentation
- All types must be Send + Sync
- Inference types must not depend on the HTTP module (strict dependency hierarchy)
- `tests/` must pass without external services (use mockito for HTTP mocking)
- `mod.rs` files are facades only — no logic, only module declarations and re-exports

## Operations

Available operations for session management and common tasks:

| Operation | Purpose |
|-----------|---------|
| Resume Session | Load context from `.stateful-spec/`, summarize state, pick up where you left off |
| Save Session | Update `memory.md` and iteration files before ending a session |
| Create Technical Spec | Analyze a request and produce a specification (stored in `impl/`) |
| Write Tests | Generate tests following project conventions (mockito, tokio::test) |
| Debug Issue | Root cause analysis, diagnostic steps, minimal fix, regression test |
| Refactor Code | Step-by-step refactoring with behavioral preservation |
| Review Changes | Self-review against correctness, conventions, security, performance |
| Write Commit Message | Structured commit message (imperative subject, what/why body) |
| Update Documentation | Update README, CHANGELOG, ARCHITECTURE, API docs after changes |

See `.cursor/rules/stateful-spec.mdc` for detailed operation instructions (Cursor users), or reference this file directly for other AI agents.
