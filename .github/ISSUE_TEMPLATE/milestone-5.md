---
name: Milestone 5 - Render Graph Foundation
about: Design and implement core render graph system
title: '[M5] Render Graph Foundation'
labels: milestone, architecture, render-graph
assignees: ''
---

## Milestone 5: Render Graph Foundation

### Goal
Design and implement the core render graph system with automatic dependency resolution, execution scheduling, and barrier insertion.

### Tasks

#### Core Data Structures
- [ ] Design render graph node structure
- [ ] Design resource handle and tracking system
- [ ] Design pass descriptor (inputs/outputs/execution)
- [ ] Implement graph storage and management

#### Dependency Resolution
- [ ] Implement dependency tracking between passes
- [ ] Implement topological sort for execution order
- [ ] Detect and report cyclic dependencies
- [ ] Validate graph before execution

#### Resource Management
- [ ] Implement resource lifetime tracking
- [ ] Implement automatic resource state transitions
- [ ] Insert barriers between passes automatically
- [ ] Handle transient resources (basic allocation, no pooling yet)

#### Pass Execution
- [ ] Implement graph compilation step
- [ ] Implement graph execution with backend
- [ ] Handle per-pass context and state
- [ ] Support read/write resource declarations

#### Refactoring
- [ ] Refactor triangle demo to use render graph
- [ ] Create "triangle pass" with proper resource declarations
- [ ] Verify render graph executes correctly

#### Testing
- [ ] Unit tests for dependency resolution
- [ ] Unit tests for topological sorting
- [ ] Unit tests for cycle detection
- [ ] Unit tests for resource lifetime tracking
- [ ] Integration test: triangle via render graph
- [ ] Test complex graph scenarios (multiple passes, dependencies)

### Acceptance Criteria
- [ ] Render graph can register passes with resource dependencies
- [ ] Graph automatically determines execution order
- [ ] Barriers are inserted automatically between passes
- [ ] Triangle demo works through render graph system
- [ ] All unit tests pass
- [ ] Graph validation catches common errors (cycles, invalid resources)
- [ ] Well-documented API with examples

### Dependencies
- Milestone 4 (Multi-Backend Triangle) must be complete

### Estimated Effort
7-10 days

### Notes
- Start with runtime graph (compile-time can be explored later)
- Keep API simple and ergonomic
- Focus on correctness over optimization
- Document the graph execution model clearly
