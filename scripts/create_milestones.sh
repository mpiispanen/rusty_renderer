#!/usr/bin/env bash
# Script to create GitHub milestones for Rusty Renderer

set -e

# Get repository info
REPO_OWNER=$(gh repo view --json owner -q .owner.login)
REPO_NAME=$(gh repo view --json name -q .name)

echo "Creating milestones for ${REPO_OWNER}/${REPO_NAME}"

# Create Milestones
echo "Creating Milestone 1..."
gh api repos/${REPO_OWNER}/${REPO_NAME}/milestones \
  -f title="M1: Project Foundation" \
  -f description="Establish the basic Rust project structure with proper module organization, command-line interface, application loop, and CI/CD pipeline." \
  -f state="open"

echo "Creating Milestone 2..."
gh api repos/${REPO_OWNER}/${REPO_NAME}/milestones \
  -f title="M2: Backend Abstraction - Stub Implementation" \
  -f description="Define the core backend abstraction traits and create stub implementations for all three graphics backends (Vulkan, DirectX, wgpu)." \
  -f state="open"

echo "Creating Milestone 3..."
gh api repos/${REPO_OWNER}/${REPO_NAME}/milestones \
  -f title="M3: Vulkan Triangle Rendering" \
  -f description="Implement the Vulkan backend (using vulkanalia) to render a single hardcoded triangle to the screen." \
  -f state="open"

echo "Creating Milestone 4..."
gh api repos/${REPO_OWNER}/${REPO_NAME}/milestones \
  -f title="M4: Multi-Backend Triangle Rendering" \
  -f description="Implement DirectX 12 and wgpu backends to render the same triangle, ensuring the backend abstraction works correctly across all three APIs." \
  -f state="open"

echo "Creating Milestone 5..."
gh api repos/${REPO_OWNER}/${REPO_NAME}/milestones \
  -f title="M5: Render Graph Foundation" \
  -f description="Design and implement the core render graph system with automatic dependency resolution, execution scheduling, and barrier insertion." \
  -f state="open"

echo ""
echo "✓ Milestones created successfully!"
echo ""
echo "To view milestones: gh api repos/${REPO_OWNER}/${REPO_NAME}/milestones"
