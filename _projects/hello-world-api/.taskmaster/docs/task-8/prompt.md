# Autonomous Task Prompt: Create Main Server File (Verification)

This task appears to duplicate Task 2's server implementation. Your job is to verify the existing implementation and make any necessary enhancements.

## Context
Task 2 already created src/index.js with:
- Express server setup
- Request logging middleware
- Server listening on port 3000

Task 8 specifies similar requirements with potential additions.

## Verification Requirements

### 1. Check Existing Implementation
```bash
# Verify file exists
cat src/index.js
```

Confirm it contains:
- Express import and initialization
- Request logging middleware
- Server listening setup

### 2. Compare Requirements

**Task 2 Implementation:**
- ✅ Basic Express server
- ✅ Request logging
- ✅ Port 3000
- ✅ Startup message

**Task 8 Additional Requirements:**
- ⚠️ Environment variable for PORT
- ⚠️ Error handling for startup

### 3. Enhancement Decision

**Option A: No Changes Needed**
If current implementation is sufficient:
1. Document that requirements are met
2. No code changes required
3. Mark task as verification complete

**Option B: Add Enhancements**
If missing environment variable support or error handling:

```javascript
// Update PORT definition
const PORT = process.env.PORT || 3000;

// Add error handling
const server = app.listen(PORT, () => {
  console.log(`Server running on http://localhost:${PORT}`);
});

server.on('error', (error) => {
  if (error.code === 'EADDRINUSE') {
    console.error(`Port ${PORT} is already in use`);
  } else {
    console.error('Server startup error:', error);
  }
  process.exit(1);
});
```

## Implementation Steps

### Step 1: Analyze Current Code
```bash
cat src/index.js
```

### Step 2: Identify Gaps
Check for:
- [ ] PORT uses process.env.PORT || 3000
- [ ] Error handling exists
- [ ] All other requirements met

### Step 3: Make Updates (if needed)
Only if gaps identified:
1. Update PORT configuration
2. Add error event listener
3. Test changes

### Step 4: Verification Testing
```bash
# Test default port
npm start

# Test custom port
PORT=4000 npm start

# Test port conflict (run in two terminals)
npm start
# In second terminal:
npm start  # Should show error
```

## Common Scenarios

### Scenario 1: All Requirements Met
- Current implementation from Task 2 is complete
- No changes needed
- Document verification complete

### Scenario 2: Missing ENV Support
- Update PORT to use process.env.PORT
- Test with custom ports
- Verify backward compatibility

### Scenario 3: Missing Error Handling
- Add server error event listener
- Test port conflicts
- Ensure graceful failure

## Expected Final Code
```javascript
const express = require('express');
const app = express();
const PORT = process.env.PORT || 3000;

// Middleware for logging requests
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  next();
});

// Routes (from Tasks 3 & 4)
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});

app.get('/health', (req, res) => {
  res.status(200).json({
    status: 'healthy',
    timestamp: new Date().toISOString()
  });
});

// Error handling (from Task 5)
app.use((err, req, res, next) => {
  console.error(err.stack);
  res.status(500).json({ error: 'Something went wrong!' });
});

app.use((req, res) => {
  res.status(404).json({ error: 'Not found' });
});

// Server setup with error handling
const server = app.listen(PORT, () => {
  console.log(`Server running on http://localhost:${PORT}`);
});

server.on('error', (error) => {
  if (error.code === 'EADDRINUSE') {
    console.error(`Port ${PORT} is already in use`);
  } else {
    console.error('Server startup error:', error);
  }
  process.exit(1);
});
```

## Success Criteria
- Server implementation verified
- Any gaps identified and fixed
- All tests pass
- Documentation updated

Since this duplicates Task 2, focus on verification rather than reimplementation.