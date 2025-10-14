# Setting Up GitHub Milestones and Issues

This guide explains how to properly set up milestones and issues for the Rusty Renderer project.

## GitHub Milestones vs Issues

- **Milestone**: A collection of related issues representing a major goal (e.g., "M1: Project Foundation")
- **Issue**: A specific task or feature to be completed (e.g., "Set up Cargo project structure")

## Step 1: Create Milestones

Go to your repository on GitHub → Issues → Milestones → New Milestone

### Milestone 1: Project Foundation
- **Title:** M1: Project Foundation
- **Due Date:** (2-3 days from start)
- **Description:**
  ```
  Establish the basic Rust project structure with proper module organization, 
  command-line interface, application loop, and CI/CD pipeline.
  ```

### Milestone 2: Backend Abstraction - Stub Implementation
- **Title:** M2: Backend Abstraction - Stub Implementation
- **Due Date:** (3-4 days from M1 completion)
- **Description:**
  ```
  Define the core backend abstraction traits and create stub implementations 
  for all three graphics backends (Vulkan, DirectX, wgpu).
  ```

### Milestone 3: Vulkan Triangle Rendering
- **Title:** M3: Vulkan Triangle Rendering
- **Due Date:** (5-7 days from M2 completion)
- **Description:**
  ```
  Implement the Vulkan backend (using vulkanalia) to render a single 
  hardcoded triangle to the screen.
  ```

### Milestone 4: Multi-Backend Triangle Rendering
- **Title:** M4: Multi-Backend Triangle Rendering
- **Due Date:** (7-10 days from M3 completion)
- **Description:**
  ```
  Implement DirectX 12 and wgpu backends to render the same triangle, 
  ensuring the backend abstraction works correctly across all three APIs.
  ```

### Milestone 5: Render Graph Foundation
- **Title:** M5: Render Graph Foundation
- **Due Date:** (7-10 days from M4 completion)
- **Description:**
  ```
  Design and implement the core render graph system with automatic dependency 
  resolution, execution scheduling, and barrier insertion.
  ```

## Step 2: Create Issues for Each Milestone

### Issues for M1: Project Foundation

1. **Set up Cargo workspace structure**
   - Labels: `setup`, `M1`
   - Milestone: M1: Project Foundation
   - Tasks:
     - [ ] Initialize Cargo workspace
     - [ ] Create module directories (backends/, render_graph/, scene/, etc.)
     - [ ] Set up Cargo.toml with initial dependencies
     - [ ] Add .gitignore for Rust projects

2. **Implement basic application framework**
   - Labels: `setup`, `M1`
   - Milestone: M1: Project Foundation
   - Tasks:
     - [ ] Create main.rs with entry point
     - [ ] Implement app.rs with winit event loop
     - [ ] Test window creation and event handling

3. **Add command-line argument parsing**
   - Labels: `setup`, `M1`
   - Milestone: M1: Project Foundation
   - Tasks:
     - [ ] Add clap dependency
     - [ ] Implement CLI arguments (backend, width, height, debug)
     - [ ] Add help text and documentation

4. **Set up CI/CD pipeline**
   - Labels: `ci`, `M1`
   - Milestone: M1: Project Foundation
   - Tasks:
     - [ ] Create GitHub Actions workflow
     - [ ] Add build, test, clippy, fmt checks
     - [ ] Document local runner setup for graphics tests
     - [ ] Add README badges

5. **Create testing infrastructure**
   - Labels: `testing`, `M1`
   - Milestone: M1: Project Foundation
   - Tasks:
     - [ ] Set up unit test framework
     - [ ] Create integration test skeleton
     - [ ] Add test utilities module

### Issues for M2: Backend Abstraction - Stub Implementation

1. **Define core backend traits**
   - Labels: `architecture`, `M2`
   - Milestone: M2: Backend Abstraction - Stub Implementation

2. **Create Vulkan backend stub**
   - Labels: `vulkan`, `M2`
   - Milestone: M2: Backend Abstraction - Stub Implementation

3. **Create DirectX backend stub**
   - Labels: `directx`, `M2`
   - Milestone: M2: Backend Abstraction - Stub Implementation

4. **Create wgpu backend stub**
   - Labels: `wgpu`, `M2`
   - Milestone: M2: Backend Abstraction - Stub Implementation

5. **Implement backend selection logic**
   - Labels: `architecture`, `M2`
   - Milestone: M2: Backend Abstraction - Stub Implementation

6. **Add backend trait unit tests**
   - Labels: `testing`, `M2`
   - Milestone: M2: Backend Abstraction - Stub Implementation

### Issues for M3: Vulkan Triangle Rendering

1. **Implement Vulkan instance and device**
   - Labels: `vulkan`, `rendering`, `M3`
   - Milestone: M3: Vulkan Triangle Rendering

2. **Implement Vulkan swapchain**
   - Labels: `vulkan`, `rendering`, `M3`
   - Milestone: M3: Vulkan Triangle Rendering

3. **Create triangle vertex buffer**
   - Labels: `vulkan`, `rendering`, `M3`
   - Milestone: M3: Vulkan Triangle Rendering

4. **Implement shader loading and pipeline**
   - Labels: `vulkan`, `shaders`, `M3`
   - Milestone: M3: Vulkan Triangle Rendering

5. **Implement rendering loop**
   - Labels: `vulkan`, `rendering`, `M3`
   - Milestone: M3: Vulkan Triangle Rendering

6. **Add integration tests**
   - Labels: `testing`, `M3`
   - Milestone: M3: Vulkan Triangle Rendering

### Issues for M4: Multi-Backend Triangle Rendering

1. **Implement DirectX 12 backend**
   - Labels: `directx`, `rendering`, `M4`
   - Milestone: M4: Multi-Backend Triangle Rendering

2. **Test DirectX on Linux via Proton**
   - Labels: `directx`, `testing`, `M4`
   - Milestone: M4: Multi-Backend Triangle Rendering

3. **Implement wgpu backend**
   - Labels: `wgpu`, `rendering`, `M4`
   - Milestone: M4: Multi-Backend Triangle Rendering

4. **Validate backend abstraction**
   - Labels: `architecture`, `M4`
   - Milestone: M4: Multi-Backend Triangle Rendering

5. **Add cross-backend tests**
   - Labels: `testing`, `M4`
   - Milestone: M4: Multi-Backend Triangle Rendering

### Issues for M5: Render Graph Foundation

1. **Design render graph data structures**
   - Labels: `render-graph`, `architecture`, `M5`
   - Milestone: M5: Render Graph Foundation

2. **Implement dependency resolution**
   - Labels: `render-graph`, `M5`
   - Milestone: M5: Render Graph Foundation

3. **Implement automatic barrier insertion**
   - Labels: `render-graph`, `M5`
   - Milestone: M5: Render Graph Foundation

4. **Implement resource lifetime tracking**
   - Labels: `render-graph`, `M5`
   - Milestone: M5: Render Graph Foundation

5. **Refactor triangle demo to use render graph**
   - Labels: `render-graph`, `rendering`, `M5`
   - Milestone: M5: Render Graph Foundation

6. **Add render graph unit tests**
   - Labels: `render-graph`, `testing`, `M5`
   - Milestone: M5: Render Graph Foundation

## Step 3: Using GitHub CLI (Alternative)

```bash
# Create a milestone
gh api repos/:owner/:repo/milestones -f title="M1: Project Foundation" -f description="..." -f due_on="2024-11-01T00:00:00Z"

# Create an issue assigned to a milestone
gh issue create \
  --title "Set up Cargo workspace structure" \
  --body "..." \
  --label "setup,M1" \
  --milestone "M1: Project Foundation"
```

## Labels to Create

Create these labels in your repository (Settings → Labels):

- `M1`, `M2`, `M3`, `M4`, `M5` - Milestone markers
- `setup` - Project setup tasks
- `architecture` - Architecture and design
- `vulkan`, `directx`, `wgpu` - Backend-specific
- `rendering` - Rendering-related
- `shaders` - Shader-related
- `render-graph` - Render graph system
- `testing` - Test-related
- `ci` - CI/CD related
- `documentation` - Documentation

## Progress Tracking

Track milestone completion in the [DESIGN.md](DESIGN.md) under "Current State" section.
