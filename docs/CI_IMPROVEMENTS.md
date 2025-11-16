# CI Improvements - November 2025

## Summary

Updated the CI pipeline to use the default rendering pipeline (forward pass with damaged helmet scene) and ensured all quality checks pass.

## Changes Made

### 1. Fixed Clippy Warnings

All clippy warnings have been resolved with `-D warnings` (treat warnings as errors):

- **Fixed `clone_on_copy`**: Removed unnecessary `.clone()` calls on `Transform` (which implements `Copy`)
  - `src/passes/forward_simple.rs`: Changed `transform.clone()` to `*transform`
  - `src/resources/gltf_loader.rs`: Removed `.clone()` on `combined_transform`

- **Fixed unnecessary borrows**: `src/app.rs` - removed `&` from `format!()` call

- **Fixed redundant closures**: Changed `.map(|d| glam::Vec3::from_array(d))` to `.map(glam::Vec3::from_array)`

- **Fixed unnecessary returns**: Removed redundant `return` statement in event loop

- **Fixed inefficient pattern matching**: Collapsed nested `match`/`if let` in Vulkan backend

- **Fixed unnecessary casts**: Removed `as u64` cast when hashing already-u64 values

- **Fixed inefficient access**: Changed `.get(0)` to `.first()`

- **Fixed `and_then` with `Some`**: Changed `.and_then(|x| Some(y))` to `.map(|x| y)`

### 2. Updated CI Workflow

**Test Scene**: Changed from `gltf_textured.toml` to `damaged_helmet.toml`
- More complex test case (PBR materials, detailed geometry)
- Better represents real-world usage
- Already working in Vulkan backend

**CLI Arguments**:
- Removed non-existent `--pipeline` flag
- Added `--max-frames 1` for deterministic output
- Changed log level from `info` to `warn` for cleaner output

**Golden Reference Images**:
- Created `references/damaged_helmet/` directory
- Generated Vulkan golden reference
- Added README documenting the scene and update process
- DirectX reference will be added once DX backend is stable

### 3. Visual Regression Testing

**Golden Reference Comparison**:
- Changed from warning to error when golden references don't match
- This ensures visual regressions are caught immediately
- Tests will fail if output differs from known-good reference

**Backend Parity**:
- Still reports as warning (not blocking)
- Small differences expected until DX backend is fully stable

### 4. Code Quality

**Formatting**: All code now passes `cargo fmt --check`

**Linting**: All code passes `cargo clippy --all-targets --all-features -- -D warnings`

**Build**: Clean build with no errors (only expected shader compilation messages)

## Current Test Pipeline

### Default Scene: Damaged Helmet

- **Model**: glTF DamagedHelmet.glb (Khronos sample)
- **Resolution**: 1280x720
- **Features Tested**:
  - glTF model loading
  - PBR material rendering
  - Texture mapping (albedo, metallic, roughness)
  - Complex geometry
  - Forward rendering pass
  - Camera positioning
  - Lighting (ambient + directional)

### CI Jobs

1. **Build** (Ubuntu)
   - Debug and release builds
   - Artifact upload for both

2. **Test** (Ubuntu)
   - Unit tests

3. **Test Rendering - Vulkan** (Self-hosted with GPU)
   - Headless rendering
   - Screenshot capture
   - Compare against golden reference
   - **Fails if output differs from reference**

4. **Build - Windows** (Windows)
   - Debug and release builds
   - Unit tests
   - DirectX 12 headless rendering
   - Screenshot capture

5. **Visual Regression** (Ubuntu)
   - Downloads artifacts from rendering tests
   - Compares Vulkan vs DirectX (warning only)
   - Compares both against golden references (fails on mismatch)
   - Generates HTML report with FLIP comparisons
   - Uploads comparison results

6. **Clippy** (Ubuntu)
   - Strict linting with `-D warnings`

7. **Format** (Ubuntu)
   - Code formatting check

8. **Docs** (Ubuntu)
   - Documentation build with `-D warnings`

## Known Limitations

### DirectX Backend

The DirectX backend currently has issues:
- Synchronization problems in headless mode
- May not produce output or may hang
- Golden reference not yet available

**Status**: DirectX tests are allowed to fail temporarily. Once DX rendering is fixed:
1. Generate golden reference image
2. Enable strict comparison for DX

### Future Improvements

1. **Add More Test Scenes**:
   - Simple triangle (basic sanity check)
   - Textured cube (simple textured geometry)
   - Multi-object scene (instancing test)
   - Shadow mapping test

2. **Platform-Specific References**:
   - May need different references for different GPU vendors
   - Consider tolerance thresholds for platform differences

3. **Performance Benchmarking**:
   - Track frame times
   - Detect performance regressions

4. **Shader Validation**:
   - Add shader compilation tests
   - Verify SPIR-V and DXIL output

## How to Update Golden References

### When to Update

- After intentional visual improvements
- After bug fixes that correct rendering
- After backend updates that change output
- **Never** for unexplained differences (investigate first)

### Update Process

```bash
# Build release binary
cargo build --release

# Generate new Vulkan reference
./target/release/rusty_renderer \
  --scene scenes/damaged_helmet.toml \
  --backend vulkan \
  --headless \
  --max-frames 1 \
  --screenshot references/damaged_helmet/damaged_helmet_vulkan.png

# Verify the new image looks correct
# Then commit
git add references/damaged_helmet/damaged_helmet_vulkan.png
git commit -m "Update Vulkan reference: [reason for change]"
```

See `references/damaged_helmet/README.md` for full details.

## Verification

To verify CI will pass locally:

```bash
# Check formatting
cargo fmt --all -- --check

# Check linting
cargo clippy --all-targets --all-features -- -D warnings

# Build release
cargo build --release

# Test Vulkan rendering
./target/release/rusty_renderer \
  --scene scenes/damaged_helmet.toml \
  --backend vulkan \
  --headless \
  --max-frames 1 \
  --screenshot test_output.png

# Compare against golden reference (requires FLIP)
python3 scripts/flip_compare.py \
  references/damaged_helmet/damaged_helmet_vulkan.png \
  test_output.png
```

## Next Steps

1. **Fix DirectX Backend**: Resolve synchronization and rendering issues
2. **Add DirectX Golden Reference**: Once DX is stable
3. **Add More Test Scenes**: Expand test coverage
4. **Document Render Pass Architecture**: Clarify that all rendering should be driven by render pass definitions
5. **Move Hardcoded Values to Render Passes**: Clear colors, attachments, etc.
