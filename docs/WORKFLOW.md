# Development Workflow

This document describes the standard development workflow for the Rusty Renderer project.

## Overview

We follow a **CI-validated** workflow where all changes must pass continuous integration checks before being considered complete.

## Standard Development Flow

### 1. Pick an Issue
- Browse open issues on GitHub
- Comment on the issue to indicate you're working on it
- Understand acceptance criteria before starting

### 2. Make Changes
- Write code following Rust best practices
- Add/update tests as needed
- Add/update documentation as needed

### 3. Local Validation (REQUIRED)
Before committing, run all checks locally:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test

# Build in release mode
cargo build --release
```

**All of these must pass before pushing.**

### 4. Commit and Push

Use descriptive commit messages:
```
<Summary line (50 chars max)>

<Blank line>

<Detailed description if needed>
<Reference to issue>

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
