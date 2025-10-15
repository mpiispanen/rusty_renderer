# Setting Up GitHub Actions Self-Hosted Runner on Bazzite Linux

This guide walks you through setting up a GitHub Actions self-hosted runner on Bazzite Linux that will automatically start on boot.

## CI Architecture

### Hybrid Runner Strategy

The project uses a **hybrid CI approach** for optimal resource usage:

**GitHub-Hosted Runners (ubuntu-latest):**
- Build jobs (debug + release)
- Unit tests (no GPU required)
- Clippy (linting)
- Format checking
- Documentation build

**Self-Hosted Runner with GPU:**
- GPU-specific integration tests (M3+)
- Graphics API validation
- Visual regression tests (future)

### Benefits

✅ **Cost Efficient:** Free GitHub runners for most jobs  
✅ **Fast:** Parallel execution, artifact sharing between jobs  
✅ **GPU Access:** Self-hosted runner tagged with `gpu` for graphics tests  
✅ **Reduced Load:** Self-hosted runner only for GPU-requiring tasks

### Build Artifact Flow

```
Build Job (GitHub-hosted)
    ├─> Debug binary → artifact
    └─> Release binary → artifact
            ↓
    Test-GPU Job (Self-hosted)
        ├─> Downloads release binary
        └─> Runs GPU tests without rebuilding
```

This saves ~2-3 minutes on GPU test jobs by reusing build artifacts.

## Why Self-Hosted Runner?

For a graphics renderer project, we need:
- Access to GPU hardware (Vulkan, DirectX via Wine/Proton, etc.)
- Ability to run graphics tests with real hardware
- Consistent environment for reproducible builds

**Note:** Currently GPU tests are disabled (`if: false`) until M3 when we have actual GPU test cases.

## Prerequisites

- Bazzite Linux installed and running
- Administrator/sudo access
- GitHub repository admin access

## Setup Instructions

### 1. Generate GitHub Runner Token

1. Go to your repository on GitHub: https://github.com/mpiispanen/rusty_renderer
2. Navigate to **Settings** → **Actions** → **Runners**
3. Click **New self-hosted runner**
4. Select **Linux** as the operating system
5. Keep the page open - you'll need the commands shown

### 2. Create Runner Directory

```bash
# Create a directory for the runner
mkdir -p ~/actions-runner
cd ~/actions-runner
```

### 3. Download and Extract Runner

Use the commands from the GitHub page, or:

```bash
# Download latest runner (verify version from GitHub page)
curl -o actions-runner-linux-x64-2.319.1.tar.gz -L https://github.com/actions/runner/releases/download/v2.319.1/actions-runner-linux-x64-2.319.1.tar.gz

# Extract
tar xzf ./actions-runner-linux-x64-2.319.1.tar.gz
```

### 4. Configure the Runner

```bash
# Run the configuration script
./config.sh --url https://github.com/mpiispanen/rusty_renderer --token YOUR_TOKEN_HERE

# When prompted:
# - Enter runner name: bazzite-gpu-runner (or your preference)
# - Enter runner group: Default
# - Enter labels: self-hosted,Linux,X64,gpu (IMPORTANT: include 'gpu' tag)
# - Enter work folder: _work (default is fine)
```

**Important:** The `gpu` label is required for GPU-specific test jobs. The CI workflow uses `runs-on: [self-hosted, gpu]` to target this runner.

### 5. Test the Runner Manually

Before setting up auto-start, test it works:

```bash
./run.sh
```

You should see "Listening for Jobs". Press Ctrl+C to stop.

### 6. Set Up Systemd Service for Auto-Start

Create a systemd user service:

```bash
# Create systemd user directory if it doesn't exist
mkdir -p ~/.config/systemd/user

# Create the service file
cat > ~/.config/systemd/user/github-runner.service << 'EOF'
[Unit]
Description=GitHub Actions Runner
After=network.target

[Service]
Type=simple
WorkingDirectory=%h/actions-runner
ExecStart=%h/actions-runner/run.sh
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=default.target
EOF
```

### 7. Enable and Start the Service

```bash
# Reload systemd to recognize the new service
systemctl --user daemon-reload

# Enable the service to start on boot
systemctl --user enable github-runner.service

# Start the service now
systemctl --user start github-runner.service

# Check status
systemctl --user status github-runner.service
```

### 8. Enable Lingering (Important!)

This ensures your user services start even when you're not logged in:

```bash
loginctl enable-linger $USER
```

### 9. Verify It's Working

```bash
# Check service status
systemctl --user status github-runner.service

# View logs
journalctl --user-unit github-runner.service -f
```

You should see the runner listening for jobs.

### 10. Verify on GitHub

1. Go to repository **Settings** → **Actions** → **Runners**
2. You should see your runner listed as "Idle" (green circle)

## Managing the Runner

### Check Status
```bash
systemctl --user status github-runner.service
```

### View Logs
```bash
# Live logs
journalctl --user-unit github-runner.service -f

# Recent logs
journalctl --user-unit github-runner.service -n 100
```

### Restart Runner
```bash
systemctl --user restart github-runner.service
```

### Stop Runner
```bash
systemctl --user stop github-runner.service
```

### Disable Auto-Start
```bash
systemctl --user disable github-runner.service
```

## Known Issues

### Disk Space Warning on Bazzite

You may see warnings like "You are running out of disk space. Free space left: 0 MB" in GitHub Actions logs. This is a **false positive** due to Bazzite's architecture:

- Bazzite uses composefs for the root filesystem (`/`), which is read-only and shows as 100% full
- This is normal and expected for immutable Linux distributions
- Actual work happens in `/var/home` which has plenty of space
- The warning doesn't affect job execution - jobs will complete successfully

To verify actual disk space:
```bash
df -h /var/home
```

This is a known limitation of the GitHub Actions runner's disk space check and can be safely ignored on Bazzite.

## Troubleshooting

### Runner Not Starting After Reboot

Check if lingering is enabled:
```bash
loginctl show-user $USER | grep Linger
# Should show: Linger=yes
```

If not:
```bash
loginctl enable-linger $USER
```

### Check Runner Logs
```bash
journalctl --user-unit github-runner.service -n 100 --no-pager
```

### Verify Network Connectivity
```bash
# Test GitHub connectivity
curl -I https://github.com
```

### Runner Shows as Offline

1. Check service is running: `systemctl --user status github-runner.service`
2. Check logs for errors: `journalctl --user-unit github-runner.service -n 50`
3. Verify network connectivity
4. May need to reconfigure with a new token if token expired

## Updating the Runner

When GitHub releases a new runner version:

```bash
# Stop the service
systemctl --user stop github-runner.service

# Go to runner directory
cd ~/actions-runner

# Download new version
curl -o actions-runner-linux-x64-NEW_VERSION.tar.gz -L https://github.com/actions/runner/releases/download/vNEW_VERSION/actions-runner-linux-x64-NEW_VERSION.tar.gz

# Extract (this updates in place)
tar xzf ./actions-runner-linux-x64-NEW_VERSION.tar.gz

# Start service again
systemctl --user start github-runner.service
```

## Security Considerations

- The runner has access to your repository secrets
- Only use on a trusted machine
- Keep Bazzite system updated
- Consider running in a container for additional isolation (advanced)
- Monitor runner logs for suspicious activity

## GPU Access Verification

Once the runner is set up, verify GPU access:

```bash
# Check Vulkan
vulkaninfo | head -20

# Check GPU devices
ls -la /dev/dri/
```

The runner should have access to GPU devices for graphics testing.

## Next Steps

After setup:
1. Push a commit to trigger the CI pipeline
2. Monitor the Actions tab in GitHub
3. Verify all jobs run successfully on your self-hosted runner
4. Check that GPU tests work (will be added in M3+)

---

**Note**: Keep this document for reference. The runner should now automatically start whenever you boot your Bazzite system.
