# Autonomous Agent Prompt for Task 6: Initialize Node.js Project

## Task Context
You are tasked with initializing a new Node.js project for a Hello World API. This is the foundational task that sets up the project structure and configuration for all subsequent development.

## Your Mission
Create a properly configured Node.js project with the following specifications:
1. Project name: `hello-world-api`
2. Main entry point: `src/index.js`
3. Proper npm configuration with start scripts
4. Version control setup with .gitignore

## Step-by-Step Instructions

### 1. Create Project Directory Structure
```bash
mkdir -p hello-world-api/src
cd hello-world-api
touch src/index.js
```

### 2. Initialize NPM Project
```bash
npm init -y
```

### 3. Update package.json
Modify the generated package.json file to include:
```json
{
  "name": "hello-world-api",
  "version": "1.0.0",
  "description": "A simple Hello World API built with Node.js",
  "main": "src/index.js",
  "private": true,
  "scripts": {
    "start": "node src/index.js",
    "dev": "nodemon src/index.js"
  },
  "keywords": ["api", "hello-world", "express", "nodejs"],
  "author": "",
  "license": "ISC"
}
```

### 4. Create .gitignore File
Create a .gitignore file with the following content:
```
# Dependencies
node_modules/

# Environment files
.env
.env.local
.env.*.local

# Logs
npm-debug.log*
yarn-debug.log*
yarn-error.log*

# OS files
.DS_Store
Thumbs.db

# IDE files
.vscode/
.idea/
*.swp
*.swo

# Build output
dist/
build/
```

### 5. Create Initial index.js
Add a placeholder comment to src/index.js:
```javascript
// Main entry point for Hello World API
// Express server will be configured here
```

## Validation Steps
1. Verify package.json exists and contains correct metadata
2. Confirm src/index.js file exists
3. Check that .gitignore is properly configured
4. Ensure npm start script is defined
5. Run `npm run start` to verify no syntax errors (it's okay if it exits immediately)

## Expected Result
A fully initialized Node.js project ready for Express.js installation with:
- Proper directory structure (hello-world-api/src/)
- Configured package.json with scripts
- Version control setup with .gitignore
- Placeholder entry point file

## Notes
- Do not install any dependencies yet (that's Task 7)
- Focus only on project initialization and configuration
- Ensure all file paths are relative to the project root