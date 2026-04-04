# Iteration 021 — Stateful Spec upstream sync

| Field | Value |
|-------|--------|
| **Type** | chore |
| **Status** | done |
| **Source** | [franciscotbjr/stateful-spec](https://github.com/franciscotbjr/stateful-spec) `main` |
| **Date** | 2026-04-03 |

## Description

Full refresh per `prompts/initialization/update-project.md`: methodology, templates, operation prompts, Cursor rules, and agent docs aligned with upstream. `memory.md` and `project-definition.md` were not replaced.

## Acceptance criteria

- [x] `.stateful-spec/methodology/` matches upstream `methodology/`
- [x] `.stateful-spec/templates/` vendored from upstream `templates/`
- [x] `.stateful-spec/prompts/operations/` vendored; `.cursor/rules/*.mdc` regenerated with ollama-oxide footers
- [x] Initialization prompts `new-project.md`, `onboard-existing.md`, `update-project.md` present under `.stateful-spec/prompts/initialization/`
- [x] Phase-transition prompts under `.stateful-spec/prompts/phase-transitions/`
- [x] `AGENTS.md` and `stateful-spec.mdc` updated (templates, `@` table, upstream pointer)
- [x] `memory.md` records the sync

## Tasks completed

- [x] Download methodology, templates, operations, and selected init prompts from raw.githubusercontent.com
- [x] Regenerate nine operation `.mdc` files from upstream markdown (UTF-8)
- [x] Update facade documentation

## References

- Workflow: `.stateful-spec/prompts/initialization/update-project.md`

## Follow-up

- [x] Working tree committed (2026-04-03).
