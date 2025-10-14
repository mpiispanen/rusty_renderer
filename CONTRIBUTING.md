# Contributing to Rusty Renderer

Thank you for your interest in contributing! This document provides guidelines for contributing to the project.

## Getting Started

1. **Read the Documentation**
   - [README.md](README.md) - Project overview
   - [docs/DESIGN.md](docs/DESIGN.md) - Architecture and design
   - [docs/WORKFLOW.md](docs/WORKFLOW.md) - **Development workflow** (start here!)
   - [docs/MILESTONES.md](docs/MILESTONES.md) - Project roadmap

2. **Set Up Your Environment**
   - Install Rust (1.70+): https://rustup.rs
   - Install development tools:
     ```bash
     rustup component add rustfmt clippy
     ```
   - Clone and build:
     ```bash
     git clone https://github.com/mpiispanen/rusty_renderer.git
     cd rusty_renderer
     cargo build
     cargo test
     ```

## Before You Start

### Check Existing Work
- Browse [open issues](https://github.com/mpiispanen/rusty_renderer/issues)
- Check [milestones](docs/MILESTONES.md) for planned work
- Comment on an issue to claim it

### Understand Requirements
- Read issue description and acceptance criteria
- Ask questions in the issue if unclear
- Understand how your change fits the architecture

## Development Process

**⚠️ CRITICAL: Follow the workflow in [docs/WORKFLOW.md](docs/WORKFLOW.md)**

The key points:

### 1. Run Local Checks (BEFORE pushing)
```bash
# Must all pass:
cargo fmt --check          # Code formatting
cargo clippy --all-targets --all-features -- -D warnings  # Linting
cargo test                 # All tests
cargo build --release      # Release build
```

### 2. Commit and Push
```bash
git add <files>
git commit -m "Descriptive message

Resolves #<issue-number>"
git push origin main
```

### 3. Wait for CI ✅
```bash
# Watch CI run - MUST complete before closing issue
gh run watch

# All jobs must pass:
# ✓ Build
# ✓ Test
# ✓ Clippy
# ✓ Format
# ✓ Documentation
```

### 4. Only Then Close Issue
```bash
# After CI is green:
gh issue close <number> --comment "✅ Brief summary

CI passed: <link>"
```

## Code Standards

### Rust Style
- Follow official [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting (enforced by CI)
- Address all `cargo clippy` warnings (enforced by CI)
- Write idiomatic Rust code

### Documentation
```rust
//! Module-level documentation

/// Function documentation with examples
///
/// # Examples
/// ```
/// let result = my_function();
/// assert_eq!(result, expected);
/// ```
pub fn my_function() { }
```

### Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Arrange
        let input = setup_test();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

### Commit Messages
```
<type>: <subject>

<body>

Resolves #<issue>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `refactor`: Code refactoring
- `test`: Test additions/changes
- `chore`: Build process, tooling changes

## Testing Guidelines

### Required Tests
- **Unit tests**: Test individual functions/modules
- **Integration tests**: Test module interactions
- **All tests must pass**: No exceptions

### Test Organization
- Unit tests: In same file as code, in `#[cfg(test)] mod tests`
- Integration tests: In `tests/` directory
- Use `tests/common/` for shared test utilities

### Running Tests
```bash
# All tests
cargo test

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture

# With logging
RUST_LOG=debug cargo test -- --nocapture
```

## Pull Request Process

Currently using **trunk-based development** (direct to main):
- All commits go directly to main
- CI validates every commit
- Issues must be small, focused changes

In the future, we may use:
1. Fork the repository
2. Create a feature branch
3. Make changes, following workflow
4. Submit PR when CI passes
5. Address review comments
6. Merge after approval

## CI/CD

All changes are automatically tested via GitHub Actions:
- **Build**: Release build must succeed
- **Test**: All tests must pass
- **Clippy**: No warnings allowed
- **Format**: Must match `cargo fmt`
- **Docs**: Documentation must build

**You must wait for CI to pass before closing issues.**

See the [Workflow Guide](docs/WORKFLOW.md) for details on monitoring CI.

## Common Pitfalls

### ❌ Don't Do This
1. Close issue before CI completes
2. Push without running local checks
3. Ignore clippy warnings
4. Skip writing tests
5. Leave TODO comments without issues

### ✅ Do This Instead
1. Wait for green CI, then close issue
2. Run full validation locally first
3. Fix all clippy warnings
4. Write tests for all changes
5. Create issues for future work

## Getting Help

- **Questions**: Ask in the issue you're working on
- **CI Failures**: Check `gh run view --log`
- **Local Issues**: Run with `RUST_LOG=debug`
- **Architecture**: See [docs/DESIGN.md](docs/DESIGN.md)
- **Workflow**: See [docs/WORKFLOW.md](docs/WORKFLOW.md)

## Code of Conduct

- Be respectful and professional
- Focus on the code, not the person
- Provide constructive feedback
- Help others learn and improve
- Assume good intentions

## Learning Resources

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/)
- [Graphics Programming Resources](docs/DESIGN.md#references)

## Questions?

Feel free to:
- Open an issue with your question
- Comment on existing issues
- Check the documentation in `docs/`

---

**Remember**: Code is not done until CI is green! ✅
