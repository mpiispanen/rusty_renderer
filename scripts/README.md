# Scripts

This directory contains helper scripts for development, testing, and GitHub management.

## Testing Scripts

### flip_compare.py

Python wrapper for NVIDIA FLIP perceptual image comparison. This script provides direct access to the FLIP Python API with JSON output for easy integration with Rust tests.

**Installation:**
```bash
pip install flip-evaluator numpy pillow
```

**Usage:**
```bash
# Basic comparison
python3 scripts/flip_compare.py reference.png test.png

# With custom parameters
python3 scripts/flip_compare.py reference.png test.png \
    --ppd 67 \
    --error-map error_map.png \
    --output results.json \
    --verbosity 2

# Silent mode (only JSON)
python3 scripts/flip_compare.py reference.png test.png -v 0
```

**Output:**
```json
{
  "mean": 0.081237,
  "median": 0.001462,
  "q1": 0.001462,
  "q3": 0.001462,
  "min": 0.001462,
  "max": 0.997351,
  "ppd": 67.0,
  "dynamic_range": "LDR",
  "reference": "reference.png",
  "test": "test.png",
  "error_map": "error_map.png"
}
```

**Exit codes:**
- 0: Success (mean error < 0.15)
- 1: Error too high (mean error ≥ 0.15)
- 2: Script error

See `src/testing/README.md` for integration with Rust tests.

### batch_flip_compare.sh

Batch comparison script for comparing multiple images in directories.

**Usage:**
```bash
# Compare all images in test directory against reference directory
./scripts/batch_flip_compare.sh reference_images/ test_images/ output/

# Results saved to output/ with JSON and error maps
```

**Output:**
- JSON results for each comparison
- Error map images showing differences
- Summary with pass/fail statistics

## Build Scripts

### setup_windows_crosscompile.sh

Sets up Windows cross-compilation environment for DirectX 12 development on Linux.

### build_dx12.sh

Builds the project for Windows (DirectX 12) using cross-compilation.

### test_dx12_proton.sh

Tests DirectX 12 builds using Proton/Wine on Linux.

## GitHub Setup Scripts

## Quick Start for New Sessions

```bash
# Get full context and status
./scripts/session_status.sh
```

## Usage

### 1. Create Milestones

```bash
./scripts/create_milestones.sh
```

This creates all 5 milestones (M1-M5) in your GitHub repository.

### 2. Create Issues for Milestone 1

```bash
./scripts/create_m1_issues.sh
```

This creates all 5 issues for Milestone 1: Project Foundation.

### 3. Check Session Status

```bash
./scripts/session_status.sh
```

Shows:
- Git status and pending commits
- Recent commits
- GitHub milestones status
- Project structure status
- Next steps

## Prerequisites

- GitHub CLI (`gh`) must be installed and authenticated
- You must be in the repository directory when running these scripts
- You must have write access to the repository

## Manual Commands

If you prefer to run commands manually:

### Create a milestone:
```bash
gh api repos/OWNER/REPO/milestones \
  -f title="M1: Project Foundation" \
  -f description="Description here" \
  -f state="open"
```

### Create an issue:
```bash
gh issue create \
  --title "Issue title" \
  --milestone "M1: Project Foundation" \
  --label "setup,M1" \
  --body "Issue description"
```

### List milestones:
```bash
gh api repos/OWNER/REPO/milestones
```

### List issues for a milestone:
```bash
gh issue list --milestone "M1: Project Foundation"
```

## Creating Issues for Other Milestones

Currently, only M1 has an automated script. For other milestones, refer to `docs/GITHUB_SETUP.md` for the full issue breakdown and create them manually or create similar scripts.
