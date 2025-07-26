# Task 3: Implement Hello Endpoint

## Overview
This task implements the primary functionality of the Hello World API by creating the root endpoint that returns a JSON response with a greeting message. This endpoint fulfills the core requirement of the API to respond with "Hello, World!" when accessed.

## Objectives
- Implement GET / endpoint
- Return JSON response with "Hello, World!" message
- Ensure HTTP 200 status code
- Follow RESTful API conventions
- Replace placeholder route from Task 2

## Technical Approach

### 1. Route Definition
The endpoint uses Express.js route handling to respond to GET requests at the root path (/). The handler function receives request and response objects, allowing full control over the HTTP response.

### 2. Response Format
The API returns a JSON object following common REST API practices:
```json
{
  "message": "Hello, World!"
}
```

### 3. Status Code
HTTP 200 (OK) indicates successful request processing, appropriate for a simple data retrieval operation.

## Implementation Details

### Complete Route Handler
```javascript
// Hello endpoint
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});
```

### Integration Points
1. Place after middleware setup
2. Replace the placeholder route from Task 2
3. Before the 404 catch-all handler
4. After request logging middleware

### Response Characteristics
- **Content-Type**: application/json
- **Status Code**: 200 OK
- **Response Body**: JSON object with message property
- **Encoding**: UTF-8

## Dependencies
- **Task 2**: Create Express.js Server (must have basic server running)
- **Express.js**: Web framework for route handling

## Success Criteria
- [ ] GET / endpoint defined
- [ ] Returns JSON response: `{"message":"Hello, World!"}`
- [ ] HTTP status code is 200
- [ ] Content-Type header is application/json
- [ ] Request is logged by middleware
- [ ] Replaces placeholder route successfully

## Testing Strategy

### Manual Testing
```bash
# Using curl
curl -i http://localhost:3000/

# Expected response headers:
HTTP/1.1 200 OK
Content-Type: application/json; charset=utf-8

# Expected response body:
{"message":"Hello, World!"}
```

### Alternative Testing Methods
```bash
# Using wget
wget -qO- http://localhost:3000/

# Using httpie (if installed)
http GET localhost:3000

# Browser testing
# Navigate to: http://localhost:3000
```

### Validation Points
1. Status code is exactly 200
2. Content-Type includes application/json
3. Response is valid JSON
4. Message field contains exact text
5. No extra fields in response

## Related Tasks
- **Task 2**: Provides server infrastructure
- **Task 4**: Implements health endpoint (similar pattern)
- **Task 8**: Will test this endpoint functionality
- **Task 6**: Logs requests to this endpoint

## Notes
- The `res.json()` method automatically sets Content-Type header
- Using `res.status(200)` is explicit but optional (200 is default)
- JSON response allows easy client parsing
- Consider adding CORS headers for browser access in future