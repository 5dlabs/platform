# Task Context

This workspace contains all the necessary files to complete the assigned task.

## Task Files

**@task.md** - The main task description, including title, priority, and high-level requirements

**@design-spec.md** - Comprehensive technical design specification with architecture details, key components, and implementation approach

**@prompt.md** - Step-by-step implementation instructions for you to follow

**@acceptance-criteria.md** - Success criteria checklist to ensure all requirements are met

## Repository Information

The repository should be cloned to the workspace root. When you run `pwd`, you should see `/workspace` and this should also be the root of the git repository.

## Git Configuration

The workspace is configured with:
- Git repository cloned to `/workspace` root
- GitHub authentication via personal access token
- GitHub CLI (`gh`) pre-authenticated
- Default branch is `main` (not `master`)
- Git user/email pre-configured from environment

## Git Workflow

**@git-guidelines.md** - CRITICAL: Contains mandatory git workflow rules that MUST be followed

## Important Notes

- Task files in this directory (CLAUDE.md, task.md, etc.) are excluded from git via .gitignore
- These are working files for your reference only
- Focus on creating the deliverables specified in the prompt
- All implementation instructions are in **@prompt.md**