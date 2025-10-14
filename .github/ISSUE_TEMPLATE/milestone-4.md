---
name: Milestone 4 - Multi-Backend Triangle
about: Implement DirectX and wgpu backends for triangle rendering
title: '[M4] Multi-Backend Triangle Rendering'
labels: milestone, directx, wgpu, rendering
assignees: ''
---

## Milestone 4: Multi-Backend Triangle Rendering

### Goal
Implement DirectX 12 and wgpu backends to render the same triangle, ensuring the backend abstraction works correctly across all three APIs.

### Tasks

#### DirectX 12 Backend Implementation
- [ ] Implement DirectX 12 instance and device creation
- [ ] Implement DirectX 12 swapchain management
- [ ] Implement DirectX 12 command list recording
- [ ] Implement basic render pass/render target setup
- [ ] Implement graphics pipeline state object (PSO) creation
- [ ] Port vertex/fragment shaders to HLSL
- [ ] Implement resource management (vertex buffer, etc.)

#### DirectX Testing on Linux
- [ ] Test DirectX implementation via Proton/Wine
- [ ] Document any platform-specific quirks
- [ ] Verify rendering output matches Vulkan

#### wgpu Backend Implementation
- [ ] Implement wgpu device and adapter selection
- [ ] Implement wgpu swapchain/surface management
- [ ] Implement wgpu command encoder/buffer
- [ ] Implement wgpu render pass and pipeline
- [ ] Port shaders to WGSL (or use SPIR-V)
- [ ] Implement resource management

#### Backend Abstraction Validation
- [ ] Verify trait design works for all backends
- [ ] Refine traits if needed based on implementation experience
- [ ] Ensure consistent behavior across backends
- [ ] Document backend-specific considerations

#### Testing
- [ ] Integration tests for DirectX backend
- [ ] Integration tests for wgpu backend
- [ ] Cross-backend comparison tests
- [ ] Visual validation for all backends

### Acceptance Criteria
- [ ] Triangle renders identically on all three backends (Vulkan, DirectX, wgpu)
- [ ] DirectX works on Linux via Proton
- [ ] Backend can be switched via command-line argument
- [ ] All integration tests pass for all backends
- [ ] No regressions in Vulkan implementation
- [ ] Code is well-documented with backend-specific notes

### Dependencies
- Milestone 3 (Vulkan Triangle) must be complete

### Estimated Effort
7-10 days

### Notes
- May need to adjust trait design based on DirectX/wgpu requirements
- DirectX testing on Linux requires Proton/Wine setup
- wgpu provides good fallback for portability
- Focus on API parity, not optimization yet
