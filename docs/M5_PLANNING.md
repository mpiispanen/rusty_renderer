# Milestone 5 Planning: Infrastructure & Testing

**Version:** 1.0  
**Created:** 2025-10-18  
**Status:** Approved - Ready for Implementation

## Overview

Milestone 5 focuses on building robust testing and development infrastructure before moving to more complex rendering features. After successfully completing multi-backend triangle rendering (M1-M4), we need solid validation and testing tools.

## Strategic Decision: Infrastructure First

**Rationale:** Building testing infrastructure now will enable faster, more confident development of complex features later (render graph, scene system, etc.).

### Benefits
1. **Quality Foundation**: Automated visual testing prevents regressions
2. **CI Integration**: Headless rendering enables GPU tests in CI
3. **Multi-Backend Validation**: Ensure consistent rendering across APIs
4. **Debugging Efficiency**: Better tools accelerate development
5. **Confidence**: Solid foundation for complex features

## M5 Goals

### 1. Offscreen/Headless Rendering
**Purpose:** Enable rendering without visible windows for CI testing

**Scope:**
- Implement offscreen rendering for all three backends (Vulkan, wgpu, DirectX 12)
- Add `--headless` CLI flag
- Support rendering in CI environments without displays
- Maintain full rendering functionality in headless mode

**Deliverables:**
- Offscreen rendering support in all backends
- Headless mode CLI flag
- CI-compatible rendering tests

**Estimated Effort:** 20-26 hours

### 2. Screenshot Capture
**Purpose:** Save rendered frames for manual and automated validation

**Scope:**
- Capture framebuffer/texture to memory
- Convert backend-specific formats to standard PNG
- Add `--screenshot <path>` CLI flag
- Add `--frames <count>` for controlled test runs
- Support capturing from all backends

**Deliverables:**
- Screenshot capture functionality
- PNG export support
- CLI flags for screenshot control
- Examples in documentation

**Estimated Effort:** Included in offscreen rendering effort

### 3. Visual Correctness Testing
**Purpose:** Automated validation that backends produce identical output

**Scope:**
- Golden reference image system
- Pixel-by-pixel comparison with tolerance
- Visual diff image generation
- Cross-backend output validation
- Git LFS integration for reference images

**Deliverables:**
- Visual testing framework
- Reference image management
- Automated comparison tests
- Diff visualization on failures

**Estimated Effort:** 10-15 hours

### 4. Validation Layer Improvements
**Purpose:** Ensure consistent debugging across all backends

**Scope:**
- Verify validation layers work on all backends
- Document validation layer setup per backend
- Add tests for validation flag functionality
- Improve error reporting consistency

**Deliverables:**
- Validation layer documentation
- Per-backend setup guides
- Validation flag tests

**Estimated Effort:** 5-8 hours

### 5. CI/CD Enhancements
**Purpose:** Integrate new testing capabilities into CI pipeline

**Scope:**
- Enable GPU tests in CI (with headless rendering)
- Screenshot artifact upload
- Visual validation in CI (future: compare against references)
- Multi-backend testing automation

**Deliverables:**
- Updated CI workflow
- Automated GPU tests
- Screenshot artifacts
- Multi-backend CI validation

**Estimated Effort:** 8-10 hours

### 6. Documentation & Examples
**Purpose:** Document new testing capabilities and workflows

**Scope:**
- Update DESIGN.md for M5
- Document headless rendering usage
- Document screenshot capture
- Document visual testing workflow
- Create examples demonstrating features

**Deliverables:**
- Updated design document
- Testing workflow documentation
- Usage examples
- Troubleshooting guides

**Estimated Effort:** 5-8 hours

## Implementation Plan

### Phase 1: Offscreen Rendering (Week 1)
**Goal:** Enable headless rendering on all backends

1. **Research & Design** (2-3h)
   - Survey offscreen approaches per backend
   - Design unified offscreen API
   - Plan CLI integration

2. **Vulkan Implementation** (6-8h)
   - Implement VK_EXT_headless_surface approach
   - Create offscreen framebuffer alternative
   - Test headless rendering

3. **wgpu Implementation** (4-6h)
   - Create texture without window surface
   - Implement offscreen render target
   - Test on Linux backend

4. **DirectX 12 Implementation** (6-8h)
   - Render to texture without swap chain
   - Create offscreen RTV
   - Test with WARP renderer

5. **CLI Integration** (2-3h)
   - Add `--headless` flag
   - Add `--frames <count>` flag
   - Update argument parsing

### Phase 2: Screenshot Capture (Week 2)
**Goal:** Save rendered frames to PNG files

1. **Image Capture** (4-6h)
   - Implement framebuffer readback for each backend
   - Handle format conversion
   - Add memory management

2. **PNG Export** (2-3h)
   - Integrate `image` crate
   - Convert to standard RGB8/RGBA8
   - Implement save functionality

3. **CLI Integration** (1-2h)
   - Add `--screenshot <path>` flag
   - Save on exit or after N frames
   - Error handling

4. **Testing** (2-3h)
   - Test screenshot on all backends
   - Verify image quality
   - Test in headless mode

### Phase 3: Visual Testing (Week 2-3)
**Goal:** Automated visual correctness validation

1. **Reference Images** (2-3h)
   - Generate golden reference images
   - Set up Git LFS
   - Organize reference image storage

2. **Comparison Framework** (4-6h)
   - Implement pixel-by-pixel comparison
   - Add tolerance for platform differences
   - Generate visual diff images

3. **Cross-Backend Tests** (3-4h)
   - Compare Vulkan vs wgpu output
   - Compare Vulkan vs DirectX output
   - Add tolerance configuration

4. **CI Integration** (2-3h)
   - Add visual tests to CI
   - Upload diff images on failure
   - Configure test thresholds

### Phase 4: Polish & Documentation (Week 3)
**Goal:** Clean up, document, and finalize M5

1. **Validation Layer Review** (3-4h)
   - Test validation on all backends
   - Document per-backend setup
   - Add validation flag tests

2. **Documentation** (5-8h)
   - Update DESIGN.md to v0.4.0
   - Document testing workflow
   - Create usage examples
   - Write troubleshooting guide

3. **Code Review & Cleanup** (3-4h)
   - Code review
   - Refactoring
   - Add inline documentation

4. **M5 Retrospective** (2-3h)
   - Write M5_RETROSPECTIVE.md
   - Document learnings
   - Plan M6

## Time Estimates

| Component | Estimated Hours |
|-----------|----------------|
| Offscreen Rendering | 20-26 |
| Screenshot Capture | (included above) |
| Visual Testing | 10-15 |
| Validation Improvements | 5-8 |
| CI/CD Enhancements | 8-10 |
| Documentation | 5-8 |
| **Total** | **50-70 hours** |

**Target Duration:** 2-3 weeks

## Issues to Create

Break down M5 into focused implementation issues:

1. ✅ **#9** - M5 Planning (this issue - updated)
2. ✅ **#27** - Offscreen Rendering & Screenshot Capture (updated)
3. 🆕 **Visual Testing Framework** - Comparison and validation
4. 🆕 **Reference Image Management** - Git LFS setup and golden images
5. 🆕 **CI GPU Testing** - Integrate headless tests into CI
6. 🆕 **Validation Layer Documentation** - Per-backend guides
7. 🆕 **M5 Documentation & Retrospective** - Final docs and review

## Success Criteria

### Technical
- [ ] All backends support headless rendering
- [ ] Screenshot capture works on all backends
- [ ] Visual comparison framework functional
- [ ] Reference images in Git LFS
- [ ] CI runs automated GPU tests
- [ ] All backends validated with validation layers
- [ ] Zero CI failures

### Quality
- [ ] All tests passing (unit, integration, visual)
- [ ] Clippy clean
- [ ] Well documented
- [ ] Examples working
- [ ] Troubleshooting guides complete

### Process
- [ ] DESIGN.md updated to v0.4.0
- [ ] M5 retrospective written
- [ ] M6 previewed
- [ ] All issues closed

## Risks & Mitigation

### Risk 1: Offscreen Complexity
**Impact:** High - Different approaches per backend  
**Mitigation:** Research thoroughly, start with easiest (wgpu), learn from each

### Risk 2: Visual Testing Accuracy
**Impact:** Medium - Platform differences may cause false positives  
**Mitigation:** Use generous tolerance, document expected differences

### Risk 3: Git LFS Setup
**Impact:** Low - New tool in workflow  
**Mitigation:** Follow standard Git LFS practices, document setup

### Risk 4: CI Environment Limitations
**Impact:** Medium - GitHub runners have limited GPU access  
**Mitigation:** Use software renderers (WARP, SwiftShader), test locally first

## Future Milestones Preview

After M5 infrastructure is in place:

### M6: Render Graph Foundation
- Render pass abstraction
- Dependency resolution
- Resource barriers
- Execution scheduling

### M7: Enhanced Graphics Pipeline
- Vertex/index buffers
- Uniform buffers
- Multiple objects
- Textures and samplers

### M8: Scene System
- Scene graph
- glTF loading
- Camera system
- Interactive exploration

### M9: Developer Tools
- Shader hot-reloading
- Debug UI (egui)
- Performance profiling
- Render graph visualization

## Related Documents

- M4 Retrospective: `docs/M4_RETROSPECTIVE.md`
- Design Document: `docs/DESIGN.md` (v0.3.0)
- Validation Layers: `docs/VALIDATION_LAYERS.md`
- Testing DirectX: `docs/TESTING_DIRECTX_ON_LINUX.md`
- Coordinate Systems: `docs/COORDINATE_SYSTEMS.md`

## Notes from M4

### Apply These Learnings
- ✅ Test frequently during implementation
- ✅ Keep backend implementations isolated
- ✅ Document platform differences as discovered
- ✅ Update DESIGN.md incrementally

### Avoid These Issues
- ❌ Don't defer testing until the end
- ❌ Don't make issues too broad
- ❌ Don't skip CI verification before closing
- ❌ Don't let documentation fall behind

---

**Milestone:** M5: Infrastructure & Testing  
**Status:** Approved - Ready for Implementation  
**Estimated Effort:** 50-70 hours (2-3 weeks)  
**Next Action:** Create implementation issues and begin Phase 1
