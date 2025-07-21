# Task 1: Project Setup and Configuration - Autonomous Prompt

You are tasked with setting up a new Node.js project for a Simple Todo REST API. This is the foundational task that establishes the project structure and installs all necessary dependencies.

## Your Mission

Initialize a complete Node.js project with Express.js, including all required dependencies, project structure, and configuration files for a REST API that will manage todo items.

## Required Actions

1. **Create Project Structure**
   ```
   simple-api/
   ├── src/
   │   ├── controllers/
   │   ├── models/
   │   ├── routes/
   │   └── middleware/
   ├── tests/
   │   ├── unit/
   │   │   ├── models/
   │   │   ├── controllers/
   │   │   └── middleware/
   │   └── integration/
   └── data/
   ```

2. **Initialize NPM Project**
   - Run `npm init -y` to create package.json

3. **Install Dependencies**
   
   Production dependencies:
   ```bash
   npm install express better-sqlite3 express-validator swagger-ui-express swagger-jsdoc
   ```
   
   Development dependencies:
   ```bash
   npm install --save-dev jest supertest nodemon prettier
   ```

4. **Configure NPM Scripts**
   
   Add to package.json:
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

5. **Create Configuration Files**
   
   `.env`:
   ```
   PORT=3000
   NODE_ENV=development
   ```
   
   `.env.example` (same content as .env)
   
   `.gitignore`:
   ```
   node_modules/
   .env
   coverage/
   data/
   *.log
   .DS_Store
   ```

6. **Create Initial README.md**
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

## Success Verification

Ensure all of the following are true:
- [ ] All directories exist in the correct structure
- [ ] package.json exists with all dependencies listed
- [ ] All npm scripts work when tested
- [ ] .env and .env.example files exist
- [ ] .gitignore file exists with proper entries
- [ ] README.md exists with setup instructions

## Important Notes

- Use Node.js 18 or higher
- Ensure all dependencies install without errors
- The project should be ready for immediate development after this setup
- Do not create any application code yet - only setup and configuration

## Context

This is the first task in building a Simple Todo REST API. The project will use:
- Express.js for the web framework
- SQLite for the database
- Jest for testing
- Swagger for API documentation

Once complete, other developers should be able to clone the repository, run `npm install`, and immediately start working on the application code.