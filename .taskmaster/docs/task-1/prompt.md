# Task 1: Setup Project Structure and Dependencies - Autonomous Agent Prompt

You are tasked with initializing a new Node.js project for a Hello World REST API. This is the foundational task that sets up the project structure and installs all necessary dependencies.

## Objective
Create a new Node.js project with Express.js framework, install all required dependencies, and establish the complete project directory structure for a production-ready REST API.

## Context
- This is a greenfield project - no existing code
- The API will serve as a test project for the 5D Labs orchestrator workflow
- Must follow Node.js and Express.js best practices
- Project name: `hello-world-api`

## Required Actions

1. **Create Project Directory**
   - Create directory named `hello-world-api`
   - Navigate into the directory
   - Initialize npm project with default settings

2. **Install Dependencies**
   - Install exact versions of core dependencies:
     - express@4.18.2
     - cors@2.8.5
     - helmet@7.0.0
     - pino@8.15.0
     - pino-http@8.5.0
     - dotenv@16.3.1
   - Install exact versions of dev dependencies:
     - jest@29.6.4
     - supertest@6.3.3
     - nodemon@3.0.1
     - eslint@8.48.0
     - swagger-jsdoc@6.2.8
     - swagger-ui-express@5.0.0

3. **Create Directory Structure**
   ```
   src/
     middleware/
     routes/
     utils/
     app.js
     server.js
   tests/
     unit/
     integration/
   docs/
     openapi.yaml
   .env
   .dockerignore
   Dockerfile
   kubernetes.yaml
   README.md
   ```

4. **Configure package.json**
   - Add scripts: start, dev, test, lint
   - Add Jest configuration
   - Set Node.js engine requirement (>=18.0.0)
   - Update description and main entry point

5. **Create Configuration Files**
   - .env with PORT=3000, NODE_ENV=development, LOG_LEVEL=info, API_VERSION=1.0.0
   - .gitignore with standard Node.js patterns
   - .dockerignore with appropriate exclusions
   - .eslintrc.json with basic Node.js configuration

6. **Create Empty Placeholder Files**
   - All files should be created even if empty
   - This ensures the complete structure is ready

## Success Criteria
- All dependencies installed with exact versions specified
- Complete directory structure created
- All configuration files in place
- npm scripts functional (dev, start, test, lint)
- No installation errors
- Project can be started with `npm run dev` (will fail but command should execute)

## Important Notes
- Use exact dependency versions - do not use ^ or ~ prefixes
- Create all directories and files even if empty
- Ensure .env file includes all specified variables
- Make package.json scripts match exactly as specified

## Verification
After completion, verify:
1. Run `npm list` to confirm all dependencies installed
2. Check directory structure matches specification
3. Confirm all configuration files exist
4. Test that npm scripts are defined correctly

Complete this task autonomously without user interaction. Create all files and directories as specified, ensuring the project is ready for development.