# Tests

This directory contains integration tests for the Rusty Renderer.

## Organization

- `common/` - Shared test utilities and helpers
- `config_test.rs` - Configuration module integration tests
- `backend_test.rs` - Backend module integration tests (expanded in M2+)

## Running Tests

```bash
# Run all tests (unit + integration)
cargo test

# Run only integration tests
cargo test --test '*'

# Run specific integration test file
cargo test --test config_test

# Run specific test
cargo test test_config_validation_valid

# Run with output
cargo test -- --nocapture

# Run with debug logging
RUST_LOG=debug cargo test -- --nocapture
```

## Writing Tests

### Integration Tests

Integration tests should:
1. Import `mod common;` to access test utilities
2. Call `common::setup_test_env()` at the start of each test
3. Test module integration, not internal implementation details
4. Use descriptive test names with `test_` prefix

Example:
```rust
mod common;

#[test]
fn test_my_feature() {
    common::setup_test_env();
    
    let config = common::default_test_config();
    // Test your feature
}
```

### Unit Tests

Unit tests are located in the same file as the code they test, in a `#[cfg(test)]` module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Test implementation
    }
}
```

## Test Utilities

The `common` module provides:
- `setup_test_env()` - Initialize logging for tests
- `test_config(backend, width, height)` - Create custom test config
- `default_test_config()` - Create default test config (Vulkan, 800x600)

## Future Additions

- Graphics backend tests (M3+) - Will require headless rendering or mock backends
- Render graph tests (M5)
- Performance benchmarks
- Property-based tests for complex systems

## CI/CD Integration

All tests are run automatically in CI pipeline on every push and PR.
See `.github/workflows/ci.yml` for configuration.

