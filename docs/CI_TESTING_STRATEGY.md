# CI Testing Strategy

## Overview

Our CI pipeline tests both Vulkan and DirectX 12 backends across multiple platforms and configurations to ensure backend parity and prevent regressions.

## Test Configurations

### 1. Vulkan Testing (Linux + GPU)

- **Platform**: Self-hosted Linux runner with GPU
- **Configuration**: Native Vulkan on real GPU hardware
- **Purpose**: Primary rendering validation with hardware acceleration
- **Job**: `test-rendering-vulkan`

### 2. DirectX Testing - WARP (Windows Software Renderer)

- **Platform**: GitHub-hosted Windows runner
- **Configuration**: DirectX 12 with WARP software renderer
- **Purpose**: Validate DirectX backend without requiring GPU hardware
- **Environment**: `RUSTY_RENDERER_USE_WARP=1`
- **Job**: `build-windows` → DirectX test step

**Why WARP?**
- All GitHub Windows runners have WARP available
- Provides consistent, deterministic rendering
- No GPU hardware required
- Perfect for CI validation

### 3. DirectX Testing - Proton (Linux + GPU)

- **Platform**: Self-hosted Linux runner with GPU
- **Configuration**: DirectX 12 via VKD3D-Proton translation layer
- **Purpose**: Validate DirectX backend on real GPU hardware through Proton
- **Cross-compilation**: Windows binary built with `x86_64-pc-windows-gnu` target
- **Job**: `test-rendering-directx-proton`

**Why Proton?**
- Tests DirectX backend with real GPU
- Validates VKD3D-Proton compatibility (important for Linux gaming)
- Uses same GPU as Vulkan tests for fair comparison
- Critical for Steam Deck and Linux gaming support

## Visual Regression Testing

After all rendering tests complete, the `visual-regression` job compares outputs:

### Backend Parity Checks

1. **Vulkan vs DirectX (WARP)**
   - Compares native Vulkan against DirectX software renderer
   - Some differences expected due to different implementations

2. **Vulkan vs DirectX (Proton)**
   - Compares native Vulkan against DirectX via VKD3D translation
   - Most important for gaming compatibility
   - Should be very similar when using same GPU

3. **DirectX WARP vs DirectX Proton** ⚠️
   - Compares software renderer against GPU-accelerated DirectX
   - Should produce identical results (same DirectX backend)
   - **Failure indicates a bug in our DirectX implementation**

### Golden Reference Checks

- Compares current outputs against reference images
- Fails CI if rendering regresses
- Located in `references/damaged_helmet/`

## Artifacts

Each test configuration uploads screenshots:

- `screenshots-vulkan` - Native Vulkan rendering
- `screenshots-directx` - DirectX via WARP
- `screenshots-directx-proton` - DirectX via Proton/VKD3D
- `visual-regression-results` - Comparison images and HTML report

## Running Tests Locally

### Test Vulkan (Linux)
```bash
cargo run --release -- \
  --scene scenes/damaged_helmet.toml \
  --backend vulkan \
  --headless \
  --max-frames 1 \
  --screenshot vulkan_test.png
```

### Test DirectX with WARP (Windows)
```powershell
$env:RUSTY_RENDERER_USE_WARP = "1"
.\target\release\rusty_renderer.exe `
  --scene scenes\damaged_helmet.toml `
  --backend directx `
  --headless `
  --max-frames 1 `
  --screenshot directx_warp_test.png
```

### Test DirectX via Proton (Linux)
```bash
./scripts/test_dx12_proton.sh --release
```

Or manually:
```bash
# Build Windows binary
cargo build --release --target x86_64-pc-windows-gnu

# Run via Proton
export STEAM_COMPAT_CLIENT_INSTALL_PATH="$HOME/.local/share/Steam"
export STEAM_COMPAT_DATA_PATH="$HOME/.proton_rusty_renderer"
export WINEDEBUG="-all"

PROTON="$HOME/.local/share/Steam/steamapps/common/Proton 9.0 (Beta)/proton"
$PROTON run target/x86_64-pc-windows-gnu/release/rusty_renderer.exe \
  --scene scenes/damaged_helmet.toml \
  --backend directx \
  --headless \
  --max-frames 1 \
  --screenshot directx_proton_test.png
```

## Benefits of This Approach

1. **Comprehensive Coverage**: Tests both backends on real hardware and software renderers
2. **Platform Validation**: Ensures DirectX works on both Windows and Linux (via Proton)
3. **No GPU Required for Basic CI**: WARP allows DirectX testing on standard GitHub runners
4. **Gaming Support**: Proton testing validates Linux gaming compatibility
5. **Backend Parity**: Catches rendering differences early
6. **Regression Prevention**: Golden references prevent quality degradation

## Future Improvements

- [ ] Add macOS + Metal backend testing
- [ ] Test multiple GPU vendors (AMD, NVIDIA, Intel)
- [ ] Add performance benchmarks
- [ ] Test different WARP quality levels
- [ ] Add ray tracing validation
- [ ] Test compute shader workloads
