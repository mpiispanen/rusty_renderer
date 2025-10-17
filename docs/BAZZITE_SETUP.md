# DirectX 12 Testing Setup on Bazzite

## Problem
Bazzite is an immutable OS that has dependency conflicts when trying to install MinGW cross-compilation tools via `rpm-ostree`.

## Solution: Use Distrobox

Distrobox lets you run a mutable Fedora container where you can install MinGW without affecting your base system.

## Setup (5 minutes, no reboot needed!)

### Step 1: Create a Fedora development container

```bash
distrobox create --name fedora-dev --image fedora:41
distrobox enter fedora-dev
```

### Step 2: Inside the container, install cross-compilation tools

```bash
# Inside the distrobox container
sudo dnf install -y mingw64-gcc mingw64-winpthreads-static rustup

# Set up Rust
rustup-init -y
source $HOME/.cargo/env
rustup target add x86_64-pc-windows-gnu
```

### Step 3: Configure the project (still inside container)

```bash
# Navigate to your project (it's automatically mounted)
cd ~/rusty_renderer

# The .cargo/config.toml is already set up correctly
```

### Step 4: Test DirectX 12 compilation

```bash
# Still inside the container
cargo build --release --target x86_64-pc-windows-gnu
```

### Step 5: Run with Proton (exit container first)

```bash
# Exit the container
exit

# Back on your host Bazzite system
./scripts/test_dx12_proton.sh --release
```

## Daily Workflow

### Option A: Build in container, run on host
```bash
# Terminal 1: Enter container and build
distrobox enter fedora-dev
cd ~/rusty_renderer
cargo build --release --target x86_64-pc-windows-gnu
exit

# Terminal 1: Run on host
./scripts/test_dx12_proton.sh --release
```

### Option B: One-liner from host
```bash
# Build in container, then run on host
distrobox enter fedora-dev -- bash -c "cd ~/rusty_renderer && cargo build --release --target x86_64-pc-windows-gnu" && \
./scripts/test_dx12_proton.sh --release
```

### Option C: Use a helper script (create this)
```bash
# Create ~/rusty_renderer/scripts/build_dx12.sh
cat > scripts/build_dx12.sh << 'EOF'
#!/bin/bash
distrobox enter fedora-dev -- bash -c "cd ~/rusty_renderer && cargo build --release --target x86_64-pc-windows-gnu"
EOF

chmod +x scripts/build_dx12.sh

# Then just run:
./scripts/build_dx12.sh && ./scripts/test_dx12_proton.sh --release
```

## Why This Works Better

1. **No system modifications** - Your immutable Bazzite system stays clean
2. **No reboot needed** - Containers start instantly
3. **Isolated environment** - MinGW tools don't conflict with base system
4. **Automatic mounting** - Your home directory is accessible in the container
5. **Same performance** - Container builds are just as fast as native

## Verifying VKD3D-Proton

VKD3D-Proton should already be installed on Bazzite (it comes with Proton). Verify:

```bash
ls ~/.steam/root/compatibilitytools.d/ 2>/dev/null || \
ls ~/.local/share/Steam/compatibilitytools.d/ 2>/dev/null || \
flatpak list | grep -i proton
```

If VKD3D-Proton is not installed, the script will detect it and provide instructions.

## Troubleshooting

### Container doesn't start
```bash
# List containers
distrobox list

# Remove and recreate
distrobox rm fedora-dev
distrobox create --name fedora-dev --image fedora:41
```

### Rust not in PATH inside container
```bash
# Add to container's ~/.bashrc
echo 'source $HOME/.cargo/env' >> ~/.bashrc
distrobox enter fedora-dev
```

### Can't find project files
```bash
# Your home directory is automatically mounted
# So ~/rusty_renderer on host = ~/rusty_renderer in container
pwd  # Should show /var/home/matpii01
ls ~/rusty_renderer  # Should show your project
```

## Complete Test of All 3 Backends

```bash
# Vulkan (native on host)
cargo run --release

# wgpu (native on host)  
cargo run --release -- --backend wgpu

# DirectX 12 (build in container, run via Proton on host)
distrobox enter fedora-dev -- bash -c "cd ~/rusty_renderer && cargo build --release --target x86_64-pc-windows-gnu"
./scripts/test_dx12_proton.sh --release
```

## Advanced: Persistent Container Setup

To make the container setup permanent, add to your `~/.bashrc`:

```bash
# Auto-enter fedora-dev for cargo windows builds
alias cargo-windows='distrobox enter fedora-dev -- bash -c "cd $PWD && cargo build --target x86_64-pc-windows-gnu"'
alias cargo-windows-release='distrobox enter fedora-dev -- bash -c "cd $PWD && cargo build --release --target x86_64-pc-windows-gnu"'
```

Then just use:
```bash
cargo-windows-release
./scripts/test_dx12_proton.sh --release
```

## Summary

- **Setup time**: ~5 minutes (download + install)
- **System changes**: None (container only)
- **Reboot required**: No
- **Performance impact**: Negligible
- **Maintenance**: None (container auto-updates)

This is the recommended approach for development on immutable operating systems like Bazzite, Silverblue, or Kinoite.
