# Orchestrator API Documentation

## PM Task Submission Endpoint

**URL:** `POST /api/v1/pm/tasks`

### Request Body Structure

Based on the `PmTaskRequest` struct in `orchestrator-common/src/models/pm_task.rs`:

```json
{
  "id": 999,
  "title": "Task Title",
  "description": "Task description",
  "details": "Detailed implementation information",
  "test_strategy": "Testing approach and criteria",
  "priority": "high|medium|low",
  "dependencies": [1, 2, 3],
  "status": "pending|in_progress|completed",
  "subtasks": [],
  
  "service_name": "target-service-name",
  "agent_name": "claude-agent-1",
  "model": "sonnet|opus",
  
  "markdown_files": [
    {
      "filename": "task.md",
      "content": "Markdown content here",
      "file_type": "task|design-spec|prompt|context|acceptance-criteria"
    }
  ],
  
  "agent_tools": [
    {
      "name": "bash",
      "enabled": true,
      "config": {},
      "restrictions": []
    }
  ],
  
  "repository": {
    "url": "https://github.com/org/repo",
    "branch": "main",
    "path": "/optional/subpath",
    "auth": {
      "type": "Token|SshKey|BasicAuth",
      "secret_name": "github-pat-username",
      "secret_key": "token"
    }
  }
}
```

### Field Naming Conventions

- Uses **snake_case** for JSON field names
- Repository auth type uses `"type"` key (mapped to `auth_type` in Rust)
- All fields except optional ones are required

### Response Structure

**Success (200):**
```json
{
  "success": true,
  "message": "Task submitted successfully",
  "data": {
    "name": "task-999",
    "namespace": "orchestrator", 
    "service": "target-service-name",
    "task_id": 999
  }
}
```

**Error (4xx):**
```json
{
  "success": false,
  "message": "Error description"
}
```

### Model Selection

The `model` field supports:
- `"sonnet"` (default)
- `"opus"`

This should flow through to the TaskRun CRD and Claude execution.

### Example cURL Command

```bash
curl -X POST http://orchestrator.orchestrator.svc.cluster.local/api/v1/pm/tasks \
  -H "Content-Type: application/json" \
  -d @task-request.json
```