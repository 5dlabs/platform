# Autonomous AI Agent Prompt: Initialize Node.js Project

## Task Overview
You need to initialize a new Node.js project for a Hello World API. This involves creating the project structure, initializing npm, and setting up basic configuration files.

## Detailed Instructions

### Step 1: Create Project Directory Structure
1. Create a new directory named `hello-world-api` if it doesn't exist
2. Navigate into the project directory
3. Create a `src` subdirectory for source code
4. Create an empty `src/index.js` file as the main entry point

### Step 2: Initialize NPM Package
1. Run `npm init -y` to create a package.json with default values
2. This will generate a basic package.json file in the project root

### Step 3: Update Package.json Configuration
Modify the generated package.json file with the following updates:
```json
{
  "name": "hello-world-api",
  "version": "1.0.0",
  "description": "A simple Hello World API built with Node.js and Express",
  "main": "src/index.js",
  "private": true,
  "scripts": {
    "start": "node src/index.js",
    "dev": "nodemon src/index.js"
  },
  "keywords": ["api", "express", "hello-world"],
  "author": "Your Name",
  "license": "MIT",
  "engines": {
    "node": ">=20.0.0"
  }
}
```

### Step 4: Create .gitignore File
Create a `.gitignore` file in the project root with the following content:
```
# Dependencies
node_modules/

# Environment files
.env
.env.local
.env.*.local

# Logs
logs/
*.log
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
*~

# Build output
dist/
build/

# Test coverage
coverage/
.nyc_output/
```

## Expected Outcomes
1. A properly structured Node.js project directory:
   ```
   hello-world-api/
   ├── src/
   │   └── index.js (empty file)
   ├── package.json (configured)
   └── .gitignore (configured)
   ```

2. A valid package.json with:
   - Correct project metadata
   - Start script configured
   - Node.js engine requirement specified

3. A comprehensive .gitignore file excluding common unwanted files

## Validation Steps
1. Verify the directory structure matches expectations
2. Run `npm run` to see available scripts
3. Ensure package.json is valid JSON (no syntax errors)
4. Check that .gitignore includes node_modules/

## Common Issues and Solutions
- If npm init fails, ensure Node.js and npm are properly installed
- If directory already exists, work within the existing structure
- If package.json already exists, update it rather than overwriting

## Notes
- The `private: true` field prevents accidental npm publication
- The dev script with nodemon is for future use (nodemon not installed yet)
- Keep the index.js file empty for now - it will be implemented in later tasks
- The project uses Node.js 20+ for modern JavaScript features