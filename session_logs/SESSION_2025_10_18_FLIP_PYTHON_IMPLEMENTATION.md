# Session: FLIP Python Implementation

**Date:** October 18, 2025  
**Focus:** Continued implementation of Python version of FLIP for image testing

## Summary

Successfully implemented a comprehensive Python API integration for NVIDIA FLIP perceptual image comparison, providing dual-method support (CLI and Python API) for visual regression testing.

## Implemented Features

### 1. Python FLIP Script (`scripts/flip_compare.py`)

Created a standalone Python script that provides direct access to the FLIP Python API:

**Features:**
- Direct API access via `flip_evaluator` module (bypasses CLI)
- JSON output for reliable parsing from Rust
- Error map generation with magma colormap
- Flexible parameters (PPD, verbosity, output paths)
- Exit codes for CI/CD integration (0=pass, 1=fail, 2=error)

**Usage:**
```bash
python3 scripts/flip_compare.py reference.png test.png \
    --ppd 67 \
    --error-map diff.png \
    --output results.json \
    --verbosity 2
```

**Output Format:**
```json
{
  "mean": 0.081237,
  "median": 0.001462,
  "q1": 0.001462,
  "q3": 0.001462,
  "min": 0.001462,
  "max": 0.997351,
  "ppd": 67.0,
  "dynamic_range": "LDR",
  "reference": "reference.png",
  "test": "test.png",
  "error_map": "error_map.png"
}
```

### 2. Batch Comparison Script (`scripts/batch_flip_compare.sh`)

Shell script for comparing multiple image pairs in directories:

**Features:**
- Automatic matching of test images to reference images
- JSON results and error maps for each comparison
- Summary statistics (total/passed/failed)
- Configurable threshold (default: 0.15)

**Usage:**
```bash
./scripts/batch_flip_compare.sh reference_dir/ test_dir/ output_dir/
```

### 3. Enhanced Rust FlipComparator

Updated `src/testing/flip.rs` with dual-method support:

**Changes:**
- Added `use_python_api` flag to `FlipComparator`
- New constructor: `FlipComparator::with_python_api()`
- Split `compare()` into `compare_with_cli()` and `compare_with_python_api()`
- JSON parsing method for Python API results
- Updated `FlipResult` struct (median instead of weighted_median for consistency)
- Added `weighted_median()` compatibility method

**Architecture:**
```rust
// Method 1: CLI (original)
let flip = FlipComparator::default();
let result = flip.compare("ref.png", "test.png")?;

// Method 2: Python API (new)
let flip = FlipComparator::with_python_api(None, 2);
let result = flip.compare("ref.png", "test.png")?;
```

### 4. New Tests

Added comprehensive test coverage in `tests/visual_tests.rs`:

1. **`test_vulkan_vs_wgpu_flip`** - Original CLI method test
2. **`test_vulkan_vs_wgpu_flip_python_api`** - Python API method test
3. **`test_flip_comparison_methods`** - Validates both methods produce identical results

All tests pass with mean error ~0.081 (well below 0.15 threshold).

### 5. Documentation

Created and updated multiple documentation files:

**New Documentation:**
- `docs/FLIP_INTEGRATION.md` - Comprehensive FLIP integration guide
  - Installation instructions
  - Method comparison (CLI vs Python API)
  - Threshold interpretation
  - Usage examples
  - Troubleshooting
  - CI/CD integration

**Updated Documentation:**
- `src/testing/README.md` - Added FLIP section with both methods
- `scripts/README.md` - Documented new scripts
- `README.md` - Added visual regression testing section

### 6. Dependencies

Added to `Cargo.toml`:
```toml
serde_json = "1.0"  # For JSON parsing
```

Python requirements (documented):
```bash
pip install flip-evaluator numpy pillow
```

## Test Results

All tests pass successfully:

```
running 3 tests
test test_vulkan_vs_wgpu_flip ... ok
test test_vulkan_vs_wgpu_flip_python_api ... ok
test test_flip_comparison_methods ... ok

test result: ok. 3 passed; 0 failed; 0 ignored
```

### Comparison Results

Both methods produce identical results (difference: 0.00000035):

```
Method 1 (CLI):        Mean: 0.081237
Method 2 (Python API): Mean: 0.081237
Difference:            0.00000035
```

### Performance

- CLI method: ~200-500ms per comparison
- Python API method: ~150-300ms per comparison
- Both are fast enough for automated testing

## Files Modified

### New Files
- `scripts/flip_compare.py` - Python FLIP API wrapper
- `scripts/batch_flip_compare.sh` - Batch comparison script
- `docs/FLIP_INTEGRATION.md` - FLIP integration guide

### Modified Files
- `src/testing/flip.rs` - Dual-method support
- `tests/visual_tests.rs` - Additional tests
- `Cargo.toml` - Added serde_json dependency
- `src/testing/README.md` - FLIP documentation
- `scripts/README.md` - Script documentation
- `README.md` - Visual testing section

## Benefits

### Advantages of Python API Method

1. **Reliability**: JSON output eliminates text parsing issues
2. **Flexibility**: Direct numpy array access for custom processing
3. **Error Maps**: Better control over error map generation
4. **Debugging**: Clearer error messages and stack traces
5. **CI/CD**: Exit codes and structured output for automation

### Backward Compatibility

The original CLI method remains fully functional, ensuring existing tests continue to work without modification.

## Usage Examples

### Rust Tests

```rust
#[test]
fn test_rendering_quality() {
    let flip = FlipComparator::with_python_api(Some(67.0), 2);
    let result = flip.compare("reference.png", "test.png")?;
    assert!(result.passes(0.10));
}
```

### Command Line

```bash
# Single comparison
python3 scripts/flip_compare.py ref.png test.png -o results.json

# Batch comparison
./scripts/batch_flip_compare.sh refs/ tests/ output/

# Rust test suite
cargo test --test visual_tests flip -- --ignored --nocapture
```

## Integration Quality

The implementation follows best practices:

- ✅ Dual-method architecture for flexibility
- ✅ JSON-based communication for reliability
- ✅ Comprehensive test coverage
- ✅ Detailed documentation
- ✅ Backward compatible with existing tests
- ✅ CI/CD ready with exit codes
- ✅ Batch processing support
- ✅ Error map generation
- ✅ Performance optimized

## Thresholds

Recommended FLIP mean error thresholds:

| Threshold | Quality | Use Case |
|-----------|---------|----------|
| < 0.05 | Excellent | Same backend, same hardware |
| < 0.10 | Good | Different backends |
| < 0.15 | Acceptable | Cross-platform |
| ≥ 0.15 | Investigate | Potential rendering issues |

## Future Enhancements

Potential improvements documented in `FLIP_INTEGRATION.md`:

1. HDR-FLIP support for tone-mapped content
2. Parallel batch processing
3. Historical FLIP error tracking
4. Custom spatial weighting for critical regions
5. Pure Rust FLIP implementation (long-term)

## References

- [NVIDIA FLIP Paper](https://research.nvidia.com/publication/2020-07_FLIP)
- [flip-evaluator PyPI](https://pypi.org/project/flip-evaluator/)
- [FLIP GitHub](https://github.com/NVlabs/flip)

## Status

✅ **Complete** - Python FLIP implementation successfully integrated with:
- Dual-method support (CLI + Python API)
- Comprehensive test coverage
- Full documentation
- Batch processing scripts
- Ready for production use
