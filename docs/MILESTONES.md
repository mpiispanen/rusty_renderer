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

## Milestone 5: Render Graph Foundation (7-10 days)
**Goal:** Implement core render graph system

**Issues:**
1. Design render graph data structures
2. Implement dependency resolution
3. Implement automatic barrier insertion
4. Implement resource lifetime tracking
5. Refactor triangle demo to use render graph
6. Add render graph unit tests

---

## Total Estimated Timeline
Approximately 24-38 days (5-8 weeks) for all short-term milestones.
