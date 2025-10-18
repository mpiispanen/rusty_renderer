# Windows Cross-Compilation on Bazzite

## Status

Cross-compilation from Bazzite (immutable Fedora-based OS) to Windows is challenging due to:

1. **Immutable filesystem**: Cannot easily install MinGW toolchain via rpm-ostree
2. **Dependency conflicts**: MinGW packages require redhat-rpm-config which conflicts with existing system packages

## What Works

- **Native Vulkan**: Works perfectly on Bazzite
- **wgpu backend**: Works perfectly (uses Vulkan on Linux, DX12/Metal/Vulkan on other platforms)
- **CI-based Windows builds**: GitHub Actions can build and test Windows binaries with DirectX 12

## Testing DirectX 12 Backend

### Option 1: GitHub Actions CI (Recommended)
Build and test Windows binaries in CI:
```yaml
- name: Build Windows (DirectX 12)
  run: cargo build --target x86_64-pc-windows-msvc --release
```

### Option 2: Use wgpu for Cross-Platform Testing
The wgpu backend provides DirectX 12 support on Windows while using Vulkan on Linux:
```bash
cargo run -- --backend wgpu
```

### Option 3: Windows Development Machine
For direct DirectX 12 development, use a Windows machine or VM.

## Cross-Compilation Attempts

Attempted setup on Bazzite:
```bash
# This requires MinGW toolchain which conflicts with system packages
rustup target add x86_64-pc-windows-gnu
cargo build --target x86_64-pc-windows-gnu
```

Error: `Error calling dlltool 'x86_64-w64-mingw32-dlltool': No such file or directory`

Installing MinGW via rpm-ostree causes dependency conflicts with redhat-rpm-config.

## Recommendation

For DirectX 12 development:
1. Use wgpu backend for local testing (works on all platforms)
2. Use GitHub Actions for Windows-specific builds and tests
3. Direct DX12 testing requires Windows environment (native or VM)

The wgpu backend provides excellent cross-platform coverage and uses native APIs (DX12 on Windows, Metal on macOS, Vulkan on Linux) under the hood.
