# Orchestrator MCP Server Usage Guide

> Complete guide to interacting with the Orchestrator MCP server from Claude Code/Cursor

The Orchestrator MCP server provides a natural language interface for managing task implementation across your development workflow. This guide shows you exactly how to interact with it through Claude Code or Cursor.

## 🎯 Overview

When you have the Orchestrator MCP server configured in your IDE, you can simply **talk to Claude** about what you want to implement, and Claude will:

- Submit implementation tasks to the Kubernetes orchestrator
- Handle retries with custom guidance when tasks fail
- Track progress across multiple services and tasks
- Manage different working directories and repositories
- Provide status updates and troubleshooting help

**You don't need to remember Kubernetes commands or TaskRun syntax** - just describe what you want to build in natural language.

## 🚀 Prerequisites

1. **MCP Server Running**: The orchestrator MCP server should be deployed and accessible
2. **IDE Integration**: Claude Code or Cursor configured with the MCP server
3. **Documentation Generated**: Task documentation should exist (either generated or manually created)
4. **GitHub Authentication**: Appropriate GitHub PATs and SSH keys configured in cluster

## 💬 Natural Language Conversation Examples

### **Scenario 1: Starting Your First Implementation Task**

**You (CTO)**:
> "I just finished generating docs for the trader service. The docs are merged into main. I want to start implementing task 1 using the claude-agent-1 GitHub account. The target repo is git@github.com:myorg/trader.git and I want to use the sonnet model."

**Claude Response**:
> I'll submit the first implementation task for your trader service. Let me use the orchestrator to set this up.

*[Claude uses MCP tool: `mcp_orchestrator_submit_implementation_task`]*

> ✅ **Implementation task submitted successfully!**
>
> **Details:**
> - **Task**: 1 (trader service)
> - **Agent**: claude-agent-1
> - **Target Repo**: git@github.com:myorg/trader.git
> - **Model**: sonnet
> - **Working Directory**: trader/ (default)
> - **TaskRun**: `trader-claude-agent-1-task1-attempt1`
>
> The agent will:
> 1. Clone the platform repo to access task documentation
> 2. Clone/setup the trader repository
> 3. Use Rust-specific MCP tools (rustdocs, brave-search, memory)
> 4. Follow the tool usage patterns for efficient implementation
>
> You can monitor progress with:
> ```bash
> kubectl get taskrun trader-claude-agent-1-task1-attempt1 -n orchestrator
> ```

---

### **Scenario 2: Task Failure - Need for Retry**

**You**:
> "Task 1 failed. I looked at the logs and the agent chose the wrong HTTP client library. It used 'curl' crate but should use 'reqwest'. Can you retry this with better guidance?"

**Claude Response**:
> I'll retry task 1 with specific guidance about using reqwest instead of curl. Let me submit a corrected attempt.

*[Claude uses MCP tool with prompt modification]*

> ✅ **Task 1 retry submitted with corrections!**
>
> **Retry Strategy:**
> - **Preserved**: Same feature branch for continuity
> - **Added Guidance**: "Previous attempt failed because agent chose 'curl' crate. Use 'reqwest' crate instead for HTTP requests. Query rustdocs for reqwest examples before implementing."
> - **Tool Pattern**: Agent will query rustdocs for reqwest before coding
> - **TaskRun**: `trader-claude-agent-1-task1-attempt2`
>
> The agent will see the previous attempt's work and build on it with the corrected approach.

---

### **Scenario 3: Moving to Next Task**

**You**:
> "Great! Task 1 completed successfully. Let's move on to task 2, but I want to use a custom working directory 'trading-engine' instead of the default 'trader'."

**Claude Response**:
> Perfect! I'll submit task 2 with the custom working directory. The system will automatically clean up from task 1 and create a fresh branch.

*[Claude uses MCP tool with working directory override]*

> ✅ **Task 2 submitted with custom working directory!**
>
> **Setup:**
> - **Task**: 2 (trader service)
> - **Working Directory**: trading-engine/ (custom)
> - **Repository**: Will update existing repo, create new feature branch
> - **Cleanup**: Automatic cleanup from task 1 branch
> - **TaskRun**: `trader-claude-agent-1-task2-attempt1`
>
> The agent will work in `/workspace/target-repo/trading-engine/` and create that directory if it doesn't exist.

---

### **Scenario 4: Different Service**

**You**:
> "Now I want to start working on the auth service. This is a completely separate service with its own repo at git@github.com:myorg/platform-auth.git. Start with task 1."

**Claude Response**:
> I'll submit the first task for the auth service. This will create a separate workspace and use different tool configurations.

*[Claude uses MCP tool with new service]*

> ✅ **Auth service task 1 submitted!**
>
> **New Service Setup:**
> - **Service**: auth (separate PVC: `workspace-auth`)
> - **Repository**: git@github.com:myorg/platform-auth.git
> - **Fresh Environment**: New workspace, fresh repository clone
> - **Tool Configuration**: Will use auth-service specific tools
> - **TaskRun**: `auth-claude-agent-1-task1-attempt1`
>
> This is completely isolated from the trader service workspace.

---

### **Scenario 5: Checking Status**

**You**:
> "How's that auth task coming along?"

**Claude Response**:
> Let me check the status for you.

*[Claude could use kubectl or monitoring tools if available]*

> **Task Status Update:**
> - **TaskRun**: auth-claude-agent-1-task1-attempt1
> - **Status**: Running (started 15 minutes ago)
> - **Phase**: Implementation in progress
> - **Logs**: Agent successfully queried rustdocs and is implementing authentication middleware
>
> The task appears to be progressing well. The agent followed the tool usage patterns and researched the appropriate authentication patterns before coding.

---

### **Scenario 6: Emergency Stop**

**You**:
> "Wait, I need to stop that auth task. I just realized we're changing the architecture."

**Claude Response**:
> I can help you stop the running task. However, the current MCP tools don't include a stop function - you'll need to cancel it directly:

> ```bash
> kubectl delete taskrun auth-claude-agent-1-task1-attempt1 -n orchestrator
> ```
>
> The workspace will be preserved in the `workspace-auth` PVC, so when you're ready to restart with the new architecture, just let me know and I can submit a new task with updated guidance.

---

## 🎛️ Available MCP Tools

The orchestrator provides these MCP tools that Claude can use on your behalf:

### `mcp_orchestrator_submit_implementation_task`

**Purpose**: Submit a new implementation task or retry a failed task

**Required Parameters**:
- `task_id`: The task number to implement (e.g., 1, 2, 3)
- `service`: The service name (creates `workspace-{service}` PVC)

**Optional Parameters**:
- `agent`: GitHub agent identifier (default: `claude-agent-1`)
- `model`: Claude model to use (default: `sonnet`)
- `branch`: Git branch to work on (default: `main`)
- `retry`: Whether this is a retry attempt (default: `false`)
- `repository_url`: Git repository URL (auto-detected if not specified)
- `github_user`: GitHub username (auto-detected if not specified)
- `working_directory`: Custom working directory within repo

### `mcp_orchestrator_init_docs`

**Purpose**: Generate task documentation using Claude

**Required Parameters**:
- `task_id`: Specific task to generate docs for (optional for all tasks)

**Optional Parameters**:
- `model`: Claude model for docs generation (default: `opus`)
- `force`: Overwrite existing documentation (default: `false`)
- `working_directory`: Project directory containing `.taskmaster` folder

## 🔧 Natural Language Patterns

Here are common ways to interact with Claude when you have MCP access:

### **Starting Work**
- "Start implementing task 3 for the analytics service"
- "Begin task 1 using the opus model"
- "I want to implement task 2 in the 'backend-api' working directory"

### **Handling Failures**
- "Task 5 failed because [reason]. Retry with [specific guidance]"
- "The agent made a mistake in task 2. It should use [correct approach] instead"
- "Retry the last task but tell the agent to [specific instruction]"

### **Service Management**
- "Switch to working on the user-service repository"
- "Start fresh work on the notifications service"
- "I need to implement task 1 for a new service called 'payments'"

### **Monitoring**
- "How's the current task progressing?"
- "Check the status of the last implementation"
- "Is task 3 still running?"

### **Documentation**
- "Generate documentation for task 5"
- "Create docs for all tasks in the trader service"
- "Regenerate the documentation with the opus model"

## 🏗️ Architecture Context

### **What Happens Behind the Scenes**

When you ask Claude to implement a task:

1. **Task Submission**: Claude calls the MCP tool to create a TaskRun resource in Kubernetes
2. **Resource Creation**: The controller creates ConfigMaps with documentation and scripts
3. **Job Execution**: A Kubernetes Job runs the Claude agent in a container
4. **Repository Management**: The agent clones repositories and manages Git branches
5. **Implementation**: The agent reads documentation and implements the task
6. **Result**: Code changes are committed and pushed to the target repository

### **Workspace Management**

- **Persistent Storage**: Each service gets its own PVC (`workspace-{service}`)
- **Repository Isolation**: Target repositories are separate from the platform repo
- **Branch Strategy**: Each task gets its own feature branch (`feature/task-{id}-implementation`)
- **Cleanup**: Automatic cleanup between tasks and attempts

### **Agent Capabilities**

Implementation agents have access to:
- **Rust Documentation**: Live rustdocs queries for current best practices
- **Web Search**: Research current library recommendations and solutions
- **Memory**: Store and recall implementation patterns and decisions
- **Repository Tools**: Git operations, file management, code analysis
- **Kubernetes**: Resource inspection and debugging tools

## 🚨 Troubleshooting

### **Common Issues**

**"Task failed during repository setup"**
- Check GitHub authentication (PAT secrets, SSH keys)
- Verify repository URL and permissions
- Ensure the specified branch exists

**"Agent chose wrong approach"**
- Use retry with specific guidance about the correct approach
- Include examples or specific library recommendations
- Reference successful patterns from previous tasks

**"Workspace permission errors"**
- Check PVC status and node capacity
- Verify service account permissions
- Look for storage quota issues

### **Getting Help**

If Claude can't help directly, you can:

1. **Check TaskRun status**: `kubectl get taskrun <name> -n orchestrator -o yaml`
2. **View logs**: `kubectl logs job/<job-name> -n orchestrator`
3. **Inspect workspace**: `kubectl exec -it <pod> -n orchestrator -- /bin/bash`
4. **Monitor resources**: Use Grafana dashboards for task execution metrics

## 📋 Best Practices

### **For Task Submission**
- Be specific about requirements and constraints
- Mention preferred libraries or patterns when relevant
- Provide context about previous successful approaches
- Use descriptive working directory names for complex projects

### **For Retries**
- Include specific guidance about what went wrong
- Reference successful examples from other tasks
- Be explicit about library or approach preferences
- Don't just retry without additional context

### **For Multi-Service Projects**
- Use consistent naming conventions for services
- Keep related services in the same GitHub organization
- Plan working directory structure across services
- Consider dependencies between services

---

**💡 Remember**: The goal is natural conversation. You don't need to memorize commands or syntax - just tell Claude what you want to build, and the MCP integration will handle the orchestration details.