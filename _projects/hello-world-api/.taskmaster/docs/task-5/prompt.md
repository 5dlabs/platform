# Autonomous Agent Prompt: Add Error Handling and Documentation

You are an autonomous agent tasked with completing the Hello World API by adding comprehensive error handling and creating user documentation. This final task ensures the API is production-ready.

## Prerequisites

Verify before starting:
- Express server is running with logging middleware
- Hello endpoint (/) is implemented
- Health endpoint (/health) is implemented
- 404 handler exists

## Task Requirements

### Part 1: Implement Error Handling Middleware

1. **Add Error Handler to src/index.js**

Add this error handling middleware AFTER all routes but BEFORE the 404 handler:

```javascript
// Error handling middleware
app.use((err, req, res, next) => {
  console.error(err.stack);
  res.status(500).json({ error: 'Something went wrong!' });
});
```

2. **Ensure Correct Middleware Order**

The final order must be:
- Logging middleware
- Routes (/, /health)
- **Error handling middleware** ← Add here
- 404 handler (must remain last)

### Part 2: Create README Documentation

Create a `README.md` file in the project root with the following content:

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

### GET / - Hello World
Returns a greeting message.

**Response:**
```json
{
  "message": "Hello, World!"
}
```

### GET /health - Health Check
Returns the service health status and current timestamp.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T14:32:17.845Z"
}
```

## Example Requests

### Using curl

```bash
# Hello endpoint
curl http://localhost:3000/

# Health check
curl http://localhost:3000/health
```

### Using HTTPie

```bash
# Hello endpoint
http GET localhost:3000

# Health check
http GET localhost:3000/health
```

## Error Handling

The API includes comprehensive error handling:

- **404 Not Found**: Returned for undefined routes
- **500 Internal Server Error**: Returned for unexpected server errors

## Development

### Project Structure
```
hello-world-api/
├── src/
│   └── index.js      # Main server file
├── package.json      # Dependencies
├── README.md         # This file
└── .gitignore       # Git ignore rules
```

### Requirements
- Node.js 20 or higher
- npm (comes with Node.js)

### Running Tests
```bash
# Start the server
npm start

# In another terminal, test endpoints
curl http://localhost:3000/
curl http://localhost:3000/health
```

## License

ISC
```

## Implementation Steps

1. **Open src/index.js**
2. **Locate the position after routes but before 404 handler**
3. **Insert the error handling middleware**
4. **Save the file**
5. **Create README.md in project root**
6. **Copy the documentation content exactly**
7. **Save README.md**

## Validation Steps

### Validate Error Handler

1. **Test that server still works normally**:
   ```bash
   curl http://localhost:3000/
   curl http://localhost:3000/health
   ```

2. **Test 404 still works**:
   ```bash
   curl http://localhost:3000/invalid
   ```
   Should return: `{"error":"Not found"}`

3. **Verify error handler positioning**:
   - Check that error handler is BEFORE 404 handler
   - Error handler must have 4 parameters: (err, req, res, next)

### Validate Documentation

1. **Check README exists**:
   ```bash
   ls README.md
   ```

2. **Verify all sections present**:
   - Installation
   - Usage
   - Endpoints
   - Examples
   - Error Handling
   - Development

3. **Test example commands**:
   Copy and run each curl command from the README

## Important Notes

### Error Handler Requirements
- Must have exactly 4 parameters
- First parameter must be named `err`
- Must be placed before 404 handler
- Should log error but not expose it to client

### README Requirements
- Use exact markdown formatting
- Include all sections
- Provide working examples
- Document both endpoints
- Explain error responses

## Common Mistakes to Avoid

1. **Wrong parameter count**: Error handlers need exactly 4 parameters
   ```javascript
   // WRONG - only 3 parameters
   app.use((req, res, next) => { ... });
   
   // CORRECT - 4 parameters
   app.use((err, req, res, next) => { ... });
   ```

2. **Wrong placement**: Error handler must be before 404
   ```javascript
   // WRONG ORDER
   app.use(notFoundHandler);  // 404 first
   app.use(errorHandler);     // Error second
   
   // CORRECT ORDER
   app.use(errorHandler);     // Error first
   app.use(notFoundHandler);  // 404 last
   ```

3. **Exposing error details**:
   ```javascript
   // WRONG - exposes stack trace
   res.status(500).json({ error: err.message, stack: err.stack });
   
   // CORRECT - generic message
   res.status(500).json({ error: 'Something went wrong!' });
   ```

## Final Code Structure

Your src/index.js should have this structure:
```javascript
const express = require('express');
const app = express();
const PORT = 3000;

// Logging middleware
app.use((req, res, next) => { ... });

// Routes
app.get('/', (req, res) => { ... });
app.get('/health', (req, res) => { ... });

// Error handling middleware (NEW)
app.use((err, req, res, next) => {
  console.error(err.stack);
  res.status(500).json({ error: 'Something went wrong!' });
});

// 404 handler (must be last)
app.use((req, res) => { ... });

// Server listen
app.listen(PORT, () => { ... });
```

## Success Confirmation

The task is complete when:
- Error handler is in the correct position
- README.md exists with all content
- All existing endpoints still work
- 404 errors still work
- Server doesn't crash on errors

Run this final check:
```bash
# Check files exist
ls -la README.md src/index.js

# Test endpoints
curl http://localhost:3000/
curl http://localhost:3000/health
curl http://localhost:3000/invalid
```

All tests should pass!