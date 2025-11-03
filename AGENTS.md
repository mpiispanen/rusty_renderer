# Agent Log

## 2025-11-03 – Codex Session
- Closed obsolete GitHub issues #55 (Material system), #68 (shadow map pass), and #69 (PCF) after consolidating work under #90.
- Gathered context on issue #88: remaining legacy paths live mostly in `src/app.rs` and the deprecated `pipelines/` module.
- Removed the legacy `application`/`pipelines` modules, updated `src/main.rs` to launch the new `App`, dropped unused fields in `src/app.rs`, and deleted pipeline-based examples.
- Next up: verify docs/tests as needed for the new architecture and continue iterating on issue #88 tasks (e.g., forward pass polish).
- Updated active docs (`README.md`, `CURRENT_STATE.md`, `ROADMAP.md`, `docs/DESIGN.md`, `docs/WORKFLOW.md`) to reflect the render-graph flow and deleted redundant status/design documents to keep a single source of truth.
