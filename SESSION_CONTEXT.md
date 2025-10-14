# Rusty Renderer - Session Context

**Last Updated:** 2025-10-14  
**Current Phase:** Initial Setup & Planning

## 📍 Current Status

### What's Done
- ✅ Initial design document created (docs/DESIGN.md)
- ✅ Project structure planned and documented
- ✅ GitHub issue templates created (bug report, feature request)
- ✅ Milestone breakdown documented (docs/MILESTONES.md, docs/GITHUB_SETUP.md)
- ✅ GitHub CLI scripts created for milestone/issue creation
- ✅ Documentation conversion script (markdown → HTML)

### What's Next
1. **Push changes to GitHub** (`git push origin main`)
2. **Create GitHub milestones** (`./scripts/create_milestones.sh`)
3. **Create Milestone 1 issues** (`./scripts/create_m1_issues.sh`)
4. **Start Milestone 1 implementation**

### Current Branch
- `main` - 4 commits ahead of origin

## 🎯 Active Milestone

**Milestone 1: Project Foundation** (Not yet started)

Issues to create (via script):
1. Set up Cargo workspace structure
2. Implement basic application framework
3. Add command-line argument parsing
4. Set up CI/CD pipeline
5. Create testing infrastructure

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
│   └── ISSUE_TEMPLATE/     # Issue templates
├── LICENSE
└── README.md

# To be created in M1:
├── src/                    # Source code (M1)
│   ├── main.rs
│   ├── app.rs
│   ├── backends/
│   ├── render_graph/
│   ├── scene/
│   ├── shaders/
│   ├── ui/
│   └── profiling/
├── tests/                  # Integration tests (M1)
├── shaders/               # Shader files (M3+)
└── assets/                # Test assets (M7+)
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

### Documentation
```bash
# Generate HTML docs
./docs/convert.sh

# View design doc
cat docs/DESIGN.md
```

### GitHub Setup (when ready)
```bash
# Push changes
git push origin main

# Create milestones
./scripts/create_milestones.sh

# Create M1 issues
./scripts/create_m1_issues.sh

# View issues
gh issue list --milestone "M1: Project Foundation"
```

### Development (after M1 setup)
```bash
# Build
cargo build

# Test
cargo test

# Lint
cargo clippy

# Format
cargo fmt
```

## 🚀 Starting a New Session

1. **Review this file** (`SESSION_CONTEXT.md`)
2. **Check git status** (`git status`)
3. **Review current milestone** (`gh issue list --milestone "M1: Project Foundation"` or check docs/MILESTONES.md)
4. **Check design doc** for architecture reference (`docs/DESIGN.md`)

## 💡 Notes for AI Assistant

- Project is in **initial planning phase**
- No Rust code exists yet - will be created in Milestone 1
- Focus on Vulkan first, then DirectX, then wgpu for each feature
- Keep implementations minimal and focused
- Update SESSION_CONTEXT.md when significant progress is made
- macOS support is explicitly out of scope

## 📝 Recent Commits

```bash
git log --oneline -5
```

Latest:
- Add GitHub CLI scripts for milestone and issue creation
- Restructure milestone tracking to use proper GitHub milestones
- Add GitHub issue templates and milestone tracking
- Initial design document and documentation structure
