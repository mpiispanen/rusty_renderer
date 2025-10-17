# Quick Start: Testing DirectX 12 on Linux

Want to test the DirectX 12 backend on Linux? Here's the fastest way to get started using Proton's VKD3D translation layer.

## For Bazzite/Immutable OS Users (RECOMMENDED)

**If you're on Bazzite, Silverblue, or Kinoite**, use Distrobox - no reboot needed!

See [`BAZZITE_SETUP.md`](BAZZITE_SETUP.md) for details. Quick version:

```bash
# One-time setup (5 minutes)
distrobox create --name fedora-dev --image fedora:41
distrobox enter fedora-dev
sudo dnf install -y mingw64-gcc mingw64-winpthreads-static rustup
rustup-init -y && source ~/.cargo/env
rustup target add x86_64-pc-windows-gnu
exit

# Daily workflow (30 seconds)
./scripts/build_dx12.sh              # Build in container
./scripts/test_dx12_proton.sh --release  # Run on host
```

## For Traditional Linux (Fedora/Ubuntu/etc)

```bash
# 1. Install Windows Rust target
rustup target add x86_64-pc-windows-gnu

# 2. Install MinGW (Fedora)
sudo dnf install mingw64-gcc mingw64-winpthreads-static

# 3. Build and test
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
