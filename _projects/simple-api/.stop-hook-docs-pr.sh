#!/bin/bash
# Post-completion hook for docs generation - PR creation

echo "🔗 STOP HOOK - Starting PR creation..."
echo "Timestamp: $(date)"
echo "Working directory: $(pwd)"

# === GIT AUTHENTICATION SETUP ===
echo "=== GIT AUTHENTICATION SETUP ==="

# Source GitHub environment if available
if [ -f ".github-env" ]; then
    source .github-env
    echo "✅ Loaded GitHub environment"
elif [ -f "/workspace/.github-env" ]; then
    source /workspace/.github-env
    echo "✅ Loaded GitHub environment from /workspace"
fi

# Ensure git credentials are properly configured
if [ -n "$GITHUB_TOKEN" ] && [ -n "$GITHUB_USER" ]; then
    echo "✅ GitHub credentials available"

    # Configure git user if not already set
    if [ -z "$(git config user.name)" ]; then
        git config user.name "$GITHUB_USER"
        git config user.email "${GITHUB_USER}@users.noreply.github.com"
        echo "✅ Configured git user: $GITHUB_USER"
    fi

    # Ensure git credentials file exists
    if [ ! -f "/workspace/.git-credentials" ] || ! grep -q "github.com" /workspace/.git-credentials 2>/dev/null; then
        echo "https://${GITHUB_USER}:${GITHUB_TOKEN}@github.com" > /workspace/.git-credentials
        chmod 600 /workspace/.git-credentials
        echo "✅ Created git credentials file"
    fi

    # Configure git to use the credentials
    git config credential.helper 'store --file=/workspace/.git-credentials'
    echo "✅ Configured git credential helper"

else
    echo "❌ Missing GITHUB_TOKEN or GITHUB_USER"
    echo "   GITHUB_TOKEN present: $([ -n "$GITHUB_TOKEN" ] && echo "yes" || echo "no")"
    echo "   GITHUB_USER present: $([ -n "$GITHUB_USER" ] && echo "yes" || echo "no")"
    exit 1
fi

# Create a test file to prove the stop hook ran
echo "Stop hook executed at $(date)" > .stop-hook-executed
echo "✅ Created .stop-hook-executed file"

# Check if we have any documentation changes
if git diff --quiet HEAD -- .taskmaster/docs/ 2>/dev/null && git diff --cached --quiet HEAD -- .taskmaster/docs/ 2>/dev/null; then
    # Check for untracked files
    UNTRACKED_DOCS=$(git ls-files --others --exclude-standard .taskmaster/docs/ 2>/dev/null | wc -l)
    if [ "$UNTRACKED_DOCS" -eq 0 ]; then
        echo "No documentation changes found - exiting"
        exit 0
    fi
fi

echo "📝 Documentation changes detected - proceeding with PR creation"

# Add all documentation changes
git add .taskmaster/docs/ || { echo "❌ Failed to add docs"; exit 1; }

# Commit changes
COMMIT_MSG="docs: Auto-generated documentation via orchestrator

- Generated comprehensive task documentation
- Updated task breakdown and implementation details
- Added acceptance criteria and prompts"

if git commit -m "$COMMIT_MSG"; then
    echo "✅ Successfully committed documentation changes"
else
    echo "❌ Failed to commit changes"
    exit 1
fi

# Push changes
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if git push origin "$CURRENT_BRANCH"; then
    echo "✅ Successfully pushed changes to $CURRENT_BRANCH"
else
    echo "❌ Failed to push changes"
    exit 1
fi

# Create PR if not on main/master
if [ "$CURRENT_BRANCH" != "main" ] && [ "$CURRENT_BRANCH" != "master" ]; then
    if command -v gh >/dev/null 2>&1; then
        gh pr create --title "docs: Auto-generated documentation updates" \
                     --body "Auto-generated documentation updates from orchestrator" \
                     --base main || echo "⚠️ PR creation failed (may already exist)"
        echo "✅ PR creation attempted"
    else
        echo "⚠️ GitHub CLI not available"
    fi
fi

echo "🎉 Stop hook completed successfully!"

# Original complex logic (preserved for reference)
: '
# Source GitHub environment if available
if [ -f ".github-env" ]; then
    source .github-env
    echo "✅ Loaded GitHub environment"
else
    echo "⚠️  Warning: .github-env not found"
fi

# Setup authentication based on environment
if [ "$USE_SSH" = "true" ]; then
    echo "🔑 Using SSH authentication"

    # Ensure SSH key is available
    if [ ! -f ~/.ssh/id_rsa ]; then
        echo "❌ Error: SSH private key not found at ~/.ssh/id_rsa"
        exit 1
    fi

    # Test SSH connection
    echo "Testing SSH connection to GitHub..."
    ssh -T git@github.com 2>&1 | head -3 || echo "SSH test completed"

elif [ -n "$GITHUB_TOKEN" ]; then
    echo "🔑 Using HTTPS token authentication"

    # Configure git credential helper for token auth
    git config credential.helper store
    if [ -f "/workspace/.git-credentials" ]; then
        git config credential.helper 'store --file=/workspace/.git-credentials'
        echo "✅ Using existing git credentials"
    else
        echo "⚠️  Warning: No git credentials file found"
    fi
else
    echo "❌ Error: No authentication method available"
    echo "   Need either USE_SSH=true with SSH key or GITHUB_TOKEN"
    exit 1
fi

# Configure git user (use PM user or repository user)
PM_USER="pm0-5dlabs"
if [ -n "$PM_USER" ]; then
    git config user.name "$PM_USER"
    git config user.email "${PM_USER}@users.noreply.github.com"
    echo "✅ Configured git user: $PM_USER"
else
    echo "⚠️  Warning: No PM user configured"
fi

# Check if there are changes to commit
if git diff --quiet && git diff --cached --quiet; then
    echo "ℹ️  No changes to commit"
    exit 0
fi

# Stage all changes
echo "📝 Staging changes..."
git add .

# Create commit message
COMMIT_MSG="docs: auto-generate Task Master documentation

- Generated documentation for Task Master tasks
- Updated by Claude Code agent
- Working directory: $WORKING_DIR
- Repository: git@github.com:5dlabs/platform.git
- Source branch: feature/example-project-and-cli

🤖 Generated by Claude Code - Docs Generation"

# Commit changes
echo "💾 Committing changes..."
git commit -m "$COMMIT_MSG"

# Push to current branch
CURRENT_BRANCH=$(git branch --show-current)
echo "🚀 Pushing to branch: $CURRENT_BRANCH"

# Push using appropriate authentication method
if [ "$USE_SSH" = "true" ]; then
    echo "Pushing via SSH..."
    git push origin "$CURRENT_BRANCH"
    echo "✅ Successfully pushed to $CURRENT_BRANCH via SSH"
elif [ -n "$GITHUB_TOKEN" ]; then
    echo "Pushing via HTTPS with token..."
    git push origin "$CURRENT_BRANCH"
    echo "✅ Successfully pushed to $CURRENT_BRANCH via HTTPS"
else
    echo "❌ Error: No authentication method configured for push"
    exit 1
fi

# Create PR if we're not on the main branch
if [ "$CURRENT_BRANCH" != "main" ] && [ "$CURRENT_BRANCH" != "master" ]; then
    echo "🔀 Creating pull request..."

    PR_TITLE="docs: auto-generate Task Master documentation"
    PR_BODY="## Documentation Update

**Generated by:** Claude Code Agent
**Working Directory:** \`$WORKING_DIR\`
**Branch:** \`$CURRENT_BRANCH\`
**Timestamp:** $(date -u +"%Y-%m-%d %H:%M:%S UTC")
**Authentication:** $([ "$USE_SSH" = "true" ] && echo "SSH" || echo "HTTPS")
**Repository:** git@github.com:5dlabs/platform.git
**Source Branch:** feature/example-project-and-cli


### Changes
- Auto-generated Task Master documentation for all tasks
- Updated task files and specifications
- Documentation generated in: \`_projects/simple-api\`

### Notes
This PR was automatically created by the Claude Code documentation generation system.
Scope: Complete Task Master documentation

🤖 **Auto-generated** - Review and merge when ready"

    # Create PR using GitHub CLI
    if command -v gh >/dev/null 2>&1; then
        # Set up gh authentication
        if [ "$USE_SSH" = "true" ]; then
            echo "Configuring GitHub CLI for SSH..."
            gh auth login --with-token <<< "$GITHUB_TOKEN" 2>/dev/null || echo "GitHub CLI auth may have failed"
        elif [ -n "$GITHUB_TOKEN" ]; then
            echo "Configuring GitHub CLI for HTTPS..."
            gh auth login --with-token <<< "$GITHUB_TOKEN" 2>/dev/null || echo "GitHub CLI auth may have failed"
        fi

        gh pr create \
            --title "$PR_TITLE" \
            --body "$PR_BODY" \
            --base "feature/example-project-and-cli" \
            --head "$CURRENT_BRANCH" \
            || echo "⚠️  PR creation failed (may already exist)"
        echo "✅ Pull request created successfully"
    else
        echo "⚠️  GitHub CLI not available - skipping PR creation"
    fi
else
    echo "ℹ️  On main branch - no PR needed"
fi

echo "🎉 Documentation commit and push completed successfully!"
'