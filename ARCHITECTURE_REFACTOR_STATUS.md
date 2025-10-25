# Architecture Refactor - Status & Tracking

**Started:** 2025-10-25  
**Goal:** Fully data-driven, non-hardcoded rendering system  
**Timeline:** ~8-10 weeks

## Quick Links

- **Detailed Roadmap:** [`ARCHITECTURE_CLEANUP_ROADMAP.md`](./ARCHITECTURE_CLEANUP_ROADMAP.md)
- **Design Document:** [`docs/DESIGN.md`](./docs/DESIGN.md)
- **GitHub Milestone:** [Architecture Refactor: Data-Driven System](https://github.com/mpiispanen/rusty_renderer/milestone/19)

## Active Issues

### Phase 1: Backend Parity ⚡ (Current Focus)
- [#71](https://github.com/mpiispanen/rusty_renderer/issues/71) - Implement depth testing in DirectX
- [#72](https://github.com/mpiispanen/rusty_renderer/issues/72) - Ensure Vulkan/DirectX visual parity
- [#74](https://github.com/mpiispanen/rusty_renderer/issues/74) - Setup CI rendering tests

### Phase 2: Remove Hardcoding 🧹
- [#73](https://github.com/mpiispanen/rusty_renderer/issues/73) - Remove embedded shader fallbacks
- _More issues to be created as Phase 1 completes_

### Future Phases
- **Phase 3:** Pipeline Templates (TOML-driven rendering)
- **Phase 4:** Scene-Driven System (glTF + scene files)
- **Phase 5:** Full CI/CD Integration
- **Phase 6:** Architecture Validation

## Progress Tracking

### Week 1 (Oct 25 - Nov 1, 2025)
- [x] DirectX memory management fixed
- [x] DirectX render graph working
- [x] Backface culling enabled
- [x] Architecture roadmap created
- [x] GitHub issues/labels created
- [ ] Depth testing implementation (in progress)
- [ ] CI setup (in progress)

### Completed Milestones
- ✅ **Oct 25, 2025:** DirectX Memory Location Fix Complete
  - Fixed heap types and resource states
  - Implemented staging buffers for GPU-only uploads
  - Removed legacy hardcoded rendering
  - Cube renders with lighting and backface culling

## Vision

### Current (Partially Hardcoded)
```
Application → Backend → Some Hardcoded Data
                ↓
            Embedded Shaders
                ↓  
            Fixed Pipeline State
```

### Target (Fully Data-Driven)
```
Scene (glTF) → Pipeline Template → Render Graph
     ↓              ↓                   ↓
  Objects      Pass Configs        Resources
  Materials    Shaders             Barriers
  Transforms   Bindings            Transitions
                                       ↓
                                   Backend API
```

## Success Criteria

When complete, we should be able to:
1. **Add new scene** → Just create glTF + scene TOML (no code changes)
2. **Change rendering** → Edit pipeline template TOML (no code changes)
3. **Add new backend** → Implement traits only (no rendering logic)
4. **Verify correctness** → CI catches visual regressions automatically

## How to Contribute

1. Pick an issue from the [milestone](https://github.com/mpiispanen/rusty_renderer/milestone/19)
2. Check [`ARCHITECTURE_CLEANUP_ROADMAP.md`](./ARCHITECTURE_CLEANUP_ROADMAP.md) for context
3. Follow the task checklist in the issue
4. Submit PR with tests
5. Ensure CI passes (once implemented)

## Phase Completion Checklist

### Phase 1: Backend Parity
- [ ] DirectX depth testing works
- [ ] Vulkan and DirectX produce identical output
- [ ] CI renders and compares both backends
- [ ] All validation errors fixed

### Phase 2: Remove Hardcoding  
- [ ] No embedded shaders
- [ ] No hardcoded vertex data
- [ ] No hardcoded pipeline state
- [ ] All rendering through render graph

### Phase 3: Pipeline Templates
- [ ] TOML pipeline format defined
- [ ] Pipeline loader implemented
- [ ] Passes defined by templates
- [ ] Shader selection from templates

### Phase 4: Scene-Driven
- [ ] All geometry from glTF
- [ ] Materials from glTF
- [ ] Transforms from scene graph
- [ ] Pipeline referenced in scene file

### Phase 5: CI/CD
- [ ] Automated visual testing
- [ ] Backend comparison tests
- [ ] Reference image repository
- [ ] Test failure reporting

### Phase 6: Validation
- [ ] All hardcoding removed
- [ ] Architecture goals met
- [ ] Documentation updated
- [ ] Performance validated

---

**Last Updated:** 2025-10-25  
**Next Review:** 2025-11-01
