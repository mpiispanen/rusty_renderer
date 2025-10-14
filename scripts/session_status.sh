#!/usr/bin/env bash
# Quick status check and context for new sessions

set -e

echo "════════════════════════════════════════════════════════"
echo "  Rusty Renderer - Session Quick Start"
echo "════════════════════════════════════════════════════════"
echo ""

# Git status
echo "📋 Git Status:"
git status --short --branch
echo ""

# Check if we're ahead/behind
AHEAD=$(git rev-list --count @{u}..HEAD 2>/dev/null || echo "0")
BEHIND=$(git rev-list --count HEAD..@{u} 2>/dev/null || echo "0")

if [ "$AHEAD" -gt 0 ]; then
    echo "⚠️  You have $AHEAD commit(s) to push"
fi

if [ "$BEHIND" -gt 0 ]; then
    echo "⚠️  You are $BEHIND commit(s) behind remote (run 'git pull')"
fi
echo ""

# Recent commits
echo "📝 Recent Commits (last 3):"
git log --oneline -3
echo ""

# Check if milestones exist on GitHub
echo "🎯 GitHub Milestones:"
if command -v gh &> /dev/null; then
    REPO_OWNER=$(gh repo view --json owner -q .owner.login 2>/dev/null || echo "unknown")
    REPO_NAME=$(gh repo view --json name -q .name 2>/dev/null || echo "unknown")
    
    if [ "$REPO_OWNER" != "unknown" ]; then
        MILESTONE_COUNT=$(gh api repos/${REPO_OWNER}/${REPO_NAME}/milestones 2>/dev/null | jq length || echo "0")
        
        if [ "$MILESTONE_COUNT" -eq 0 ]; then
            echo "   ⚠️  No milestones found - run: ./scripts/create_milestones.sh"
        else
            echo "   ✓ $MILESTONE_COUNT milestone(s) exist"
            gh api repos/${REPO_OWNER}/${REPO_NAME}/milestones --jq '.[] | "   - \(.title) (\(.open_issues) open / \(.closed_issues) closed)"' 2>/dev/null || true
        fi
    fi
else
    echo "   ⚠️  GitHub CLI not installed (install 'gh' to see milestone status)"
fi
echo ""

# Check if source directory exists
echo "📁 Project Structure:"
if [ -d "src" ]; then
    echo "   ✓ src/ exists"
    echo "   Files: $(find src -type f -name '*.rs' 2>/dev/null | wc -l) Rust files"
else
    echo "   ⚠️  src/ not created yet (will be created in Milestone 1)"
fi

if [ -d "tests" ]; then
    echo "   ✓ tests/ exists"
else
    echo "   ⚠️  tests/ not created yet (will be created in Milestone 1)"
fi

if [ -f "Cargo.toml" ]; then
    echo "   ✓ Cargo.toml exists"
else
    echo "   ⚠️  Cargo.toml not created yet (will be created in Milestone 1)"
fi
echo ""

# Next steps
echo "🚀 Next Steps:"
if [ ! -f "Cargo.toml" ]; then
    echo "   1. Review SESSION_CONTEXT.md for current status"
    echo "   2. Push changes: git push origin main"
    echo "   3. Create milestones: ./scripts/create_milestones.sh"
    echo "   4. Create M1 issues: ./scripts/create_m1_issues.sh"
    echo "   5. Start Milestone 1 implementation"
elif [ ! -d "src" ]; then
    echo "   1. Review open issues: gh issue list --milestone 'M1: Project Foundation'"
    echo "   2. Continue Milestone 1 implementation"
else
    echo "   1. Check current milestone progress"
    echo "   2. Review open issues and continue development"
fi
echo ""

echo "📚 Quick References:"
echo "   - Session context:  cat SESSION_CONTEXT.md"
echo "   - Design document:  cat docs/DESIGN.md"
echo "   - Milestones:       cat docs/MILESTONES.md"
if command -v gh &> /dev/null && [ "$REPO_OWNER" != "unknown" ]; then
    echo "   - GitHub issues:    gh issue list"
    echo "   - Current milestone: gh issue list --milestone 'M1: Project Foundation'"
fi
echo ""
echo "════════════════════════════════════════════════════════"
