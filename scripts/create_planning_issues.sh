#!/usr/bin/env bash
# Script to create planning/review issues for Milestones 2-5

set -e

# Get repository info
REPO_OWNER=$(gh repo view --json owner -q .owner.login)
REPO_NAME=$(gh repo view --json name -q .name)

echo "Creating planning issues for Milestones 2-5"
echo "Repository: ${REPO_OWNER}/${REPO_NAME}"
echo ""

# Create planning label
echo "Creating 'planning' label..."
gh label create "planning" --color "c5def5" --description "Milestone planning and review tasks" || true

echo ""

# Issue for M2
echo "Creating planning issue for M2..."
gh issue create \
  --title "M2 Planning: Review M1 and plan backend abstraction" \
  --milestone "M2: Backend Abstraction - Stub Implementation" \
  --label "planning,M2" \
  --body "## Description
Review the progress from Milestone 1 and perform iterative planning for M2 (Backend Abstraction) and future milestones.

## Tasks
### Review M1 Progress
- [ ] Review all completed M1 issues
- [ ] Document lessons learned from M1
- [ ] Identify any technical debt or issues to address
- [ ] Update project structure documentation if needed

### Plan M2 Implementation
- [ ] Review backend abstraction design in DESIGN.md
- [ ] Identify specific traits and interfaces needed
- [ ] Plan stub implementation approach for all three backends
- [ ] Break down M2 into detailed implementation tasks
- [ ] Create additional issues for M2 if needed

### Update Future Milestones
- [ ] Review M3-M5 plans based on M1 learnings
- [ ] Update MILESTONES.md if architecture insights require changes
- [ ] Adjust timeline estimates if needed
- [ ] Update DESIGN.md with any architectural refinements

## Acceptance Criteria
- [ ] M1 retrospective documented
- [ ] M2 tasks clearly defined and prioritized
- [ ] Future milestones updated to reflect current understanding
- [ ] All team members aligned on M2 approach

## Related
- Milestone 1: Project Foundation (completed)
- Milestone 2: Backend Abstraction - Stub Implementation
- See docs/DESIGN.md and docs/MILESTONES.md"

# Create M2 label
gh label create "M2" --color "0e8a16" --description "Milestone 2: Backend Abstraction" || true

# Issue for M3
echo "Creating planning issue for M3..."
gh issue create \
  --title "M3 Planning: Review M2 and plan Vulkan implementation" \
  --milestone "M3: Vulkan Triangle Rendering" \
  --label "planning,M3" \
  --body "## Description
Review the progress from Milestone 2 and perform iterative planning for M3 (Vulkan Triangle Rendering) and future milestones.

## Tasks
### Review M2 Progress
- [ ] Review all completed M2 issues
- [ ] Validate backend abstraction design works as intended
- [ ] Document lessons learned from stub implementations
- [ ] Identify any abstraction gaps or improvements needed

### Plan M3 Implementation
- [ ] Review Vulkan implementation plan in DESIGN.md
- [ ] Verify vulkanalia crate compatibility and capabilities
- [ ] Plan Vulkan initialization sequence
- [ ] Design shader compilation and management approach
- [ ] Break down M3 into detailed implementation tasks
- [ ] Create additional issues for M3 if needed

### Update Future Milestones
- [ ] Review M4-M5 plans based on M2 learnings
- [ ] Update backend abstraction if needed based on stub experience
- [ ] Adjust MILESTONES.md timeline if needed
- [ ] Document any architectural refinements

## Acceptance Criteria
- [ ] M2 retrospective documented
- [ ] Backend abstraction validated and refined if needed
- [ ] M3 Vulkan tasks clearly defined
- [ ] Shader workflow planned
- [ ] Future milestones updated to reflect current understanding

## Related
- Milestone 2: Backend Abstraction - Stub Implementation (completed)
- Milestone 3: Vulkan Triangle Rendering
- See docs/DESIGN.md and docs/MILESTONES.md"

# Create M3 label
gh label create "M3" --color "0e8a16" --description "Milestone 3: Vulkan Triangle Rendering" || true

# Issue for M4
echo "Creating planning issue for M4..."
gh issue create \
  --title "M4 Planning: Review M3 and plan multi-backend implementation" \
  --milestone "M4: Multi-Backend Triangle Rendering" \
  --label "planning,M4" \
  --body "## Description
Review the progress from Milestone 3 and perform iterative planning for M4 (Multi-Backend Triangle Rendering) and future milestones.

## Tasks
### Review M3 Progress
- [ ] Review all completed M3 issues
- [ ] Validate Vulkan implementation against design goals
- [ ] Document Vulkan-specific learnings and gotchas
- [ ] Identify backend abstraction improvements from Vulkan experience
- [ ] Performance baseline established

### Plan M4 Implementation
- [ ] Review DirectX 12 and wgpu implementation plans
- [ ] Refine backend abstraction based on Vulkan experience
- [ ] Plan DirectX 12 implementation approach
- [ ] Plan wgpu implementation approach
- [ ] Define cross-backend testing strategy
- [ ] Break down M4 into detailed implementation tasks
- [ ] Create additional issues for M4 if needed

### Update Future Milestones
- [ ] Review M5 (Render Graph) plans based on multi-backend insights
- [ ] Update MILESTONES.md timeline if needed
- [ ] Document backend-specific considerations
- [ ] Plan for backend feature parity testing

## Acceptance Criteria
- [ ] M3 retrospective documented with Vulkan learnings
- [ ] Backend abstraction refined if needed
- [ ] M4 tasks clearly defined for DirectX and wgpu
- [ ] Cross-backend testing approach planned
- [ ] M5 updated to reflect multi-backend reality

## Related
- Milestone 3: Vulkan Triangle Rendering (completed)
- Milestone 4: Multi-Backend Triangle Rendering
- See docs/DESIGN.md and docs/MILESTONES.md"

# Create M4 label
gh label create "M4" --color "0e8a16" --description "Milestone 4: Multi-Backend Triangle Rendering" || true

# Issue for M5
echo "Creating planning issue for M5..."
gh issue create \
  --title "M5 Planning: Review M4 and plan render graph implementation" \
  --milestone "M5: Render Graph Foundation" \
  --label "planning,M5" \
  --body "## Description
Review the progress from Milestone 4 and perform iterative planning for M5 (Render Graph Foundation) and future development.

## Tasks
### Review M4 Progress
- [ ] Review all completed M4 issues
- [ ] Validate all three backends working correctly
- [ ] Document backend-specific differences and considerations
- [ ] Verify backend abstraction is solid across Vulkan, DirectX, and wgpu
- [ ] Establish performance baselines for all backends

### Plan M5 Implementation
- [ ] Review render graph design in DESIGN.md
- [ ] Study reference implementations (Frostbite, Unreal, etc.)
- [ ] Design graph node and edge representation
- [ ] Plan dependency resolution algorithm
- [ ] Plan barrier insertion strategy
- [ ] Design resource aliasing and lifetime management
- [ ] Plan execution scheduling approach
- [ ] Break down M5 into detailed implementation tasks
- [ ] Create additional issues for M5 if needed

### Plan Post-M5 Development
- [ ] Define next milestone(s) beyond M5
- [ ] Plan feature additions (mesh rendering, materials, etc.)
- [ ] Consider advanced render graph features
- [ ] Update long-term roadmap
- [ ] Document architectural decisions and rationale

## Acceptance Criteria
- [ ] M4 retrospective documented with multi-backend insights
- [ ] All three backends validated and benchmarked
- [ ] M5 render graph design finalized
- [ ] Render graph tasks clearly defined and prioritized
- [ ] Post-M5 roadmap outlined
- [ ] Architecture documentation updated

## Related
- Milestone 4: Multi-Backend Triangle Rendering (completed)
- Milestone 5: Render Graph Foundation
- See docs/DESIGN.md and docs/MILESTONES.md
- See render graph references in documentation"

# Create M5 label
gh label create "M5" --color "0e8a16" --description "Milestone 5: Render Graph Foundation" || true

echo ""
echo "✓ All planning issues created successfully!"
echo ""
echo "To view all planning issues: gh issue list --label planning"
