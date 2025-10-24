# Quick Reference - Build & Test Commands

## Build Commands

### Linux (Vulkan)
```bash
# Debug build
cargo build

# Release build
cargo build --release

# Build specific example
cargo build --example render_graph_triangle
```

### Windows Cross-Compile (from Linux)
```bash
# Install cross-compiler (one time)
cargo install cargo-xwin --version 0.19.2

# Build for Windows
cargo xwin build --release --target x86_64-pc-windows-msvc

# Build specific example for Windows
cargo xwin build --release --target x86_64-pc-windows-msvc --example render_graph_triangle
```

## Run Commands

### Vulkan Backend (Linux)
```bash
# Triangle example (windowed)
cargo run --example render_graph_triangle vulkan

# Triangle example (headless)
cargo run --example render_graph_triangle --headless vulkan

# With logging
RUST_LOG=info cargo run --example render_graph_triangle vulkan
RUST_LOG=debug cargo run --example render_graph_triangle vulkan
```

### DirectX Backend (Windows)
```cmd
:: From Windows command prompt
render_graph_triangle.exe directx
render_graph_triangle.exe --headless directx

:: With logging
set RUST_LOG=info
render_graph_triangle.exe --headless directx
```

### WGPU Backend (Deferred)
```bash
# Not recommended - has bind group issues
cargo run --example render_graph_triangle wgpu
```

## Test Commands

### Run All Tests
```bash
cargo test
cargo test --release
```

### Run Specific Test
```bash
cargo test test_name
```

### With Logging
```bash
RUST_LOG=debug cargo test -- --nocapture
```

## Proton Testing (Limited)

### From windows_test/ directory
```bash
cd windows_test
./run_with_proton.sh
./test_simple.sh
```

### Manual Proton
```bash
export STEAM_COMPAT_DATA_PATH=~/.wine_rusty_renderer
export STEAM_COMPAT_CLIENT_INSTALL_PATH=~/.steam/steam
~/.steam/steam/steamapps/common/"Proton 9.0 (Beta)"/proton run ./render_graph_triangle.exe --headless directx
```

## Utility Commands

### Clean Build
```bash
cargo clean
```

### Check Without Building
```bash
cargo check
cargo check --target x86_64-pc-windows-msvc
```

### Format Code
```bash
cargo fmt
```

### Lint
```bash
cargo clippy
```

### Build Documentation
```bash
cargo doc --open
```

## File Locations

### Linux Builds
```
target/debug/examples/render_graph_triangle
target/release/examples/render_graph_triangle
```

### Windows Builds
```
target/x86_64-pc-windows-msvc/release/examples/render_graph_triangle.exe
target/x86_64-pc-windows-msvc/release/examples/test_scene_loading.exe
```

### Test Directory
```
windows_test/
├── render_graph_triangle.exe
├── test_scene_loading.exe
├── run_with_proton.sh
└── assets/, scenes/, shaders/
```

## Environment Variables

### Logging Levels
```bash
export RUST_LOG=error     # Errors only
export RUST_LOG=warn      # Warnings and errors
export RUST_LOG=info      # Info, warnings, errors
export RUST_LOG=debug     # Debug and above
export RUST_LOG=trace     # Everything
```

### Backend Selection
```bash
# Set via command line argument
cargo run --example render_graph_triangle vulkan
cargo run --example render_graph_triangle wgpu
cargo run --example render_graph_triangle directx  # Windows only
```

### Vulkan Debugging
```bash
export VK_INSTANCE_LAYERS=VK_LAYER_KHRONOS_validation
export VK_LAYER_PATH=/usr/share/vulkan/explicit_layer.d
```

## Quick Tests

### Verify Vulkan Works
```bash
RUST_LOG=info cargo run --example render_graph_triangle --headless vulkan
# Should create render_graph_triangle.png
```

### Verify Windows Build
```bash
cargo xwin build --release --target x86_64-pc-windows-msvc --example render_graph_triangle
ls -lh target/x86_64-pc-windows-msvc/release/examples/render_graph_triangle.exe
```

### Check for Issues
```bash
cargo check 2>&1 | grep -i error
cargo clippy 2>&1 | grep -i warning
```

## Troubleshooting

### Vulkan Not Found
```bash
# Install Vulkan SDK
sudo dnf install vulkan-validation-layers vulkan-tools
```

### Windows Build Fails
```bash
# Reinstall cargo-xwin
cargo install cargo-xwin --version 0.19.2 --force

# Clear cache
rm -rf .xwin-cache
```

### Proton Issues
```bash
# Clear Wine prefix
rm -rf ~/.wine_rusty_renderer

# Use different Proton version
export PROTON_PATH=~/.steam/steam/steamapps/common/"Proton - Experimental"/proton
```

## Performance Profiling

### Build with Debug Info
```bash
cargo build --release --profile release-with-debug
```

### Run with Profiler
```bash
perf record cargo run --release --example render_graph_triangle vulkan
perf report
```

## Common Workflows

### Full Clean Build
```bash
cargo clean && cargo build --release
```

### Test All Backends (Linux)
```bash
RUST_LOG=info cargo run --example render_graph_triangle --headless vulkan
# WGPU deferred due to issues
```

### Prepare for Windows Testing
```bash
cargo xwin build --release --target x86_64-pc-windows-msvc --example render_graph_triangle
cp target/x86_64-pc-windows-msvc/release/examples/render_graph_triangle.exe windows_test/
```

## CI/CD Commands

### GitHub Actions
```yaml
- run: cargo check
- run: cargo test --all-features
- run: cargo build --release
- run: cargo xwin build --target x86_64-pc-windows-msvc
```

---

**Last Updated**: 2025-10-24
