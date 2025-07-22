# Acceptance Criteria: Implement Express Server and Middleware

## Core Requirements

### 1. Server File Structure
- [ ] `/src/index.js` file exists
- [ ] File uses ES6 module syntax (import/export)
- [ ] Proper imports for express and dotenv

### 2. Environment Configuration
- [ ] `dotenv.config()` called before any environment variable usage
- [ ] PORT variable read from `process.env.PORT`
- [ ] Fallback to port 3000 if PORT not set
- [ ] NODE_ENV variable accessible

### 3. Express App Setup
- [ ] Express app instance created
- [ ] JSON middleware configured with `express.json()`
- [ ] Middleware applied in correct order
- [ ] Server listens on configured port

### 4. Startup Logging
- [ ] Server startup message includes:
  - [ ] Timestamp in ISO format
  - [ ] Port number
  - [ ] Environment (development/production)
- [ ] Console output is clear and informative
- [ ] No excessive logging

### 5. Package.json Configuration
- [ ] `"type": "module"` added for ES6 modules
- [ ] Scripts section includes:
  - [ ] `"start": "node src/index.js"`
  - [ ] `"dev": "nodemon src/index.js"`
- [ ] Scripts work without errors

### 6. Error Handling Preparation
- [ ] Basic error handling structure in place
- [ ] Server handles uncaught exceptions gracefully
- [ ] Proper shutdown handling prepared

## Test Cases

### Test 1: Server Startup
```bash
npm start
# Expected output:
# [2025-01-22T10:00:00.000Z] Server running on port 3000
# [2025-01-22T10:00:00.000Z] Environment: development
```

### Test 2: Environment Variable Loading
```bash
PORT=4000 npm start
# Server should start on port 4000
```

### Test 3: JSON Parsing
```bash
# With server running:
curl -X POST http://localhost:3000/test \
  -H "Content-Type: application/json" \
  -d '{"name": "test", "value": 123}'
# Should parse JSON body (even if endpoint returns 404)
```

### Test 4: Nodemon Development Mode
```bash
npm run dev
# Should start with nodemon
# Modify src/index.js
# Server should auto-restart
```

### Test 5: Module System
```javascript
// Verify ES6 imports work:
import express from 'express';
// Should not throw errors
```

## Performance Criteria
- [ ] Server starts in less than 2 seconds
- [ ] Memory usage stays under 100MB on startup
- [ ] No memory leaks during operation
- [ ] Graceful shutdown on SIGTERM/SIGINT

## Security Checks
- [ ] No sensitive data logged to console
- [ ] Environment variables not exposed
- [ ] JSON parsing has size limits
- [ ] No deprecated middleware used

## Definition of Done
- All test cases pass successfully
- Server runs stably without crashes
- Code is clean and well-commented
- Middleware order follows best practices
- Ready for route additions in next tasks