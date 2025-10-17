# Testing DirectX 12 on Linux via Proton/VKD3D

This guide explains how to test the DirectX 12 backend on Linux using Proton's VKD3D-Proton translation layer, which translates DirectX 12 API calls to Vulkan.

## Why This Works

- **VKD3D-Proton**: Ships with Proton, translates D3D12 → Vulkan
- **Your System**: Bazzite includes Proton out of the box
- **Result**: Real DirectX 12 code running on Linux!

## Prerequisites

Your Bazzite system already has:
- ✅ Proton 9.0 (Beta) or Experimental
- ✅ VKD3D-Proton (included with Proton)
- ✅ Vulkan drivers

You need to install:
- Windows Rust target
- MinGW-w64 cross-compiler

## Setup (One-Time)

### 1. Install Windows Rust Target

```bash
rustup target add x86_64-pc-windows-gnu
```

### 2. Install MinGW Cross-Compiler

```bash
# On Fedora/Bazzite (rpm-ostree)
rpm-ostree install mingw64-gcc mingw64-winpthreads-static

# Reboot after installation
sudo systemctl reboot

# Alternative: Use toolbox if you don't want to layer packages
toolbox create crosscompile
toolbox enter crosscompile
sudo dnf install mingw64-gcc mingw64-winpthreads-static
```

### 3. Configure Cargo for Cross-Compilation

Create or edit `.cargo/config.toml` in your project:

```toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
ar = "x86_64-w64-mingw32-ar"
```

## Building for Windows

### Debug Build

```bash
cargo build --target x86_64-pc-windows-gnu
```

The binary will be at:
```
target/x86_64-pc-windows-gnu/debug/rusty_renderer.exe
```

### Release Build

```bash
cargo build --target x86_64-pc-windows-gnu --release
```

The binary will be at:
```
target/x86_64-pc-windows-gnu/release/rusty_renderer.exe
```

## Running via Proton

### Method 1: Direct Proton Run

```bash
# Using Proton 9.0
~/.local/share/Steam/steamapps/common/"Proton 9.0 (Beta)"/proton run \
    target/x86_64-pc-windows-gnu/release/rusty_renderer.exe -- --backend directx

# Using Proton Experimental  
~/.local/share/Steam/steamapps/common/"Proton - Experimental"/proton run \
    target/x86_64-pc-windows-gnu/release/rusty_renderer.exe -- --backend directx
```

### Method 2: Using the Helper Script

We provide a helper script for easier testing:

```bash
# Make it executable
chmod +x scripts/test_dx12_proton.sh

# Run with default settings
./scripts/test_dx12_proton.sh

# Run with debug output
./scripts/test_dx12_proton.sh --debug

# Use specific Proton version
./scripts/test_dx12_proton.sh --proton experimental

# Build in release mode
./scripts/test_dx12_proton.sh --release
```

## Environment Variables

### VKD3D Debug Output

```bash
export VKD3D_DEBUG=warn          # Show warnings
export VKD3D_DEBUG=info          # Show info messages
export VKD3D_DEBUG=trace         # Show everything (verbose!)
export VKD3D_SHADER_DEBUG=trace  # Shader compilation debug
```

### Force Software Rendering (WARP equivalent)

While VKD3D doesn't have direct WARP support, you can use:

```bash
# Use llvmpipe software renderer
export VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.x86_64.json
```

### Wine/Proton Debug

```bash
export WINEDEBUG=warn+d3d12,warn+dxgi  # D3D12 and DXGI debug
export WINEDEBUG=-all                  # Disable all debug (faster)
export PROTON_LOG=1                    # Enable Proton logging
```

## Troubleshooting

### Issue: "x86_64-w64-mingw32-gcc: command not found"

**Solution:** Install mingw64-gcc (see Setup step 2)

### Issue: "error: linker `x86_64-w64-mingw32-gcc` not found"

**Solution:** Check `.cargo/config.toml` and ensure mingw is in PATH

### Issue: Window doesn't appear

**Possible causes:**
1. DirectX backend not fully implemented (rendering pipeline incomplete)
2. VKD3D translation issue
3. Winit window creation issue

**Debug steps:**
```bash
# Enable debug output
export VKD3D_DEBUG=warn
export WINEDEBUG=warn+d3d12
export RUST_LOG=debug

# Run and check logs
./scripts/test_dx12_proton.sh --debug 2>&1 | tee dx12_test.log
```

### Issue: "Failed to create D3D12 device"

**Possible causes:**
1. VKD3D-Proton not found/working
2. Vulkan drivers not available
3. DirectX backend initialization issue

**Verify Vulkan works:**
```bash
vulkaninfo | grep "deviceName"
```

**Check VKD3D in Proton:**
```bash
ls ~/.local/share/Steam/steamapps/common/"Proton 9.0 (Beta)"/files/lib64/vkd3d*/
```

### Issue: Building fails with Windows-specific errors

**Solution:** Use conditional compilation - our code already supports this!

Check that `src/backends/directx/mod.rs` has:
```rust
#[cfg(windows)]
mod dx12_impl;
```

## Testing Workflow

### Full Testing Cycle

1. **Test Vulkan (native):**
   ```bash
   cargo run --release
   ```

2. **Test wgpu (native):**
   ```bash
   cargo run --release -- --backend wgpu
   ```

3. **Build for Windows:**
   ```bash
   cargo build --target x86_64-pc-windows-gnu --release
   ```

4. **Test DirectX via Proton:**
   ```bash
   ./scripts/test_dx12_proton.sh --release
   ```

### Quick Iteration for DirectX Development

```bash
# Terminal 1: Watch for changes and auto-build
cargo watch -x "build --target x86_64-pc-windows-gnu"

# Terminal 2: Run tests
./scripts/test_dx12_proton.sh
```

## Performance Considerations

**Translation Overhead:**
- VKD3D-Proton adds ~5-10% overhead
- Still useful for testing correctness
- Real Windows testing via CI is authoritative

**When to Use Each Method:**
- **Development/Testing**: Proton on Linux (fast iteration)
- **Verification**: CI on real Windows (authoritative)
- **Release**: Real Windows hardware (best performance)

## Advanced: Using Wine Directly

For more control, you can use Wine instead of Proton:

```bash
# Create wine prefix
export WINEPREFIX=~/rusty_wine
wineboot --init

# Install VKD3D-Proton manually (optional, for latest version)
# Download from https://github.com/HansKristian-Work/vkd3d-proton/releases

# Run
export WINEPREFIX=~/rusty_wine
wine target/x86_64-pc-windows-gnu/release/rusty_renderer.exe -- --backend directx
```

## CI vs Local Testing

| Aspect | Local (Proton) | CI (Windows) |
|--------|---------------|--------------|
| Speed | Fast (no push) | Slow (~5 min) |
| Accuracy | Good (translation) | Perfect (native) |
| Visual | Yes (can see window) | No (headless) |
| Debug | Easy (local logs) | Harder (remote logs) |
| Cost | Free | Free (GitHub) |

**Recommendation:** Use both!
- Local Proton for quick iteration
- CI for final verification

## What You're Actually Testing

When running via Proton/VKD3D:

1. **Your DirectX code**: Real `ID3D12Device` calls
2. **VKD3D-Proton**: Translates D3D12 → Vulkan calls
3. **Vulkan driver**: Renders using your GPU
4. **Result**: Validates DirectX API usage + rendering correctness

This is **very valuable** because:
- Catches D3D12 API misuse
- Validates rendering logic
- Faster than pushing to CI
- Can see visual output

## Example Session

```bash
# Setup (once)
rustup target add x86_64-pc-windows-gnu
rpm-ostree install mingw64-gcc mingw64-winpthreads-static
sudo systemctl reboot

# Build and test
cd ~/rusty_renderer
cargo build --target x86_64-pc-windows-gnu --release

# Run with debug output
export VKD3D_DEBUG=warn
export RUST_LOG=info
./scripts/test_dx12_proton.sh --release --debug

# Compare with native Vulkan
cargo run --release

# All backends working? 🎉
```

## Next Steps

Once this is working:
1. Compare rendering output: DirectX (via Proton) vs Vulkan (native)
2. Profile performance overhead
3. Test more complex rendering scenarios
4. Add automated screenshot comparison tests

---

**This setup gives you a complete cross-platform development environment!** You can develop and test all three backends (Vulkan, wgpu, DirectX 12) entirely on Linux. 🚀
