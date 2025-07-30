# Setup Project Structure and Dependencies

You are tasked with initializing a Node.js project for a Hello World API that serves as a testing ground for the 5D Labs orchestrator workflow. This is the foundational task that establishes the project structure and dependencies.

## Your Mission
Create a complete Node.js Express.js project setup with all necessary dependencies, proper directory structure, and configuration files for a production-ready REST API service.

## Technical Requirements

### Project Initialization
1. Create project directory: `hello-world-api`
2. Initialize with `npm init -y`
3. Install exact dependency versions as specified

### Core Dependencies (Production)
Install these exact versions:
```bash
npm install express@4.18.2 cors@2.8.5 helmet@7.0.0 pino@8.15.0 pino-http@8.5.0 dotenv@16.3.1
```

### Development Dependencies
Install these exact versions:
```bash
npm install --save-dev jest@29.6.4 supertest@6.3.3 nodemon@3.0.1 eslint@8.48.0 swagger-jsdoc@6.2.8 swagger-ui-express@5.0.0
```

### Directory Structure
Create this exact structure:
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
├── Dockerfile
├── kubernetes.yaml
└── README.md
```

### Configuration Files

**Package.json Scripts:**
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

**.env File:**
```env
PORT=3000
NODE_ENV=development
LOG_LEVEL=info
API_VERSION=v1
```

**.gitignore:**
```
node_modules/
.env
*.log
coverage/
.nyc_output/
```

**.dockerignore:**
```
node_modules/
.git/
.env
*.log
coverage/
tests/
README.md
```

## Critical Success Factors
- Use EXACT dependency versions specified above
- Create ALL directories and files in the structure
- Ensure `npm install` completes without errors
- Verify `npm run dev` starts without errors (even if server.js is empty)
- All configuration files must be properly formatted

## Architecture Context
This project follows a three-layer architecture:
- **API Layer**: Routes for HTTP handling
- **Business Logic**: Utils for services
- **Infrastructure**: Middleware for cross-cutting concerns

## Validation
After completion:
1. Run `npm install` - should complete successfully
2. Run `npm run dev` - should start without errors
3. Verify all files and directories exist
4. Check that package.json contains all specified scripts

## Important Notes
- This is a foundational task - all other tasks depend on this setup
- Use exact versions specified to ensure compatibility
- Create empty files where content will be added later
- Follow Node.js community best practices for project structure

Complete this task systematically, verifying each step before proceeding to the next.