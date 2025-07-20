# Task 1: Initialize Express Project and Configure Environment

## Overview

This task establishes the foundation for the Express.js application by setting up the project structure, installing core dependencies, configuring the development environment, and implementing a basic Express server with essential middleware.

## Objectives

- Create a well-organized project directory structure
- Initialize npm project with proper configuration
- Install Express.js and essential middleware packages
- Configure environment variables for different deployment stages
- Implement a basic Express server with security and logging middleware
- Set up development tooling for efficient workflow
- Implement graceful shutdown handling

## Related Project Context

This task aligns with the Product Requirements Document's objective to "Build a minimal but functional Express.js application" and establishes the foundation for all subsequent tasks. It implements the core technical requirements for Express server setup, middleware configuration, and environment management.

## Technical Requirements

### Dependencies
- **Express.js** (^4.19.2): Core web framework
- **dotenv** (^16.4.5): Environment variable management
- **cors** (^2.8.5): Cross-origin resource sharing
- **morgan** (^1.10.0): HTTP request logger
- **helmet** (^7.1.0): Security headers middleware
- **nodemon** (^3.1.0): Development auto-restart tool

### Project Structure
```
/
├── src/
│   ├── app.js              # Main application entry point
│   ├── config/             # Configuration files
│   ├── middleware/         # Custom middleware
│   ├── routes/             # Route definitions
│   ├── models/             # Database models (future)
│   └── utils/              # Utility functions
├── public/                 # Static files
├── tests/                  # Test files
├── .env                    # Environment variables (not committed)
├── .env.example            # Environment template
├── .gitignore              # Git ignore rules
├── package.json            # Project metadata and dependencies
├── package-lock.json       # Dependency lock file
└── README.md               # Project documentation
```

## Implementation Steps

### 1. Project Structure and npm Setup (Subtask 1.1)
```bash
# Create project directories
mkdir -p src/{config,middleware,routes,models,utils}
mkdir -p public tests

# Initialize npm project
npm init -y

# Update package.json with proper metadata
```

### 2. Core Dependencies Installation (Subtask 1.2)
```bash
# Install production dependencies
npm install express@^4.19.2 dotenv@^16.4.5 cors@^2.8.5 morgan@^1.10.0 helmet@^7.1.0

# Install development dependencies
npm install --save-dev nodemon@^3.1.0
```

### 3. Basic Express Server Implementation (Subtask 1.3)
Create `src/app.js`:
```javascript
const express = require('express');
const app = express();

// Health check route
app.get('/health', (req, res) => {
  res.status(200).json({ 
    status: 'OK', 
    timestamp: new Date().toISOString() 
  });
});

const PORT = process.env.PORT || 3000;

const server = app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
});

module.exports = { app, server };
```

### 4. Middleware Configuration (Subtask 1.4)
Update `src/app.js` with middleware stack:
```javascript
const helmet = require('helmet');
const cors = require('cors');
const morgan = require('morgan');

// Security headers
app.use(helmet());

// CORS configuration
app.use(cors());

// Request logging
app.use(morgan('combined'));

// Body parsing
app.use(express.json());
app.use(express.urlencoded({ extended: true }));

// Global error handler
app.use((err, req, res, next) => {
  console.error(err.stack);
  res.status(err.status || 500).json({
    error: {
      message: err.message || 'Internal Server Error',
      status: err.status || 500
    }
  });
});
```

### 5. Environment and Scripts Setup (Subtask 1.5)
Create `.env.example`:
```
PORT=3000
NODE_ENV=development
JWT_SECRET=your-secret-key-here
DATABASE_URL=./database.sqlite
```

Update `package.json` scripts:
```json
{
  "scripts": {
    "start": "node src/app.js",
    "dev": "nodemon src/app.js",
    "test": "jest"
  }
}
```

Create `.gitignore`:
```
node_modules/
.env
*.log
.DS_Store
database.sqlite
coverage/
.vscode/
.idea/
```

### 6. Graceful Shutdown Implementation
Add to `src/app.js`:
```javascript
// Graceful shutdown handling
process.on('SIGTERM', gracefulShutdown);
process.on('SIGINT', gracefulShutdown);

function gracefulShutdown() {
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
}
```

## Acceptance Criteria

- [ ] Project directory structure is created as specified
- [ ] npm is initialized with proper package.json configuration
- [ ] All required dependencies are installed with correct versions
- [ ] Express server starts successfully on configured port
- [ ] Environment variables are loaded from .env file
- [ ] Health check endpoint responds with 200 status
- [ ] Middleware is applied in correct order (helmet → cors → morgan → body parsing)
- [ ] Global error handler catches and formats errors properly
- [ ] npm scripts work correctly (start, dev, test)
- [ ] Server handles SIGTERM and SIGINT signals gracefully
- [ ] .gitignore excludes sensitive files and directories

## Testing

### Manual Testing
1. Start the server: `npm run dev`
2. Visit http://localhost:3000/health
3. Verify response: `{"status":"OK","timestamp":"..."}`
4. Check console for morgan logs
5. Test graceful shutdown with Ctrl+C

### Automated Testing
```javascript
// tests/server.test.js
const request = require('supertest');
const { app, server } = require('../src/app');

describe('Server Setup', () => {
  afterAll(() => {
    server.close();
  });

  test('Health check endpoint', async () => {
    const response = await request(app)
      .get('/health')
      .expect(200);
    
    expect(response.body).toHaveProperty('status', 'OK');
    expect(response.body).toHaveProperty('timestamp');
  });
});
```

## Common Issues and Solutions

### Issue: Port already in use
**Solution**: Change PORT in .env file or kill the process using the port

### Issue: Nodemon not restarting
**Solution**: Check nodemon.json configuration or restart manually

### Issue: Environment variables not loading
**Solution**: Ensure dotenv is configured before other imports

## Next Steps

After completing this task, the project will have:
- A functioning Express server
- Proper project structure
- Essential middleware configured
- Development environment ready

This foundation enables proceeding to Task 2: Setup SQLite Database with Models.