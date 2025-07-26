# Autonomous Agent Prompt: Initialize Node.js Project

You are an autonomous agent tasked with initializing a Node.js project for a Hello World API. Your goal is to set up the foundational structure and dependencies needed for an Express.js web application.

## Task Requirements

1. **Create Project Directory Structure**
   - Create a new directory named `hello-world-api`
   - Navigate into this directory for all subsequent operations
   - Create a `src` subdirectory for source code

2. **Initialize Node.js Project**
   - Run `npm init -y` to create a default package.json
   - Update package.json with the following configuration:
     ```json
     {
       "name": "hello-world-api",
       "version": "1.0.0",
       "description": "A simple Hello World API",
       "main": "src/index.js",
       "scripts": {
         "start": "node src/index.js"
       }
     }
     ```

3. **Install Dependencies**
   - Install Express.js: `npm install express`
   - Ensure the dependency is added to package.json
   - Verify installation with `npm list`

4. **Create Project Files**
   - Create an empty `src/index.js` file
   - Create a `.gitignore` file with:
     ```
     node_modules/
     .env
     ```

5. **Implement Basic Server Structure**
   Add the following initial code to `src/index.js`:
   ```javascript
   const express = require('express');
   const app = express();
   const PORT = process.env.PORT || 3000;

   // Middleware for request logging
   app.use((req, res, next) => {
     console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
     next();
   });

   // Server startup
   app.listen(PORT, () => {
     console.log(`Server running on http://localhost:${PORT}`);
   });
   ```

## Validation Steps

After completing the setup:

1. Verify package.json contains Express.js in dependencies
2. Confirm the directory structure:
   ```
   hello-world-api/
   ├── node_modules/
   ├── src/
   │   └── index.js
   ├── .gitignore
   ├── package.json
   └── package-lock.json
   ```
3. Test the start script: `npm start` (server should start without errors)
4. Verify console output shows "Server running on http://localhost:3000"

## Expected Outcomes

- A properly initialized Node.js project with Express.js installed
- Correct project structure with src directory
- Configured npm scripts for easy server startup
- Basic Express server that can be started and logs requests
- Version-controlled setup with appropriate .gitignore

## Error Handling

If you encounter errors:
- For npm errors: Try clearing cache with `npm cache clean --force`
- For permission issues: Ensure you have write permissions in the current directory
- For missing Node.js: This task requires Node.js 20+ to be pre-installed

Complete all steps sequentially and verify each step before proceeding to the next.