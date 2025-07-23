# MCP Tools Template Design

## Problem Statement

Claude Code CRD tasks need to understand what MCP tools are available to them, but currently this information is not dynamically provided. This leads to:

- Claude not knowing what capabilities it has
- Suboptimal tool usage
- Manual documentation that gets out of sync
- Inability to adapt behavior based on available tools

## REFINED APPROACH: Template-Driven Tool Documentation

### Enhanced Concept

Instead of a static catalog, use the tool configuration as a **source template** to dynamically generate an `mcp-tools.md` file that gets included in Claude's memory. This provides:

- **Dynamic Documentation**: Only shows tools that are actually enabled
- **Conditional Usage Guidelines**: Different recommendations based on tool availability
- **Single Source of Truth**: Tool config drives both deployment AND documentation
- **Context-Aware**: Can vary by task type, environment, or service

### Implementation Architecture

#### 1. Enhanced Tool Configuration
```yaml
# values.yaml
codeRunConfig:
  toolConfigurations:
    minimal:
      localTools:
        - name: "read_file"
          enabled: true
          description: "Read file contents with line-range support"
          category: "file-ops"
          riskLevel: "low"
          usageGuidelines:
            - "Always read before editing to understand context"
            - "Use line ranges for large files"
            - "Prefer for understanding code structure"
          examples:
            - "Reading package.json to understand dependencies"
            - "Checking existing code before modifications"

        - name: "edit_file"
          enabled: true
          description: "Edit files with precise diff-based changes"
          category: "file-ops"
          riskLevel: "medium"
          usageGuidelines:
            - "Always include sufficient context around changes"
            - "Use for targeted modifications, not wholesale rewrites"
            - "Verify changes with read_file afterward"
          examples:
            - "Adding new functions to existing modules"
            - "Updating configuration files"

        - name: "run_terminal_cmd"
          enabled: true
          description: "Execute shell commands in the workspace"
          category: "system"
          riskLevel: "high"
          usageGuidelines:
            - "Use sparingly and with caution"
            - "Prefer specific tools over generic shell commands"
            - "Always explain what command does before running"
            - "Never run commands that could damage the system"
          conditionalUsage:
            - condition: "Installing dependencies"
              recommendation: "Use npm install, pip install, etc."
            - condition: "Running tests"
              recommendation: "Use npm test, pytest, cargo test, etc."

      remoteTools:
        - name: "web_search"
          enabled: false  # Not available in minimal config
          description: "Search the web for current information"
          category: "research"

    advanced:
      localTools:
        # ... includes all minimal tools plus:
        - name: "codebase_search"
          enabled: true
          description: "Semantic search across the codebase"
          category: "code-analysis"
          riskLevel: "low"
          usageGuidelines:
            - "Use for understanding unfamiliar codebases"
            - "Great for finding patterns and implementations"
            - "Prefer over grep for conceptual searches"

      remoteTools:
        - name: "web_search"
          enabled: true
          description: "Search the web for current information"
          category: "research"
          riskLevel: "low"
          usageGuidelines:
            - "Use for latest documentation and best practices"
            - "Helpful for debugging error messages"
            - "Good for checking library compatibility"

        - name: "github_create_issue"
          enabled: true
          description: "Create GitHub issues"
          category: "collaboration"
          requirements: ["github_token"]
```

#### 2. Template Generation Logic
```handlebars
{{!-- mcp-tools.md.hbs --}}
# Available MCP Tools

Based on your current configuration (**{{toolConfig}}**), you have access to:

## Local Tools (Always Available)

{{#each localTools}}
{{#if enabled}}
### 🔧 {{name}}

**Description:** {{description}}
**Category:** {{category}}
**Risk Level:** {{riskLevel}}

**Usage Guidelines:**
{{#each usageGuidelines}}
- {{this}}
{{/each}}

{{#if conditionalUsage}}
**When to Use:**
{{#each conditionalUsage}}
- **{{condition}}**: {{recommendation}}
{{/each}}
{{/if}}

**Examples:**
{{#each examples}}
- {{this}}
{{/each}}

---
{{/if}}
{{/each}}

## Remote Tools (Network Required)

{{#if remoteTools}}
{{#each remoteTools}}
{{#if enabled}}
### 🌐 {{name}}

**Description:** {{description}}
**Category:** {{category}}
{{#if requirements}}**Requirements:** {{join requirements ", "}}{{/if}}

**Usage Guidelines:**
{{#each usageGuidelines}}
- {{this}}
{{/each}}

---
{{/if}}
{{/each}}
{{else}}
*No remote tools enabled in {{toolConfig}} configuration.*
{{/if}}

## Tool Selection Strategy

{{#if (eq toolConfig "minimal")}}
**Minimal Configuration Strategy:**
- Focus on core file operations (read, edit)
- Use terminal commands only when necessary
- Rely on built-in capabilities
- Prioritize simple, direct approaches
{{/if}}

{{#if (eq toolConfig "advanced")}}
**Advanced Configuration Strategy:**
- Leverage semantic search for code understanding
- Use web search for current best practices
- Consider remote tools for enhanced capabilities
- Balance efficiency with available tooling
{{/if}}

## Best Practices

1. **Start with Low-Risk Tools**: Always try `read_file` and `codebase_search` first
2. **Understand Before Acting**: Read existing code before making changes
3. **Verify Changes**: Use `read_file` to confirm edits worked as expected
4. **Progressive Enhancement**: Use higher-risk tools only when necessary
5. **Document Decisions**: Explain tool choices in your implementation notes

## Tool Combinations

**For Understanding Code:**
1. `codebase_search` → Find relevant files/patterns
2. `read_file` → Examine specific implementations
3. `web_search` → Research unfamiliar patterns (if available)

**For Making Changes:**
1. `read_file` → Understand current state
2. `edit_file` → Make targeted changes
3. `read_file` → Verify changes
4. `run_terminal_cmd` → Test/build (if needed)

**For Research & Documentation:**
1. `web_search` → Find current best practices (if available)
2. `codebase_search` → Find existing patterns in project
3. `read_file` → Study implementation details
```

#### 3. Integration with Container Script
```bash
# In container.sh.hbs
# Generate MCP tools documentation based on current tool config
echo "🔧 Generating MCP tools documentation for {{toolConfig}} configuration..."

# Template rendering would happen during ConfigMap creation
# The resulting mcp-tools.md would be available via @ pointer
```

#### 4. CLAUDE.md Integration
```markdown
# Claude Code Project Memory

## Tool Capabilities
See @mcp-tools.md for your available tools and usage guidelines

## Project Guidelines & Standards
See @coding-guidelines.md for project coding standards and best practices
See @github-guidelines.md for git workflow and commit message standards

## Current Task Documentation
See @task/task.md for requirements and description
```

### Benefits of This Approach

✅ **Automatic Synchronization**: Documentation always matches actual tool availability
✅ **Configuration-Aware**: Different docs for minimal vs advanced setups
✅ **Usage Guidance**: Not just "what" tools but "when" and "how" to use them
✅ **Risk Management**: Clear guidance on tool risk levels and best practices
✅ **Contextual Help**: Tool combinations and workflow recommendations
✅ **Maintenance-Free**: No manual documentation updates needed

### Template Rendering Process

1. **ConfigMap Generation**: Helm renders tool config into tool documentation template
2. **File Creation**: Generated `mcp-tools.md` included in task ConfigMap
3. **Claude Memory**: Loaded via `@mcp-tools.md` pointer in CLAUDE.md
4. **Dynamic Adaptation**: Each task gets documentation matching its tool configuration

This creates a **self-documenting** system where the tool configuration itself becomes the source of truth for both deployment and Claude's understanding of capabilities.

## Original Approach: ConfigMap Tool Catalog

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