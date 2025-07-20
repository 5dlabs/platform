# Task 1: Project Setup and Configuration - Autonomous Prompt

You are an AI agent tasked with setting up the initial project structure and configuration for a Simple Todo REST API. This is the foundational task that must be completed before any other development work can begin.

## Context
- **Project**: Simple Todo REST API
- **Technology Stack**: Node.js 18+, Express.js 4.x, SQLite with better-sqlite3, Jest for testing
- **Working Directory**: You should be in the project root directory
- **Architecture Reference**: .taskmaster/docs/architecture.md
- **Requirements Reference**: .taskmaster/docs/prd.txt

## Your Mission

Set up a complete Node.js project with all necessary dependencies, configuration files, and directory structure required for building a Simple Todo REST API.

## Detailed Steps

1. **Create Project Structure**
   - Create all directories as specified in the architecture document
   - Ensure the following structure exists:
     ```
     simple-api/
     ├── src/
     │   ├── controllers/
     │   ├── models/
     │   ├── routes/
     │   └── middleware/
     ├── tests/
     │   ├── unit/
     │   ├── integration/
     │   └── fixtures/
     ├── data/
     └── docs/
     ```

2. **Initialize Node.js Project**
   - Run `npm init -y`
   - Update package.json with appropriate project metadata

3. **Install Dependencies**
   - Production dependencies: express, better-sqlite3, express-validator, swagger-ui-express, swagger-jsdoc, dotenv
   - Development dependencies: jest, supertest, nodemon, prettier, @types/jest

4. **Configure NPM Scripts**
   - Add all required scripts to package.json:
     - start, dev, test, test:watch, test:coverage, format, lint, db:init

5. **Set Up Configuration Files**
   - Create `.env.example` with all required environment variables
   - Create `.env` by copying from `.env.example`
   - Create `.prettierrc` with team code style preferences
   - Create `.prettierignore` to exclude non-source files

6. **Configure Git**
   - Create comprehensive `.gitignore` file
   - Ensure sensitive files and build artifacts are excluded

7. **Create Initial Documentation**
   - Create README.md with setup instructions
   - Include all necessary information for new developers

8. **Verify Setup**
   - Run `npm list` to verify all dependencies installed
   - Run `npm run format` to test Prettier configuration
   - Ensure Node version is 18 or higher

## Success Criteria
- ✅ All directories created according to architecture specification
- ✅ package.json contains all required dependencies and scripts
- ✅ Environment configuration files created and properly formatted
- ✅ Prettier configuration working correctly
- ✅ Git ignore file comprehensive and appropriate
- ✅ README provides clear setup instructions
- ✅ All commands run without errors

## Important Notes
- Do NOT start the server yet - just set up the project
- Do NOT create any application code - only configuration
- Ensure all file paths are relative to the project root
- Use exact versions specified in the dependencies where provided
- Follow the exact directory structure from the architecture document

## Error Handling
If you encounter any errors:
1. For SQLite build errors, note that build tools may need to be installed
2. For permission errors, ensure proper file permissions are set
3. For dependency conflicts, use the latest compatible versions

## Validation Commands
After setup, run these commands to validate:
```bash
node --version  # Should show v18 or higher
npm list       # Should show all dependencies
npm run format # Should complete without errors
ls -la src/    # Should show all subdirectories
```

Remember: This is the foundation for the entire project. Take care to set it up correctly as all subsequent tasks depend on this configuration.