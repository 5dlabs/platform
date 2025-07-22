# Autonomous Task Prompt: Initialize Project and Environment Configuration

You are tasked with setting up a new Express.js API project with proper environment configuration and project structure.

## Context
- This is a new project for a Simple Express API
- Target Node.js version: 18+ LTS
- Project will use Express.js 5, dotenv for environment variables, and nodemon for development

## Your Mission
Initialize and configure a new Express.js project with proper directory structure, dependencies, and environment setup.

## Steps to Complete

1. **Initialize the project**
   - Run `npm init -y` to create package.json
   - Verify Node.js version compatibility (18+)

2. **Install dependencies**
   - Install production dependencies: `express@5 dotenv@16`
   - Install development dependencies: `nodemon@3`

3. **Create directory structure**
   ```
   src/
   ├── index.js
   ├── routes/
   ├── controllers/
   ├── middleware/
   └── utils/
   ```

4. **Configure environment**
   - Create `.env` file with `PORT=3000` and `NODE_ENV=development`
   - Create `.gitignore` with standard Node.js exclusions

5. **Create initial documentation**
   - Generate README.md with project setup instructions
   - Document Node.js version requirement
   - Include installation and usage instructions

## Success Criteria
- All directories exist in the correct structure
- Dependencies are installed with exact versions specified
- Environment configuration is working
- Documentation is clear and complete
- Project can be started with npm scripts

## Notes
- Use exact versions specified for all dependencies
- Ensure all configuration follows Express.js best practices
- Keep the setup minimal but production-ready