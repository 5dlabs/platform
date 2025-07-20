# Task 1: Project Setup and Configuration

## Overview
This task establishes the foundation for the Simple Todo REST API project by initializing the project structure, installing necessary dependencies, and configuring the development environment. This setup follows the architecture specifications and ensures all team members can quickly begin development.

## Task Details

### Priority
High

### Dependencies
None - This is the initial setup task

### Status
Pending

## Implementation Guide

### 1. Create Project Directory Structure

Following the architecture document's prescribed structure:

```bash
mkdir -p simple-api/{src/{controllers,models,routes,middleware},tests/{unit/{models,controllers,middleware},integration,fixtures},data,docs}
cd simple-api
```

### 2. Initialize Node.js Project

```bash
npm init -y
```

Update `package.json` with project details:
```json
{
  "name": "simple-todo-api",
  "version": "1.0.0",
  "description": "A lightweight REST API for managing todo items",
  "main": "server.js",
  "author": "Development Team",
  "license": "MIT"
}
```

### 3. Install Production Dependencies

Install the core dependencies required for the application:

```bash
npm install express@^4.18.2 \
           better-sqlite3@^9.0.0 \
           express-validator@^7.0.1 \
           swagger-ui-express@^5.0.0 \
           swagger-jsdoc@^6.2.8 \
           dotenv@^16.3.1
```

### 4. Install Development Dependencies

Install development tools for testing, formatting, and development workflow:

```bash
npm install --save-dev jest@^29.7.0 \
                      supertest@^6.3.3 \
                      nodemon@^3.0.1 \
                      prettier@^3.0.3 \
                      @types/jest@^29.5.5
```

### 5. Configure NPM Scripts

Update `package.json` with the following scripts section:

```json
{
  "scripts": {
    "start": "node server.js",
    "dev": "nodemon server.js",
    "test": "jest --coverage",
    "test:watch": "jest --watch",
    "test:coverage": "jest --coverage --coverageReporters=text-lcov | coveralls",
    "format": "prettier --write \"**/*.js\"",
    "lint": "prettier --check \"**/*.js\"",
    "db:init": "node scripts/initDb.js"
  }
}
```

### 6. Create Environment Configuration

Create `.env.example` file:

```env
# Server Configuration
PORT=3000
NODE_ENV=development

# Database Configuration
DB_PATH=./data/todos.db

# API Configuration
API_PREFIX=/api
API_VERSION=v1

# Logging
LOG_LEVEL=info
```

Create actual `.env` file for local development:
```bash
cp .env.example .env
```

### 7. Configure Prettier

Create `.prettierrc` file:

```json
{
  "semi": true,
  "trailingComma": "es5",
  "singleQuote": true,
  "printWidth": 80,
  "tabWidth": 2,
  "useTabs": false,
  "bracketSpacing": true,
  "arrowParens": "always",
  "endOfLine": "lf"
}
```

Create `.prettierignore` file:

```
node_modules/
coverage/
data/
*.db
```

### 8. Create Git Configuration

Create `.gitignore` file:

```gitignore
# Dependencies
node_modules/
npm-debug.log*

# Environment
.env
.env.local
.env.*.local

# Database
data/
*.db
*.sqlite

# Testing
coverage/
.nyc_output/

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Logs
logs/
*.log
```

### 9. Create Basic README

Create initial `README.md`:

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
3. Copy environment variables: `cp .env.example .env`
4. Start the development server: `npm run dev`

### Available Scripts
- `npm start` - Start production server
- `npm run dev` - Start development server with auto-reload
- `npm test` - Run test suite
- `npm run format` - Format code with Prettier

## Project Status
🚧 Under Development - See [Task Master Documentation](.taskmaster/docs) for implementation progress.
```

### 10. Verify Setup

Run the following commands to verify the setup:

```bash
# Check Node version
node --version  # Should be 18+

# Verify dependencies installed
npm list

# Test prettier configuration
npm run format

# Ensure project structure is correct
tree -d -L 3
```

## Key Considerations

### Architecture Alignment
- Project structure matches the architecture document specifications
- All required directories are created following the prescribed layout
- Dependencies align with the technology stack defined in the PRD

### Development Experience
- Hot-reloading enabled with nodemon for efficient development
- Consistent code formatting with Prettier
- Environment-based configuration for flexibility
- Clear separation between production and development dependencies

### Testing Foundation
- Jest configured for unit and integration testing
- Supertest included for API endpoint testing
- Coverage reporting enabled to meet the 90% requirement

## Common Issues and Solutions

### Issue: SQLite3 Build Errors
**Solution**: Ensure you have build tools installed:
- macOS: `xcode-select --install`
- Ubuntu/Debian: `sudo apt-get install build-essential`
- Windows: Install Visual Studio Build Tools

### Issue: Port Already in Use
**Solution**: Either:
- Change the PORT in `.env` file
- Kill the process using the port: `lsof -ti:3000 | xargs kill`

### Issue: Permission Errors
**Solution**: Ensure proper file permissions:
```bash
chmod +x scripts/*.js
chmod 755 data/
```

## Next Steps
After completing this task:
1. Proceed to Task 2: Database Setup and Model Implementation
2. Ensure all team members have successfully set up their local environment
3. Verify CI/CD pipeline recognizes the project structure

## References
- [Architecture Document](../.taskmaster/docs/architecture.md)
- [Product Requirements](../.taskmaster/docs/prd.txt)
- [Express.js Documentation](https://expressjs.com/)
- [Better-SQLite3 Documentation](https://github.com/WiseLibs/better-sqlite3)