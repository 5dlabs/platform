#!/bin/bash
# Example script to submit a task with GitHub authentication

# Configuration
AGENT_NAME="swe-1-5dlabs"
SERVICE_NAME="todo-api"
TASK_ID=1
REPO_URL="https://github.com/5dlabs/todo-api"
BRANCH="main"

# First, create the GitHub PAT secret if it doesn't exist
echo "Ensuring GitHub PAT secret exists for agent: $AGENT_NAME"
./scripts/create-github-secret.sh "$AGENT_NAME" "$GITHUB_TOKEN"

# Submit the task with authentication
echo "Submitting task $TASK_ID for service $SERVICE_NAME"
orchestrator task submit "$TASK_ID" \
  --service "$SERVICE_NAME" \
  --agent "$AGENT_NAME" \
  --repo "$REPO_URL" \
  --branch "$BRANCH" \
  --github-user "$AGENT_NAME" \
  --tools "bash:true,read:true,write:true,edit:true"

echo "Task submitted with GitHub authentication!"