---
name: Milestone 3 - Vulkan Triangle
about: Implement Vulkan backend to render a single triangle
title: '[M3] Vulkan Triangle Rendering'
labels: milestone, vulkan, rendering
assignees: ''
---

## Milestone 3: Vulkan Triangle Rendering

### Goal
Implement the Vulkan backend (using vulkanalia) to render a single hardcoded triangle to the screen.

### Tasks

#### Vulkan Backend Implementation
- [ ] Implement Vulkan instance and device creation
- [ ] Implement Vulkan swapchain management
- [ ] Implement Vulkan command buffer recording
- [ ] Implement basic render pass creation
- [ ] Implement graphics pipeline creation (vertex + fragment shader)

#### Resource Management
- [ ] Create hardcoded triangle vertex data (position + color)
- [ ] Implement vertex buffer creation and upload
- [ ] Implement buffer management utilities

#### Shader Pipeline
- [ ] Write simple vertex shader (passthrough position + color)
- [ ] Write simple fragment shader (output interpolated color)
- [ ] Compile shaders to SPIR-V
- [ ] Load and create shader modules

#### Rendering Loop
- [ ] Implement frame rendering logic
- [ ] Handle swapchain image acquisition
- [ ] Record and submit command buffers
- [ ] Present rendered frames
- [ ] Handle window resize

#### Error Handling
- [ ] Proper error propagation and handling
- [ ] Validation layer integration (debug builds)
- [ ] Meaningful error messages

#### Testing
- [ ] Integration test: verify triangle renders
- [ ] Visual validation (manual)
- [ ] Test window resize behavior
- [ ] Test error cases (device lost, etc.)

### Acceptance Criteria
- [ ] Triangle renders correctly on screen with Vulkan backend
- [ ] Colors interpolate correctly across triangle
- [ ] Window can be resized without crashes
- [ ] Validation layers report no errors (debug build)
- [ ] Integration test passes
- [ ] Code is well-documented

### Dependencies
- Milestone 2 (Backend Abstraction Stubs) must be complete

### Estimated Effort
5-7 days

### Notes
- Use vulkanalia for Vulkan bindings
- Keep it simple - no complex features yet
- Hardcoded vertex data is fine for this milestone
- Focus on getting the basics right
