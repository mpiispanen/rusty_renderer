# GitHub Issues and Milestones Updated - 2025-10-21

## Summary
Updated GitHub project management to reflect current state and roadmap.

---

## Closed Issues (Completed Work)

✅ **#59** - M10 Phase 2: Camera System  
✅ **#60** - M10 Phase 3: Forward Rendering Pipeline  
✅ **#52** - M8.3: Shader Resource Binding  

These were completed during today's session. See `SESSION_COMPLETE_2025-10-21.md` for details.

---

## Closed Milestones

✅ **M10: Unified Application & Scene-Driven Rendering**  
- Forward rendering with Blinn-Phong lighting complete
- Camera system working
- Scene system with TOML definitions
- Per-object transforms via push constants
- Vulkan backend production-ready

---

## New Milestones Created

### Phase 1: Multi-Backend Completion
**Due:** 2025-10-28 (1 week)  
**Focus:** Complete wgpu and DirectX backends

**Issues:**
- #62 - Implement wgpu push constants via dynamic uniforms
- #63 - Implement DirectX root constants for transforms
- #64 - Create unified shader pipeline (HLSL → WGSL/SPIR-V)

### Phase 2: Resource Management
**Due:** 2025-11-11 (3 weeks)  
**Focus:** Stop hardcoding, automatic resource allocation

**Issues:**
- #65 - Design and implement pass requirement system
- #66 - Implement automatic resource allocation in render graph
- #67 - Implement shader hot-reload

### Phase 3: Shadow Mapping
**Due:** 2025-11-25 (5 weeks)  
**Focus:** Implement shadow mapping techniques

**Issues:**
- #68 - Implement basic shadow maps (depth-only rendering)
- #69 - Implement PCF for soft shadows
- #70 - Implement Cascaded Shadow Maps (CSM)

### Phase 4: Advanced Renderers
**Due:** 2025-12-16 (8 weeks)  
**Focus:** Forward+ and Deferred rendering

**Issues:** To be created when Phase 3 nears completion

---

## Issue Details

### #62: wgpu Push Constants (2-3 hours)
- Follow guide in `docs/wgpu_push_constants.md`
- Dynamic uniform buffer approach
- Create WGSL shaders
- Test rendering matches Vulkan

### #63: DirectX Root Constants (2-3 hours)
- Update root signature
- Create HLSL shaders
- Test on Windows/Proton
- Cross-compilation documented in WORKFLOW.md

### #64: Unified Shader Pipeline (3-4 hours)
- HLSL as source of truth
- Auto-compile to SPIR-V for Vulkan
- Manual port to WGSL for wgpu
- Build script automation

### #65: Pass Requirement System (2-3 days)
- Passes declare what they need
- Type-safe requirement specification
- Validation at graph build time
- Documentation for pass authors

### #66: Automatic Resource Allocation (3-4 days)
- Render graph creates resources
- Shader cache system
- Pipeline creation
- Memory tracking

### #67: Shader Hot-Reload (2-3 days)
- File watching with `notify` crate
- Automatic recompilation
- Pipeline updates in next frame
- Error handling

### #68: Basic Shadow Maps (2 days)
- Depth-only render pass
- Shadow map sampling
- Works for directional + point lights
- Debug visualization

### #69: PCF Filtering (2-3 days)
- Percentage Closer Filtering
- Configurable kernel size
- Soft shadow edges
- Performance testing

### #70: Cascaded Shadow Maps (4-5 days)
- 2-4 cascades
- Frustum split calculation
- Cascade selection in shader
- Debug visualization

---

## Timeline

| Week | Phase | Issues |
|------|-------|--------|
| 1 | Multi-Backend | #62, #63, #64 |
| 2-3 | Resource Mgmt | #65, #66, #67 |
| 4-5 | Shadows | #68, #69, #70 |
| 6-8 | Advanced Renderers | TBD |

---

## Old Issues Still Open

These remain from previous milestones and may be superseded by new work:

- #61 - M10 Phase 4: Basic Materials & Textures
- #56 - M8.7: Complete glTF Rendering Pipeline
- #55 - M8.6: Material System
- #54 - M8.5: glTF Model Loading
- #48 - M7.6: Performance benchmarks

**Action:** Review these after Phase 1-2 completion to determine if they're still relevant or should be closed/updated.

---

## Milestone Strategy

We've shifted from sequential milestones (M8, M9, M10, etc.) to **phase-based planning**:

**Old Approach:**
- M8: Real rendering
- M9: Advanced features  
- M10: Scene system
- etc.

**New Approach:**
- Phase 1: Multi-Backend
- Phase 2: Resource Management
- Phase 3: Shadow Mapping
- Phase 4: Advanced Renderers
- etc.

**Why:** Clearer focus, better parallel work, aligned with roadmap.

---

## Next Actions

### Immediate (This Week)
1. Start on #62 (wgpu push constants)
2. Start on #63 (DirectX root constants)
3. Complete both before moving to #64

### Short Term (Next 2 Weeks)
4. Complete Phase 1 milestone
5. Begin Phase 2 (resource management)

### Medium Term (Next Month)
6. Complete Phase 2
7. Begin Phase 3 (shadow mapping)

---

## References

- `docs/ROADMAP_2025.md` - Complete roadmap
- `docs/IMPLEMENTATION_PLAN.md` - Detailed tasks
- `SESSION_COMPLETE_2025-10-21.md` - Today's work
- `PLANNING_COMPLETE.md` - Planning summary

---

**Updated:** 2025-10-21 22:00 UTC  
**Status:** All issues/milestones aligned with roadmap ✅
