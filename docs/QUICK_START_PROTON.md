# Quick Start: Testing DirectX 12 on Linux

Want to test the DirectX 12 backend on Linux? Here's the fastest way to get started using Proton's VKD3D translation layer.

## Prerequisites

You need:
- Bazzite/Fedora with Proton installed (you already have this!)
- MinGW cross-compiler
- Windows Rust target

## 5-Minute Setup

```bash
# 1. Install Windows Rust target (fast)
rustup target add x86_64-pc-windows-gnu

# 2. Install MinGW cross-compiler (requires reboot)
rpm-ostree install mingw64-gcc mingw64-winpthreads-static
sudo systemctl reboot

# After reboot, you're ready!
```

## Build and Test

```bash
# Build Windows binary and run via Proton (one command!)
./scripts/test_dx12_proton.sh --release

# That's it! You're running DirectX 12 on Linux via VKD3D translation
```

## What Just Happened?

1. **Cross-compiled** your Rust code to Windows .exe
2. **Proton** loaded the .exe with Wine compatibility layer
3. **VKD3D-Proton** translated DirectX 12 calls to Vulkan
4. **Your GPU** rendered the scene via Vulkan drivers

You just ran DirectX 12 code on Linux! 🎉

## Troubleshooting

### Build fails: "x86_64-w64-mingw32-gcc: command not found"

You haven't rebooted after installing mingw. Reboot and try again.

### Window doesn't appear

The DirectX rendering pipeline is not yet complete. Check the implementation status:
- Device initialization: ✅ Done
- Swap chain: ✅ Done  
- Shader compilation: ⏳ TODO
- Pipeline creation: ⏳ TODO
- Rendering: ⏳ TODO

### Want more details?

See the full guide: [docs/TESTING_DIRECTX_ON_LINUX.md](./TESTING_DIRECTX_ON_LINUX.md)

## Comparing Backends

```bash
# Test native Vulkan
cargo run --release

# Test wgpu (native)
cargo run --release -- --backend wgpu

# Test DirectX 12 (via Proton)
./scripts/test_dx12_proton.sh --release
```

Now you can test all three backends on one machine!

## Debug Mode

Enable verbose output to see what's happening:

```bash
./scripts/test_dx12_proton.sh --debug --release
```

This shows:
- VKD3D translation messages
- D3D12/DXGI API calls
- Rust application logs
- Proton/Wine debug info

## Advanced Usage

See `./scripts/test_dx12_proton.sh --help` for all options.

---

**Full documentation:** [docs/TESTING_DIRECTX_ON_LINUX.md](./TESTING_DIRECTX_ON_LINUX.md)
