# Short-Term Milestones

This document provides a quick reference for the short-term development milestones and their breakdown into issues.

See [GITHUB_SETUP.md](GITHUB_SETUP.md) for detailed instructions on creating these milestones and issues in GitHub.

## Milestone 1: Project Foundation (2-3 days)
**Goal:** Set up basic project structure, tooling, and CI/CD

**Issues:**
1. Set up Cargo workspace structure
2. Implement basic application framework
3. Add command-line argument parsing
4. Set up CI/CD pipeline
5. Create testing infrastructure

---

## Milestone 2: Backend Abstraction - Stub Implementation (3-4 days)
**Goal:** Define backend traits and create stub implementations for all backends

**Issues:**
1. Define core backend traits
2. Create Vulkan backend stub
3. Create DirectX backend stub
4. Implement backend selection logic
5. Add backend trait unit tests

---

## Milestone 3: Vulkan Triangle (5-7 days)
**Goal:** Implement Vulkan backend to render a triangle

**Issues:**
1. Implement Vulkan instance and device
2. Implement Vulkan swapchain
3. Create triangle vertex buffer
4. Implement shader loading and pipeline
5. Implement rendering loop
6. Add integration tests

---

## Milestone 4: Multi-Backend Triangle (7-10 days)
**Goal:** Implement DirectX backend for triangle rendering

**Issues:**
1. Implement DirectX 12 backend
2. Test DirectX on Linux via Proton
3. Validate backend abstraction
4. Add cross-backend tests

---

## Milestone 5: Render Graph Architecture Fix (7-10 days)
**Goal:** Fix architectural flaws to enable true multi-pass rendering without hardcoding
**Priority:** Critical

**Issues:**
1. Update `RenderPass` with Clear/Load/Store state
2. Implement per-pass framebuffer creation (Vulkan)
3. Implement per-pass render pass creation (Vulkan)
4. Fix barrier insertion logic
5. Remove hardcoded clear values from backends

---

## Milestone 6: Experimentation Foundation (5-7 days)
**Goal:** Enable optional passes and basic post-processing
**Priority:** High

**Issues:**
1. Implement basic Tone Mapping pass
2. Make Shadow Pass optional/toggleable
3. Add input handling for toggling passes
4. Verify Shadow + Tone Mapping combination
5. Add CI test for multi-pass configuration

---

## Total Estimated Timeline
Approximately 24-38 days (5-8 weeks) for all short-term milestones.
