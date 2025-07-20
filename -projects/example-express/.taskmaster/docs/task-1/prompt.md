# Task 1: Initialize Express Project - Autonomous AI Agent Prompt

You are tasked with setting up a new Express.js project from scratch. This is the foundational task that establishes the project structure and basic server configuration.

## Your Mission

Initialize a complete Express.js project with proper directory structure, install all necessary dependencies, configure the development environment, and implement a basic Express server with essential middleware for security, logging, and error handling.

## Step-by-Step Instructions

### 1. Create Project Structure
Execute these commands to create the directory structure:
```bash
mkdir -p src/{config,middleware,routes,models,utils}
mkdir -p public tests
```

### 2. Initialize npm Project
```bash
npm init -y
```

Update the generated package.json with:
- Proper project name and description
- Main entry point: "src/app.js"
- Author information
- License: "MIT"

### 3. Install Dependencies
Install production dependencies:
```bash
npm install express@^4.19.2 dotenv@^16.4.5 cors@^2.8.5 morgan@^1.10.0 helmet@^7.1.0
```

Install development dependencies:
```bash
npm install --save-dev nodemon@^3.1.0
```

### 4. Create Environment Configuration
Create `.env.example` with:
```
PORT=3000
NODE_ENV=development
JWT_SECRET=your-secret-key-here
DATABASE_URL=./database.sqlite
```

Create `.env` by copying `.env.example` and set appropriate values.

### 5. Implement Express Server
Create `src/app.js` with the following implementation:

```javascript
require('dotenv').config();
const express = require('express');
const helmet = require('helmet');
const cors = require('cors');
const morgan = require('morgan');

const app = express();

// Middleware stack
app.use(helmet());
app.use(cors());
app.use(morgan('combined'));
app.use(express.json());
app.use(express.urlencoded({ extended: true }));

// Health check endpoint
app.get('/health', (req, res) => {
  res.status(200).json({ 
    status: 'OK', 
    timestamp: new Date().toISOString(),
    environment: process.env.NODE_ENV
  });
});

// Global error handling middleware
app.use((err, req, res, next) => {
  console.error(err.stack);
  
  const status = err.status || 500;
  const message = err.message || 'Internal Server Error';
  
  res.status(status).json({
    error: {
      message,
      status,
      ...(process.env.NODE_ENV === 'development' && { stack: err.stack })
    }
  });
});

// 404 handler
app.use((req, res) => {
  res.status(404).json({
    error: {
      message: 'Not Found',
      status: 404
    }
  });
});

const PORT = process.env.PORT || 3000;

const server = app.listen(PORT, () => {
  console.log(`Server running on port ${PORT} in ${process.env.NODE_ENV} mode`);
});

// Graceful shutdown handling
const gracefulShutdown = () => {
  console.log('Received shutdown signal, closing server gracefully...');
  
  server.close(() => {
    console.log('Server closed');
    process.exit(0);
  });
  
  // Force close after 10 seconds
  setTimeout(() => {
    console.error('Could not close connections in time, forcefully shutting down');
    process.exit(1);
  }, 10000);
};

process.on('SIGTERM', gracefulShutdown);
process.on('SIGINT', gracefulShutdown);

module.exports = { app, server };
```

### 6. Configure npm Scripts
Update package.json with these scripts:
```json
{
  "scripts": {
    "start": "node src/app.js",
    "dev": "nodemon src/app.js",
    "test": "jest"
  }
}
```

### 7. Create .gitignore
Create `.gitignore` file:
```
node_modules/
.env
*.log
.DS_Store
database.sqlite
coverage/
.vscode/
.idea/
*.swp
*.swo
dist/
build/
```

### 8. Create Basic README
Create `README.md`:
```markdown
# Express Task Management API

A simple Express.js application with JWT authentication and task management features.

## Installation

1. Clone the repository
2. Install dependencies: `npm install`
3. Copy `.env.example` to `.env` and configure
4. Run development server: `npm run dev`

## Available Scripts

- `npm start` - Start production server
- `npm run dev` - Start development server with auto-reload
- `npm test` - Run tests

## API Endpoints

- `GET /health` - Health check endpoint

More endpoints will be added in subsequent tasks.
```

## Verification Steps

After completing all steps, verify:

1. **Server starts**: Run `npm run dev` and check for console message
2. **Health endpoint works**: Visit http://localhost:3000/health
3. **Environment variables load**: Check that PORT from .env is used
4. **Middleware is active**: 
   - Check response headers for helmet security headers
   - Check console for morgan request logs
5. **Error handling works**: Visit non-existent route like /test
6. **Graceful shutdown**: Press Ctrl+C and verify clean shutdown

## Success Criteria

- All directories created according to structure
- All dependencies installed with correct versions
- Server starts without errors
- Health endpoint returns proper JSON response
- Environment variables are loaded correctly
- All middleware is functioning
- Graceful shutdown works properly
- Git ignores sensitive files

## Important Notes

- Ensure dotenv.config() is called before any other imports that use environment variables
- Middleware order matters: helmet and cors should come before routes
- Always use environment variables for configuration
- Never commit .env file to version control

You have now successfully initialized the Express project. This foundation is ready for the next tasks: database setup, authentication, and API endpoints.