# Agent Log

## 2025-11-03 – Codex Session
- Closed obsolete GitHub issues #55 (Material system), #68 (shadow map pass), and #69 (PCF) after consolidating work under #90.
- Gathered context on issue #88: remaining legacy paths live mostly in `src/app.rs` and the deprecated `pipelines/` module.
- Removed the legacy `application`/`pipelines` modules, updated `src/main.rs` to launch the new `App`, dropped unused fields in `src/app.rs`, and deleted pipeline-based examples.
- Next up: verify docs/tests as needed for the new architecture and continue iterating on issue #88 tasks (e.g., forward pass polish).
- Updated active docs (`README.md`, `CURRENT_STATE.md`, `ROADMAP.md`, `docs/DESIGN.md`, `docs/WORKFLOW.md`) to reflect the render-graph flow and deleted redundant status/design documents to keep a single source of truth.
- Shifted scene resource preparation into `ForwardSimplePass::prepare_scene_resources`, so `App` just calls the helper and lets the pass/render graph handle geometry, camera, and lighting uploads.
- Added index buffer support to the forward pass helper so we bind indices and issue `draw_indexed` (Vulkan path already wired; DX12 still to implement).
- Implemented DirectX 12 `bind_vertex_buffer`/`bind_index_buffer`/`draw[_indexed]` so the backend no longer panics when the render graph executes indexed draws.
- Updated `run_with_proton.sh` to auto-build the Windows binary (using `cargo build` or `cargo xwin build`) before syncing assets and launching via Proton.
