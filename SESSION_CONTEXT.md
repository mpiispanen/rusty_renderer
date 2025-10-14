# Rusty Renderer - Session Context

**Last Updated:** 2025-10-14  
**Current Phase:** Milestone 1 Complete - Ready for Milestone 2

## 📍 Current Status

### What's Done
- ✅ Initial design document created (docs/DESIGN.md)
- ✅ Project structure planned and documented
- ✅ GitHub issue templates created (bug report, feature request)
- ✅ Milestone breakdown documented (docs/MILESTONES.md, docs/GITHUB_SETUP.md)
- ✅ GitHub CLI scripts created for milestone/issue creation
- ✅ Documentation conversion script (markdown → HTML)
- ✅ **MILESTONE 1 COMPLETE:** Project Foundation
  - ✅ Cargo workspace structure set up
  - ✅ Basic application framework implemented (app.rs, main.rs, lib.rs)
  - ✅ Command-line argument parsing (clap with config.rs)
  - ✅ CI/CD pipeline configured (GitHub Actions with caching)
  - ✅ Testing infrastructure created (12 tests passing)
  - ✅ CI validated and passing (run #18511049136)

### What's Next
1. **Start Milestone 2: Window Management & Event Loop**
   - Implement window creation with winit
   - Event loop and input handling
   - Window resize handling
   - Basic error reporting

### Current Branch
- `main` - up to date with origin, all CI passing

## 🎯 Active Milestone

**Milestone 1: Project Foundation** ✅ COMPLETE

Completed items:
1. ✅ Set up Cargo workspace structure - Basic Cargo.toml with dependencies
2. ✅ Implement basic application framework - app.rs with App struct, renderer framework
3. ✅ Add command-line argument parsing - clap integration with Config struct
4. ✅ Set up CI/CD pipeline - GitHub Actions with build, test, clippy, format, docs jobs
5. ✅ Create testing infrastructure - Unit tests for config, integration test framework

## 📁 Project Structure

```
rusty_renderer/
├── docs/                    # All documentation
│   ├── DESIGN.md           # Main design document
│   ├── MILESTONES.md       # Milestone overview
│   ├── GITHUB_SETUP.md     # GitHub setup guide
│   ├── README.md           # Docs readme
│   └── convert.sh          # MD → HTML converter
├── scripts/                # Helper scripts
│   ├── create_milestones.sh    # Create GitHub milestones
│   ├── create_m1_issues.sh     # Create M1 issues
│   └── README.md
├── .github/
│   ├── ISSUE_TEMPLATE/     # Issue templates
│   └── workflows/
│       └── ci.yml          # CI/CD pipeline (M1 ✅)
├── src/                    # Source code (M1 ✅)
│   ├── main.rs            # Entry point with CLI parsing
│   ├── lib.rs             # Library exports
│   ├── app.rs             # Main App struct and run loop
│   ├── config.rs          # Configuration handling
│   ├── backends/          # Backend trait (stub)
│   ├── render_graph/      # Render graph (stub)
│   ├── scene/             # Scene management (stub)
│   ├── shaders/           # Shader utilities (stub)
│   ├── ui/                # UI integration (stub)
│   └── profiling/         # Profiling (stub)
├── tests/                 # Integration tests (M1 ✅)
│   └── backend_test.rs    # Backend loading tests
├── shaders/               # Shader files (M3+)
├── assets/                # Test assets (M7+)
├── Cargo.toml             # Project metadata and deps
└── LICENSE
```

## 🔑 Key Decisions & Architecture

### Graphics Backends
- **Primary:** Vulkan (vulkanalia) - develop on Linux
- **Secondary:** DirectX 12 - test via Proton
- **Tertiary:** wgpu - portability layer
- **Implementation order:** Vulkan first → validate with DirectX → wgpu

### Core Architecture
- **Backend Abstraction:** Trait-based, all backends in same crate
- **Render Graph:** Runtime graph with automatic dependency resolution
- **Scene Format:** glTF (with custom abstraction)
- **Shaders:** Online (runtime) and offline (pre-compiled) workflows
- **UI:** egui for debug interface
- **Window:** winit for cross-platform windowing

### Development Workflow
1. Create detailed GitHub issues for planned work
2. Write tests (unit/integration)
3. Implement feature to pass tests
4. Update design document if architecture changes

## 📚 Important Documentation

- **Design Document:** `docs/DESIGN.md`
- **Architecture:** See "Architecture Overview" in DESIGN.md
- **Milestones:** `docs/MILESTONES.md`
- **GitHub Setup:** `docs/GITHUB_SETUP.md`

## 🛠️ Quick Commands

### Build and Test
```bash
# Build (debug)
cargo build

# Build (release)
cargo build --release

# Test
cargo test

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Format
cargo fmt --all

# Check formatting
cargo fmt --all -- --check

# Build docs
cargo doc --no-deps

# Run application
cargo run -- --help
cargo run -- --backend vulkan --width 1920 --height 1080
```

### Documentation
```bash
# Generate HTML docs
./docs/convert.sh

# View design doc
cat docs/DESIGN.md

# View milestones
cat docs/MILESTONES.md
```

### CI Status
```bash
# View recent workflow runs
gh run list --limit 5

# View specific run details
gh run view <run_id>

# Check failed job logs
gh run view <run_id> --log-failed
```

### Git
```bash
# Check status
git status

# View recent commits
git log --oneline -10

# View specific commit
git show <commit_sha>
```

## 🚀 Starting a New Session

1. **Review this file** (`SESSION_CONTEXT.md`) - Current status and recent work
2. **Check git status** (`git status`) - Verify clean working tree
3. **Review current milestone** - Check docs/MILESTONES.md for M2 details
4. **Run tests** (`cargo test`) - Ensure everything still works
5. **Check CI status** (`gh run list --limit 3`) - Verify latest runs passed

## 💡 Notes for AI Assistant

- **Milestone 1 is COMPLETE** ✅
- Project has basic Rust structure with:
  - App framework (app.rs with run loop)
  - CLI parsing (clap with config.rs)  
  - CI/CD pipeline (GitHub Actions with caching)
  - Testing infrastructure (12 tests passing)
- Ready to start **Milestone 2: Window Management & Event Loop**
- Focus on Vulkan first, then DirectX, then wgpu for each feature
- Keep implementations minimal and focused
- Update SESSION_CONTEXT.md when significant progress is made
- macOS support is explicitly out of scope
- CI uses self-hosted runner on Bazzite Linux

## 🐛 Known Issues

- CI job completion detection: GitHub Actions can show "in_progress" status even after job completes successfully. Always verify with `gh run view <run_id>` or check the actual workflow conclusion field.

## 📝 Recent Commits

Latest (from `git log --oneline -5`):
```
6ba10e8 (HEAD -> main, origin/main) Establish proper CI-validated workflow
1467b10 Create testing infrastructure
03a1120 Document disk space warning on Bazzite
507b6ca Set up CI/CD pipeline
2b7bf4e Add command-line argument parsing
```

**Latest CI Run:** #18511049136 - ✅ SUCCESS (2025-10-14T21:49:03Z)
