#!/usr/bin/env bash
# Script to create GitHub issues for Milestone 1: Project Foundation

set -e

# Get repository info
REPO_OWNER=$(gh repo view --json owner -q .owner.login)
REPO_NAME=$(gh repo view --json name -q .name)

echo "Creating issues for Milestone 1: Project Foundation"
echo "Repository: ${REPO_OWNER}/${REPO_NAME}"
echo ""

# Issue 1: Set up Cargo workspace structure
echo "Creating issue: Set up Cargo workspace structure..."
gh issue create \
  --title "Set up Cargo workspace structure" \
  --milestone "M1: Project Foundation" \
  --label "setup,M1" \
  --body "## Description
Set up the initial Rust project structure with Cargo and organize modules according to the architecture plan.

## Tasks
- [ ] Initialize Cargo workspace with \`cargo init\`
- [ ] Create module directories:
  - [ ] \`src/backends/\`
  - [ ] \`src/render_graph/\`
  - [ ] \`src/scene/\`
  - [ ] \`src/shaders/\`
  - [ ] \`src/ui/\`
  - [ ] \`src/profiling/\`
- [ ] Create placeholder \`mod.rs\` files in each module
- [ ] Configure \`Cargo.toml\` with project metadata and initial dependencies
- [ ] Add comprehensive \`.gitignore\` for Rust projects (target/, Cargo.lock for libs, etc.)
- [ ] Create \`tests/\`, \`shaders/\`, and \`assets/\` directories

## Acceptance Criteria
- [ ] Project builds with \`cargo build\`
- [ ] Module structure matches design document
- [ ] All placeholder files compile without errors

## Related
- See docs/DESIGN.md Architecture Overview section"

# Issue 2: Implement basic application framework
echo "Creating issue: Implement basic application framework..."
gh issue create \
  --title "Implement basic application framework" \
  --milestone "M1: Project Foundation" \
  --label "setup,M1" \
  --body "## Description
Create the main application entry point and basic event loop using winit for window management.

## Tasks
- [ ] Add \`winit\` dependency to Cargo.toml
- [ ] Create \`src/main.rs\` with application entry point
- [ ] Create \`src/app.rs\` with \`App\` struct
- [ ] Implement basic winit event loop
  - [ ] Window creation
  - [ ] Event handling (close, resize)
  - [ ] Clean shutdown
- [ ] Add basic error handling and logging (consider \`env_logger\` or \`tracing\`)

## Acceptance Criteria
- [ ] Application starts and creates a window
- [ ] Window displays with correct title (\"Rusty Renderer\")
- [ ] Window can be closed cleanly (no panics or hangs)
- [ ] Resize events are handled
- [ ] Basic logging works

## Related
- Milestone 1: Project Foundation
- See docs/DESIGN.md Application Framework section"

# Issue 3: Add command-line argument parsing
echo "Creating issue: Add command-line argument parsing..."
gh issue create \
  --title "Add command-line argument parsing" \
  --milestone "M1: Project Foundation" \
  --label "setup,M1" \
  --body "## Description
Implement command-line interface using clap for configuring the application at startup.

## Tasks
- [ ] Add \`clap\` dependency to Cargo.toml (with derive feature)
- [ ] Define CLI arguments struct:
  - [ ] \`--backend <vulkan|directx|wgpu>\` - Select graphics backend
  - [ ] \`--width <WIDTH>\` - Window width (default: 1280)
  - [ ] \`--height <HEIGHT>\` - Window height (default: 720)
  - [ ] \`--debug\` - Enable debug mode/validation layers
  - [ ] \`--vsync\` - Enable/disable vsync
- [ ] Integrate argument parsing in main.rs
- [ ] Pass configuration to App initialization
- [ ] Add \`--help\` and \`--version\` support

## Acceptance Criteria
- [ ] All arguments parse correctly
- [ ] Help text is clear and useful (\`--help\`)
- [ ] Invalid arguments show helpful error messages
- [ ] Arguments affect application behavior (window size, etc.)
- [ ] Default values work when arguments not provided

## Related
- Milestone 1: Project Foundation"

# Issue 4: Set up CI/CD pipeline
echo "Creating issue: Set up CI/CD pipeline..."
gh issue create \
  --title "Set up CI/CD pipeline" \
  --milestone "M1: Project Foundation" \
  --label "ci,M1" \
  --body "## Description
Create GitHub Actions workflow for continuous integration with build, test, lint, and format checks.

## Tasks
- [ ] Create \`.github/workflows/ci.yml\`
- [ ] Add workflow triggers (push, pull_request)
- [ ] Add job: Build (\`cargo build --release\`)
- [ ] Add job: Test (\`cargo test\`)
- [ ] Add job: Clippy (\`cargo clippy -- -D warnings\`)
- [ ] Add job: Format check (\`cargo fmt --check\`)
- [ ] Add job: Docs build (\`cargo doc --no-deps\`)
- [ ] Configure Rust toolchain (stable)
- [ ] Add caching for cargo dependencies
- [ ] Document local runner setup for graphics tests (in docs/)
- [ ] Update README.md with CI status badges

## Acceptance Criteria
- [ ] CI workflow runs on every push and PR
- [ ] All checks pass on main branch
- [ ] Build artifacts are cached appropriately
- [ ] Failed checks provide clear error messages
- [ ] README displays build status badge

## Related
- Milestone 1: Project Foundation
- See docs/DESIGN.md for CI/CD requirements"

# Issue 5: Create testing infrastructure
echo "Creating issue: Create testing infrastructure..."
gh issue create \
  --title "Create testing infrastructure" \
  --milestone "M1: Project Foundation" \
  --label "testing,M1" \
  --body "## Description
Set up the testing framework with unit test structure and integration test skeleton.

## Tasks
- [ ] Create \`tests/\` directory for integration tests
- [ ] Create \`tests/common/mod.rs\` for shared test utilities
- [ ] Add unit test examples in relevant modules
- [ ] Set up test module structure:
  - [ ] \`tests/integration_test.rs\` - Basic integration test skeleton
  - [ ] \`src/backends/mod.rs\` - Backend trait test utilities
- [ ] Configure test features in Cargo.toml if needed
- [ ] Add test documentation in tests/README.md
- [ ] Create helper functions for test setup/teardown

## Acceptance Criteria
- [ ] \`cargo test\` runs successfully
- [ ] Unit tests and integration tests are properly separated
- [ ] Test utilities are available and documented
- [ ] At least one example unit test exists
- [ ] At least one example integration test exists
- [ ] Tests can be run individually and in parallel

## Related
- Milestone 1: Project Foundation
- See docs/DESIGN.md Testing Strategy section"

echo ""
echo "✓ All Milestone 1 issues created successfully!"
echo ""
echo "To view issues: gh issue list --milestone 'M1: Project Foundation'"
