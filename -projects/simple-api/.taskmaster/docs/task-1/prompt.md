# Task 1: Project Setup and Configuration - Autonomous Prompt

You are an AI agent tasked with initializing a Node.js project for the Simple Todo REST API. Your goal is to set up the complete project structure, install all necessary dependencies, and configure the development environment.

## Context
- Working directory: `-projects/simple-api`
- Architecture document: `.taskmaster/docs/architecture.md`
- Product requirements: `.taskmaster/docs/prd.txt`
- This is the first task in the project, so no prior implementation exists

## Your Mission
Set up the complete project foundation including directory structure, dependencies, configuration files, and initial documentation. Ensure all components align with the architecture specification and prepare the project for subsequent implementation tasks.

## Required Actions

### 1. Create Project Structure
Create the following directory structure:
```
simple-api/
├── src/
│   ├── controllers/
│   ├── models/
│   ├── routes/
│   ├── middleware/
│   ├── config/
│   └── utils/
├── tests/
│   ├── unit/
│   │   ├── models/
│   │   ├── controllers/
│   │   └── middleware/
│   ├── integration/
│   ├── fixtures/
│   └── helpers/
├── data/
└── docs/
```

### 2. Initialize NPM Project
- Run `npm init -y` to create package.json
- Update package.json with appropriate project metadata

### 3. Install Dependencies
Production dependencies:
- express
- better-sqlite3
- express-validator
- swagger-ui-express
- swagger-jsdoc
- dotenv

Development dependencies:
- jest
- supertest
- nodemon
- prettier
- @types/jest

### 4. Configure NPM Scripts
Add the following scripts to package.json:
- `start`: Run the production server
- `dev`: Run development server with nodemon
- `test`: Run Jest tests with coverage
- `test:watch`: Run tests in watch mode
- `format`: Format code with prettier
- `lint`: Check code formatting

### 5. Create Configuration Files

**.env**:
```
PORT=3000
NODE_ENV=development
```

**.env.example**: Same as .env but for version control

**.gitignore**:
- node_modules/
- .env
- coverage/
- data/
- *.log
- .DS_Store

**.prettierrc.json**:
```json
{
  "semi": true,
  "singleQuote": true,
  "tabWidth": 2,
  "trailingComma": "es5"
}
```

### 6. Create Initial README.md
Include:
- Project title and description
- Prerequisites (Node.js 18+)
- Installation instructions
- Available npm scripts
- Basic project structure

## Validation Criteria
- All directories exist in the correct structure
- package.json contains all required dependencies and scripts
- Configuration files are properly formatted
- npm install completes without errors
- npm scripts are executable
- Project follows the architecture specification

## Important Notes
- Ensure all file paths are created relative to the working directory
- Lock dependency versions in package-lock.json
- Follow the exact structure specified in the architecture document
- Do not implement any application code - only setup and configuration
- Ensure the project is ready for Task 2 (Database Setup)

## Expected Outcome
A fully configured Node.js project with:
- Complete directory structure matching architecture specs
- All dependencies installed and locked
- Development environment configured
- NPM scripts ready for use
- Basic documentation in place
- Project ready for implementation of subsequent tasks

Execute all steps systematically and verify each component is correctly configured before proceeding to the next.