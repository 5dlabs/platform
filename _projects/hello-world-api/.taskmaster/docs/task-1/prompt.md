# Task 1: Setup Project Structure and Dependencies - Autonomous Agent Prompt

You are an experienced Node.js developer tasked with setting up the initial project structure for a REST API called "Hello World API". This is a critical first task that establishes the foundation for all subsequent development.

## Your Mission
Initialize a new Node.js project with Express.js, install all required dependencies, create the recommended directory structure, and configure the project for development.

## Detailed Instructions

### 1. Project Initialization
- Create a new directory called `hello-world-api` if it doesn't exist
- Navigate into the directory
- Initialize a new Node.js project using `npm init -y`
- This creates a default package.json that you'll customize

### 2. Dependency Installation
Install the following exact versions to ensure compatibility:

**Core Dependencies:**
```bash
npm install express@4.18.2 cors@2.8.5 helmet@7.0.0 pino@8.15.0 pino-http@8.5.0 dotenv@16.3.1
```

**Development Dependencies:**
```bash
npm install --save-dev jest@29.6.4 supertest@6.3.3 nodemon@3.0.1 eslint@8.48.0 swagger-jsdoc@6.2.8 swagger-ui-express@5.0.0
```

### 3. Directory Structure Creation
Create the following directory structure:
```
hello-world-api/
├── src/
│   ├── middleware/
│   ├── routes/
│   ├── utils/
│   ├── app.js
│   └── server.js
├── tests/
│   ├── unit/
│   └── integration/
├── docs/
│   └── openapi.yaml
├── .env
├── .dockerignore
├── .gitignore
├── Dockerfile
├── kubernetes.yaml
├── package.json
└── README.md
```

### 4. Configure package.json
Update the package.json file with:
- Proper project metadata (name, version, description)
- Main entry point: `src/server.js`
- Scripts for start, dev, test, and lint

Example scripts section:
```json
{
  "scripts": {
    "start": "node src/server.js",
    "dev": "nodemon src/server.js",
    "test": "jest --coverage",
    "lint": "eslint ."
  }
}
```

### 5. Create Configuration Files

**.env file** should contain:
```
PORT=3000
NODE_ENV=development
API_VERSION=1.0.0
LOG_LEVEL=info
```

**.gitignore file** should include standard Node.js patterns:
- node_modules/
- .env files
- logs/
- coverage/
- IDE files (.vscode/, .idea/)
- OS files (.DS_Store, Thumbs.db)

**.dockerignore file** should exclude:
- node_modules/
- npm-debug.log
- .git/
- .env files
- coverage/
- test files
- documentation

### 6. Create Initial Files
Create empty placeholder files:
- `src/app.js` - Will contain Express app configuration
- `src/server.js` - Will contain server startup code
- `docs/openapi.yaml` - Will contain API documentation
- `README.md` - Add basic project title and description
- `Dockerfile` - Empty for now
- `kubernetes.yaml` - Empty for now

## Validation Steps
After completing the setup:
1. Run `npm install` to ensure all dependencies are properly installed
2. Check that all directories and files exist as specified
3. Verify package.json has all the required scripts
4. Ensure .env file contains all required environment variables
5. Test that `npm run dev` command is available (don't run it, just verify)

## Important Notes
- Use exact dependency versions specified to avoid compatibility issues
- Create all directories even if they'll be empty initially
- Ensure all configuration files are created, even if minimal
- The project should be ready for immediate development after this setup

## Expected Outcome
A fully initialized Node.js project with:
- All dependencies installed and locked in package-lock.json
- Complete directory structure ready for development
- Configuration files prepared for development and deployment
- Project scripts configured for common development tasks

Complete this task methodically, ensuring each step is done correctly before moving to the next. The success of the entire project depends on this proper foundation.