# Task 2: Implement Express Server and Middleware

## Overview
Create the Express server with JSON parsing, environment variable support, and clean startup logging. This establishes the foundation for the API server.

## Task Details
- **Priority**: High
- **Dependencies**: Task 1 (Initialize Project and Environment Configuration)
- **Status**: Pending

## Implementation Guide

### 1. Create Main Server File
Create `/src/index.js`:

```javascript
import express from 'express';
import dotenv from 'dotenv';

// Load environment variables
dotenv.config();

// Create Express app
const app = express();
const PORT = process.env.PORT || 3000;

// Middleware
app.use(express.json());

// Server startup
app.listen(PORT, () => {
  const timestamp = new Date().toISOString();
  console.log(`[${timestamp}] Server running on port ${PORT}`);
  console.log(`[${timestamp}] Environment: ${process.env.NODE_ENV || 'development'}`);
});
```

### 2. Configure Package.json
Update `package.json`:
```json
{
  "type": "module",
  "scripts": {
    "start": "node src/index.js",
    "dev": "nodemon src/index.js"
  }
}
```

### 3. Middleware Setup Order
1. **dotenv.config()** - Load environment variables first
2. **express.json()** - Parse JSON request bodies
3. **Routes** - (to be added in subsequent tasks)
4. **Error handlers** - (to be added in Task 5)

### 4. Best Practices Implementation
- Use ES modules (type: "module" in package.json)
- Proper middleware ordering
- Environment-based configuration
- Clean, informative startup logging
- Graceful error handling preparation

## Acceptance Criteria
- [ ] Express server starts on configured port
- [ ] Environment variables load correctly from .env
- [ ] JSON parsing middleware is active
- [ ] Startup logs show port and environment
- [ ] Server responds to requests (even if 404)
- [ ] Both npm start and npm run dev work correctly

## Test Strategy
1. Start server with `npm start` - verify startup logs
2. Start server with `npm run dev` - verify nodemon watches for changes
3. Send JSON POST request to verify body parsing:
   ```bash
   curl -X POST http://localhost:3000/test \
     -H "Content-Type: application/json" \
     -d '{"test": "data"}'
   ```
4. Check that PORT from .env is used
5. Verify server runs when PORT env var is not set (defaults to 3000)