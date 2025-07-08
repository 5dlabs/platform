# Implementation Instructions - User API Service

You are implementing a simple Express.js REST API for user management. 

## Current Task: Initialize Express TypeScript Project

Please complete the following subtasks in order:

### Subtask 11.1: Initialize npm project
- Navigate to the repository root
- Run `npm init -y` to create package.json
- Update package.json with appropriate metadata:
  - name: "user-api"
  - description: "Simple Express.js user management API"
  - main: "dist/index.js"
  - author: "5dlabs"

### Subtask 11.2: Install dependencies
Install the following packages:
```bash
# Production dependencies
npm install express

# Development dependencies
npm install -D typescript @types/node @types/express ts-node nodemon
```

### Subtask 11.3: Configure TypeScript
Create `tsconfig.json` with:
- Target: ES2022
- Module: CommonJS
- Strict mode enabled
- Output directory: ./dist
- Root directory: ./src
- Include type definitions

### Subtask 11.4: Create project structure
Create the following directories:
```
src/
├── routes/
├── middleware/
└── types/
```

## Implementation Guidelines
- Use ES modules syntax in TypeScript files
- Port should be configurable via PORT env var (default 3000)
- Add npm scripts:
  - "dev": "nodemon src/index.ts"
  - "build": "tsc"
  - "start": "node dist/index.js"

## Task Tracking
As you complete each subtask, update its status:
- `task-master set-status --id=11.1 --status=done`
- `task-master set-status --id=11.2 --status=done`
- etc.

When all subtasks are complete, mark the parent task as done:
- `task-master set-status --id=11 --status=done`

Then get the next task with `task-master next`