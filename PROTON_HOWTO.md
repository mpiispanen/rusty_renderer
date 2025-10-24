# Running rusty_renderer DirectX Backend with Proton

This guide shows how to run the Windows DirectX backend on Linux using Proton.

## Prerequisites

1. **Steam** installed with Proton
2. **Windows cross-compilation target** installed:
   ```bash
   rustup target add x86_64-pc-windows-msvc
   ```
3. **xwin** for Windows SDK (see build.rs)

## Quick Start

### 1. Build the Windows Binary

```bash
cargo build --release --target x86_64-pc-windows-msvc
```

### 2. Copy to Test Directory

```bash
mkdir -p windows_test_directx
cp target/x86_64-pc-windows-msvc/release/rusty_renderer.exe windows_test_directx/
cp -r assets scenes windows_test_directx/
```

### 3. Run with the Helper Script

```bash
# Run with textured cube (default)
./run_with_proton.sh

# Run with triangle scene
./run_with_proton.sh scenes/triangle.toml

# Run with more verbose VKD3D debug output
./run_with_proton.sh scenes/textured_cube.toml info
```

## Manual Proton Commands

If you prefer to run Proton manually:

```bash
cd windows_test_directx

# Set up environment
PROTON_DIR="$HOME/.local/share/Steam/steamapps/common/Proton 9.0 (Beta)"
COMPAT_DATA="$HOME/.proton_rusty_renderer"

# Run with DirectX
STEAM_COMPAT_CLIENT_INSTALL_PATH="$HOME/.local/share/Steam" \
STEAM_COMPAT_DATA_PATH="$COMPAT_DATA" \
VKD3D_DEBUG=warn \
"$PROTON_DIR/proton" run rusty_renderer.exe --backend directx --scene scenes/textured_cube.toml
```

## VKD3D Debug Levels

Control VKD3D-Proton logging with `VKD3D_DEBUG`:
- `none` - No debug output (fastest)
- `err` - Errors only
- `warn` - Warnings and errors (recommended for testing)
- `info` - Informational messages, warnings, and errors
- `trace` - Everything (very verbose)

## What's Happening Under the Hood

When you run with Proton:

1. **Wine/Proton** provides Windows API compatibility
2. **VKD3D-Proton** translates DirectX 12 calls to Vulkan
3. Your Linux GPU drivers execute the Vulkan commands

This means:
- ✅ No Windows required
- ✅ DirectX 12 API → Vulkan translation
- ✅ Native performance (Vulkan on your GPU)
- ✅ DX Ultimate features supported (Shader Model 6.8, DXR 1.1, etc.)

## Troubleshooting

### Proton Not Found

If you get "Proton not found", check available versions:
```bash
ls -1 "$HOME/.local/share/Steam/steamapps/common/" | grep -i proton
```

Then update the `PROTON_DIR` variable in `run_with_proton.sh`.

### Binary Not Found

Make sure you've built and copied the Windows binary:
```bash
cargo build --release --target x86_64-pc-windows-msvc
mkdir -p windows_test_directx
cp target/x86_64-pc-windows-msvc/release/rusty_renderer.exe windows_test_directx/
```

### Scene File Not Found

Ensure scenes and assets are copied:
```bash
cp -r assets scenes windows_test_directx/
```

## Performance Notes

- VKD3D-Proton caches compiled shaders in `vkd3d-proton.cache`
- First run may be slower as shaders are compiled
- Subsequent runs use cached shaders for better performance
- The translation overhead is minimal (~1-5%)

## Expected Output

Successful runs will show:
- Wine/fsync initialization
- VKD3D-Proton version and config
- DX Ultimate support confirmation
- Shader Model 6.8 support
- Exit code: 0

## Testing Both Backends

Compare DirectX (via Proton) vs native Vulkan:

```bash
# DirectX via Proton
./run_with_proton.sh scenes/textured_cube.toml

# Native Vulkan (on Linux)
cargo run --release -- --backend vulkan --scene scenes/textured_cube.toml
```

Both should produce identical results, as VKD3D-Proton translates DirectX to Vulkan.
