# Claude Agent Environment Issues and Proposed Remediations

## Executive Summary
During testing of Claude agent task 9999, several environment configuration issues were identified. Analysis shows Claude was able to successfully use tools despite configuration anomalies, but other issues impacted the overall experience.

## Issues Identified

### 1. **Claude Configuration Overwrite (Needs Investigation)**
**Issue**: The `.claude.json` file shows different content between init and main container
- **Init Container**: Created config with `"allowedTools": ["Bash", "Edit", "Read", "Write", "Glob", "Grep"]`
- **Main Container**: Shows config with `"allowedTools": []` (empty array)
- **However**: Claude successfully used tools (TodoWrite, Bash, Edit) despite empty array

**Investigation Needed**: 
- Claude Code may be using different configuration than `.claude.json`
- May need to specify configuration via CLI flags
- The empty allowedTools might not actually prevent tool usage

**Root Cause**: Unknown - requires further investigation of Claude Code's configuration loading

### 2. **Export Script HTML Entity Encoding**
**Issue**: The `export-session.sh` script has HTML entities instead of proper bash syntax
- `&#x3D;` instead of `=`
- `&quot;` instead of `"`
- `&#x27;` instead of `'`

**Impact**: Script fails with syntax errors when Claude session ends

**Root Cause**: Handlebars templating is HTML-encoding the script content

### 3. **Workspace Not Clean Between Runs**
**Issue**: Previous run artifacts persisted:
- `.backup/` directory with old task files
- Existing `test-results-task-9999.md` and `workspace-snapshot.md`
- Repository on branch `test/task-9999-workspace-verification-v2` instead of `main`

**Impact**: Claude sees files from previous runs, potentially confusing task context

**Root Cause**: PVC not being cleaned between runs

### 4. **Repository State Management Issues**
**Issue**: Init container reports "Different repository detected" and authentication errors
- Line 12: "Different repository detected, updating remote and fetching..."
- Line 13: "fatal: could not read Username for 'https://github.com': No such device or address"

**Impact**: Potential issues with repository state and git operations

**Root Cause**: Previous repository state persisting in workspace

### 5. **GitHub Token Exposure**
**Issue**: GitHub PAT was written to `.github-env` file in plain text
- Claude attempted to commit this file
- GitHub push protection blocked the push

**Impact**: Security risk and blocked PR creation

### 6. **Init Container Complexity**
**Question**: Is having separate init and main containers adding unnecessary complexity?
- Different configurations between containers
- Potential for state inconsistencies
- May be simpler to do everything in main container

**Consideration**: Since we can run commands before starting Claude in the main container, the init container pattern might be adding complexity without clear benefits

## Proposed Remediations

### 1. **Investigate Claude Configuration Management**
```bash
# Need to investigate:
# 1. How Claude Code loads configuration
# 2. Whether CLI flags can override config
# 3. If allowedTools being empty actually matters

# Potential solutions to test:
# - Use --allowedTools CLI flag when starting Claude
# - Check if settings.local.json takes precedence
# - Verify if Claude Code has different config locations
```

### 2. **Fix Export Script Encoding**
**Test locally first** since we're running Claude Code:
```bash
# Create test script and check if we see HTML encoding
cat > test-export.sh << 'EOF'
#!/bin/bash
HOOK_INPUT=$(cat)
echo "Test quotes"
echo 'Test single quotes'
EOF

# If no encoding issues locally, problem is in Handlebars templating
```

In the orchestrator template rendering:
```rust
// Add to template context or use raw block
// Option A: Use Handlebars raw block
{{{{raw}}}}
#!/bin/bash
HOOK_INPUT=$(cat)
# ... rest of script
{{{{/raw}}}}

// Option B: Disable HTML escaping for this template
handlebars.register_escape_fn(handlebars::no_escape);
```

### 3. **Implement Comprehensive Workspace Cleanup**
Use the comprehensive script at `/scripts/clean-workspace-and-test.sh` that:
- Fully cleans the PVC including hidden folders
- Creates a fresh repository from https://github.com/5dlabs/agent-template
- Waits for GitHub Actions build to complete
- Restarts the orchestrator
- Submits the test job with correct repository URL

**Key features**:
```bash
# Complete PVC cleanup using Kubernetes Job
kubectl apply -f - <<EOF
apiVersion: batch/v1
kind: Job
spec:
  template:
    spec:
      containers:
      - name: cleaner
        image: busybox
        command: ["find", "/workspace", "-mindepth", "1", "-delete"]
EOF

# Fresh repo creation from template
gh repo create "${ORG}/${REPO}" --template="${ORG}/agent-template"

# Wait for builds before proceeding
gh run watch --exit-status
```

### 4. **Fix Repository Clone Logic**
```bash
# Force fresh clone instead of trying to update
if [ -d "/workspace/.git" ]; then
  echo "Removing existing repository..."
  rm -rf /workspace/.git
fi

# Clone fresh
git clone --branch "${REPOSITORY_BRANCH}" "${REPOSITORY_URL}" /tmp/repo
mv /tmp/repo/* /tmp/repo/.* /workspace/ 2>/dev/null || true
```

### 5. **Secure GitHub Token Handling**
Simple fix - add to `.gitignore` in the agent-template repository:
```gitignore
# GitHub environment file with sensitive data
.github-env
.git-credentials

# Other sensitive files
*.pem
*.key
.env*
```

This prevents Claude from accidentally committing sensitive files.

## Implementation Priority
1. **High**: Fix export script encoding (Issue #2) - Prevents session export
2. **High**: Add .github-env to .gitignore (Issue #5) - Quick fix that prevents PR blocking
3. **Medium**: Implement workspace cleanup script (Issue #3) - Ensures clean testing environment
4. **Medium**: Fix repository clone logic (Issue #4) - Current workaround exists
5. **Low**: Investigate Claude configuration (Issue #1) - Not blocking since tools still work
6. **Future**: Consider removing init container (Issue #6) - Architectural decision

## Testing Checklist for Clean Slate
- [ ] Run `/scripts/clean-workspace-and-test.sh` for automated clean testing
- [ ] Verify PVC is completely empty before starting
- [ ] Confirm fresh repository created from agent-template
- [ ] Check that .github-env is in .gitignore
- [ ] Verify export-session.sh has proper bash syntax (no HTML entities)
- [ ] Test that Claude can use all expected tools
- [ ] Verify session export works when task completes
- [ ] Confirm no push blocking due to secrets

## Quick Start Testing
```bash
# Make script executable
chmod +x /Users/jonathonfritz/platform/scripts/clean-workspace-and-test.sh

# Run comprehensive clean test
./scripts/clean-workspace-and-test.sh
```