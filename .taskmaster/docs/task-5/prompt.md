# Autonomous Agent Prompt: Implement Docs Agent Tool Discovery

## Implementation Status (July 26, 2025)

### Completed ✅
- **Toolman RBAC**: Role and RoleBinding templates added for ConfigMap permissions
- **Tool Discovery Fixed**: Deployed image `main-5724488` - now discovers 48 tools
- **ConfigMap Creation**: Implemented in Toolman (deployed as `main-adfad50`)
- **Orchestrator Mounting**: ConfigMap mounted to `/etc/tool-catalog` in agents
- **Local Tools Discovery**: Dynamic discovery from local MCP servers (no hardcoding)
- **Server-side Apply**: Fixed conflicts with `.force()` for ConfigMap updates

### Current State 🎯
- **Task 5 is COMPLETE**: All orchestrator and Toolman components implemented
- **Toolman creates**: `toolman-tool-catalog` ConfigMap with full tool metadata
- **Local tools**: Discovered dynamically from `toolman-local-tools` ConfigMap
- **Remote tools**: Discovered from Toolman-proxied MCP servers

### Pending (Separate Work) 🔄
- **Docs Agent Implementation**: Read mounted catalog and implement matching logic
- This is separate from orchestrator work and runs inside the agent container

---

## Context
You are implementing the tool discovery functionality for the docs agent. This is a critical component that reads the Toolman ConfigMap to discover available MCP tools and generates optimal tool configurations based on project analysis. This implementation must follow the zero-hardcoding principle - no tool names should be hardcoded.

**Important**: The primary use case is **greenfield projects** where the docs agent must determine needed tools based on task requirements, not by scanning existing code.

**Key Architecture Point**: The docs agent's intelligence comes from the **prompt template**, not from code. The orchestrator only needs to mount the tool catalog ConfigMap. All tool selection logic is handled by AI through carefully crafted prompts.

## Your Mission
Ensure the tool catalog ConfigMap is properly mounted for the docs agent. The actual tool discovery and recommendation logic will be handled entirely through the prompt template.

## Implementation Requirements

### Orchestrator Changes (Minimal)

The ONLY code change needed is to mount the ConfigMap:

```rust
// In orchestrator/core/src/controllers/task_controller/resources.rs
// Add to build_job_spec function for docs tasks:

if task_type.is_docs() {
    // Mount tool catalog for docs agent
    volumes.push(json!({
        "name": "tool-catalog",
        "configMap": {
            "name": "toolman-tool-catalog",
            "optional": true
        }
    }));
    volume_mounts.push(json!({
        "name": "tool-catalog",
        "mountPath": "/etc/tool-catalog",
        "readOnly": true
    }));
}
```

That's it for code changes!

### Prompt Template Responsibilities

The docs agent prompt template (in Helm charts) will handle everything else:

1. **Reading the Catalog**: Instruct the AI to read `/etc/tool-catalog/tool-catalog.json`
2. **Understanding Tools**: AI analyzes each tool's description, capabilities, and use cases
3. **Task Analysis**: AI deeply understands the task requirements
4. **Intelligent Matching**: AI makes thoughtful recommendations based on understanding
5. **Output Generation**: AI creates a minimal `tools.json` in the task folder

### Example Prompt Template Structure

```handlebars
You are analyzing task requirements to recommend appropriate MCP tools.

## Available Tools
Read the tool catalog from: /etc/tool-catalog/tool-catalog.json
This contains detailed information about all available MCP tools.

## Your Task
1. Carefully read and understand the task description
2. Analyze what technologies and capabilities are needed
3. Match task requirements with available tools
4. Consider tool descriptions and use cases
5. Make intelligent recommendations

## Output Format
Create a file at: .taskmaster/docs/task-{{TASK_ID}}/tools.json
```json
{
  "tools": {
    "local": ["filesystem"],  // Always include for file operations
    "remote": [/* Your intelligent selections */]
  },
  "reasoning": "Explain your tool choices based on task needs",
  "generated_at": "ISO timestamp"
}
```

Be thoughtful and selective - only recommend tools that directly support the task objectives.
```

### No Code Logic Needed

All the intelligence comes from the prompt engineering. No need for:
- Pattern matching code
- Keyword detection functions
- Complex analysis algorithms
- Tool matching logic

The AI model handles all of this through understanding, guided by the prompt.

## Key Implementation Points

1. **No Hardcoding**: Use pattern matching, not exact tool names
2. **Best Effort**: Handle failures gracefully, continue with what works
3. **Logging**: Log all important steps for debugging
4. **Error Handling**: Don't fail the entire process for partial errors
5. **Performance**: Use efficient algorithms and early exits

## Success Criteria

- [ ] Reads ConfigMap successfully
- [ ] Discovers all available tools
- [ ] Analyzes project files correctly
- [ ] Matches tools based on patterns
- [ ] Saves configuration for code agents
- [ ] No hardcoded tool names
- [ ] Handles errors gracefully
- [ ] Well-tested implementation

Proceed with implementing this tool discovery system, ensuring it's robust, maintainable, and follows the zero-hardcoding principle throughout.
Proceed with implementing this tool discovery system, ensuring it's robust, maintainable, and follows the zero-hardcoding principle throughout.