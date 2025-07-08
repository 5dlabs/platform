# Git Guidelines - MUST FOLLOW

## CRITICAL RULES

1. **NEVER PUSH TO MAIN BRANCH** - This is absolutely forbidden
2. **ALWAYS CREATE A FEATURE BRANCH** - Use format: `feature/task-{task-id}`
3. **DO NOT OVERRIDE GIT CONFIG** - Git user/email is pre-configured from environment
4. **NO CO-AUTHORED-BY** - Commits should only show the configured GitHub user
5. **NEVER COMMIT BUILD ARTIFACTS** - See .gitignore section below

## .gitignore Requirements

**CRITICAL**: Before committing ANY files, ensure you have a proper .gitignore file. If one doesn't exist, create it with these entries:

```gitignore
# Dependencies
node_modules/
bower_components/

# Build outputs
dist/
build/
*.js (if using TypeScript)
*.js.map
*.d.ts

# Logs and caches
npm-debug.log*
yarn-debug.log*
yarn-error.log*
.npm
.yarn-integrity
*.log

# Environment files
.env
.env.local
.env.*.local

# IDE files
.vscode/
.idea/
*.swp
*.swo
.DS_Store

# Testing
coverage/
.nyc_output/

# Temporary files
tmp/
temp/
```

## Required Workflow

```bash
# 0. FIRST: Create/verify .gitignore exists
cat .gitignore || echo "node_modules/
dist/
*.log
.env
.DS_Store" > .gitignore

# 1. Start from main and create feature branch
git checkout main
git pull origin main
git checkout -b feature/task-{task-id}

# 2. Make changes and commit (git add will respect .gitignore)
git add .
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
4. No build artifacts are included: `git status --porcelain | grep -E "node_modules|dist|\.log"`
5. .gitignore exists and is properly configured

## If You Already Committed Wrong Files

If you accidentally committed node_modules/, dist/, or other ignored files:

```bash
# 1. Remove from git but keep locally
git rm -r --cached node_modules/
git rm -r --cached dist/
git rm --cached *.log

# 2. Create/update .gitignore
echo "node_modules/
dist/
*.log
.env
.DS_Store" > .gitignore

# 3. Commit the fix
git add .gitignore
git commit -m "Add .gitignore and remove tracked files that should be ignored"
```