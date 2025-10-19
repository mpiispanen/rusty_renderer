# Milestone 6 Planning: Render Graph Foundation

**Version:** 1.0  
**Created:** 2025-10-19  
**Status:** Draft - Ready for Review

## Overview

Milestone 6 focuses on implementing a render graph system - the foundation for automatic resource management, dependency tracking, and optimized execution. With M5's comprehensive testing infrastructure in place, we can confidently build complex features knowing they'll be validated automatically.

## Context from M5

**M5 Achievements:**
- ✅ Visual regression testing (FLIP integration)
- ✅ Multi-backend comparison (Vulkan, wgpu, DirectX)
- ✅ HTML report generation
- ✅ Git LFS baseline system
- ✅ Safe baseline removal workflow
- ✅ Production-ready quality gates

**Impact on M6:**
- Every render graph change will be visually validated
- Multi-backend consistency guaranteed by CI
- Clear visual feedback on rendering changes
- Baseline comparisons prevent regressions

## M6 Goals

### 1. Render Graph Data Structures
**Purpose:** Core data model for describing rendering operations

**Scope:**
- Define render pass representation
  - Pass name and type (graphics, compute, transfer)
  - Input/output resources
  - Dependencies on other passes
- Define resource representation
  - Images, buffers, samplers
  - Format, size, usage flags
  - Lifetime and ownership
- Define dependency graph
  - Directed acyclic graph (DAG)
  - Topological sorting
  - Cycle detection

**Deliverables:**
```rust
// Core types
struct RenderGraph {
    passes: Vec<RenderPass>,
    resources: Vec<Resource>,
    dependencies: Graph<PassId>,
}

struct RenderPass {
    name: String,
    kind: PassKind,
    inputs: Vec<ResourceId>,
    outputs: Vec<ResourceId>,
    callback: Box<dyn PassCallback>,
}

struct Resource {
    name: String,
    kind: ResourceKind,
    descriptor: ResourceDescriptor,
    lifetime: ResourceLifetime,
}
```

**Estimated Effort:** 15-20 hours

### 2. Dependency Resolution
**Purpose:** Automatic execution order and validation

**Scope:**
- Topological sort of passes
- Cycle detection and error reporting
- Resource producer/consumer tracking
- Validation of resource usage
  - Format compatibility
  - Access patterns
  - Lifetime correctness

**Deliverables:**
- Dependency resolver
- Execution order calculator
- Validation system
- Clear error messages

**Example:**
```rust
let graph = RenderGraph::new()
    .add_pass("shadow_map", /* ... */)
    .add_pass("gbuffer", /* ... */)
    .add_pass("lighting", /* ... */)  // depends on gbuffer
    .add_pass("compose", /* ... */)   // depends on lighting
    .compile()?; // Validates and sorts

// Execution order: shadow_map -> gbuffer -> lighting -> compose
```

**Estimated Effort:** 12-18 hours

### 3. Automatic Barrier Insertion
**Purpose:** Correct synchronization without manual barriers

**Scope:**
- Analyze resource access patterns
- Insert pipeline barriers automatically
  - Layout transitions
  - Memory dependencies
  - Queue ownership transfers
- Optimize barrier placement
  - Merge adjacent barriers
  - Batch layout transitions
- Backend-specific barrier generation
  - Vulkan: VkImageMemoryBarrier, VkBufferMemoryBarrier
  - DirectX: Resource barriers
  - wgpu: Texture state transitions

**Deliverables:**
```rust
// Automatic barrier insertion
impl RenderGraph {
    fn insert_barriers(&mut self, backend: &dyn Backend) {
        for (pass_a, pass_b) in self.execution_order.windows(2) {
            let barriers = analyze_transition(pass_a, pass_b);
            self.barriers.insert_between(pass_a, pass_b, barriers);
        }
    }
}
```

**Estimated Effort:** 18-25 hours

### 4. Resource Lifetime Tracking
**Purpose:** Automatic resource allocation and cleanup

**Scope:**
- Determine when resources are created/destroyed
- Track first and last usage
- Automatic allocation/deallocation
- Memory aliasing opportunities
  - Reuse memory for non-overlapping resources
  - Reduce total memory usage
- Transient vs persistent resources
  - Transient: Created/destroyed each frame
  - Persistent: Preserved across frames

**Deliverables:**
```rust
struct ResourceLifetime {
    first_use: PassId,
    last_use: PassId,
    can_alias: bool,
}

impl RenderGraph {
    fn optimize_memory(&mut self) {
        // Find aliasing opportunities
        let aliasing_sets = find_non_overlapping_resources(&self.lifetimes);
        
        // Allocate memory pools
        self.memory_pools = allocate_pools(aliasing_sets);
    }
}
```

**Estimated Effort:** 15-20 hours

### 5. Refactor Triangle Demo
**Purpose:** Validate render graph with real-world usage

**Scope:**
- Port triangle rendering to render graph
- Define triangle render passes
  - Single graphics pass
  - Color attachment output
- Compare output with current implementation
  - Use M5 visual testing to validate
  - Ensure identical rendering
- Demonstrate render graph benefits
  - Automatic barriers
  - Resource lifetime management
  - Clear pass structure

**Deliverables:**
```rust
// Before (M1-M5): Manual rendering
impl Backend {
    fn render(&mut self) {
        // Manual barriers
        // Manual resource management
        // Implicit dependencies
        // ...
    }
}

// After (M6): Render graph
let graph = RenderGraph::new()
    .add_pass("triangle", |pass| {
        pass.set_color_attachment("swapchain", /* ... */);
        pass.set_render_callback(|cmd, resources| {
            // Draw triangle
        });
    })
    .compile()?;

graph.execute(&mut backend)?;
```

**Visual Validation:**
- Compare graph-based vs direct rendering
- Use FLIP to ensure identical output
- Baseline images should match exactly

**Estimated Effort:** 10-15 hours

### 6. Unit Tests
**Purpose:** Comprehensive test coverage for render graph

**Scope:**
- Dependency resolution tests
  - Valid graphs
  - Cyclic dependencies (should error)
  - Missing resources (should error)
- Barrier insertion tests
  - Correct layout transitions
  - Proper synchronization
- Lifetime tracking tests
  - First/last usage detection
  - Memory aliasing correctness
- Resource validation tests
  - Format compatibility
  - Usage flags
  - Access patterns

**Deliverables:**
- ~20-30 unit tests
- Edge case coverage
- Error path validation
- Integration with visual testing

**Estimated Effort:** 8-12 hours

## Technical Design

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     RenderGraph                         │
│  - Passes (nodes)                                       │
│  - Resources (edges)                                    │
│  - Dependencies (DAG)                                   │
└─────────────────┬───────────────────────────────────────┘
                  │
                  ├─> Compilation Phase
                  │   - Dependency resolution
                  │   - Topological sort
                  │   - Validation
                  │   - Lifetime analysis
                  │
                  ├─> Optimization Phase
                  │   - Barrier insertion
                  │   - Memory aliasing
                  │   - Pass merging (future)
                  │
                  └─> Execution Phase
                      - Resource allocation
                      - Barrier submission
                      - Pass execution
                      - Resource cleanup
```

### Key Abstractions

```rust
// Pass callback - executed during rendering
trait PassCallback {
    fn execute(&self, cmd: &mut CommandBuffer, resources: &ResourceMap);
}

// Resource descriptor - defines resource properties
enum ResourceDescriptor {
    Image {
        format: Format,
        extent: Extent3D,
        usage: ImageUsageFlags,
        samples: SampleCount,
    },
    Buffer {
        size: usize,
        usage: BufferUsageFlags,
    },
}

// Resource access - how a pass uses a resource
struct ResourceAccess {
    resource: ResourceId,
    access_type: AccessType,    // Read, Write, ReadWrite
    stage: PipelineStage,        // Vertex, Fragment, Compute, etc.
    layout: ImageLayout,         // Vulkan-specific, optional for others
}
```

### Error Handling

```rust
#[derive(Debug, Error)]
enum RenderGraphError {
    #[error("Cyclic dependency detected: {0}")]
    CyclicDependency(String),
    
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    
    #[error("Incompatible formats: {0} vs {1}")]
    IncompatibleFormats(Format, Format),
    
    #[error("Invalid resource usage: {0}")]
    InvalidUsage(String),
}
```

## Implementation Strategy

### Phase 1: Core Infrastructure (Week 1)
1. Define data structures
2. Implement dependency graph
3. Basic validation
4. Unit tests

### Phase 2: Advanced Features (Week 2)
1. Barrier insertion
2. Lifetime tracking
3. Memory optimization
4. Integration tests

### Phase 3: Integration (Week 3)
1. Refactor triangle demo
2. Visual validation
3. Performance testing
4. Documentation

## Testing Strategy

### Unit Tests
- Dependency resolution
- Resource validation
- Lifetime analysis
- Barrier insertion

### Integration Tests
- Triangle demo conversion
- Multi-pass rendering
- Resource reuse

### Visual Tests (using M5 infrastructure)
- Compare graph-based vs direct rendering
- Multi-backend consistency
- Baseline validation

### Performance Tests
- Execution overhead
- Memory usage
- Barrier efficiency

## Dependencies

### M5 Infrastructure (Available)
- ✅ Visual regression testing
- ✅ Multi-backend validation
- ✅ HTML reports
- ✅ Baseline comparison

### Backend Features (Available)
- ✅ Command buffers
- ✅ Pipeline states
- ✅ Resource management
- ✅ Synchronization primitives

### New Dependencies (None)
- All required features already implemented

## Risks and Mitigations

### Risk 1: Over-Engineering
**Risk:** Render graph too complex for current needs

**Mitigation:** 
- Start simple (triangle only)
- Iterate based on requirements
- Focus on core features first

### Risk 2: Performance Overhead
**Risk:** Graph overhead slows down rendering

**Mitigation:**
- Profile early and often
- Optimize hot paths
- Compare with direct rendering

### Risk 3: Backend Differences
**Risk:** Barrier semantics differ across backends

**Mitigation:**
- Abstract barrier insertion per backend
- Extensive testing on all platforms
- Use M5 visual validation

### Risk 4: API Complexity
**Risk:** Render graph API too difficult to use

**Mitigation:**
- Clear examples (triangle demo)
- Builder pattern for ease of use
- Comprehensive documentation

## Success Criteria

### Functional Requirements
- ✅ Automatic dependency resolution
- ✅ Correct barrier insertion
- ✅ Automatic resource lifetime management
- ✅ Triangle demo works with render graph
- ✅ Identical visual output (validated by FLIP)

### Quality Requirements
- ✅ 90%+ code coverage
- ✅ All tests passing on all backends
- ✅ Performance within 5% of direct rendering
- ✅ Clear error messages
- ✅ Comprehensive documentation

### Timeline
- **Estimated:** 78-110 hours (10-14 days)
- **Target:** 3 weeks including buffer for learning/iteration

## Documentation Plan

### Technical Documentation
- Render graph design document
- API reference
- Usage examples
- Performance characteristics

### User Documentation
- Getting started guide
- Migration from direct rendering
- Best practices
- Common pitfalls

### Developer Documentation
- Implementation notes
- Backend integration guide
- Extension points
- Testing strategy

## Future Enhancements (Post-M6)

### Near-term (M7-M8)
- Pass merging optimization
- Async compute support
- Multi-queue execution
- Resource aliasing refinement

### Long-term (M9+)
- Automatic LOD selection
- Culling integration
- GPU-driven rendering
- Ray tracing support

## Milestone Timeline

```
Week 1: Core Infrastructure
├─ Day 1-2: Data structures
├─ Day 3-4: Dependency resolution
└─ Day 5: Validation

Week 2: Advanced Features
├─ Day 1-2: Barrier insertion
├─ Day 3-4: Lifetime tracking
└─ Day 5: Memory optimization

Week 3: Integration & Polish
├─ Day 1-2: Triangle demo refactor
├─ Day 3: Visual validation
├─ Day 4: Performance testing
└─ Day 5: Documentation & cleanup
```

## Conclusion

M6 builds on the solid testing foundation from M5 to implement a production-ready render graph system. The visual regression testing ensures we can refactor confidently, knowing that any rendering changes will be caught immediately.

**Key Advantages:**
- Automatic validation via M5 infrastructure
- Multi-backend testing built-in
- Clear visual feedback on changes
- Baseline comparison for regressions

**Readiness:**
- All required backend features implemented
- Testing infrastructure in place
- Clear scope and deliverables
- Well-defined success criteria

---

**Status:** Ready for implementation  
**Estimated Duration:** 10-14 days (78-110 hours)  
**Dependencies:** None (all M1-M5 work complete)  
**Risk Level:** Low-Medium (well-scoped, good foundation)

See [MILESTONES.md](MILESTONES.md) for high-level milestone overview.
