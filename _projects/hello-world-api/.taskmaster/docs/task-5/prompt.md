# Autonomous Agent Prompt: Add Error Handling and Documentation

You are tasked with completing the Hello World API by adding robust error handling and comprehensive documentation.

## Your Mission
Implement error handling middleware and create a complete README file that documents the API usage and features.

## Prerequisites
- Tasks 3 & 4 completed (both endpoints implemented)
- Server structure finalized
- All routes defined

## Required Actions

### Part 1: Error Handling Implementation

#### 1. Add Error Handling Middleware
Add to `src/index.js` AFTER all route definitions:

```javascript
// Error handling middleware
app.use((err, req, res, next) => {
  console.error(err.stack);
  res.status(500).json({ error: 'Something went wrong!' });
});
```

**Important**: This MUST have 4 parameters (err, req, res, next) to be recognized as error middleware.

#### 2. Add 404 Handler
Add AFTER error handling middleware:

```javascript
// 404 handler for undefined routes
app.use((req, res) => {
  res.status(404).json({ error: 'Not found' });
});
```

**Important**: This MUST be the very last middleware before `app.listen()`.

#### 3. Verify Middleware Order
Your `src/index.js` should have this structure:
1. Imports and setup
2. Request logging middleware
3. Route handlers (/, /health)
4. Error handling middleware (4 params)
5. 404 handler
6. app.listen()

### Part 2: README Documentation

#### 1. Create README.md
Create `README.md` in the project root with this exact content:

```markdown
# Hello World API

A simple Express.js API that serves a Hello World message and health check endpoint.

## Installation

\`\`\`
npm install
\`\`\`

## Usage

Start the server:

\`\`\`
npm start
\`\`\`

The server will run on http://localhost:3000

## Endpoints

- GET / - Returns a Hello World message
- GET /health - Returns the service health status and timestamp

## Example Responses

### Root Endpoint

\`\`\`json
{
  "message": "Hello, World!"
}
\`\`\`

### Health Endpoint

\`\`\`json
{
  "status": "healthy",
  "timestamp": "2023-11-14T12:00:00.000Z"
}
\`\`\`
```

## Validation Tests

### Test 1: 404 Handler
```bash
curl http://localhost:3000/unknown
```
**Expected:** `{"error":"Not found"}`

### Test 2: Wrong Method on Root
```bash
curl -X POST http://localhost:3000/
```
**Expected:** `{"error":"Not found"}`

### Test 3: Error Handling (Optional Test)
Temporarily add this route to test error handling:
```javascript
app.get('/test-error', (req, res) => {
  throw new Error('Test error');
});
```

Test it:
```bash
curl http://localhost:3000/test-error
```
**Expected:** `{"error":"Something went wrong!"}`
**Console:** Should show error stack trace

Remove the test route after verification.

### Test 4: README Rendering
Open README.md in a markdown viewer or push to GitHub to verify:
- Proper formatting
- Code blocks render correctly
- All sections present

## Common Mistakes to Avoid

### 1. Wrong Middleware Order
❌ Placing 404 handler before routes
❌ Placing error handler after app.listen()
✅ Error handler → 404 handler → app.listen()

### 2. Missing Error Handler Parameters
❌ `app.use((req, res, next) => { ... })` - Won't catch errors
✅ `app.use((err, req, res, next) => { ... })` - 4 parameters required

### 3. Exposing Sensitive Information
❌ `res.status(500).json({ error: err.stack })`
✅ `res.status(500).json({ error: 'Something went wrong!' })`

### 4. README Formatting
❌ Using single backticks for code blocks
✅ Using triple backticks for code blocks

## Full Integration Test
Run this script to verify everything works:

```bash
#!/bin/bash
echo "Testing API endpoints and error handling..."

# Test root endpoint
echo "1. Testing root endpoint..."
curl -s http://localhost:3000/ | grep -q "Hello, World!" && echo "✓ Root endpoint works" || echo "✗ Root endpoint failed"

# Test health endpoint
echo "2. Testing health endpoint..."
curl -s http://localhost:3000/health | grep -q "healthy" && echo "✓ Health endpoint works" || echo "✗ Health endpoint failed"

# Test 404 handler
echo "3. Testing 404 handler..."
response=$(curl -s http://localhost:3000/nonexistent)
echo "$response" | grep -q "Not found" && echo "✓ 404 handler works" || echo "✗ 404 handler failed"

# Test wrong method
echo "4. Testing method not allowed..."
response=$(curl -s -X PUT http://localhost:3000/)
echo "$response" | grep -q "Not found" && echo "✓ Method handling works" || echo "✗ Method handling failed"

# Check README exists
echo "5. Checking README..."
[ -f README.md ] && echo "✓ README.md exists" || echo "✗ README.md missing"

echo "Tests complete!"
```

## Success Criteria
- Error middleware catches and logs errors
- 404 handler responds to undefined routes
- All error responses use JSON format
- README.md exists with all required sections
- Documentation accurately describes the API
- All validation tests pass

Complete both parts and run all validation tests before marking this task as complete.