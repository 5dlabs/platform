# Autonomous Task Prompt: Add Error Handling and Documentation

You need to implement error handling middleware and create project documentation. This task involves modifying the Express server and creating a new README file.

## Prerequisites
- Tasks 2, 3, and 4 completed
- Express server running with endpoints
- 404 handler may already exist from Task 2

## Task Requirements

### Part 1: Error Handling Implementation

#### 1. Check Current Middleware
First, check if a 404 handler already exists in src/index.js. If it does, keep it. If not, add it.

#### 2. Add Error Handling Middleware
Add this BEFORE the existing 404 handler (if present):

```javascript
// Error handling middleware
app.use((err, req, res, next) => {
  console.error(err.stack);
  res.status(500).json({ error: 'Something went wrong!' });
});
```

#### 3. Ensure Correct Order
The final middleware order should be:
1. Request logging middleware
2. Route handlers (/, /health)
3. Error handling middleware (new)
4. 404 handler
5. app.listen()

### Part 2: Create README Documentation

Create a new file `README.md` in the project root with this content:

```markdown
# Hello World API

A simple Express.js API that serves a Hello World message and health check endpoint.

## Installation

```bash
npm install
```

## Usage

Start the server:

```bash
npm start
```

The server will run on http://localhost:3000

## Endpoints

- GET / - Returns a Hello World message
- GET /health - Returns the service health status and timestamp

## Example Responses

### Root Endpoint

```json
{
  "message": "Hello, World!"
}
```

### Health Endpoint

```json
{
  "status": "healthy",
  "timestamp": "2023-11-14T12:00:00.000Z"
}
```
```

## Implementation Steps

### For Error Handling:
1. Open src/index.js
2. Locate the 404 handler (if it exists)
3. Add error handler BEFORE the 404 handler
4. Ensure error handler has 4 parameters
5. Save the file

### For Documentation:
1. Create README.md in project root
2. Copy the provided content exactly
3. Ensure proper Markdown formatting
4. Save the file

## Expected File Structure After Task
```
hello-world-api/
├── src/
│   └── index.js (with error handling)
├── package.json
├── README.md (new)
└── .gitignore
```

## Verification Steps

### 1. Test Error Handler
Create a temporary test route:
```javascript
app.get('/test-error', (req, res) => {
  throw new Error('Test error');
});
```

Test it:
```bash
curl http://localhost:3000/test-error
```
Expected: `{"error":"Something went wrong!"}`

Then remove the test route.

### 2. Test 404 Handler
```bash
curl http://localhost:3000/invalid-endpoint
```
Expected: `{"error":"Not found"}`

### 3. Verify Error Logging
When testing error handler, check console for:
- Error stack trace
- Detailed error information

### 4. Test README
```bash
cat README.md
```
Verify all sections are present and formatted correctly.

## Common Issues

### Issue 1: Error Handler Not Working
**Cause**: Missing parameter in function signature
**Fix**: Must have exactly (err, req, res, next)

### Issue 2: 404 Handler Not Working
**Cause**: Placed before routes
**Fix**: Must be after all route definitions

### Issue 3: Duplicate 404 Handlers
**Cause**: One already exists from Task 2
**Fix**: Don't add a second one, use existing

## Final Server Structure
```javascript
const express = require('express');
const app = express();
const PORT = 3000;

// Middleware for logging requests
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  next();
});

// Hello endpoint
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});

// Health check endpoint
app.get('/health', (req, res) => {
  res.status(200).json({
    status: 'healthy',
    timestamp: new Date().toISOString()
  });
});

// Error handling middleware
app.use((err, req, res, next) => {
  console.error(err.stack);
  res.status(500).json({ error: 'Something went wrong!' });
});

// 404 handler for undefined routes
app.use((req, res) => {
  res.status(404).json({ error: 'Not found' });
});

// Server setup
app.listen(PORT, () => {
  console.log(`Server running on http://localhost:${PORT}`);
});
```

## Success Criteria
- Error handler catches and logs errors
- 500 errors return JSON response
- 404 errors return JSON response
- README.md exists with all content
- Server remains stable after errors

Complete both parts of this task before marking as done.