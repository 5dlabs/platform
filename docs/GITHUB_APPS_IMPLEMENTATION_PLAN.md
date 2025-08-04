# GitHub Apps Implementation Plan - 2025-08-04

## Overview
Complete the migration from GitHub user authentication to GitHub Apps for all AI agents, enabling secure, scalable agent operations with proper permissions and authentication.

## Current State Analysis

### ✅ **What's Working (Docs Workflow)**
- **MCP Server**: Full GitHub Apps support for `docs()` function with agent name resolution
- **DocsRun Template**: Updated with `github-app` parameter and GitHub App authentication  
- **Morgan Agent**: Fully configured with GitHub App `5DLabs-Morgan` (App ID: 1723711)
- **Agent Resolution**: Smart resolution by name ("Morgan"), key ("morgan"), or GitHub App ("5DLabs-Morgan")
- **Secrets Management**: External Secrets integration working for Morgan

### ❌ **What's Missing (Code Workflow)**

#### 1. CodeRun Template - Missing GitHub App Support
**File**: `infra/workflow-templates/coderun-template.yaml`
- Still uses `github-user` parameter only (line 34)
- No `github-app` parameter or GitHub App authentication
- Uses deprecated user-based authentication

#### 2. MCP Server - Task Function Incomplete  
**File**: `controller/mcp/src/main.rs`
- `handle_task_workflow()` doesn't have GitHub App resolution like docs workflow
- Still requires `github_user` parameter (line 189)
- No agent-based GitHub App selection

#### 3. Additional Agents - Need Setup
- **Rex** (Backend): App exists, credentials need to be stored
- **Blaze** (Performance): App exists, credentials need to be stored
- **Cipher** (Security): App exists, credentials need to be stored
- All have private keys available but need Kubernetes secret creation

## Implementation Plan

### **Phase 1: Complete GitHub Apps Support for Code Tasks** 🔧

#### Task 1.1: Update CodeRun Workflow Template
**File**: `infra/workflow-templates/coderun-template.yaml`

**Changes Needed**:
```yaml
# Add github-app parameter (similar to docsrun-template.yaml:25-27)
- name: github-app
  description: "GitHub App name for authentication (e.g., 'Rex', 'Blaze', '5DLabs-Rex')"
  default: "5DLabs-Rex"  # Default to Rex for code tasks

# Mark github-user as deprecated (similar to docsrun-template.yaml:23-24)  
- name: github-user
  description: "GitHub username for commits and authentication (deprecated - use github-app)"
  default: ""

# Update CRD manifest to include githubApp field
spec:
  githubApp: "{{workflow.parameters.github-app}}"
  # Keep githubUser for backward compatibility during transition
  githubUser: "{{workflow.parameters.github-user}}"
```

#### Task 1.2: Update MCP Server Task Function
**File**: `controller/mcp/src/main.rs`

**Changes Needed**:
```rust
// Add GitHub App resolution to handle_task_workflow() (mirror handle_docs_workflow lines 113-129)
let github_app = if let Some(input) = arguments.get("github_app").and_then(|v| v.as_str()) {
    // Try to resolve agent name (e.g., "Rex" -> "5DLabs-Rex")
    if let Some(agent) = agents_config.resolve_agent(input) {
        agent.github_app.clone()
    } else {
        input.to_string() // Use as-is if not found
    }
} else if let Ok(env_app) = std::env::var("FDL_DEFAULT_GITHUB_APP") {
    env_app
} else if let Some(default_agent) = agents_config.get_code_agent() {
    default_agent.github_app.clone()
} else {
    return Err(anyhow!("No GitHub App configured for code workflow and no default code agent found"));
};

// Add to workflow parameters
params.push(format!("github-app={github_app}"));
```

#### Task 1.3: Update Tool Schema
**File**: `controller/mcp/src/tools.rs`

**Add github_app parameter to task tool schema** (similar to docs tool):
```rust
"github_app": {
    "type": "string",
    "description": "GitHub App name or agent name (e.g., 'Rex', 'Blaze', '5DLabs-Rex')"
}
```

### **Phase 2: Complete Agent Setup** 👥

#### Task 2.1: Update Helm Values
**File**: `infra/charts/controller/values.yaml`

**Add agent definitions** (after morgan section, line 61):
```yaml
  rex:
    name: "Rex"
    githubApp: "5DLabs-Rex"
    appId: "TBD"  # Get from GitHub API
    clientId: "TBD"  # Get from GitHub API
    role: "Senior Backend Architect & Systems Engineer"
    model: "claude-sonnet-4-20250514"
    expertise: ["backend", "architecture", "systems", "apis", "databases"]
    description: "Senior Backend Architect specializing in distributed systems and high-performance infrastructure"
    
  blaze:
    name: "Blaze"
    githubApp: "5DLabs-Blaze"
    appId: "TBD"  # Get from GitHub API
    clientId: "TBD"  # Get from GitHub API
    role: "Performance Engineer & Optimization Specialist"
    model: "claude-sonnet-4-20250514"
    expertise: ["performance", "optimization", "profiling", "caching"]
    description: "Performance optimization specialist focused on making systems blazingly fast"
    
  cipher:
    name: "Cipher"
    githubApp: "5DLabs-Cipher"
    appId: "TBD"  # Get from GitHub API
    clientId: "TBD"  # Get from GitHub API
    role: "Security Engineer & Code Analysis Specialist"  
    model: "claude-sonnet-4-20250514"
    expertise: ["security", "analysis", "authentication", "compliance"]
    description: "Security engineer focused on building secure, resilient systems"

# Update defaults
defaults:
  docsAgent: "morgan"
  codeAgent: "rex"  # Change from morgan to rex
```

#### Task 2.2: Store Agent Credentials
**Script**: `scripts/store-all-agent-credentials.sh`

**Actions**:
1. Run the existing script to store credentials for Rex, Blaze, and Cipher
2. Script will:
   - Look up App IDs and Client IDs from GitHub API
   - Create External Secrets for each agent
   - Store private keys in Kubernetes secrets

**Command**:
```bash
cd /Users/jonathonfritz/platform
./scripts/store-all-agent-credentials.sh
```

#### Task 2.3: Update Agent Configuration
**File**: `controller/config/agents.yaml` (if exists) or ConfigMap

**Ensure all agents are available** in the MCP server agent configuration.

### **Phase 3: Testing & Validation** ✅

#### Task 3.1: Integration Testing
**Test both workflows**:
```javascript
// Test docs workflow with Morgan
docs({
  working_directory: "_projects/test-project",
  github_app: "Morgan"  // Test agent name resolution
});

// Test code workflow with Rex  
task({
  task_id: 1,
  service: "test-service",
  repository: "5dlabs/platform",
  docs_repository: "5dlabs/docs",
  docs_project_directory: "_projects/test",
  github_app: "Rex"  // Test agent name resolution
});
```

#### Task 3.2: Validation Checklist
- [ ] All four agents (Morgan, Rex, Blaze, Cipher) have stored credentials
- [ ] MCP server can resolve agent names to GitHub Apps
- [ ] Both `docs()` and `task()` functions support GitHub Apps
- [ ] Workflow templates create proper CRDs with GitHub App authentication
- [ ] External Secrets are syncing properly

### **Phase 4: Documentation & Cleanup** 📚

#### Task 4.1: Update Documentation
**Files to update**:
- `README.md` - Update MCP tool examples to use GitHub Apps
- `docs/agent-profiles.md` - Ensure all agents are documented
- `infra/charts/controller/README.md` - Update deployment instructions

#### Task 4.2: Deprecation Notices
- Add deprecation warnings for `github_user` parameters
- Update examples to use `github_app` parameters
- Plan removal timeline for user-based authentication

## Technical Details

### GitHub App vs User Authentication
**Benefits of GitHub Apps**:
- Fine-grained permissions per repository
- Higher rate limits (5000 requests/hour vs 1000)
- No dependency on personal user accounts
- Audit trail of app actions
- Automatic token refresh

### Agent Resolution Logic
The MCP server supports flexible agent resolution:
1. **Agent Key**: `"rex"` → Rex agent
2. **Agent Name**: `"Rex"` → Rex agent  
3. **GitHub App**: `"5DLabs-Rex"` → Rex agent
4. **Fallback**: Environment variable or default agent

### Security Model
- Private keys stored in Kubernetes secrets
- External Secrets for credential management
- App-level permissions per repository
- Automatic JWT token generation

## Files Modified

### Core Implementation
- `infra/workflow-templates/coderun-template.yaml` - GitHub App support
- `controller/mcp/src/main.rs` - Agent resolution for task workflow
- `controller/mcp/src/tools.rs` - Tool schema updates
- `infra/charts/controller/values.yaml` - Agent definitions

### Configuration
- `controller/config/agents.yaml` - Agent configuration (if exists)
- Kubernetes secrets via External Secrets

### Documentation  
- `README.md` - Updated examples
- `docs/agent-profiles.md` - Complete agent documentation

## Success Criteria

### Must Have
- [x] Morgan docs workflow working with GitHub Apps
- [ ] Rex code workflow working with GitHub Apps  
- [ ] All four agents have stored credentials
- [ ] MCP server resolves all agent names properly

### Should Have
- [ ] Backward compatibility during transition period
- [ ] Complete documentation updates
- [ ] Integration test coverage

### Nice to Have
- [ ] Automated agent provisioning pipeline
- [ ] Performance metrics for GitHub App authentication
- [ ] Agent assignment based on task type/complexity

## Risk Mitigation

### Potential Issues
1. **GitHub API Rate Limits**: Mitigated by GitHub App higher limits
2. **Private Key Management**: Using External Secrets for secure storage
3. **Backward Compatibility**: Maintaining dual parameter support during transition
4. **Agent Resolution Conflicts**: Clear precedence order in resolution logic

### Rollback Plan
1. Revert MCP server changes to user-based authentication
2. Update workflow templates to remove GitHub App parameters
3. Fall back to existing user-based secrets

## Timeline

**Today (2025-08-04)**:
- Phase 1: Complete code workflow GitHub Apps support (2-3 hours)
- Phase 2: Set up remaining agents (1-2 hours)  
- Phase 3: Testing and validation (1 hour)

**This Week**:
- Phase 4: Documentation and cleanup
- Monitor and optimize GitHub App usage

---

*This plan completes the GitHub Apps migration started with Morgan, extending it to all agents and workflows for a unified, secure authentication system.*