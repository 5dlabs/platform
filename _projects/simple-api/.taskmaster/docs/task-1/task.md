# Task 1: Project Setup and Configuration

## Overview

This task involves initializing the project structure, installing all necessary dependencies, and configuring the development environment for the Simple Todo REST API. This foundational task sets up the entire project infrastructure that subsequent tasks will build upon.

## Context

As outlined in the [architecture document](../architecture.md), this project uses Node.js 18+ with Express.js as the web framework. The project follows a standard Node.js application structure with clear separation of concerns and modular organization.

## Implementation Guide

### 1. Create Project Directory Structure

Create the following directory structure as specified in the architecture:

```bash
mkdir -p simple-api/{src/{controllers,models,routes,middleware},tests/{unit/{models,controllers,middleware},integration},data}
cd simple-api
```

### 2. Initialize NPM Project

```bash
npm init -y
```

This creates a `package.json` file with default values.

### 3. Install Production Dependencies

```bash
npm install express better-sqlite3 express-validator swagger-ui-express swagger-jsdoc
```

**Dependencies explained:**
- `express`: Web framework for Node.js
- `better-sqlite3`: Synchronous SQLite3 bindings for Node.js
- `express-validator`: Middleware for request validation
- `swagger-ui-express`: Serve Swagger UI for API documentation
- `swagger-jsdoc`: Generate OpenAPI specification from JSDoc comments

### 4. Install Development Dependencies

```bash
npm install --save-dev jest supertest nodemon prettier
```

**Dev dependencies explained:**
- `jest`: Testing framework
- `supertest`: HTTP assertion library for testing Express apps
- `nodemon`: Auto-restart server on file changes during development
- `prettier`: Code formatter for consistent style

### 5. Configure NPM Scripts

Update `package.json` to include the following scripts:

```json
{
  "scripts": {
    "start": "node server.js",
    "dev": "nodemon server.js",
    "test": "jest --coverage",
    "format": "prettier --write \"**/*.js\""
  }
}
```

### 6. Create Environment Configuration

Create a `.env` file in the root directory:

```bash
PORT=3000
NODE_ENV=development
```

Also create `.env.example` with the same content for version control.

### 7. Create Basic README

Create a `README.md` file with initial setup instructions:

```markdown
# Simple Todo REST API

A lightweight REST API for managing todo items.

## Quick Start

1. Clone the repository
2. Install dependencies: `npm install`
3. Copy `.env.example` to `.env`
4. Start the server: `npm start`

## Development

Run in development mode with auto-reload:
```
npm run dev
```
```

### 8. Create .gitignore

Create a `.gitignore` file:

```
node_modules/
.env
coverage/
data/
*.log
.DS_Store
```

## Dependencies and Relationships

This task has no dependencies as it's the foundational setup. All subsequent tasks depend on this task being completed successfully.

## Success Criteria

1. ✅ Project directory structure matches the architecture specification
2. ✅ All production dependencies are installed and listed in `package.json`
3. ✅ All development dependencies are installed and listed in `devDependencies`
4. ✅ NPM scripts are configured and functional:
   - `npm start` runs the application
   - `npm run dev` runs with nodemon
   - `npm test` runs Jest tests
   - `npm run format` formats code with Prettier
5. ✅ Environment configuration files exist (`.env` and `.env.example`)
6. ✅ Basic README.md with setup instructions exists
7. ✅ `.gitignore` file properly configured

## Testing

To verify the setup:

1. Run `npm list` to ensure all dependencies are installed
2. Run `npm run format` to verify Prettier is working
3. Create a simple `server.js` file and run `npm start` to verify the start script
4. Run `npm run dev` to verify nodemon is working
5. Run `npm test` to verify Jest is configured (it will report no tests found, which is expected)

## Common Issues and Solutions

1. **Port already in use**: Change the PORT in `.env` file
2. **Permission errors during install**: Use `npm install --force` or check npm permissions
3. **SQLite installation issues**: Ensure you have build tools installed for your OS

## Next Steps

After completing this task, proceed to:
- Task 2: Database Setup and Model Implementation
- Task 3: Implement Express Application and Middleware

Both tasks can be started in parallel as they have minimal interdependencies at the start.