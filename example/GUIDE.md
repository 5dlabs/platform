 # Complete Guide: Building with AI Agents

This guide walks through using the agent platform to build a complete application from scratch.

## Understanding the System

### Components

1. **Task Files** (Markdown)
   - Detailed descriptions of what to build
   - Success criteria for validation
   - Technical requirements and constraints

2. **CLAUDE.md**
   - Project-specific instructions for agents
   - Coding standards and conventions
   - Technology stack preferences

3. **Task Master**
   - Local task tracking for the project
   - Dependencies between tasks
   - Progress monitoring

4. **Orchestrator**
   - Manages agent deployments
   - Handles task submission
   - Monitors execution

5. **TaskRun CRD**
   - Kubernetes resource for task execution
   - Tracks agent job status
   - Enables context updates

## Workflow

### 1. Project Setup

```bash
# Create project directory
mkdir my-service
cd my-service

# Initialize task management
mkdir .taskmaster
echo '{"version": "1.0", "context": {"current": "my-service", "tasks": []}}' > .taskmaster/tasks.json

# Create CLAUDE.md with project guidelines
cat > CLAUDE.md << 'EOF'
# My Service Instructions

Project-specific guidelines for AI agents...
EOF

# Create initial package.json or project file
npm init -y  # for Node.js projects
```

### 2. Writing Task Files

Good task files have:
- Clear objectives
- Specific requirements
- Success criteria
- Examples where helpful

```markdown
# Task: Implement User Authentication

## Objective
Add JWT-based authentication to the API

## Requirements
1. Install required packages (jsonwebtoken, bcrypt)
2. Create auth middleware
3. Add login and register endpoints
4. Secure existing endpoints

## Success Criteria
- [ ] Users can register with email/password
- [ ] Users can login and receive JWT
- [ ] Protected endpoints require valid JWT
- [ ] Passwords are hashed with bcrypt
```

### 3. Submitting Tasks

```bash
# Submit a task
orchestrator-cli task submit \
  --service my-service \
  --task-file tasks/add-auth.md

# Check status
orchestrator-cli task status --service my-service

# View agent logs
kubectl logs -n orchestrator -l service=my-service -f
```

### 4. Monitoring Progress

The agent will:
1. Read the task file
2. Analyze the existing code
3. Make necessary changes
4. Run tests if available
5. Update task status

### 5. Context Updates

If an agent needs clarification:

```bash
# Add context to running task
orchestrator-cli task add-context \
  --service my-service \
  --context "Use Passport.js for authentication strategy"
```

## Best Practices

### Task Design

1. **Start Small**: Break large features into smaller tasks
2. **Be Specific**: Clear requirements lead to better results
3. **Include Examples**: Show desired input/output
4. **Define Success**: How will you know it's done?

### Project Organization

```
my-service/
├── .taskmaster/          # Local task tracking
├── CLAUDE.md            # Agent instructions
├── tasks/               # Task descriptions
│   ├── 01-setup.md
│   ├── 02-feature-a.md
│   └── 03-feature-b.md
├── docs/                # Design documents
│   └── architecture.md
└── src/                 # Source code
```

### Working with Agents

1. **Provide Context**: Good CLAUDE.md files help agents understand your preferences
2. **Iterative Development**: Submit follow-up tasks to refine features
3. **Review Changes**: Always review agent-generated code
4. **Test Thoroughly**: Agents can miss edge cases

## Example: Full Application Build

Here's how to build a complete application:

### Phase 1: Foundation
1. Project setup and structure
2. Basic server/framework setup
3. Development environment configuration

### Phase 2: Core Features
1. Data models and database
2. CRUD operations
3. Business logic

### Phase 3: Advanced Features
1. Authentication/authorization
2. API documentation
3. Error handling and logging

### Phase 4: Production Readiness
1. Test suite
2. Performance optimization
3. Deployment configuration

## Troubleshooting

### Agent Issues

**Agent can't find files:**
- Check CLAUDE.md has correct project structure
- Ensure files are in the expected locations

**Agent makes unwanted changes:**
- Be more specific in task descriptions
- Add constraints to CLAUDE.md

**Task fails repeatedly:**
- Break into smaller subtasks
- Provide more examples
- Check agent logs for errors

### Platform Issues

**Task not starting:**
```bash
kubectl get taskruns -n orchestrator
kubectl describe taskrun <name> -n orchestrator
```

**Agent pod crashes:**
```bash
kubectl logs -n orchestrator <pod-name> --previous
```

## Advanced Usage

### Custom Agent Images

You can use different AI models:

```bash
orchestrator-cli task submit \
  --service my-service \
  --task-file task.md \
  --agent-image ghcr.io/5dlabs/gemini-cli:latest
```

### Batch Operations

Submit multiple related tasks:

```bash
for task in tasks/*.md; do
  orchestrator-cli task submit \
    --service my-service \
    --task-file "$task"
  sleep 5  # Give time between submissions
done
```

### Task Templates

Create reusable task templates:

```markdown
# Template: Add CRUD Endpoints

## Objective
Add CRUD endpoints for ${RESOURCE}

## Requirements
1. Create routes: GET, POST, PUT, DELETE
2. Add validation for ${RESOURCE} fields
3. Implement controllers with proper error handling
4. Add integration tests

## Variables
- RESOURCE: The resource name (e.g., "user", "product")
- FIELDS: List of fields and their types
```

## Conclusion

The agent platform enables rapid development by automating coding tasks while maintaining control over architecture and quality. Start with small, well-defined tasks and gradually build up to complex features.

Remember: AI agents are tools to augment development, not replace developer judgment. Always review and test agent-generated code before deploying to production.