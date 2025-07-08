# Implementation Instructions - User API Service

You are implementing a simple Express.js REST API for user management. Follow the tasks in order, marking each as complete before moving to the next.

## IMPORTANT: Fix Previous Mistakes

The previous implementation made critical git mistakes that MUST be fixed:
1. **node_modules/ was committed** - This should NEVER be in git
2. **dist/ was committed** - Build outputs should NEVER be in git  
3. **.gitignore was missing** - Every project needs a proper .gitignore

**BEFORE doing anything else:**
1. Remove wrongly committed files from git
2. Create a proper .gitignore file
3. Ensure only source code is tracked

## Getting Started

1. First, run `task-master list` to see all tasks
2. Start with Task 11: Initialize Express TypeScript Project
3. Mark tasks complete as you finish them: `task-master set-status --id=<id> --status=done`
4. Get the next task with `task-master next`

## Task Implementation Guide

### Task 11: Initialize Express TypeScript Project

Complete the following subtasks:

**11.1: Initialize npm project**
- Run `npm init -y` to create package.json
- Update package.json with appropriate metadata

**11.2: Install dependencies**
```bash
# Production dependencies
npm install express

# Development dependencies
npm install -D typescript @types/node @types/express ts-node nodemon
```

**11.3: Configure TypeScript**
Create `tsconfig.json` with Node.js appropriate settings

**11.4: Create project structure**
Create the src directory structure as specified in the design

### Task 12: Create User Type Definition

- Create `src/types/user.ts`
- Define the User interface with proper TypeScript types

### Task 13: Implement Health Check Endpoint

- Create `src/routes/health.ts`
- Implement GET /health handler
- Wire it to the main Express app

### Task 14: Implement User Routes

- Create `src/routes/users.ts`
- Implement GET /users (return all users)
- Implement POST /users (create new user with validation)
- Wire routes to the main app

### Task 15: Add Error Handling Middleware

- Create `src/middleware/error.ts`
- Implement global error handler
- Add it to Express app (must be last middleware)

### Task 16: Create README Documentation

- Document all endpoints
- Include setup instructions
- Add curl examples for testing

## Important Implementation Notes

1. **Main Entry Point**: Create `src/index.ts` as the Express server entry point
2. **Port Configuration**: Use `process.env.PORT || 3000`
3. **NPM Scripts**: Add these to package.json:
   - `"dev": "nodemon src/index.ts"`
   - `"build": "tsc"`
   - `"start": "node dist/index.js"`

## Testing Your Implementation

After each major task:
1. Run `npm run dev` to start the development server
2. Test endpoints with curl or Postman
3. Verify TypeScript compilation with `npm run build`

## Git Workflow

Remember to follow the git guidelines:
1. Create a feature branch: `git checkout -b feature/user-api`
2. Commit your changes regularly
3. Create a pull request when complete

## Success Criteria

Your implementation is complete when:
- All 6 tasks and their subtasks are marked as done
- The server starts without errors
- All endpoints return correct responses
- TypeScript compiles without errors
- README is comprehensive and accurate
- **CRITICAL: You have created a pull request with all your changes**

## Final Steps - Creating the Pull Request

**IMPORTANT**: After completing all tasks and verifying the implementation works:

1. Create a feature branch and commit all changes:
   ```bash
   git checkout -b feature/user-api-implementation
   git add .
   git commit -m "Implement User API service with Express and TypeScript
   
   - Initialize Express TypeScript project
   - Add user type definitions
   - Implement health check endpoint
   - Create user management routes (GET/POST)
   - Add global error handling
   - Create comprehensive README
   
   🤖 Generated with [Claude Code](https://claude.ai/code)"
   ```

2. Push to your feature branch:
   ```bash
   git push origin feature/user-api-implementation
   ```

3. Create a pull request:
   ```bash
   gh pr create --base main --title "Implement User API Service" --body "## Summary
   
   Implemented a complete Express.js REST API for user management with TypeScript.
   
   ## Changes
   - Set up Express server with TypeScript configuration
   - Created User type definitions
   - Implemented /health endpoint
   - Implemented /users endpoints (GET and POST)
   - Added global error handling middleware
   - Created comprehensive documentation
   
   ## Testing
   - Server starts successfully on port 3000
   - All endpoints respond correctly
   - TypeScript compiles without errors
   - Validation works as expected
   
   🤖 Generated with [Claude Code](https://claude.ai/code)"
   ```

**The task is NOT complete until the pull request is created and you provide the PR URL.**