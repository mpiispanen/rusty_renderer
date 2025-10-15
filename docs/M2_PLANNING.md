# Milestone 2 Planning: Backend Abstraction - Stub Implementation

**Date:** 2025-10-15  
**Milestone:** M2 - Backend Abstraction  
**Estimated Duration:** 3-4 days  
**Status:** Planning

## Goals

1. Define comprehensive backend traits for graphics abstraction
2. Create stub implementations for all three backends (Vulkan, DirectX 12, wgpu)
3. Implement backend selection and initialization logic
4. Establish thorough unit testing for trait contracts
5. Validate abstraction design before real implementation in M3

## Background: M1 Learnings

From M1 Retrospective:
- ✅ Modular architecture working well
- ✅ CI/CD pipeline robust and fast
- ✅ Test infrastructure ready for expansion
- ✅ Window management already functional
- ✅ Clean foundation with no technical debt

Key insight: **Trait-based backend abstraction is validated by stub structure**

## Backend Abstraction Design

### Core Traits (from DESIGN.md)

#### 1. GraphicsBackend (Main Interface)
```rust
pub trait GraphicsBackend {
    fn backend_type() -> BackendType;
    fn initialize(window: &Window) -> Result<Self>;
    fn begin_frame(&mut self) -> Result<()>;
    fn end_frame(&mut self) -> Result<()>;
    fn resize(&mut self, width: u32, height: u32) -> Result<()>;
    fn cleanup(&mut self);
    fn device(&self) -> &dyn Device;
    fn swapchain(&self) -> &dyn Swapchain;
}
```

**Purpose:** Unified entry point for all backend operations

#### 2. Device (Device Management)
```rust
pub trait Device {
    fn name(&self) -> &str;
    fn supports_feature(&self, feature: &str) -> bool;
    // Backend-specific access for advanced usage
    fn as_any(&self) -> &dyn Any;
}
```

**Purpose:** GPU device abstraction, capability queries

#### 3. CommandBuffer (Command Recording)
```rust
pub trait CommandBuffer {
    fn begin(&mut self) -> Result<()>;
    fn end(&mut self) -> Result<()>;
    fn clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) -> Result<()>;
    fn bind_pipeline(&mut self, pipeline: &dyn Pipeline) -> Result<()>;
    fn draw(&mut self, vertex_count: u32, instance_count: u32) -> Result<()>;
}
```

**Purpose:** Recording rendering commands (minimal set for M2/M3)

#### 4. Pipeline (Pipeline State)
```rust
pub trait Pipeline {
    fn name(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
}
```

**Purpose:** Graphics pipeline abstraction (will expand in M3)

#### 5. Resource (Buffers/Textures)
```rust
pub trait Resource {
    fn size(&self) -> usize;
    fn resource_type(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
}
```

**Purpose:** GPU resource management (basic for M2)

#### 6. Swapchain (Presentation)
```rust
pub trait Swapchain {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn current_frame(&self) -> usize;
    fn acquire_next_image(&mut self) -> Result<()>;
    fn present(&mut self) -> Result<()>;
    fn recreate(&mut self, width: u32, height: u32) -> Result<()>;
}
```

**Purpose:** Window surface and presentation management

### Design Principles

1. **Minimal but Complete**
   - Include only what's needed for M3 (triangle rendering)
   - Design for future expansion
   - Avoid premature optimization

2. **Backend Parity**
   - Same interface for all backends
   - Backend-specific features behind `as_any()` escape hatch
   - Clear capability querying

3. **Error Handling**
   - Use `anyhow::Result` for now
   - Plan for custom error types in future
   - Descriptive error messages

4. **Testability**
   - All traits easily mockable
   - Stub implementations validate contracts
   - Unit tests for each trait

## Detailed Task Breakdown

### Task 1: Define Core Backend Traits (Issue #1)
**Estimated Time:** 4-6 hours

**Subtasks:**
- [ ] Create trait definitions in `src/backends/mod.rs`
- [ ] Define `BackendType` enum
- [ ] Document all traits with rustdoc
- [ ] Define factory function `create_backend()`
- [ ] Add trait contract tests

**Deliverables:**
- Complete trait definitions
- Comprehensive documentation
- Unit tests for trait contracts

**Acceptance Criteria:**
- All traits compile
- Documentation builds without warnings
- Basic tests pass

---

### Task 2: Create Vulkan Backend Stub (Issue #2)
**Estimated Time:** 3-4 hours

**Subtasks:**
- [ ] Create `src/backends/vulkan/mod.rs`
- [ ] Implement `VulkanBackend` struct
- [ ] Implement all trait methods (return dummy values)
- [ ] Implement stub Device, Swapchain, etc.
- [ ] Add unit tests for Vulkan backend

**Deliverables:**
- Complete Vulkan stub implementation
- All traits implemented
- Tests validating stub behavior

**Acceptance Criteria:**
- Vulkan backend creates successfully
- All trait methods callable
- Tests pass
- Clippy clean

---

### Task 3: Create DirectX Backend Stub (Issue #3)
**Estimated Time:** 3-4 hours

**Subtasks:**
- [ ] Create `src/backends/directx/mod.rs`
- [ ] Implement `DirectXBackend` struct
- [ ] Implement all trait methods (return dummy values)
- [ ] Implement stub Device, Swapchain, etc.
- [ ] Add unit tests for DirectX backend

**Deliverables:**
- Complete DirectX stub implementation
- All traits implemented
- Tests validating stub behavior

**Acceptance Criteria:**
- DirectX backend creates successfully
- All trait methods callable
- Tests pass
- Clippy clean

---

### Task 4: Create wgpu Backend Stub (Issue #4)
**Estimated Time:** 3-4 hours

**Subtasks:**
- [ ] Create `src/backends/wgpu_backend/mod.rs`
- [ ] Implement `WgpuBackend` struct
- [ ] Implement all trait methods (return dummy values)
- [ ] Implement stub Device, Swapchain, etc.
- [ ] Add unit tests for wgpu backend

**Deliverables:**
- Complete wgpu stub implementation
- All traits implemented
- Tests validating stub behavior

**Acceptance Criteria:**
- wgpu backend creates successfully
- All trait methods callable
- Tests pass
- Clippy clean

---

### Task 5: Implement Backend Selection Logic (Issue #5)
**Estimated Time:** 4-6 hours

**Subtasks:**
- [ ] Integrate backend creation with App struct
- [ ] Wire CLI config to backend selection
- [ ] Add backend initialization in app.rs
- [ ] Handle initialization errors gracefully
- [ ] Add integration tests for backend selection

**Deliverables:**
- App successfully creates selected backend
- CLI backend flag works correctly
- Error handling for unsupported backends
- Integration tests

**Acceptance Criteria:**
- Can select backend via CLI
- App initializes correct backend
- Proper error messages for failures
- All integration tests pass

---

### Task 6: Add Backend Trait Unit Tests (Issue #6)
**Estimated Time:** 3-4 hours

**Subtasks:**
- [ ] Create `tests/backend_traits.rs`
- [ ] Test backend creation for all types
- [ ] Test trait method contracts
- [ ] Test error handling
- [ ] Test capability queries
- [ ] Add test coverage documentation

**Deliverables:**
- Comprehensive unit test suite
- Edge case coverage
- Documentation of test strategy

**Acceptance Criteria:**
- All backend types tested
- Trait contracts validated
- 100% of stub methods covered
- Tests pass in CI

---

## Testing Strategy

### Unit Tests
- Each trait method tested individually
- Backend creation tests
- Capability query tests
- Error handling tests

### Integration Tests
- Backend selection from CLI
- Backend initialization flow
- Multiple backend switching (if supported)

### CI Validation
- All tests must pass
- Clippy clean (no warnings)
- Format check passes
- Documentation builds

## Dependencies

### Crate Additions (M2)
```toml
# None needed for M2 - stubs only!
# Real implementations in M3+:
# vulkanalia (M3)
# windows (M4)  
# wgpu (M4)
```

### Existing Dependencies
- `winit` - for Window type
- `anyhow` - for error handling
- Standard library

## Success Criteria

### Definition of Done for M2
- [ ] All 6 traits defined and documented
- [ ] All 3 backends have complete stub implementations
- [ ] Backend selection logic working
- [ ] CLI can select any backend
- [ ] All unit tests passing (target: 30+ tests)
- [ ] All integration tests passing
- [ ] CI fully green
- [ ] Documentation complete
- [ ] No clippy warnings
- [ ] Code formatted

### Quality Gates
1. **Trait Design:** Comprehensive enough for M3 triangle rendering
2. **Stub Quality:** All methods implemented, none panicking
3. **Test Coverage:** Every public method tested
4. **Documentation:** Every trait and method documented
5. **CI:** All jobs passing

## Risk Assessment

### Low Risk ✅
- Trait definition (well-understood problem)
- Stub implementation (simple, no external deps)
- Testing (infrastructure already exists)

### Medium Risk ⚠️
- Trait completeness (might need additions in M3)
  - **Mitigation:** Keep traits extensible, use `as_any()` for backend-specific

### Minimal Risk 🟢
- CI pipeline (already proven in M1)
- Development workflow (well-documented)

## Timeline

**Total Estimated Time:** 20-28 hours (2.5-3.5 days)

### Day 1
- Task 1: Define core traits (4-6h)
- Task 2: Vulkan stub (3-4h)

### Day 2
- Task 3: DirectX stub (3-4h)
- Task 4: wgpu stub (3-4h)

### Day 3
- Task 5: Backend selection (4-6h)
- Task 6: Unit tests (3-4h)

**Buffer:** 0.5-1 day for unexpected issues

## Impact on Future Milestones

### M3: Vulkan Triangle Rendering
- **Enabled by:** Complete trait definitions
- **Ready to implement:** Vulkan backend with real code
- **Validated:** Abstraction layer proven by stubs

### M4: Multi-Backend Triangle
- **Enabled by:** Proven abstraction layer
- **Ready to implement:** DirectX and wgpu with confidence
- **Validated:** Same interface works for all backends

### M5: Render Graph Foundation
- **Not blocked:** Render graph can build on backend traits
- **May need:** Additional resource management traits
- **Plan:** Extend traits if needed (non-breaking)

## Architecture Refinements

### Changes to DESIGN.md
None needed - current design is sound.

Potential future additions:
- Async resource loading (post-M5)
- Multi-threading support (post-M5)
- Advanced pipeline features (as needed)

### Changes to MILESTONES.md
No timeline changes needed. M1 experience suggests estimates are accurate.

## Open Questions

1. **Command Buffer Lifetime:** Who owns command buffers?
   - **Decision:** Backend creates, App uses. Clarify in M3.

2. **Resource Management:** How to handle resource cleanup?
   - **Decision:** RAII via Drop trait. Validate in M3.

3. **Error Recovery:** Can we recover from backend errors?
   - **Decision:** Graceful degradation where possible. Test in M3.

## Conclusion

M2 is well-defined and ready to execute. The stub implementation approach will:
1. Validate trait design before complex Vulkan implementation
2. Enable parallel backend development later
3. Establish comprehensive test coverage early
4. Prove the abstraction layer works

**Ready to proceed with implementation!**

---

**Next Steps:**
1. Review and approve this plan
2. Create detailed GitHub issues for tasks 1-6
3. Begin implementation starting with Task 1
4. Follow CI-validated workflow from docs/WORKFLOW.md
