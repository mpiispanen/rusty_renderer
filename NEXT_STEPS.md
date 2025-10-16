# Next Steps After Reboot

## Immediate Actions

### 1. Verify Validation Layers
```bash
vulkaninfo --summary | grep -i khronos
```
**Expected:** Should see `VK_LAYER_KHRONOS_validation` listed

### 2. Clean Rebuild
```bash
cd /var/home/matpii01/rusty_renderer
cargo clean
cargo build --release
```
**Expected:** Should see messages about shader compilation and validation

### 3. Run with Validation Enabled
```bash
# Debug mode automatically enables validation
cargo run --example triangle 2>&1 | tee validation_output.txt
```
**Look for:** "Validation Error" or "Validation Warning" messages

### 4. Analyze Output
Read `validation_output.txt` for Vulkan validation errors that explain the crash.

## What We're Looking For

Validation layers will tell us exactly what's wrong:
- Missing required pipeline state
- Invalid SPIR-V usage
- Incompatible features
- API misuse

## If Validation Shows Errors
Fix the issues it reports, rebuild, and test again.

## If No Validation Errors
This would indicate a driver bug. Next steps:
1. Simplify pipeline (remove optional features)
2. Try minimal shader (just output solid color)
3. Consider filing RADV bug report
4. Proceed with wgpu backend (M4) as alternative

## Files to Commit After Fix
```bash
git add build.rs DEBUGGING_STATUS.md SESSION_SUMMARY.md
git add src/backends/vulkan/mod.rs  # After cleaning up debug code
git commit -m "Fix validation layer issues and improve build system"
```

## Current Commit
Last commit: `c9cb7cb` - "Add GPU testing infrastructure and --test-duration flag"
