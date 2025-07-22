# Task 3: Create Health and Welcome Endpoints

## Overview
Implement GET `/health` and GET `/` endpoints for health checks and API information. These endpoints provide essential monitoring and discovery capabilities.

## Task Details
- **Priority**: High
- **Dependencies**: Task 2 (Implement Express Server and Middleware)
- **Status**: Pending

## Implementation Guide

### 1. Create Route Files Structure
```
src/
├── routes/
│   ├── index.js         # Main route aggregator
│   ├── welcome.js       # Welcome endpoint
│   └── health.js        # Health check endpoint
└── controllers/
    ├── welcome.js       # Welcome logic
    └── health.js        # Health check logic
```

### 2. Implement Welcome Controller
`/src/controllers/welcome.js`:
```javascript
import { readFileSync } from 'fs';
import { join } from 'path';

export const getWelcome = (req, res) => {
  const packageJson = JSON.parse(
    readFileSync(join(process.cwd(), 'package.json'), 'utf-8')
  );
  
  res.json({
    message: 'Welcome to Simple Express API',
    version: packageJson.version,
    timestamp: new Date().toISOString()
  });
};
```

### 3. Implement Health Controller
`/src/controllers/health.js`:
```javascript
export const getHealth = (req, res) => {
  res.json({
    status: 'ok',
    uptime: process.uptime(),
    timestamp: new Date().toISOString()
  });
};
```

### 4. Create Route Definitions
`/src/routes/welcome.js`:
```javascript
import { Router } from 'express';
import { getWelcome } from '../controllers/welcome.js';

const router = Router();
router.get('/', getWelcome);

export default router;
```

`/src/routes/health.js`:
```javascript
import { Router } from 'express';
import { getHealth } from '../controllers/health.js';

const router = Router();
router.get('/health', getHealth);

export default router;
```

### 5. Integrate Routes in Server
Update `/src/index.js`:
```javascript
import welcomeRoutes from './routes/welcome.js';
import healthRoutes from './routes/health.js';

// After middleware setup
app.use('/', welcomeRoutes);
app.use('/', healthRoutes);
```

## Acceptance Criteria
- [ ] GET `/` returns welcome message with version and timestamp
- [ ] GET `/health` returns status, uptime, and timestamp
- [ ] Both endpoints return proper JSON responses
- [ ] Response time under 100ms
- [ ] Controllers separated from route definitions
- [ ] Clean, modular code structure

## Test Strategy
1. Test welcome endpoint:
   ```bash
   curl http://localhost:3000/
   ```
   Verify: message, version, timestamp fields

2. Test health endpoint:
   ```bash
   curl http://localhost:3000/health
   ```
   Verify: status "ok", uptime > 0, valid timestamp

3. Verify JSON format and content-type headers
4. Test response times are acceptable
5. Ensure endpoints work after server restart