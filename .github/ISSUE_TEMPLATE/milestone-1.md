---
name: Milestone 1 - Project Foundation
about: Set up basic project structure and tooling
title: '[M1] Project Foundation'
labels: milestone, setup
assignees: ''
---

## Milestone 1: Project Foundation

### Goal
Establish the basic Rust project structure with proper module organization, command-line interface, application loop, and CI/CD pipeline.

### Tasks

#### Project Structure
- [ ] Initialize Cargo workspace with proper directory layout
- [ ] Set up module structure (backends/, render_graph/, scene/, shaders/, ui/, profiling/)
- [ ] Configure Cargo.toml with initial dependencies
- [ ] Add .gitignore for Rust projects

#### Application Framework
- [ ] Implement main.rs with application entry point
- [ ] Create app.rs with basic application loop using winit
- [ ] Add command-line argument parsing (using clap)
  - Backend selection (--backend vulkan|directx|wgpu)
  - Window dimensions (--width, --height)
  - Debug flags (--debug)

#### CI/CD Pipeline
- [ ] Create GitHub Actions workflow for:
  - Build verification (cargo build)
  - Unit tests (cargo test)
  - Clippy linting (cargo clippy)
  - Format checking (cargo fmt --check)
- [ ] Set up local runner configuration for graphics tests (document only, actual runner setup separate)
- [ ] Add README badges for build status

#### Testing Infrastructure
- [ ] Set up unit test framework structure
- [ ] Create basic integration test skeleton in tests/
- [ ] Add test utilities module

### Acceptance Criteria
- [ ] Project builds successfully with `cargo build`
- [ ] Window opens and closes cleanly with winit
- [ ] Command-line arguments are parsed correctly
- [ ] CI/CD pipeline runs and passes
- [ ] Basic test infrastructure is in place

### Dependencies
None (first milestone)

### Estimated Effort
2-3 days
