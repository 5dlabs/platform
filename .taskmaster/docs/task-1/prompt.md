# Autonomous Prompt for Task 1: Setup Basic Express Server

## Context

You are tasked with creating a simple Express.js server with a single health check endpoint. This is a minimal example application designed to test the Task Master workflow and demonstrate basic Claude Code execution capabilities.

## Task Requirements

### Primary Objective
Create a working Express.js server with one `/health` endpoint that returns JSON status information.

### Required Steps
1. **Initialize Node.js Project**
   - Run `npm init -y` to create package.json
   - Verify package.json was created successfully

2. **Install Dependencies**
   - Install Express.js: `npm install express`
   - Verify installation by checking package.json dependencies

3. **Create Server File**
   - Create `server.js` with Express server code
   - Include a GET `/health` endpoint
   - Add proper error handling and logging

4. **Configure Package Scripts**
   - Add a `start` script to package.json
   - Ensure the script points to the correct entry file

### Implementation Requirements

#### Server Code Structure
```javascript
const express = require('express');
const app = express();
const port = process.env.PORT || 3000;

// Health check endpoint
app.get('/health', (req, res) => {
  res.json({
    status: 'healthy',
    timestamp: new Date().toISOString(),
    message: 'Express server is running!'
  });
});

// Start server
app.listen(port, () => {
  console.log(`Server running on port ${port}`);
  console.log(`Health check available at http://localhost:${port}/health`);
});
```

#### Package.json Scripts
Ensure the package.json includes:
```json
{
  "scripts": {
    "start": "node server.js"
  }
}
```

## Verification Steps

After implementation, verify the solution by:

1. **Test Server Startup**
   ```bash
   npm start
   ```
   - Should see console logs indicating server started
   - Should not see any error messages

2. **Test Health Endpoint**
   ```bash
   curl http://localhost:3000/health
   ```
   - Should return JSON response with status, timestamp, and message
   - Should receive 200 HTTP status code

3. **Verify File Structure**
   - `package.json` exists with correct dependencies and scripts
   - `server.js` exists with complete server implementation
   - `node_modules/` directory exists with Express installed

## Expected Deliverables

1. **package.json** - Node.js project configuration with Express dependency
2. **server.js** - Express server with health endpoint
3. **Working server** - Able to start with `npm start` and respond to requests

## Success Criteria

The task is complete when:
- ✅ `npm start` successfully starts the server without errors
- ✅ GET request to `/health` returns proper JSON response
- ✅ Console shows startup messages with port information
- ✅ All files are properly created and configured

## Notes

- Use port 3000 as default (allow PORT environment variable override)
- Include proper error handling in the server code
- Add clear console logging for debugging
- Keep the implementation simple and focused on the core requirements