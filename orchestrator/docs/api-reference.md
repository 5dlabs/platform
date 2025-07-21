# Orchestrator API Reference

## Base URL
```
http://orchestrator.orchestrator.svc.cluster.local/api/v1
```

## Health Check

### GET /health
Check service health status.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2025-07-10T22:13:20.462638009+00:00",
  "version": "0.1.0"
}
```

## Task Management

### POST /pm/tasks
Submit a new task for execution.

**Request Body:**
```json
{
  "task_id": "string",
  "service_name": "string", 
  "agent_name": "string",
  "repository": {
    "url": "string",
    "branch": "string",
    "working_directory": "string",
    "github_user": "string"
  },
  "markdown_files": [
    {
      "content": "string",
      "filename": "string", 
      "file_type": "Context|Task|DesignSpec|Prompt|AcceptanceCriteria"
    }
  ],
  "agent_tools": [
    {
      "name": "string",
      "enabled": true,
      "config": {},
      "restrictions": ["string"]
    }
  ],
  "retry_count": 0,
  "model": "string"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Task submitted successfully",
  "data": {
    "taskrun_name": "taskrun-xyz123",
    "namespace": "orchestrator"
  }
}
```

### GET /pm/tasks
List all tasks.

**Response:**
```json
{
  "tasks": [
    {
      "id": "string",
      "status": "pending|running|completed|failed",
      "service_name": "string",
      "created_at": "2025-07-10T22:13:20Z"
    }
  ]
}
```

### GET /pm/tasks/{task_id}
Get details for a specific task.

**Response:**
```json
{
  "id": "string",
  "status": "pending|running|completed|failed",
  "service_name": "string",
  "agent_name": "string",
  "repository": {
    "url": "string",
    "branch": "string",
    "working_directory": "string",
    "github_user": "string"
  },
  "created_at": "2025-07-10T22:13:20Z",
  "completed_at": "2025-07-10T22:15:30Z",
  "logs": ["string"]
}
```

### POST /pm/tasks/{task_id}/context
Add context to an existing task.

**Request Body:**
```json
{
  "context": "string",
  "metadata": {}
}
```

### POST /pm/tasks/{task_id}/session
Update session information for a task.

**Request Body:**
```json
{
  "session_data": "string",
  "status": "string"
}
```

## Documentation Generation

### POST /pm/docs/generate
Generate documentation for Task Master tasks using AI agents.

**Request Body:**
```json
{
  "repository_url": "https://github.com/owner/repo.git",
  "working_directory": "-projects/example-express", 
  "source_branch": "feature/example-project-and-cli",
  "target_branch": "docs-generation-20250710-143600",
  "service_name": "docs-generator",
  "agent_name": "claude-docs-opus",
  "model": "opus",
  "github_user": "pm0-5dlabs",
  "task_id": null,
  "force": false,
  "dry_run": false
}
```

**Response:**
```json
{
  "success": true,
  "message": "Documentation generation job submitted successfully",
  "data": {
    "taskrun_name": "docs-taskrun-xyz123",
    "namespace": "orchestrator"
  }
}
```

**Parameters:**
- `repository_url`: Git repository URL
- `working_directory`: Relative path from repo root to Task Master directory
- `source_branch`: Base branch to create PR against
- `target_branch`: Branch name for the documentation changes
- `service_name`: Service identifier for the job
- `agent_name`: Claude agent identifier
- `model`: AI model to use (`opus`, `sonnet`)
- `github_user`: GitHub account for commits and PRs
- `task_id`: Optional - generate docs for specific task only
- `force`: Overwrite existing documentation
- `dry_run`: Preview mode (server-side dry run)

## Error Responses

All endpoints may return error responses in this format:

```json
{
  "success": false,
  "message": "Error description",
  "error_code": "ERROR_TYPE",
  "details": {}
}
```

**Common Error Codes:**
- `400` - Bad Request: Invalid request format or missing required fields
- `404` - Not Found: Resource not found
- `422` - Unprocessable Entity: Valid format but invalid data
- `500` - Internal Server Error: Server-side error

## Authentication

Currently the API uses Kubernetes service account authentication within the cluster. External access requires appropriate network policies and ingress configuration.

## Rate Limiting

No explicit rate limiting is currently implemented, but Kubernetes resource limits apply to prevent resource exhaustion.

## Monitoring

TaskRun jobs can be monitored using:
```bash
kubectl -n orchestrator get taskrun
kubectl -n orchestrator describe taskrun <taskrun-name>
kubectl -n orchestrator logs -f job/<job-name>
```

## CLI Integration

The orchestrator CLI provides convenient access to these APIs:

```bash
# Documentation generation
orchestrator task init-docs --model opus --github-user pm0-5dlabs

# Task submission  
orchestrator task submit --task-id 1 --service myservice --agent claude

# Task listing
orchestrator task list

# Job monitoring
orchestrator job list
orchestrator job get <job-id>
```

Refer to `orchestrator --help` for complete CLI documentation.