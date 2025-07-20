# Task 1: Project Setup and Configuration

## Overview
Initialize the project structure, install dependencies, and configure the development environment for the Simple Todo REST API. This foundational task establishes the project framework following the architecture patterns defined in the system design.

## Task Details
**ID**: 1  
**Title**: Project Setup and Configuration  
**Priority**: High  
**Dependencies**: None  
**Status**: Pending

## Architecture Context
This task implements the project structure outlined in the [architecture document](../../architecture.md):
- Follows the defined directory structure with clear separation of concerns
- Sets up the technology stack: Node.js 18+, Express.js 4.x, SQLite, Jest
- Establishes development workflow with npm scripts and prettier formatting

## Product Requirements Alignment
Aligns with PRD requirements for:
- Technical stack implementation (Node.js, Express, SQLite, Jest)
- Development setup with npm scripts for dev, test, and start
- Environment configuration via .env
- Auto-restart with nodemon for development
- Consistent code formatting with prettier

## Implementation Steps

### 1. Create Project Directory Structure
```bash
mkdir -p src/{controllers,models,routes,middleware}
mkdir -p tests/{unit/{models,controllers,middleware},integration,fixtures}
mkdir -p data docs
```

### 2. Initialize NPM Project
```bash
npm init -y
```

### 3. Install Production Dependencies
```bash
npm install express better-sqlite3 express-validator swagger-ui-express swagger-jsdoc
```

### 4. Install Development Dependencies
```bash
npm install --save-dev jest supertest nodemon prettier @types/jest
```

### 5. Configure Package.json Scripts
Update `package.json` with the following scripts:
```json
{
  "scripts": {
    "start": "node server.js",
    "dev": "nodemon server.js",
    "test": "jest --coverage",
    "test:watch": "jest --watch",
    "format": "prettier --write \"**/*.js\"",
    "lint": "prettier --check \"**/*.js\""
  }
}
```

### 6. Create Environment Configuration
Create `.env` file:
```
PORT=3000
NODE_ENV=development
```

Create `.env.example` file:
```
PORT=3000
NODE_ENV=development
```

### 7. Create Basic README.md
```markdown
# Simple Todo REST API

A lightweight REST API for managing todo items, built with Node.js and Express.

## Getting Started

### Prerequisites
- Node.js 18 or higher
- npm

### Installation
1. Clone the repository
2. Install dependencies: `npm install`
3. Copy `.env.example` to `.env`
4. Start the server: `npm start`

For development with auto-restart:
```
npm run dev
```

### Testing
Run tests with:
```
npm test
```
```

### 8. Create .gitignore
```
node_modules/
.env
coverage/
data/
*.log
.DS_Store
```

### 9. Configure Prettier
Create `.prettierrc.json`:
```json
{
  "semi": true,
  "singleQuote": true,
  "tabWidth": 2,
  "trailingComma": "es5"
}
```

## Success Criteria
- Project directory structure matches architecture specification
- All required dependencies are installed and versions locked in package-lock.json
- NPM scripts work correctly:
  - `npm start` runs the server
  - `npm run dev` runs with nodemon auto-restart
  - `npm test` runs Jest tests
  - `npm run format` formats code with prettier
- Environment configuration is properly set up
- README provides clear setup instructions
- Project is ready for implementation of subsequent tasks

## Related Tasks
- **Next**: [Task 2: Database Setup and Model Implementation](../task-2/task.md)
- This task provides the foundation for all subsequent implementation tasks

## References
- [Architecture Document](../../architecture.md) - Section: Project Structure
- [Product Requirements](../../prd.txt) - Section: Technical Stack, Development Setup