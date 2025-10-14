# Tests

This directory contains integration tests for the Rusty Renderer.

## Organization

- `common/` - Shared test utilities and helpers
- Individual test files for different subsystems

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

Test infrastructure will be set up in Issue #5 of Milestone 1.
