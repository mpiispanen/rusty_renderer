# FIXED: MinGW Installation on Bazzite

## The Problem

```
error: Could not depsolve transaction; 2 problems detected:
 Problem 1: package mingw64-winpthreads requires mingw64-filesystem >= 95...
 Problem 2: conflicting requests with redhat-rpm-config...
```

The `rpm-ostree install` command fails on Bazzite because it's an immutable OS with package dependency conflicts.

## The Solution: Use Distrobox ✅

Instead of modifying your base system, use a Fedora container:

### Quick Setup (5 minutes, no reboot!)

```bash
# 1. Create development container
distrobox create --name fedora-dev --image fedora:41

# 2. Enter container and install tools
distrobox enter fedora-dev

# 3. Inside container, install everything
sudo dnf install -y mingw64-gcc mingw64-winpthreads-static rustup
rustup-init -y
source ~/.cargo/env
rustup target add x86_64-pc-windows-gnu
exit

# 4. Use the automated build script
./scripts/build_dx12.sh

# 5. Run with Proton
./scripts/test_dx12_proton.sh --release
```

### What This Does

1. **Creates isolated container** - Fedora 41 container with full package manager
2. **Installs MinGW in container** - No conflicts, no system modifications
3. **Auto-mounts your home** - Your project files are accessible
4. **Builds Windows .exe** - Cross-compiles DirectX 12 code
5. **Runs on host via Proton** - Tests actual DirectX API calls

### Daily Workflow

Just run the helper script - it handles everything automatically:

```bash
./scripts/build_dx12.sh                  # Builds in container
./scripts/test_dx12_proton.sh --release  # Runs on host
```

The `build_dx12.sh` script:
- Auto-creates container if missing
- Installs tools on first run
- Builds Windows binary
- Takes ~2 minutes first time, ~30 seconds after

## Why This is Better

✅ **No system changes** - Base OS stays immutable  
✅ **No reboot needed** - Containers start instantly  
✅ **No conflicts** - Isolated from system packages  
✅ **Automatic** - Helper script handles everything  
✅ **Professional** - Industry standard approach  

## Documentation

- **Bazzite Guide**: `docs/BAZZITE_SETUP.md` - Complete setup and workflow
- **Quick Start**: `docs/QUICK_START_PROTON.md` - Fast track guide
- **Full Details**: `docs/TESTING_DIRECTX_ON_LINUX.md` - How it works

## Testing All Backends

```bash
# Vulkan (native)
cargo run --release

# wgpu (native)
cargo run --release -- --backend wgpu

# DirectX 12 (container + Proton)
./scripts/build_dx12.sh
./scripts/test_dx12_proton.sh --release
```

All three backends can now be tested on your Bazzite system!

---

**TL;DR**: Don't use `rpm-ostree` for MinGW on Bazzite. Use Distrobox instead - it's cleaner, faster, and actually works.
