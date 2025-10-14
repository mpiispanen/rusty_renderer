# GitHub Setup Scripts

This directory contains helper scripts for setting up GitHub milestones and issues.

## Usage

### 1. Create Milestones

```bash
./scripts/create_milestones.sh
```

This creates all 5 milestones (M1-M5) in your GitHub repository.

### 2. Create Issues for Milestone 1

```bash
./scripts/create_m1_issues.sh
```

This creates all 5 issues for Milestone 1: Project Foundation.

## Prerequisites

- GitHub CLI (`gh`) must be installed and authenticated
- You must be in the repository directory when running these scripts
- You must have write access to the repository

## Manual Commands

If you prefer to run commands manually:

### Create a milestone:
```bash
gh api repos/OWNER/REPO/milestones \
  -f title="M1: Project Foundation" \
  -f description="Description here" \
  -f state="open"
```

### Create an issue:
```bash
gh issue create \
  --title "Issue title" \
  --milestone "M1: Project Foundation" \
  --label "setup,M1" \
  --body "Issue description"
```

### List milestones:
```bash
gh api repos/OWNER/REPO/milestones
```

### List issues for a milestone:
```bash
gh issue list --milestone "M1: Project Foundation"
```

## Creating Issues for Other Milestones

Currently, only M1 has an automated script. For other milestones, refer to `docs/GITHUB_SETUP.md` for the full issue breakdown and create them manually or create similar scripts.
