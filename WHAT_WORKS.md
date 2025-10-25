# Quick Reference: What Actually Works

## ✅ VERIFIED WORKING

### Vulkan + Forward Pipeline
```bash
# Headless rendering with screenshot
cargo run --release -- \
  --backend vulkan \
  --pipeline forward \
  --scene scenes/textured_cube.toml \
  --headless \
  --screenshot output.png \
  --max-frames 1

# Interactive windowed mode
cargo run --release -- \
  --backend vulkan \
  --pipeline forward \
  --scene scenes/gltf_textured.toml
```

**Status**: ✅ Production ready, zero validation errors

## ❌ BROKEN - DO NOT USE

### Simple Pipeline
```bash
# This will fail with validation errors and device loss
cargo run --release -- \
  --pipeline simple \
  --scene scenes/triangle.toml
```

**Status**: ❌ Broken - validation errors, GPU device loss

## ⚠️ UNTESTED

### DirectX Backend
```bash
# Headless mode appears to hang/timeout
# Windowed mode may work but unverified
```

**Status**: ⚠️ Needs testing on real Windows hardware

## 📋 Working Scenes

All three scenes verified with Vulkan + forward pipeline:

1. **scenes/triangle.toml** - RGB triangle, basic test
2. **scenes/textured_cube.toml** - Lit cube with checkerboard  
3. **scenes/gltf_textured.toml** - GLTF model with texture

## 🧪 Quick Tests

```bash
# Verify Vulkan works completely
./verify_vulkan.sh

# Check screenshots
ls -lh vk_*.png
```

## 🎯 For Development

**Always use**:
- Backend: `vulkan`
- Pipeline: `forward`
- Validation: Enabled (default)

**Never use**:
- Pipeline: `simple` (broken)
- DirectX headless (untested)

## 📊 Success Criteria

A backend is "working" when it:
1. Compiles ✅
2. Runs without crashes ✅
3. Exit code 0 ✅
4. **Screenshot exists** ✅
5. **Screenshot not black** ✅
6. **File size > 10 KB** ✅
7. **Zero validation errors** ✅

---

**Last Updated**: 2025-10-25 13:50 UTC  
**Status**: Vulkan proven, ready for feature development
