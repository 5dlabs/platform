# Acceptance Criteria

## Repository Cloning
- [ ] Repository https://github.com/5dlabs/agent-sandbox is cloned successfully
- [ ] Git credentials are configured correctly
- [ ] GitHub CLI (`gh`) is authenticated

## File System Structure  
- [ ] Working directory is `/workspace` (not `/workspace/test-service`)
- [ ] Task files exist in `.task/9999/` directory
- [ ] All markdown files (task.md, design-spec.md, prompt.md) are accessible
- [ ] `.claude.json` exists with proper tool permissions

## Git Operations
- [ ] Can create and write new files
- [ ] Can stage files with `git add`
- [ ] Can commit with proper commit message
- [ ] Can push to remote repository using OAuth token

## CLI Integration
- [ ] CLI reads markdown files from disk successfully
- [ ] CLI submits task with all markdown content
- [ ] CLI properly handles repository and GitHub user parameters