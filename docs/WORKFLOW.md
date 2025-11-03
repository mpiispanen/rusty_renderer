# Development Workflow

**Last Updated:** 2025-11-03

This document describes the standard development workflow for the Rusty Renderer project, including testing with both Vulkan and DirectX (via Proton on Bazzite).

## Overview

We follow a **CI-validated** workflow where all changes must pass continuous integration checks before being considered complete. We also emphasize **incremental commits** with proper validation at each step.

---

## Quick Reference

```bash
# Before starting work
git pull origin main

# Format code (do this frequently!)
cargo fmt

# Check for issues
cargo clippy --all-targets -- -D warnings

# Run tests
cargo test --lib

# Build and test
cargo build --release
cargo run --release -- --scene scenes/cube.toml --headless

# Commit (use pre-commit hook for auto-checks)
git add -A
git commit -m "Description"
git push origin main
```

---

## Standard Development Flow

### 1. Pick an Issue / Define Task
- Browse open issues on GitHub or milestones
- Comment on the issue to indicate you're working on it
- Understand acceptance criteria before starting
- If no issue exists, create one first (helps track progress)

### 2. Set Up Your Environment

**First Time Setup:**
```bash
# Install pre-commit hook (recommended!)
cp scripts/pre-commit.sample .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

**Pre-commit hook does:**
- `cargo fmt --check` - ensures code is formatted
- `cargo clippy` - catches common mistakes
- `cargo test --lib` - runs unit tests
- Prevents broken commits from being pushed

### 3. Make Changes Incrementally

**Best Practice: Small, Tested Commits**

Instead of making large changes, work in small increments:

```bash
# 1. Make a small change (e.g., add a struct)
# 2. Format and check
cargo fmt
cargo clippy

# 3. Build to ensure it compiles
cargo build

# 4. Commit immediately
git add -A
git commit -m "Add Transform struct to scene module"

# 5. Repeat for next small change
```

**Why This Works:**
- Easy to find when bugs were introduced
- CI catches issues earlier
- Easier to review
- Can revert small changes without losing work

### 4. Local Validation (REQUIRED)

Before pushing, ALWAYS run local checks to catch issues early:

```bash
# 1. Format code (fixes most style issues)
cargo fmt

# 2. Run linter (catches bugs and bad patterns)
cargo clippy --all-targets -- -D warnings

# 3. Run all tests
cargo test

# 4. Build in release mode
cargo build --release
```

**All of these must pass before pushing.**

### 5. Test Rendering (Both Backends)

Test your changes on both Vulkan and DirectX to ensure backend parity:

```bash
# Test Vulkan (native on Linux/Bazzite)
cargo run --release -- --backend vulkan --scene cube

# Test DirectX via Proton (on Linux/Bazzite)
./run_with_proton.sh cube

# Headless mode with screenshot for comparison
cargo run --release -- --backend vulkan --scene cube --headless --screenshot vk_test.png
./run_with_proton.sh cube --headless --screenshot dx_test.png

# Compare outputs (if you have visual comparison tools)
# python3 scripts/flip_compare.py vk_test.png dx_test.png
```

**For rendering changes, both backends should produce similar output.**

See [docs/TESTING_DIRECTX_ON_LINUX.md](TESTING_DIRECTX_ON_LINUX.md) for more details on Proton testing.

### 5. Commit Messages

Use descriptive commit messages following this format:

```
<Summary line (50 chars max)>

<Blank line>

<Detailed description explaining WHY, not just WHAT>
<Any relevant context or decisions made>
<Reference to issue if applicable>

Issue: #123
```

**Good Examples:**
```
Fix GLSL struct layout for lighting uniforms

Changed uint _padding1[3] to three separate uint fields
because GLSL std140 layout treats arrays differently than
Rust repr(C). This was causing the shader to read wrong
offsets for light data.

Fixes lighting appearing flat/gray.
```

```
Add per-frame descriptor sets for synchronization

Vulkan requires separate descriptor sets for frames in flight.
Changed from Vec<DescriptorSet> to Vec<Vec<DescriptorSet>>
to prevent "descriptor set in use" validation errors.

Issue: #156
```

**Bad Examples:**
```
fix bug          # What bug? How? Why?
update shader    # Too vague
wip              # Never commit WIP to main
```

Resolves #<issue-number>
```

Example:
```
Fix clippy error in backend tests

Remove assert!(true) which clippy flags as optimized out.
The test validates module structure by compiling successfully.

Resolves #10
```

Then push:
```bash
git add <files>
git commit -m "Your message"
git push origin main
```

### 5. Wait for CI (CRITICAL)

**DO NOT** close the issue immediately after pushing!

#### Monitor the CI run:
```bash
# View recent runs
gh run list --limit 5

# Watch a specific run (get ID from list)
gh run watch <run-id>

# Or just watch the latest run
gh run watch
```

#### Wait for all jobs to complete:
- ✓ Build
- ✓ Test  
- ✓ Clippy
- ✓ Format
- ✓ Documentation
- ✓ Backend Rendering Tests (when implemented)

#### If CI fails:
1. Check the logs: `gh run view <run-id> --log`
2. Identify the failure
3. Fix the issue locally
4. Re-run local validation
5. Commit and push the fix
6. Wait for CI again

### 6. Close Issue

Only after CI passes completely:

```bash
gh issue close <issue-number> --comment "✅ <Brief summary of what was done>

<Details if needed>

CI passed: <link to successful run>"
```

## Common CI Failures

### Clippy Errors
```bash
# Run clippy with the same settings as CI
cargo clippy --all-targets --all-features -- -D warnings
```

Common issues:
- Unused imports/variables: Remove them or prefix with `_`
- Unoptimized code patterns: Follow clippy's suggestions
- Format issues: Run `cargo fmt`

### Test Failures
```bash
# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test <test_name> -- --nocapture
```

### Format Issues
```bash
# Check formatting
cargo fmt --check

# Auto-fix formatting
cargo fmt
```

### Build Failures
Usually caught locally, but if CI fails:
- Check for platform-specific code
- Verify all dependencies are in Cargo.toml
- Check for missing feature flags

## Disk Space Warnings

The self-hosted runner may show disk space warnings due to Bazzite's composefs root filesystem. These are **false positives** and can be ignored if:
- `/var/home` has adequate space (`df -h /var/home`)
- Jobs complete successfully

See `docs/SELF_HOSTED_RUNNER.md` for details.

## Branch Strategy

Currently using **trunk-based development** (direct to main):
- All work goes directly to main branch
- CI validates every commit
- Issues must be small and completable quickly

Future: May adopt feature branches for larger changes.

## Parallel Work Strategy

### When to Use GitHub Agent Runners

Some issues can be worked on in parallel without dependencies. These are perfect candidates for GitHub agent runners:

**Indicators for Parallel Work:**
- Issues with same milestone but different implementation files
- Stub implementations (e.g., separate backend stubs)
- Documentation tasks
- Independent test suites
- Non-conflicting feature additions

**Example - M2 Backend Stubs:**
After issue #11 (Define core backend traits) is complete, issues #12, #13, #14 can be worked on in parallel:
- #12: Vulkan backend stub (src/backends/vulkan/)
- #13: DirectX backend stub (src/backends/directx/)
- #14: wgpu backend stub (src/backends/wgpu_backend/)

Each touches different files, so no merge conflicts.

### How to Identify Parallel Opportunities

1. **Check Dependencies:** Issues with "Depends on" can't start until dependency completes
2. **Check File Overlap:** Issues touching different files = parallel safe
3. **Check Milestone Planning:** Planning docs often identify parallel work

### Workflow for Parallel Issues

When parallel work is identified:

1. **Flag the opportunity:**
   ```bash
   # Comment on issues
   gh issue comment <issue> --body "🔀 This issue can be worked in parallel with #X, #Y, #Z
   
   File scope: src/backends/vulkan/
   No conflicts with other issues in progress."
   ```

2. **Assign to GitHub agent runners** if available

3. **Coordinate in issue comments** to avoid conflicts

4. **Monitor CI** - each parallel branch must pass CI independently

### Current M2 Parallel Opportunity 🔀

**After #11 completes**, these 3 issues can be worked **in parallel**:
- Issue #12: Vulkan backend stub → `src/backends/vulkan/`
- Issue #13: DirectX backend stub → `src/backends/directx/`  
- Issue #14: wgpu backend stub → `src/backends/wgpu_backend/`

**Why parallel-safe:**
- Different directory scope (no file conflicts)
- Same dependencies (only #11)
- Independent test files
- Stub implementations (no complex integration)

This could reduce M2 timeline from 20-28 hours to ~16-22 hours (saving 4-6 hours).

## Issue Lifecycle

1. **Open**: Issue created, ready to work on
2. **In Progress**: Someone is actively working (comment on issue)
3. **Pushed**: Code committed and pushed, CI running
4. **CI Passed**: All checks green
5. **Closed**: Issue resolved, CI passed, work complete

## Quality Standards

### Code Quality
- All clippy warnings must be addressed
- Code must be formatted with `cargo fmt`
- No TODO comments without associated issues
- Clear, self-documenting code

### Test Coverage
- New features must include tests
- Bug fixes must include regression tests
- Tests must pass consistently

### Documentation
- Public APIs must be documented
- Complex logic needs comments
- README updated for user-facing changes
- Architecture docs updated for structural changes

## Tips

### Speed Up Feedback
```bash
# Run only clippy on changed files
cargo clippy --all-targets -- -D warnings

# Run only specific tests
cargo test <test_name>

# Watch CI without blocking terminal
gh run watch <run-id> &
```

### Avoid Common Mistakes
1. ❌ Closing issue before CI completes
2. ❌ Skipping local validation
3. ❌ Pushing broken code "to see what CI says"
4. ❌ Ignoring clippy warnings
5. ❌ Committing without running tests

### Best Practices
1. ✅ Run full local validation before pushing
2. ✅ Wait for green CI before closing issues
3. ✅ Fix issues immediately if CI fails
4. ✅ Keep commits focused and atomic
5. ✅ Write descriptive commit messages

## Getting Help

- CI logs: `gh run view <run-id> --log`
- Local test output: `cargo test -- --nocapture`
- Clippy help: `cargo clippy --help`
- Check docs: `docs/` directory
- Ask in issues: Tag with questions

## Summary: The Golden Rule

**Code is not done until CI is green. ✅**

Never close an issue until you've confirmed all CI jobs pass.

---

## Debugging Best Practices

### Systematic Shader Debugging

When shaders aren't working as expected:

**1. Verify Data Upload**
```rust
// Add logging to see what's being sent to GPU
log::info!("Uploading light: type={}, dir={:?}", 
    light.light_type, light.position_or_direction);
```

**2. Test Shader Reads**
```glsl
// Output raw values as colors to verify reads
void main() {
    Light light = lighting.lights[0];
    // Map [-1,1] to [0,1] for visualization
    outColor = vec4(light.positionOrDirection.xyz * 0.5 + 0.5, 1.0);
}
```

**3. Isolate the Problem**
```glsl
// Test one thing at a time
void main() {
    // Test 1: Are normals working?
    outColor = vec4(normalize(fragNormal) * 0.5 + 0.5, 1.0);
    
    // Test 2: Are lights being read?
    if (lighting.lightCount > 0) {
        outColor = vec4(1.0, 0.0, 0.0, 1.0); // Red if true
    }
    
    // Test 3: Is diffuse calculation working?
    float diff = max(dot(normal, lightDir), 0.0);
    outColor = vec4(vec3(diff), 1.0); // Grayscale based on angle
}
```

**4. Check Struct Layouts**
- GLSL std140 layout rules differ from Rust `repr(C)`
- Arrays in GLSL can have unexpected padding
- **Use explicit fields instead of arrays for padding**
- Example:
  ```glsl
  // BAD - array padding may not match Rust
  uint _padding[3];
  
  // GOOD - explicit fields always match
  uint padding1;
  uint padding2;
  uint padding3;
  ```

### Vulkan Validation Debugging

**Enable Validation:**
```bash
# Always develop with validation enabled
VK_INSTANCE_LAYERS=VK_LAYER_KHRONOS_validation cargo run
```

**Common Errors:**

| Error | Cause | Fix |
|-------|-------|-----|
| Descriptor set in use | Reusing across frames | Per-frame descriptor sets |
| Buffer in use | Destroying while GPU using it | `wait_idle()` before cleanup |
| Image layout mismatch | Wrong layout transition | Match render pass expectations |

### Debug Logging Strategy

```rust
// Use appropriate log levels
log::error!("Critical failure: {}", error);  // Errors only
log::warn!("Unexpected but handled: {}", issue);  // Warnings
log::info!("Important state change: {}", change);  // Key events
log::debug!("Detailed info: {}", details);  // Development
log::trace!("Very verbose: {}", data);  // Rarely used

// Add context to errors
.context("Failed to create buffer")
.with_context(|| format!("Buffer size: {}", size))
```

---

## Cross-Compilation and Testing

### Building for Windows on Linux

```bash
# Install Windows target
rustup target add x86_64-pc-windows-gnu

# Build for Windows
cargo build --release --target x86_64-pc-windows-gnu

# Binary location
ls target/x86_64-pc-windows-gnu/release/rusty_renderer.exe
```

### Testing DirectX via Proton

**Setup:**
```bash
# Install Steam and Proton
# Or install wine-staging

# Set up wine prefix
export WINEPREFIX=~/.wine-rusty-renderer
wine64 wineboot
```

**Run:**
```bash
# Via Proton
~/.steam/steam/steamapps/common/Proton\ 8.0/proton run \
  ./target/x86_64-pc-windows-gnu/release/rusty_renderer.exe \
  --backend directx --headless

# Or via Wine
wine64 ./target/x86_64-pc-windows-gnu/release/rusty_renderer.exe \
  --backend directx --headless
```

**Check DirectX Validation:**
```bash
# Enable D3D debug layer (requires Windows SDK)
# Set environment variable
export D3D12_DEBUG_LAYER=1
```

### Cross-Platform Testing Matrix

| Platform | Vulkan | wgpu | DirectX |
|----------|--------|------|---------|
| Linux    | ✅ Native | ✅ Native | 🍷 Proton/Wine |
| Windows  | ✅ Native | ✅ Native | ✅ Native |
| macOS    | ⚠️ MoltenVK | ✅ Native (Metal) | ❌ N/A |

---

## CI/CD Integration

### Visual Testing

**Setting Up Reference Images:**
```bash
# Generate reference image
cargo run --release -- --scene scenes/cube.toml --headless \
  --screenshot references/cube_reference.png

# Add to git
git add references/cube_reference.png
```

**Running Visual Tests:**
```bash
# Compare against reference
cargo test --test visual_tests

# Update reference if intentional change
cargo run --release -- --scene scenes/cube.toml --headless \
  --screenshot references/cube_reference.png
git add references/cube_reference.png
git commit -m "Update reference image for new lighting"
```

### Pre-Commit Hook

Create `.git/hooks/pre-commit`:
```bash
#!/bin/bash
set -e

echo "Running pre-commit checks..."

# Format check
echo "Checking formatting..."
cargo fmt --check || {
    echo "❌ Code not formatted. Run: cargo fmt"
    exit 1
}

# Clippy
echo "Running clippy..."
cargo clippy --all-targets -- -D warnings || {
    echo "❌ Clippy errors found"
    exit 1
}

# Tests
echo "Running tests..."
cargo test --lib || {
    echo "❌ Tests failed"
    exit 1
}

echo "✅ All pre-commit checks passed!"
```

---

## Performance Best Practices

### Profiling

```bash
# CPU profiling
cargo install cargo-flamegraph
cargo flamegraph --release -- --scene scenes/complex.toml --max-frames 1000

# Memory profiling
cargo install cargo-bloat
cargo bloat --release -n 20

# Build time profiling
cargo clean
cargo build --release --timings
# Opens browser with timing info
```

### GPU Profiling

**Vulkan:**
```bash
# RenderDoc
renderdoccmd capture ./target/release/rusty_renderer

# Nsight Graphics (NVIDIA)
nv-nsight-gfx ./target/release/rusty_renderer
```

**DirectX:**
```bash
# PIX on Windows
pix -captureOnStart ./target/release/rusty_renderer.exe
```

---

## Documentation

### Keeping Docs Updated

**When to Update:**
- Architecture changes → Update `docs/DESIGN.md`
- New features → Update `README.md` and feature docs
- Workflow changes → Update `docs/WORKFLOW.md`
- Bug fixes → Update `REMAINING_ISSUES.md` or close issue

**Documentation Standards:**
- Use Markdown
- Include code examples
- Add diagrams for complex systems (use Mermaid)
- Keep examples runnable and tested

### Auto-Generating Docs

```bash
# Generate API docs
cargo doc --no-deps --open

# Check doc coverage
cargo +nightly rustdoc -- -Z unstable-options --show-coverage
```

---

## Common Pitfalls to Avoid

### 1. Large Uncommitted Changes
**Bad:** Working for hours without committing  
**Good:** Commit every small working change

### 2. Skipping Validation
**Bad:** `git commit && git push` without testing  
**Good:** Run clippy + tests before every commit

### 3. Hardcoded Values
**Bad:** Shader paths, buffer sizes in random places  
**Good:** Constants or configuration files

### 4. Ignoring Warnings
**Bad:** "It's just a warning, I'll fix it later"  
**Good:** Fix warnings immediately (they become errors in CI)

### 5. No Error Context
**Bad:** `Result<()>` with generic errors  
**Good:** `.context("Specific failure description")`

---

## Emergency Procedures

### Broken Main Branch

```bash
# Find last good commit
git log --oneline

# Revert to it
git revert <bad-commit-hash>
git push origin main

# Or if recent
git revert HEAD
git push origin main
```

### Failed CI

```bash
# Check what failed
gh run list --limit 1

# View logs
gh run view --log

# Fix locally and force push if needed
git commit --amend
git push origin main --force-with-lease
```

### Validation Errors in Production

1. **Reproduce locally**
   ```bash
   VK_INSTANCE_LAYERS=VK_LAYER_KHRONOS_validation cargo run
   ```

2. **Isolate the issue**
   - Binary search through commits
   - Disable features until it works
   - Add targeted logging

3. **Fix and verify**
   - Fix the issue
   - Add test to prevent regression
   - Update documentation if needed

---

**Last Updated:** 2025-10-21  
**Next Review:** When major workflow changes occur
