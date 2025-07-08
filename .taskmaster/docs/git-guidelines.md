# Git Guidelines - MUST FOLLOW

## CRITICAL RULES

1. **NEVER PUSH TO MAIN BRANCH** - This is absolutely forbidden
2. **ALWAYS CREATE A FEATURE BRANCH** - Use format: `feature/task-{task-id}`
3. **DO NOT OVERRIDE GIT CONFIG** - Git user/email is pre-configured from environment
4. **NO CO-AUTHORED-BY** - Commits should only show the configured GitHub user

## Required Workflow

```bash
# 1. Start from main and create feature branch
git checkout main
git pull origin main
git checkout -b feature/task-{task-id}

# 2. Make changes and commit
git add <files>
git commit -m "Clear description of changes

🤖 Generated with [Claude Code](https://claude.ai/code)"

# 3. Push ONLY to feature branch
git push origin feature/task-{task-id}

# 4. Create PR from feature branch to main
gh pr create --base main --title "Task {task-id}: Brief description" --body "..."
```

## Git Configuration

The following is already configured by the environment:
- `user.name` is set to the GitHub username (e.g., `swe-1-5dlabs`)
- `user.email` is set to `{username}@users.noreply.github.com`
- Credentials are stored in `/workspace/.git-credentials`

**DO NOT** run any of these commands:
- ❌ `git config user.name "Claude"`
- ❌ `git config user.email "claude@example.com"`
- ❌ `git push origin main`
- ❌ Adding `Co-Authored-By: Claude` to commits

## Branch Protection

The `main` branch has protection rules that will reject direct pushes. You MUST use pull requests.

## Verification

Before creating a PR, verify:
1. You're on a feature branch: `git branch --show-current`
2. Your commits show the correct author: `git log -1 --pretty=format:"%an <%ae>"`
3. You're pushing to the feature branch, not main