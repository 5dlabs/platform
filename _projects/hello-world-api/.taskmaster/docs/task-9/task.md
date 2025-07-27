# Task 9: Implement Root Endpoint

## Overview
This task implements the primary API endpoint that responds to GET requests at the root path (/) with a "Hello, World!" message in JSON format. This endpoint serves as the main functionality of the Hello World API and confirms the API is operational.

## Purpose and Objectives
- Implement the core GET / endpoint as specified in requirements
- Return a properly formatted JSON response
- Set appropriate HTTP status code (200)
- Provide a simple health indicator for the API
- Follow RESTful API conventions
- Complete the minimal viable API functionality

## Technical Approach

### Endpoint Design
1. **HTTP Method**: GET for retrieving data
2. **Path**: Root path (/) for simplicity
3. **Response Format**: JSON with message field
4. **Status Code**: 200 OK for successful responses
5. **Content Type**: application/json (handled by Express)

### Key Technical Decisions
- Use Express's res.json() method for automatic JSON serialization
- Explicitly set status code for clarity
- Place route before error handling middleware
- Keep response structure simple and consistent
- Add descriptive comment for documentation

## Implementation Details

### Route Handler Implementation
```javascript
// Root endpoint - Returns a welcome message to confirm the API is working
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});
```

### Integration Point in src/index.js
```javascript
const express = require('express');
const app = express();

// Port configuration
const PORT = process.env.PORT || 3000;

// Middleware for request logging
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  next();
});

// Routes
// Root endpoint - Returns a welcome message to confirm the API is working
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});

// Error handling middleware
app.use((err, req, res, next) => {
  console.error(`${new Date().toISOString()} - ERROR:`, err.message);
  res.status(500).json({ error: 'Internal Server Error' });
});

// Server startup code continues...
```

### Response Format
```json
{
  "message": "Hello, World!"
}
```

### HTTP Response Headers
- Status: 200 OK
- Content-Type: application/json
- Content-Length: (automatically set)

## Dependencies and Requirements

### Prerequisites
- Completed Task 8: Main server file created
- Express.js server running
- Request logging middleware in place

### Technical Requirements
- Express.js 4.x route handling
- JSON response capability
- HTTP status code support

## Testing Strategy

### Manual Testing

1. **Using curl**
   ```bash
   curl -i http://localhost:3000/
   
   # Expected response:
   HTTP/1.1 200 OK
   Content-Type: application/json; charset=utf-8
   
   {"message":"Hello, World!"}
   ```

2. **Using browser**
   - Navigate to http://localhost:3000/
   - Should see JSON response displayed

3. **Using httpie** (if available)
   ```bash
   http GET localhost:3000/
   
   # Expected:
   HTTP/1.1 200 OK
   Content-Type: application/json; charset=utf-8
   
   {
       "message": "Hello, World!"
   }
   ```

### Automated Testing
```javascript
// test-endpoint.js
const http = require('http');

const options = {
  hostname: 'localhost',
  port: 3000,
  path: '/',
  method: 'GET'
};

const req = http.request(options, (res) => {
  let data = '';
  
  res.on('data', (chunk) => {
    data += chunk;
  });
  
  res.on('end', () => {
    const response = JSON.parse(data);
    console.log('Status:', res.statusCode === 200 ? '✓ 200' : '✗ ' + res.statusCode);
    console.log('Message:', response.message === 'Hello, World!' ? '✓ Correct' : '✗ Incorrect');
    console.log('Content-Type:', res.headers['content-type'].includes('application/json') ? '✓ JSON' : '✗ Not JSON');
  });
});

req.on('error', console.error);
req.end();
```

### Success Criteria
- ✅ GET / returns 200 status code
- ✅ Response is valid JSON
- ✅ Response contains "message" field
- ✅ Message value is "Hello, World!"
- ✅ Content-Type header is application/json
- ✅ Request is logged by middleware

## Related Tasks
- **Previous**: Task 8 - Create Main Server File
- **Next**: Task 10 - Implement Health Endpoint
- **Related**: Task 11 - Add Basic Error Handling

## Notes and Considerations
- The endpoint returns JSON, not plain text
- Status code 200 is explicitly set for clarity
- This endpoint can serve as a basic connectivity test
- The message format matches the PRD requirements exactly
- Consider adding API versioning in production (e.g., /api/v1/)
- The root endpoint is typically used for API information in larger projects
- Request logging will show all access to this endpoint