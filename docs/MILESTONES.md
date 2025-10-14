# Short-Term Milestones

This document provides a quick reference for the short-term development milestones.

## Milestone 1: Project Foundation (2-3 days)
**Goal:** Set up basic project structure, tooling, and CI/CD

**Key Deliverables:**
- Cargo workspace with proper module structure
- Basic application loop with winit
- Command-line argument parsing
- GitHub Actions CI/CD pipeline
- Unit test framework

**Issue Template:** `.github/ISSUE_TEMPLATE/milestone-1.md`

---

## Milestone 2: Backend Abstraction - Stub Implementation (3-4 days)
**Goal:** Define backend traits and create stub implementations for all backends

**Key Deliverables:**
- Core backend traits (GraphicsBackend, Device, CommandBuffer, Pipeline, Resource, Swapchain)
- Stub implementations for Vulkan, DirectX, and wgpu
- Backend selection logic
- Unit tests for trait contracts

**Issue Template:** `.github/ISSUE_TEMPLATE/milestone-2.md`

---

## Milestone 3: Vulkan Triangle (5-7 days)
**Goal:** Implement Vulkan backend to render a triangle

**Key Deliverables:**
- Working Vulkan backend using vulkanalia
- Hardcoded triangle vertex data
- Simple vertex and fragment shaders
- Rendering loop with swapchain management
- Integration tests

**Issue Template:** `.github/ISSUE_TEMPLATE/milestone-3.md`

---

## Milestone 4: Multi-Backend Triangle (7-10 days)
**Goal:** Implement DirectX and wgpu backends for triangle rendering

**Key Deliverables:**
- DirectX 12 backend implementation
- DirectX testing on Linux via Proton
- wgpu backend implementation
- Cross-backend validation
- Integration tests for all backends

**Issue Template:** `.github/ISSUE_TEMPLATE/milestone-4.md`

---

## Milestone 5: Render Graph Foundation (7-10 days)
**Goal:** Implement core render graph system

**Key Deliverables:**
- Render graph data structures
- Automatic dependency resolution and execution ordering
- Automatic barrier insertion
- Resource lifetime tracking
- Refactored triangle demo using render graph
- Comprehensive unit tests

**Issue Template:** `.github/ISSUE_TEMPLATE/milestone-5.md`

---

## Creating GitHub Issues

To create these milestones as GitHub issues:

1. Go to the repository on GitHub
2. Navigate to Issues → New Issue
3. Select the appropriate milestone template
4. Review and create the issue

Alternatively, use the GitHub CLI:

```bash
# Create from template
gh issue create --template milestone-1.md
gh issue create --template milestone-2.md
# ... etc
```

## Progress Tracking

Track overall progress in the main [DESIGN.md](DESIGN.md) document under "Current State" section.
