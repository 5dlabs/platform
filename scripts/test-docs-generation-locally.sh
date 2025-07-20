#!/bin/bash
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🧪 Testing Docs Generation Authentication Locally${NC}"
echo "=================================================="

# Check for required environment variables
REQUIRED_VARS=("GITHUB_TOKEN" "GITHUB_USER")
MISSING_VARS=""

for var in "${REQUIRED_VARS[@]}"; do
    if [ -z "${!var:-}" ]; then
        MISSING_VARS="$MISSING_VARS $var"
    fi
done

if [ -n "$MISSING_VARS" ]; then
    echo -e "${RED}❌ Missing required environment variables:${NC}"
    for var in $MISSING_VARS; do
        echo "  - $var"
    done
    echo ""
    echo "Please set them and try again:"
    echo "  export GITHUB_TOKEN='your_token_here'"
    echo "  export GITHUB_USER='your_username_here'"
    exit 1
fi

# Configuration
CLAUDE_CODE_IMAGE="${CLAUDE_CODE_IMAGE:-ghcr.io/5dlabs/platform/claude-code:latest}"

# Test repository configuration
TEST_REPO_URL="${TEST_REPO_URL:-https://github.com/5dlabs/agent-sandbox}"
TEST_REPO_BRANCH="${TEST_REPO_BRANCH:-main}"

echo -e "${GREEN}✓ Environment variables found${NC}"
echo "  GITHUB_USER: $GITHUB_USER"
echo "  GITHUB_TOKEN: ${GITHUB_TOKEN:0:8}..."
echo "  Claude Code image: $CLAUDE_CODE_IMAGE"
echo "  Test repo: $TEST_REPO_URL"
echo "  Test branch: $TEST_REPO_BRANCH"
echo ""

# Step 1: Pull the Claude Code image
echo -e "${YELLOW}Step 1: Pulling Claude Code Docker image...${NC}"
if ! docker pull "$CLAUDE_CODE_IMAGE"; then
    echo -e "${RED}❌ Failed to pull Claude Code image${NC}"
    echo "You may need to log in to the container registry:"
    echo "  docker login ghcr.io"
    exit 1
fi

echo -e "${GREEN}✓ Claude Code image pulled successfully${NC}"

# Step 2: Create test configuration files
echo -e "${YELLOW}Step 2: Creating test configuration files...${NC}"

# Create temporary directory for test files
TEST_DIR="/tmp/orchestrator-test-$(date +%s)"
mkdir -p "$TEST_DIR"

# Create a minimal CLAUDE.md file
cat > "$TEST_DIR/CLAUDE.md" << 'EOF'
# Test Documentation Generation

This is a test run to verify GitHub authentication and repository access.

## Tasks
1. Verify GitHub authentication works
2. Test repository cloning
3. Confirm write permissions

## Expected Outcome
The authentication tests should pass and the container should be ready to run Claude.
EOF

# Create a test stop hook script
cat > "$TEST_DIR/.stop-hook-docs-pr.sh" << 'EOF'
#!/bin/sh
echo "Test stop hook executed successfully"
echo "In a real scenario, this would create a PR with generated docs"
EOF
chmod +x "$TEST_DIR/.stop-hook-docs-pr.sh"

# Create test Claude settings
cat > "$TEST_DIR/claude-settings.json" << 'EOF'
{
  "model": "claude-3-5-sonnet-20241022",
  "tools": {
    "computer": false,
    "bash": true,
    "editor": true
  },
  "maxTokens": 8000,
  "temperature": 0.0,
  "acceptEdits": true
}
EOF

echo -e "${GREEN}✓ Test configuration files created in: $TEST_DIR${NC}"

# Step 3: Create authentication test script
echo -e "${YELLOW}Step 3: Creating authentication test script...${NC}"

# Create a test script that will be passed to the container
cat > "$TEST_DIR/test-script.sh" << EOF
#!/bin/sh

echo '════════════════════════════════════════════════════════════════'
echo '║                 DOCS GENERATION TESTING                      ║'
echo '════════════════════════════════════════════════════════════════'

# Disable interactive Git prompts globally
export GIT_TERMINAL_PROMPT=0
export GIT_ASKPASS=/bin/true
export SSH_ASKPASS=/bin/true

# Setup authentication (HTTPS mode)
echo "=== HTTPS AUTHENTICATION SETUP ==="

if [ -n "\$GITHUB_TOKEN" ] && [ -n "\$GITHUB_USER" ]; then
    echo "✓ GitHub environment variables found"

    # Configure git user FIRST
    git config --global user.name "\$GITHUB_USER"
    git config --global user.email "\${GITHUB_USER}@users.noreply.github.com"

    # Disable interactive prompts for git operations
    git config --global credential.helper ""
    git config --global core.askPass ""

    # Create credentials file with proper format
    mkdir -p /root/.git-credentials-dir
    echo "https://\${GITHUB_USER}:\${GITHUB_TOKEN}@github.com" > /root/.git-credentials
    chmod 600 /root/.git-credentials

    # Configure git to use the credentials file
    git config --global credential.helper 'store --file=/root/.git-credentials'

    # Also set up workspace-specific credentials (for compatibility)
    echo "https://\${GITHUB_USER}:\${GITHUB_TOKEN}@github.com" > /workspace/.git-credentials
    chmod 600 /workspace/.git-credentials

    echo "✓ Git credentials configured for user: \$GITHUB_USER"
else
    echo "❌ Missing GITHUB_TOKEN or GITHUB_USER environment variables"
    exit 1
fi

# =============================================================================
# COMPREHENSIVE GITHUB AUTHENTICATION TESTING
# =============================================================================
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "║                 GITHUB AUTHENTICATION TESTING                ║"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Test 1: Basic GitHub API access
echo "=== TEST 1: GitHub API Authentication ==="
if [ -n "\$GITHUB_TOKEN" ]; then
  echo "Testing GitHub API access..."
  API_RESPONSE=\$(curl -s -H "Authorization: token \$GITHUB_TOKEN" https://api.github.com/user)
  if echo "\$API_RESPONSE" | grep -q '"login"'; then
    API_USER=\$(echo "\$API_RESPONSE" | grep '"login"' | sed 's/.*"login": *"\([^"]*\)".*/\1/')
    echo "✓ GitHub API authentication successful"
    echo "  Authenticated as: \$API_USER"

    # Verify the API user matches the configured user
    if [ "\$API_USER" = "\$GITHUB_USER" ]; then
      echo "✓ API user matches configured GITHUB_USER"
    else
      echo "⚠️  WARNING: API user (\$API_USER) differs from GITHUB_USER (\$GITHUB_USER)"
      echo "   This may cause permission issues"
    fi
  else
    echo "❌ GitHub API authentication failed"
    echo "   Response: \$API_RESPONSE"
    exit 1
  fi
else
  echo "❌ No GITHUB_TOKEN available for API testing"
  exit 1
fi

# Test 2: Repository access permissions
echo ""
echo "=== TEST 2: Repository Access Verification ==="
REPO_URL="$TEST_REPO_URL"
# Extract owner/repo from URL
if echo "\$REPO_URL" | grep -q "github.com"; then
  REPO_PATH=\$(echo "\$REPO_URL" | sed 's|.*github.com[/:]||' | sed 's|\.git\$||')
  echo "Testing repository access for: \$REPO_PATH"

  REPO_RESPONSE=\$(curl -s -H "Authorization: token \$GITHUB_TOKEN" "https://api.github.com/repos/\$REPO_PATH")
  if echo "\$REPO_RESPONSE" | grep -q '"full_name"'; then
    echo "✓ Repository is accessible via API"

    # Check permissions
    PERMISSIONS=\$(echo "\$REPO_RESPONSE" | grep -o '"permissions":{[^}]*}' || echo "")
    if echo "\$PERMISSIONS" | grep -q '"push":true'; then
      echo "✓ Push permissions confirmed"
    else
      echo "⚠️  WARNING: Push permissions may be limited"
      echo "   Permissions: \$PERMISSIONS"
    fi
  else
    echo "❌ Repository not accessible via API"
    echo "   Response: \$REPO_RESPONSE"
    exit 1
  fi
else
  echo "⚠️  Non-GitHub repository detected, skipping API permission check"
fi

# Test 3: Git credential functionality
echo ""
echo "=== TEST 3: Git Credential Testing ==="
echo "Testing git credential configuration..."

# Show current git config
echo "Git configuration:"
git config --global --list | grep -E "(user\.|credential\.|core\.askPass)" || echo "No relevant git config found"

# Test git credential helper
echo ""
echo "Testing credential helper..."
echo "Testing HTTPS credential helper..."
# Create a minimal test to verify credentials work
TEST_REPO_URL="https://github.com/\$REPO_PATH.git"
echo "Testing with URL: \$TEST_REPO_URL"

# Test git ls-remote (read operation)
if git ls-remote "\$TEST_REPO_URL" HEAD >/dev/null 2>&1; then
  echo "✓ Git HTTPS authentication working (read access confirmed)"
else
  echo "❌ Git HTTPS authentication failed"
  echo "   Testing credential helper directly..."

  # Debug credential helper
  echo "url=https://github.com" | git credential fill 2>&1 || echo "Credential helper test completed"

  echo ""
  echo "🚫 ABORTING: Git authentication not working"
  exit 1
fi

# Test 4: Repository cloning
echo ""
echo "=== TEST 4: Repository Cloning Test ==="
echo "Attempting to clone test repository..."

cd /workspace
if git clone --depth 1 --branch "$TEST_REPO_BRANCH" "\$REPO_URL" test-clone; then
  echo "✓ Repository cloned successfully"
  cd test-clone
  echo "✓ Repository contents:"
  ls -la | head -10

  # Test 4.5: Working Directory Credentials Test
  echo ""
  echo "=== TEST 4.5: Working Directory Credentials Test ==="
  echo "Testing Git credentials from a subdirectory (simulating Claude's working directory)..."

  # Create a subdirectory to simulate Claude's working directory
  mkdir -p test-service
  cd test-service
  echo "Current directory: \$(pwd)"

  # Test git operations from subdirectory
  echo "Testing git operations from subdirectory..."
  if git status >/dev/null 2>&1; then
    echo "✓ Git operations work from subdirectory"
  else
    echo "❌ Git operations failed from subdirectory"
    echo "Setting up local git credentials in working directory..."

    # Copy credentials to current directory
    echo "https://\${GITHUB_USER}:\${GITHUB_TOKEN}@github.com" > .git-credentials
    chmod 600 .git-credentials
    git config credential.helper 'store --file=.git-credentials'

    if git status >/dev/null 2>&1; then
      echo "✓ Git operations work after setting local credentials"
    else
      echo "❌ Git operations still failing from subdirectory"
      exit 1
    fi
  fi

  # Test git remote operations from subdirectory
  echo "Testing git remote operations from subdirectory..."
  if git fetch origin >/dev/null 2>&1; then
    echo "✓ Git remote operations work from subdirectory"
  else
    echo "❌ Git remote operations failed from subdirectory"
    echo "This could cause issues when Claude tries to push from working directory"
    exit 1
  fi

  # Go back to repository root for the write test
  cd ..

  # Test 5: Write access test
  echo ""
  echo "=== TEST 5: Write Access Test ==="
  echo "Creating test branch to verify write permissions..."

  TEST_BRANCH="auth-test-\$(date +%s)"
  echo "Test branch: \$TEST_BRANCH"

  if git checkout -b "\$TEST_BRANCH"; then
    echo "✓ Created test branch locally"

    # Create a small test file
    echo "# Authentication Test" > ".auth-test-\$(date +%s).md"
    echo "This file was created to test GitHub write access." >> ".auth-test-\$(date +%s).md"
    echo "Generated at: \$(date)" >> ".auth-test-\$(date +%s).md"

    if git add ".auth-test-"*.md && git commit -m "Test: Verify GitHub write access"; then
      echo "✓ Created test commit"

      # Try to push the test branch
      if git push origin "\$TEST_BRANCH"; then
        echo "✓ Successfully pushed test branch to GitHub"
        echo "✓ Write access confirmed!"

        # Test 5.5: Push from subdirectory
        echo ""
        echo "=== TEST 5.5: Push from Subdirectory Test ==="
        echo "Testing push operations from subdirectory (simulating Claude's workflow)..."

        # Create another test file from subdirectory
        cd test-service
        echo "Current directory: \$(pwd)"
        echo "# Subdirectory Test" > "../.auth-test-subdir-\$(date +%s).md"
        echo "This file was created from a subdirectory." >> "../.auth-test-subdir-\$(date +%s).md"

        # Try git operations from subdirectory
        if git add "../.auth-test-subdir-"*.md && git commit -m "Test: Verify subdirectory git operations"; then
          echo "✓ Created commit from subdirectory"

          if git push origin "\$TEST_BRANCH"; then
            echo "✓ Successfully pushed from subdirectory"
            echo "✓ Subdirectory git operations confirmed!"
          else
            echo "❌ Failed to push from subdirectory"
            echo "   This could cause issues when Claude tries to create PRs"
            exit 1
          fi
        else
          echo "❌ Failed to create commit from subdirectory"
          exit 1
        fi

        cd .. # Back to repo root

        # Clean up test branch immediately
        echo "Cleaning up test branch..."
        git push origin --delete "\$TEST_BRANCH" 2>/dev/null || echo "Test branch cleanup completed"
        rm -f ".auth-test-"*.md 2>/dev/null || true

        echo "✓ Test branch cleaned up"
      else
        echo "❌ Failed to push test branch to GitHub"
        echo "   This indicates insufficient write permissions"
        exit 1
      fi
    else
      echo "❌ Failed to create test commit"
      exit 1
    fi
  else
    echo "❌ Failed to create test branch"
    exit 1
  fi
else
  echo "❌ Failed to clone repository"
  exit 1
fi

echo ""
echo "✅ ALL GITHUB AUTHENTICATION TESTS PASSED"
echo "   • API access: ✓"
echo "   • Repository access: ✓"
echo "   • Git credentials: ✓"
echo "   • Repository cloning: ✓"
echo "   • Subdirectory git operations: ✓"
echo "   • Write permissions: ✓"
echo "   • Subdirectory push operations: ✓"
echo ""
echo "🚀 Authentication testing complete!"
echo ""
echo "In a real docs generation run, Claude would now start with:"
echo "  - Verified GitHub access"
echo "  - Cloned repository with .taskmaster directory"
echo "  - Configured credentials for PR creation"
echo ""
echo "🎯 TEST SUMMARY: GitHub authentication is working correctly!"
EOF

chmod +x "$TEST_DIR/test-script.sh"

echo -e "${GREEN}✓ Authentication test script created${NC}"

# Step 4: Run the container with authentication testing
echo ""
echo -e "${BLUE}Running authentication test in Claude Code container...${NC}"
echo "This will test all GitHub authentication steps without starting Claude"
echo ""

# Run the test in the Claude Code container (which has git, curl, etc.)
if docker run --rm \
    -e GITHUB_TOKEN="$GITHUB_TOKEN" \
    -e GITHUB_USER="$GITHUB_USER" \
    -v "$TEST_DIR:/config:ro" \
    -v "/tmp/test-workspace:/workspace" \
    --name orchestrator-auth-test \
    --user root \
    "$CLAUDE_CODE_IMAGE" \
    /bin/sh /config/test-script.sh; then

    echo ""
    echo -e "${GREEN}🎉 SUCCESS: All authentication tests passed!${NC}"
    echo ""
    echo "The docs generation container should now work correctly with:"
    echo "  ✓ GitHub API access"
    echo "  ✓ Repository cloning"
    echo "  ✓ Git credentials"
    echo "  ✓ Write permissions"
    echo ""
    echo "You can now deploy this version with confidence."
    echo ""
    echo -e "${BLUE}Next steps:${NC}"
    echo "1. The updated docs-generation-container.sh.hbs template is ready"
    echo "2. Deploy the orchestrator with the new authentication logic"
    echo "3. Test with a real docs generation task (task_id 999999)"

else
    echo ""
    echo -e "${RED}❌ FAILURE: Authentication tests failed${NC}"
    echo ""
    echo "Please check the output above for specific issues."
    echo "Common problems:"
    echo "  • Invalid GitHub token"
    echo "  • Insufficient repository permissions"
    echo "  • Network connectivity issues"
    echo "  • Container registry access issues"
    exit 1
fi

# Cleanup
echo ""
echo -e "${YELLOW}Cleaning up...${NC}"
rm -rf "$TEST_DIR"

echo -e "${GREEN}✓ Cleanup complete${NC}"
echo ""
echo -e "${BLUE}🚀 Ready to deploy the updated docs generation container!${NC}"