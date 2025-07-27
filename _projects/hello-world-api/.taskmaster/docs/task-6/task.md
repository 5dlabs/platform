# Task 6: Initialize Node.js Project

## Overview
This task establishes the foundation for the Hello World API by creating a new Node.js project with proper configuration and structure. It sets up the essential package.json file, project directories, and npm scripts needed for development.

## Purpose and Objectives
- Create a properly structured Node.js project directory
- Initialize npm package management with appropriate metadata
- Configure npm scripts for running the application
- Establish project structure following Node.js best practices
- Set up version control exclusions with .gitignore

## Technical Approach

### Project Initialization Strategy
1. **Directory Structure Creation**: Establish a clean project structure with separation of concerns
2. **NPM Initialization**: Use npm to create package.json with project metadata
3. **Script Configuration**: Set up essential npm scripts for development workflow
4. **Version Control Setup**: Configure .gitignore for proper repository management

### Key Technical Decisions
- Use `npm init -y` for quick initialization with defaults
- Organize source code in a dedicated `src` directory
- Configure start script to use Node.js directly (no build step needed)
- Include common .gitignore patterns for Node.js projects

## Implementation Details

### Step 1: Create Project Directory Structure
```bash
mkdir hello-world-api
cd hello-world-api
mkdir src
touch src/index.js
```

### Step 2: Initialize NPM Project
```bash
npm init -y
```

### Step 3: Update Package.json
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
  "license": "MIT"
}
```

### Step 4: Create .gitignore
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

## Dependencies and Requirements

### System Requirements
- Node.js 20.x or higher
- npm 10.x or higher
- Git (for version control)

### Development Dependencies
- None required for this task
- Optional: nodemon for development (will be added in later tasks)

## Testing Strategy

### Verification Steps
1. **Package.json Validation**
   - Verify file exists and is valid JSON
   - Check all required fields are present
   - Confirm scripts section contains start command

2. **Directory Structure Verification**
   ```bash
   # Expected structure
   hello-world-api/
   ├── src/
   │   └── index.js
   ├── package.json
   └── .gitignore
   ```

3. **NPM Script Testing**
   ```bash
   # Test that npm start command is configured
   npm run start # Should attempt to run src/index.js
   ```

### Success Criteria
- ✅ Project directory exists with proper structure
- ✅ package.json contains correct metadata and scripts
- ✅ .gitignore excludes node_modules and sensitive files
- ✅ npm start command is properly configured
- ✅ Project is ready for Express.js installation

## Related Tasks
- **Next**: Task 7 - Install Express.js Dependency
- **Blocked By**: None (this is the first task)
- **Related**: All subsequent tasks depend on this foundation

## Notes and Considerations
- The `private: true` flag prevents accidental npm publication
- Using src directory follows common Node.js project conventions
- The dev script with nodemon is optional but improves developer experience
- Consider adding a README.md file for project documentation
- The MIT license is suggested but can be changed based on requirements