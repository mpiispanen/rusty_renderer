# Documentation Index

**Last Updated:** 2025-10-21

This directory contains all design documents, planning materials, and technical guides for the Rusty Renderer project.

---

## 📋 Quick Navigation

### For New Contributors
1. Start with [../README.md](../README.md) - Project overview
2. Read [DESIGN.md](DESIGN.md) - Architecture and principles
3. Review [WORKFLOW.md](WORKFLOW.md) - Development practices
4. Check [ROADMAP_2025.md](ROADMAP_2025.md) - Current plan

### For Active Development
- [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md) - Concrete tasks (next 8 weeks)
- [WORKFLOW.md](WORKFLOW.md) - Best practices and debugging
- [wgpu_push_constants.md](wgpu_push_constants.md) - wgpu implementation guide

### For Understanding Current State
- [../LIGHTING_WORKING.md](../LIGHTING_WORKING.md) - Latest achievement
- [../VALIDATION_FIXED.md](../VALIDATION_FIXED.md) - Validation error fixes
- [../BACKEND_STATUS.md](../BACKEND_STATUS.md) - Backend comparison
- [../SESSION_COMPLETE_2025-10-21.md](../SESSION_COMPLETE_2025-10-21.md) - Latest session

---

## 📚 Document Purposes

### Core Design Documents

**[DESIGN.md](DESIGN.md)**
- Architecture overview
- Core principles
- Component descriptions
- Current capabilities and limitations
- **Read this:** To understand system design

**[ROADMAP_2025.md](ROADMAP_2025.md)**
- 8-phase development plan through 2026
- Vision for interactive rendering laboratory
- Short-term (4 weeks) and long-term goals
- Success metrics
- **Read this:** To understand where we're going

**[IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md)**
- Concrete tasks for next 8 weeks
- Step-by-step implementation guides
- Code examples
- Acceptance criteria
- **Read this:** Before starting new work

### Process Documents

**[WORKFLOW.md](WORKFLOW.md)**
- Standard development flow
- Incremental commit strategy
- Debugging best practices
- Cross-compilation instructions
- Pre-commit hook setup
- Common pitfalls
- **Read this:** Before making your first commit

**[CONTRIBUTING.md](../CONTRIBUTING.md)**
- How to contribute
- Code style guidelines
- PR process
- **Read this:** Before submitting a PR

### Implementation Guides

**[wgpu_push_constants.md](wgpu_push_constants.md)**
- Complete guide for implementing wgpu push constants
- Dynamic uniform buffer approach
- WGSL shader requirements
- **Read this:** When implementing wgpu backend

### Status Documents (Root Directory)

**[BACKEND_STATUS.md](../BACKEND_STATUS.md)**
- Current state of each backend
- What's working, what's not
- Implementation guides for missing features

**[LIGHTING_WORKING.md](../LIGHTING_WORKING.md)**
- How we fixed the lighting
- Debugging process
- GLSL struct layout issue
- **Read this:** To learn about shader debugging

**[VALIDATION_FIXED.md](../VALIDATION_FIXED.md)**
- How we achieved zero validation errors
- Descriptor set synchronization
- Buffer cleanup
- **Read this:** To understand Vulkan best practices

**[SESSION_COMPLETE_2025-10-21.md](../SESSION_COMPLETE_2025-10-21.md)**
- Latest session summary
- Accomplishments
- Commits made
- Next steps

**[REMAINING_ISSUES.md](../REMAINING_ISSUES.md)**
- Known issues and their analysis
- Prioritized fix recommendations

---

## 📖 Specialized Documentation

### By Directory

**`../shaders/`**
- Shader source files
- README.md with shader documentation

**`../scenes/`**
- Scene definition files (TOML)
- Example scenes

**`../scripts/`**
- Build and utility scripts
- Pre-commit hook template

**`../tests/`**
- Test documentation
- Visual testing guides

**`../src/testing/`**
- Testing utilities
- FLIP integration docs

---

## 🎯 Quick Answers

**Q: How do I get started?**  
A: Read [../README.md](../README.md), then [WORKFLOW.md](WORKFLOW.md), then pick an issue from [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md)

**Q: How do I debug shader issues?**  
A: See "Systematic Shader Debugging" in [WORKFLOW.md](WORKFLOW.md) and [LIGHTING_WORKING.md](../LIGHTING_WORKING.md)

**Q: What's the architecture?**  
A: See [DESIGN.md](DESIGN.md) sections on "Architecture Overview" and "Core Components"

**Q: How do I add a new render pass?**  
A: See [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md) Task 2.1 "Pass Requirement System"

**Q: Why is lighting broken?**  
A: Read [LIGHTING_WORKING.md](../LIGHTING_WORKING.md) - it's probably a struct layout issue

**Q: How do I cross-compile for Windows?**  
A: See "Cross-Compilation and Testing" in [WORKFLOW.md](WORKFLOW.md)

**Q: What should I work on next?**  
A: Check [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md) for prioritized tasks

**Q: How do I run tests?**  
A: `cargo test --lib` for unit tests, see [WORKFLOW.md](WORKFLOW.md) for complete validation

---

## 📅 Document Update Schedule

- **DESIGN.md** - Update on architecture changes
- **ROADMAP_2025.md** - Review monthly
- **IMPLEMENTATION_PLAN.md** - Update weekly
- **WORKFLOW.md** - Update when processes change
- **Status docs** - Update after each session
- **Session summaries** - Create after each major session

---

## 🔗 External Links

- [GitHub Repository](https://github.com/mpiispanen/rusty_renderer)
- [Vulkan Specification](https://www.khronos.org/registry/vulkan/specs/1.3/)
- [wgpu Documentation](https://wgpu.rs/)
- [GLSL Specification](https://www.khronos.org/registry/OpenGL/specs/gl/GLSLangSpec.4.60.pdf)

---

**Maintained by:** Project Contributors  
**Questions?** Open an issue on GitHub
