# wgpu Backend Investigation - Oct 25, 2025

## Issue Found & Partially Fixed

### Root Cause of Bind Group Error ✅ FIXED
The bind group validation error was caused by **legacy hardcoded rendering in `end_frame()`**:

```rust
// OLD CODE (BROKEN)
fn end_frame(&mut self) -> Result<()> {
    // ... create render pass ...
    render_pass.set_pipeline(pipeline);
    render_pass.draw(0..3, 0..1); // ❌ NO BIND GROUPS SET!
    // ...
}
```

This was drawing 3 vertices (hardcoded triangle) without setting any bind groups, causing:
```
wgpu error: The current set RenderPipeline expects a BindGroup to be set at index 0
```

### Fix Applied
Removed all rendering logic from `end_frame()`, same as DirectX fix:

```rust
// NEW CODE (FIXED)
fn end_frame(&mut self) -> Result<()> {
    // The render graph has already handled all rendering in execute_graph()
    // No additional work needed here
    Ok(())
}
```

## Remaining Issues ❌

### 1. Initialization Hang
- Forward pipeline with glTF scene hangs during initialization or early execution
- Does not produce error message
- Timeout after 10+ seconds
- Root cause unknown

### 2. Simple Pipeline Incompatibility  
- Simple pipeline has empty `prepare()` method (no uniform buffers)
- wgpu's `finalize()` requires at least 2 uniform buffers to create bind groups
- Mismatch between pipeline expectations and backend requirements

## Testing Results

| Test Case | Result | Notes |
|-----------|--------|-------|
| `--list-scenes` | ✅ Works | No rendering, just initialization |
| Triangle + Simple Pipeline | ❌ Bind group error | Fixed in code, but simple pipeline incompatible |
| glTF + Forward Pipeline | ❌ Hangs | Initialization or early execution issue |

## Recommendation

**Keep wgpu deferred** for Phase 2 or later:

**Reasons:**
1. Vulkan + DirectX cover all platforms (via Proton)
2. Multiple unresolved issues remain
3. Would slow down Phase 1 (Backend Parity) progress
4. Architectural changes may be needed (pipeline/backend matching)

**When to revisit:**
- After Phase 3 (Pipeline Templates) - proper pipeline/backend matching
- When web deployment becomes a priority
- When macOS native support is needed

## Files Modified

- `src/backends/wgpu_backend/mod.rs` - Removed legacy `end_frame()` rendering

## Related Issues

- Same fix as DirectX #71
- Part of architecture cleanup removing hardcoded rendering

---

**Status:** Bind group error fixed, but initialization hang remains  
**Action:** Defer to Phase 2+, focus on Vulkan/DirectX parity
