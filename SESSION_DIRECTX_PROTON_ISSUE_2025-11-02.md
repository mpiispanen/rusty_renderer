# DirectX Proton Issue - Session 2025-11-02

## Summary

Fixed the Proton run script to properly sync all required directories and support both Windows build targets, but discovered a critical issue preventing the DirectX backend from running under Proton.

## What Was Fixed

### 1. Updated `run_with_proton.sh`
- ✅ Now supports both `x86_64-pc-windows-gnu` and `x86_64-pc-windows-msvc` targets
- ✅ Automatically syncs all required directories:
  - `shaders/` - All shader files
  - `assets/` - Textures and models
  - `scenes/` - Scene definition files
- ✅ Removed invalid `--pipeline` argument that doesn't exist in Config
- ✅ Auto-creates test directory if missing
- ✅ Better error messages

### 2. Created Documentation
- ✅ `DIRECTX_PROTON_HOWTO.md` - Comprehensive guide for running DirectX with Proton
  - Prerequisites
  - Build instructions  
  - Usage examples
  - Environment variables
  - Troubleshooting
  - How it works (Proton → VKD3D → Vulkan)

### 3. Verification
- ✅ Linux Vulkan backend works correctly (tested and verified)
- ✅ Windows cross-compilation works (binary builds successfully)
- ✅ Script properly syncs and prepares environment

## Current Issue

### Problem: Argument Parsing Crash

**Symptoms:**
- Windows `.exe` exits with code 2 (or 0) immediately
- Debug log shows crash during `Config::parse()` (clap argument parser)
- Never gets past "Parsing arguments" log line
- Happens even with minimal arguments

**What We Tried:**
```bash
# All of these crash during argument parsing:
./run_with_proton.sh --scene triangle --max-frames 1 --headless
./run_with_proton.sh --scene scenes/triangle.toml --max-frames 1
./run_with_proton.sh --backend directx --headless --max-frames 1

# Even minimal arguments crash:
rusty_renderer.exe --backend directx --headless --max-frames 1
```

**Debug Evidence:**
```
# rusty_renderer_debug.log always shows:
Starting Rusty Renderer v0.1.0
Initializing logging
Parsing arguments
# <crashes here, never reaches "Arguments parsed successfully">
```

**Root Cause (Suspected):**
The `clap` argument parser library may have compatibility issues when running under Wine/Proton. The crash happens during `Config::parse()` which calls clap's `Parser::parse()`.

**Why This Matters:**
- Can't test DirectX backend on Linux via Proton
- Cross-compilation works, but runtime fails
- Native Windows might work (untested)

## Next Steps

### Option 1: Investigate clap Issue
- Test with older/newer clap versions
- Try clap derive vs builder API
- Check clap Windows compatibility issues

### Option 2: Alternative Argument Parsing
- Switch to a different argument parser (e.g., `structopt`, `argh`, or manual parsing)
- This is a significant refactor

### Option 3: Native Windows Testing
- Test the `.exe` on actual Windows hardware
- If it works there, document Proton as unsupported for now
- Focus on native Windows development environment

### Option 4: Workaround for CI
- Use Vulkan backend for Linux CI testing
- Add native Windows CI runner for DirectX testing
- Keep Proton as "nice to have" but not required

## Recommendation

**Short term**: Option 4 - Use Vulkan for Linux CI, add Windows runner for DirectX  
**Long term**: Option 3 - Verify on native Windows, Option 1 - investigate clap if needed

The current implementation works for:
- ✅ Vulkan backend on Linux (tested, verified)
- ✅ Cross-compilation to Windows (builds successfully)
- ❓ DirectX on Windows (needs native testing)
- ❌ DirectX via Proton (clap parsing crash)

## Files Modified

1. `run_with_proton.sh` - Fixed and improved
2. `DIRECTX_PROTON_HOWTO.md` - New comprehensive documentation

## Status

- Script infrastructure: **COMPLETE**
- Documentation: **COMPLETE**
- Vulkan backend: **WORKS**
- DirectX via Proton: **BLOCKED** (clap issue)
- DirectX on Windows: **UNTESTED**
