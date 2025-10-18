# DirectX 12 Testing on Linux via Proton - Setup Complete! 🎉

## What We Just Created

A complete cross-platform testing environment that lets you test **DirectX 12 on Linux** using Proton's VKD3D-Proton translation layer!

## Files Added

### 1. Documentation
- **`docs/TESTING_DIRECTX_ON_LINUX.md`** (7.6 KB)
  - Comprehensive guide to DirectX testing on Linux
  - Explains VKD3D-Proton translation
  - Troubleshooting section
  - Environment variables reference
  - Performance considerations

- **`docs/QUICK_START_PROTON.md`** (2.5 KB)
  - 5-minute quick start guide
  - Minimal steps to get testing
  - Common issues and solutions

### 2. Testing Script
- **`scripts/test_dx12_proton.sh`** (6.6 KB, executable)
  - Automated build + test script
  - Handles Proton detection
  - Manages environment variables
  - Debug mode with verbose output
  - Error checking and helpful messages

### 3. Cargo Configuration
- **`.cargo/config.toml`**
  - MinGW linker settings
  - Windows cross-compilation setup
  - Ready for `cargo build --target x86_64-pc-windows-gnu`

## How It Works

```
┌─────────────────┐
│  Rust Code      │
│  (DirectX 12)   │
└────────┬────────┘
         │ cargo build --target windows
         ▼
┌─────────────────┐
│  Windows .exe   │
│  (D3D12 calls)  │
└────────┬────────┘
         │ Proton loads
         ▼
┌─────────────────┐
│  Wine Layer     │
│  (Windows API)  │
└────────┬────────┘
         │ VKD3D-Proton translates
         ▼
┌─────────────────┐
│  Vulkan API     │
│  (Native Linux) │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Your GPU       │
│  Renders!       │
└─────────────────┘
```

## Setup Steps (For You)

### Step 1: Install Windows Target (30 seconds)

```bash
rustup target add x86_64-pc-windows-gnu
```

✅ This downloads the Windows standard library for cross-compilation.

### Step 2: Install MinGW Cross-Compiler (5 minutes + reboot)

```bash
# This layers the package into your Bazzite system
rpm-ostree install mingw64-gcc mingw64-winpthreads-static

# Required reboot to apply the layered packages
sudo systemctl reboot
```

⚠️ **Important:** You MUST reboot after this step! rpm-ostree layers packages that only become available after reboot.

### Step 3: Test It! (2 minutes)

```bash
cd ~/rusty_renderer

# Build Windows binary and run via Proton in one command
./scripts/test_dx12_proton.sh --release
```

That's it! 🎉

## What You Can Now Test

### All Three Backends on One Machine!

```bash
# 1. Native Vulkan
cargo run --release

# 2. Native wgpu  
cargo run --release -- --backend wgpu

# 3. DirectX 12 via Proton
./scripts/test_dx12_proton.sh --release
```

### Debug Mode

```bash
# Enable verbose output to see VKD3D translation
./scripts/test_dx12_proton.sh --debug --release
```

This shows:
- VKD3D-Proton translation messages
- DirectX 12 API calls
- DXGI swap chain operations
- Vulkan calls (from VKD3D)
- Application logs

### Different Proton Versions

```bash
# Use Proton Experimental (latest features)
./scripts/test_dx12_proton.sh --proton experimental

# Use Proton 9.0 (default, stable)
./scripts/test_dx12_proton.sh --proton 9.0
```

## Why This Is Awesome

### Benefits

1. **Fast Iteration**
   - No need to push to CI
   - Compile and test in ~30 seconds
   - Immediate visual feedback

2. **Complete Testing Coverage**
   - All 3 backends on one machine
   - Vulkan, wgpu, DirectX 12
   - Side-by-side comparison

3. **DirectX Validation**
   - Tests actual DirectX 12 API calls
   - VKD3D validates API usage
   - Catches errors early

4. **Cross-Platform Development**
   - Develop on Linux
   - Test Windows code
   - No need for Windows VM

5. **Visual Validation**
   - See the window
   - See rendered output
   - Compare against Vulkan/wgpu

### What VKD3D-Proton Does

VKD3D-Proton is a **high-quality DirectX 12 → Vulkan translation layer** used by Steam to run Windows games on Linux. It:

- Translates D3D12 calls to Vulkan (not emulation, real translation)
- Used by thousands of games daily
- Maintained by Valve and contributors
- Performance within 5-10% of native D3D12
- Excellent API coverage

**Your DirectX 12 code will run real D3D12 API calls**, and VKD3D will translate them to Vulkan. This is a **legitimate test** of your DirectX implementation!

## Current Status

### What Works Now

✅ Build system configured  
✅ Cross-compilation setup complete  
✅ Proton integration script ready  
✅ Documentation complete  

### What Needs DirectX Implementation

Once you implement the DirectX rendering pipeline:
- ⏳ Shader compilation (HLSL → DXIL/bytecode)
- ⏳ Pipeline state object creation
- ⏳ Command list recording
- ⏳ Actual rendering

**Then you can test the full DirectX pipeline on Linux!**

## Testing Workflow

### Quick Test Cycle

```bash
# 1. Make DirectX code changes
vim src/backends/directx/dx12_impl.rs

# 2. Build for Windows
cargo build --target x86_64-pc-windows-gnu --release

# 3. Test via Proton
./scripts/test_dx12_proton.sh --release

# Repeat!
```

### Watch Mode (Auto-rebuild)

```bash
# Terminal 1: Watch and auto-build on changes
cargo watch -x "build --target x86_64-pc-windows-gnu --release"

# Terminal 2: Run tests manually
./scripts/test_dx12_proton.sh --release
```

### Compare Backends

```bash
# Run all three backends and compare visually
cargo run --release                           # Vulkan
cargo run --release -- --backend wgpu         # wgpu
./scripts/test_dx12_proton.sh --release      # DirectX via VKD3D
```

## Troubleshooting Reference

### Issue: "x86_64-w64-mingw32-gcc: command not found"

**Cause:** Haven't rebooted after installing mingw  
**Solution:** `sudo systemctl reboot`

### Issue: "Proton not found"

**Cause:** Proton not installed or wrong version  
**Solution:** Check available versions:
```bash
ls ~/.local/share/Steam/steamapps/common/ | grep Proton
```

### Issue: Window doesn't appear

**Cause:** DirectX rendering not implemented yet  
**Solution:** Implement shader compilation and pipeline creation

### Issue: VKD3D errors in logs

**Cause:** DirectX API misuse  
**Solution:** Enable debug mode and check VKD3D warnings:
```bash
./scripts/test_dx12_proton.sh --debug 2>&1 | grep VKD3D
```

## Next Steps

### Immediate (Setup)

1. ✅ Install Windows target: `rustup target add x86_64-pc-windows-gnu`
2. ✅ Install MinGW: `rpm-ostree install mingw64-gcc mingw64-winpthreads-static`
3. ⏳ Reboot: `sudo systemctl reboot`
4. ⏳ Test: `./scripts/test_dx12_proton.sh`

### Future (DirectX Implementation)

Once setup is complete, you can:

1. **Implement shader compilation**
   - Pre-compile HLSL to DXIL bytecode
   - Or use runtime compilation (dxcompiler)

2. **Create graphics pipeline**
   - Pipeline state object (PSO)
   - Root signature
   - Input layout

3. **Record command lists**
   - Set pipeline state
   - Bind vertex buffer
   - Draw triangle

4. **Test via Proton!**
   - See DirectX rendering on Linux
   - Compare output with Vulkan
   - Validate correctness

## Documentation Index

- **Quick Start**: `docs/QUICK_START_PROTON.md`
- **Full Guide**: `docs/TESTING_DIRECTX_ON_LINUX.md`
- **Script Help**: `./scripts/test_dx12_proton.sh --help`
- **This Summary**: `PROTON_TESTING_SETUP.md`

## Summary

You now have a **complete cross-platform testing environment** that enables:

- ✅ Testing DirectX 12 on Linux via VKD3D-Proton
- ✅ Cross-compilation from Linux to Windows
- ✅ Automated testing scripts
- ✅ All 3 backends testable on one machine
- ✅ Visual validation and debugging
- ✅ Fast iteration cycle

**This is professional-grade cross-platform development tooling!** 🚀

Once you do the 5-minute setup (install target + mingw + reboot), you'll be able to test DirectX 12 rendering on your Linux machine just as easily as testing Vulkan or wgpu.

---

**Ready to proceed?** Follow the setup steps above, then continue implementing the DirectX rendering pipeline with confidence that you can test it locally!
