# MCP Tools Template Design

## Problem Statement

Claude Code CRD tasks need to understand what MCP tools are available to them, but currently this information is not dynamically provided. This leads to:

- Claude not knowing what capabilities it has
- Suboptimal tool usage
- Manual documentation that gets out of sync
- Inability to adapt behavior based on available tools

## Proposed Solution: ConfigMap Tool Catalog

### Core Approach

Store a comprehensive catalog of both local and remote MCP tools in the Helm chart ConfigMap. This catalog would be rendered into Claude's memory to inform it of exact capabilities.

### Implementation Strategy

#### 1. Tool Catalog Structure
```yaml
# In values.yaml or dedicated config
mcpTools:
  local:
    - name: "bash"
      description: "Execute shell commands"
      category: "system"
      riskLevel: "high"
      examples: ["ls -la", "cd /workspace", "npm install"]

    - name: "edit"
      description: "Edit files with diff-based changes"
      category: "file"
      riskLevel: "medium"
      examples: ["edit src/index.js", "modify configuration"]

    - name: "read"
      description: "Read file contents"
      category: "file"
      riskLevel: "low"
      examples: ["read package.json", "view source code"]

  remote:
    - name: "github_create_issue"
      description: "Create GitHub issues"
      category: "github"
      riskLevel: "medium"
      requirements: ["github_token"]

    - name: "rustdocs_query"
      description: "Query Rust documentation"
      category: "documentation"
      riskLevel: "low"
```

#### 2. Template Generation
```handlebars
# Available MCP Tools

You have access to the following tools:

## Local Tools
{{#each mcpTools.local}}
### {{name}}
**Description:** {{description}}
**Category:** {{category}}
**Risk Level:** {{riskLevel}}
{{#if examples}}
**Examples:** {{join examples ", "}}
{{/if}}

{{/each}}

## Remote Tools
{{#each mcpTools.remote}}
### {{name}}
**Description:** {{description}}
**Category:** {{category}}
**Risk Level:** {{riskLevel}}
{{#if requirements}}
**Requirements:** {{join requirements ", "}}
{{/if}}

{{/each}}

## Tool Usage Guidelines
- Prefer low-risk tools when possible
- Always verify file changes with read operations
- Use bash sparingly and with caution
- Check remote tool requirements before use
```

### Advantages

✅ **Complete Visibility**: Both local and remote tools cataloged
✅ **Easy Maintenance**: Centralized in Helm values
✅ **Version Control**: Tool catalog versioned with infrastructure
✅ **Flexible Rendering**: Can generate different templates per task type
✅ **Risk Awareness**: Claude understands tool risk levels
✅ **Conditional Logic**: Templates can adapt based on available tools

### Disadvantages

❌ **Manual Maintenance**: Requires updating when tools change
❌ **Sync Risk**: Catalog can drift from actual available tools
❌ **Static Nature**: Can't detect runtime tool availability changes

## Alternative Approaches

### 1. Dynamic Tool Discovery
**Concept**: Query MCP server at runtime for available tools
```bash
# Hypothetical
mcp-client list-tools --format=json
```
**Pros**: Always accurate, no maintenance
**Cons**: Complex implementation, runtime dependency, local tools still problematic

### 2. Tool Introspection API
**Concept**: Claude Code exposes API to query available tools
```bash
curl localhost:8080/available-tools
```
**Pros**: Real-time accuracy
**Cons**: Requires Claude Code modification, complex orchestration

### 3. Hybrid Approach
**Concept**: ConfigMap catalog + runtime validation
- Use ConfigMap as base catalog
- Validate availability at container startup
- Generate warnings for missing tools

**Implementation**:
```bash
# In container startup
for tool in ${EXPECTED_TOOLS}; do
  if ! mcp-client check-tool "$tool"; then
    echo "⚠️ Tool $tool not available"
  fi
done
```

### 4. Tool Provider Manifests
**Concept**: Each MCP server provides a manifest of tools
```yaml
# mcp-server-manifest.yaml
tools:
  - name: github_create_issue
    description: "Create GitHub issues"
    parameters:
      - title: string
      - body: string
```
**Pros**: Self-documenting, accurate
**Cons**: Requires MCP server modifications

### 5. Template Variants by Tool Configuration
**Concept**: Pre-generate templates for common tool combinations
```yaml
toolConfigurations:
  minimal: ["read", "edit", "bash"]
  advanced: ["read", "edit", "bash", "github_*", "rustdocs_*"]
  development: ["read", "edit", "bash", "npm", "git", "docker"]
```

## Recommended Implementation Plan

### Phase 1: ConfigMap Catalog (Immediate)
1. Define tool catalog schema in values.yaml
2. Create template that renders tool documentation
3. Include in CLAUDE.md via @ pointer
4. Manual maintenance process

### Phase 2: Enhanced Metadata (Future)
1. Add tool usage patterns and best practices
2. Include conditional logic for task types
3. Risk-based tool recommendations
4. Integration examples

### Phase 3: Validation Layer (Future)
1. Container startup tool validation
2. Runtime availability checking
3. Graceful degradation for missing tools
4. Tool usage analytics

## Template Integration

The generated MCP tools documentation would be included in Claude's memory:

```markdown
# CLAUDE.md
See @coding-guidelines.md for project standards
See @github-guidelines.md for git workflow
See @mcp-tools.md for available capabilities    # <-- New addition
See @task/task.md for current requirements
```

This ensures Claude has complete visibility into its capabilities while maintaining the clean @ pointer architecture.

## Maintenance Workflow

1. **Tool Addition**: Update values.yaml, test template rendering
2. **Tool Removal**: Remove from catalog, update documentation
3. **Tool Changes**: Modify descriptions and examples
4. **Version Releases**: Review and update entire catalog

## Future Enhancements

- **Tool Usage Analytics**: Track which tools are used most frequently
- **Smart Recommendations**: Suggest optimal tools for task types
- **Performance Metrics**: Include tool execution time estimates
- **Dependency Mapping**: Show tool interdependencies
- **Security Profiles**: Tool access control based on task sensitivity