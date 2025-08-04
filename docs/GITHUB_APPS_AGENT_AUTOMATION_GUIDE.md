# GitHub Apps Agent Automation Guide

## Overview

This guide documents the complete process for creating, configuring, and managing GitHub App agents for the 5D Labs platform. Based on extensive research and testing, this covers both semi-automated and fully automated approaches.

## Current State Assessment (2025-08-04)

### ✅ What Works
- **Morgan (Docs Agent)**: Fully configured with OAuth, permissions, and repository access
- **GitHub App Creation**: Manual process through GitHub UI works reliably
- **Permission Configuration**: Can be set through GitHub UI
- **Private Key Management**: Keys stored locally and in Kubernetes secrets
- **API Actions**: GitHub Apps can perform reviews, comments, status checks
- **Workflow Integration**: Apps work with Argo Workflows and MCP server

### ❌ What Doesn't Work  
- **UI Assignment**: GitHub Apps cannot be manually assigned in UI dropdowns (by design)
- **Full API Creation**: No direct API to create GitHub Apps programmatically
- **Permission Updates via API**: Must be done through GitHub UI

### 🔄 What's Partially Automated
- **Credential Storage**: Scripts exist for Kubernetes secret management
- **Installation Management**: Can be automated via 2025 APIs
- **Permission Templates**: Can be documented and replicated

## GitHub Apps Agent Setup Process

### Phase 1: Manual GitHub App Creation

Since GitHub doesn't allow fully programmatic app creation for security reasons, this step requires manual intervention:

#### 1.1 Create GitHub App
```bash
# Navigate to GitHub Apps creation
open "https://github.com/organizations/5dlabs/settings/apps/new"
```

**Required Configuration:**
- **App Name**: `5DLabs-{AgentName}` (e.g., `5DLabs-Rex`)
- **Description**: Use agent profile description
- **Homepage URL**: `https://github.com/5dlabs/platform/blob/main/docs/agents/{agent}.md`
- **Callback URL**: `https://github.com` (for OAuth support)
- **✅ Request user authorization (OAuth) during installation**
- **Webhook**: Leave unconfigured (not needed)

#### 1.2 Set Permissions (Based on Morgan Template)
**Repository Permissions:**
```
actions: write
actions_variables: write
administration: write
attestations: write
checks: write
codespaces: write
codespaces_lifecycle_admin: write
codespaces_metadata: read
codespaces_secrets: write
codespaces_user_secrets: write
contents: write
deployments: write
discussions: write
environments: write
issues: write
issue_fields: write
issue_types: write
merge_queues: write
metadata: read
pages: write
pull_requests: write
repository_hooks: write
repository_projects: admin
secrets: write
security_events: write
statuses: write
vulnerability_alerts: write
workflows: write
```

**Organization Permissions:**
```
members: write
organization_administration: write
organization_announcement_banners: write
organization_api_insights: read
organization_campaigns: write
organization_codespaces: write
organization_codespaces_secrets: write
organization_codespaces_settings: write
organization_custom_org_roles: write
organization_custom_properties: admin
organization_custom_roles: write
organization_dependabot_secrets: write
organization_events: read
organization_hooks: write
organization_knowledge_bases: write
organization_models: read
organization_personal_access_tokens: write
organization_personal_access_token_requests: write
organization_plan: read
organization_private_registries: write
organization_projects: admin
organization_secrets: write
organization_self_hosted_runners: write
organization_user_blocking: write
```

#### 1.3 Generate and Download Private Key
- Click "Generate a private key"
- Download the `.pem` file
- Store securely for automation scripts

### Phase 2: Semi-Automated Configuration

#### 2.1 Store Credentials Using Existing Scripts
```bash
# Use the enhanced credential storage script
./scripts/store-all-agent-credentials.sh
```

This script:
- Retrieves App ID and Client ID from GitHub API
- Stores private keys in Kubernetes secrets
- Creates External Secrets for Vault integration
- Configures secrets in both `secret-store` and `agent-platform` namespaces

#### 2.2 Install App on Repository
```bash
# Semi-automated installation using 2025 APIs
./scripts/install-agent-on-repository.sh {agent-name}
```

Or manually:
1. Go to `https://github.com/organizations/5dlabs/settings/apps/{app-slug}`
2. Click "Install App"
3. Select "Only select repositories" → Add `platform`
4. Save installation

#### 2.3 Update Helm Configuration
```bash
# Add agent definition to values.yaml
./scripts/update-agent-profiles.sh
```

### Phase 3: Automation Scripts (Enhanced)

#### 3.1 Enhanced Credential Storage Script
**File**: `scripts/store-all-agent-credentials-enhanced.sh`

```bash
#!/bin/bash
# Enhanced version with API validation and error handling

AGENTS=("rex" "blaze" "cipher")
ORG="5dlabs"

for agent in "${AGENTS[@]}"; do
    app_name="5DLabs-$(echo "${agent:0:1}" | tr '[:lower:]' '[:upper:]')$(echo "${agent:1}")"
    
    # Get app details from GitHub API
    app_data=$(gh api "/orgs/${ORG}/apps" | jq -r --arg name "$app_name" '.[] | select(.name == $name)')
    
    if [[ -n "$app_data" ]]; then
        app_id=$(echo "$app_data" | jq -r '.id')
        client_id=$(echo "$app_data" | jq -r '.client_id')
        
        # Store in Kubernetes with validation
        store_credentials "$agent" "$app_id" "$client_id" "${agent}-private-key.pem"
        
        # Verify installation access
        verify_installation_access "$agent" "$app_id"
    else
        echo "❌ App $app_name not found - create manually first"
    fi
done
```

#### 3.2 Permission Validation Script
**File**: `scripts/verify-agent-permissions.sh`

```bash
#!/bin/bash
# Verify all agents have consistent permissions

AGENTS=("morgan" "rex" "blaze" "cipher")

for agent in "${AGENTS[@]}"; do
    echo "=== Checking $agent ==="
    
    # Get current permissions
    permissions=$(gh api "/orgs/5dlabs/installations" | \
        jq ".installations[] | select(.app_slug == \"5dlabs-${agent}\") | .permissions")
    
    # Compare with Morgan template
    compare_permissions "$permissions" "$morgan_template"
done
```

#### 3.3 Installation Automation (2025 APIs)
**File**: `scripts/install-agents-on-repository.sh`

```bash
#!/bin/bash
# Use 2025 installation automation APIs

ORG="5dlabs"
REPO="platform"
AGENTS=("rex" "blaze" "cipher")

for agent in "${AGENTS[@]}"; do
    # Get installation ID
    install_id=$(get_installation_id "$agent")
    
    # Add repository to installation using 2025 API
    gh api --method PUT \
        "/app/installations/${install_id}/repositories/${REPO_ID}" \
        -H "Accept: application/vnd.github.v3+json"
    
    echo "✅ Added $REPO to $agent installation"
done
```

## Automation Roadmap

### Level 1: Semi-Automated (Current)
- ✅ Manual app creation via UI
- ✅ Automated credential storage
- ✅ Automated Helm configuration
- ✅ Semi-automated installation

### Level 2: Enhanced Automation (Next Phase)
- 🔄 App manifest generation
- 🔄 Bulk permission updates via UI automation
- 🔄 Repository installation via 2025 APIs
- 🔄 Validation and verification automation

### Level 3: Full Automation (Future Goal)
- ❓ Programmatic app creation (if GitHub adds API)
- ❓ Permission management via API (if GitHub adds support)
- ❓ End-to-end agent provisioning

## File Structure and Assets

### Current Assets Available
```
/Users/jonathonfritz/platform/
├── {agent}-private-key.pem          # Private keys for Rex, Blaze, Cipher
├── {agent}-avatar.png               # Agent avatars
├── scripts/
│   ├── store-all-agent-credentials.sh
│   ├── fully-automated-github-app.py
│   └── create-agent-apps.sh
└── docs/agent-profiles.md           # Agent specifications
```

### Required Enhancements
```bash
# Create enhanced automation scripts
mkdir -p scripts/enhanced/
touch scripts/enhanced/store-credentials-with-validation.sh
touch scripts/enhanced/verify-permissions.sh  
touch scripts/enhanced/install-on-repository.sh
touch scripts/enhanced/bulk-agent-setup.sh
```

## Agent Configuration Templates

### Rex (Backend Architect)
```yaml
rex:
  name: "Rex"
  githubApp: "5DLabs-Rex"
  appId: "TBD"  # Retrieved via API
  clientId: "TBD"  # Retrieved via API
  role: "Senior Backend Architect & Systems Engineer"
  model: "claude-sonnet-4-20250514"
  expertise: ["backend", "architecture", "systems", "apis", "databases"]
  description: "Senior Backend Architect specializing in distributed systems"
```

### Blaze (Performance Specialist)
```yaml
blaze:
  name: "Blaze"
  githubApp: "5DLabs-Blaze"
  appId: "TBD"
  clientId: "TBD"
  role: "Performance Engineer & Optimization Specialist"
  model: "claude-sonnet-4-20250514"
  expertise: ["performance", "optimization", "profiling", "caching"]
  description: "Performance optimization specialist focused on speed"
```

### Cipher (Security Specialist)
```yaml
cipher:
  name: "Cipher"
  githubApp: "5DLabs-Cipher"
  appId: "TBD"
  clientId: "TBD"
  role: "Security Engineer & Code Analysis Specialist"
  model: "claude-sonnet-4-20250514"
  expertise: ["security", "analysis", "authentication", "compliance"]
  description: "Security engineer focused on building secure systems"
```

## Testing and Validation

### Agent Functionality Tests
```bash
# Test each agent after setup
for agent in rex blaze cipher; do
    echo "Testing $agent..."
    
    # Test bot account accessibility
    gh api "/users/5dlabs-${agent}[bot]" >/dev/null && echo "✅ Bot account" || echo "❌ Bot account"
    
    # Test repository installation
    gh api "/repos/5dlabs/platform/installation" | \
        jq -e ".app_slug == \"5dlabs-${agent}\"" && echo "✅ Repository access" || echo "❌ Repository access"
    
    # Test MCP integration
    test_mcp_integration "$agent"
done
```

### Integration Testing
```bash
# Test full workflow integration
./test-agent-workflow.sh rex    # Test code workflow
./test-agent-workflow.sh morgan  # Test docs workflow
```

## Troubleshooting Guide

### Common Issues and Solutions

#### App Creation Issues
- **Issue**: Permission denied during creation
- **Solution**: Ensure org owner permissions, check OAuth restrictions

#### Installation Issues  
- **Issue**: App shows installed but API says no access
- **Solution**: Wait 15-30 minutes for propagation, verify repository selection

#### Credential Issues
- **Issue**: Private key authentication fails
- **Solution**: Verify key format, check Kubernetes secret encoding

#### Permission Issues
- **Issue**: API calls return 403 forbidden
- **Solution**: Compare permissions with Morgan, update missing permissions

## Security Considerations

### Private Key Management
- Store private keys in Kubernetes secrets only
- Use External Secrets for Vault integration
- Never commit private keys to git
- Rotate keys periodically

### Permission Principle
- Use minimum required permissions
- Regular permission audits
- Document permission rationale
- Test with reduced permissions first

### Access Control
- Limit repository access to necessary repos only
- Use organization-level restrictions when appropriate
- Monitor app usage and access patterns
- Regular security reviews

## Future Enhancements

### API Improvements (Waiting on GitHub)
- Programmatic app creation API
- Permission management API
- Bulk installation management
- Enhanced webhook configuration

### Platform Enhancements
- Agent health monitoring
- Performance metrics collection
- Automated permission auditing
- Cost tracking and optimization

### Workflow Integration
- Smart agent assignment based on task type
- Load balancing across agents
- Failure handling and retries
- Metrics and observability

---

*This guide will be updated as GitHub APIs evolve and new automation capabilities become available.*