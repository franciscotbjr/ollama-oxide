# AGENTS.md — ollama-oxide

This project uses the **Stateful Spec** methodology for AI-assisted development.

## Getting Started

1. Read **`.stateful-spec/memory.md`** first — it contains the current project state, active work, and constraints.
2. Read **`.stateful-spec/project-definition.md`** — it defines the technology stack, conventions, and quality gates.
3. Read **`.stateful-spec/methodology/`** — the full methodology (phases, roles, decision framework).
4. Use **`.stateful-spec/templates/`** when creating new specs, ADRs, or iteration files (templates from [stateful-spec](https://github.com/franciscotbjr/stateful-spec)).

## Methodology

Every unit of work follows the iteration cycle: **Analyze → Plan → Specify → Implement → Verify**

- Phase guides live in `.stateful-spec/methodology/phases/`
- Roles and expectations: `.stateful-spec/methodology/roles.md`
- Decision framework: `.stateful-spec/methodology/decision-framework.md`

To refresh methodology and prompts from upstream, follow `.stateful-spec/prompts/initialization/update-project.md`.

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

| Operation | Cursor | Purpose |
|-----------|--------|---------|
| Resume Session | `@resume-session` | Load context from `.stateful-spec/`, summarize state, pick up where you left off |
| Save Session | `@save-session` | Update `memory.md` and iteration files before ending a session |
| Create Technical Spec | `@create-technical-spec` | Analyze a request and produce a specification (stored in `impl/`) |
| Write Tests | `@write-tests` | Generate tests following project conventions (mockito, tokio::test) |
| Debug Issue | `@debug-issue` | Root cause analysis, diagnostic steps, minimal fix, regression test |
| Refactor Code | `@refactor-code` | Step-by-step refactoring with behavioral preservation |
| Review Changes | `@review-changes` | Self-review against correctness, conventions, security, performance |
| Write Commit Message | `@write-commit-message` | Structured commit message (imperative subject, what/why body) |
| Update Documentation | `@update-documentation` | Update README, CHANGELOG, ARCHITECTURE, API docs after changes |

Upstream prompt sources: `.stateful-spec/prompts/operations/`. See `.cursor/rules/stateful-spec.mdc` for the methodology overview and pointers (Cursor users). Other agents can read the same paths under `.stateful-spec/`.
