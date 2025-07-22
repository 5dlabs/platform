# Acceptance Criteria: Create Health and Welcome Endpoints

## Core Requirements

### 1. File Structure
- [ ] `/src/routes/` directory exists
- [ ] `/src/controllers/` directory exists
- [ ] Controllers:
  - [ ] `/src/controllers/welcome.js` exists
  - [ ] `/src/controllers/health.js` exists
- [ ] Routes:
  - [ ] `/src/routes/welcome.js` exists
  - [ ] `/src/routes/health.js` exists

### 2. Welcome Endpoint (GET /)
- [ ] Returns 200 OK status
- [ ] Response is JSON format
- [ ] Response includes:
  - [ ] `message` field with welcome text
  - [ ] `version` field from package.json
  - [ ] `timestamp` field in ISO 8601 format
- [ ] Content-Type header is `application/json`

### 3. Health Endpoint (GET /health)
- [ ] Returns 200 OK status
- [ ] Response is JSON format
- [ ] Response includes:
  - [ ] `status` field with value "ok"
  - [ ] `uptime` field with server uptime in seconds
  - [ ] `timestamp` field in ISO 8601 format
- [ ] Content-Type header is `application/json`

### 4. Code Organization
- [ ] Routes use Express Router
- [ ] Controllers contain business logic
- [ ] Routes are thin - only routing logic
- [ ] Proper separation of concerns
- [ ] ES6 module imports used throughout

### 5. Integration
- [ ] Routes properly mounted in main server file
- [ ] No route conflicts
- [ ] Middleware order maintained
- [ ] Server starts without errors

## Test Cases

### Test 1: Welcome Endpoint
```bash
curl -i http://localhost:3000/

# Expected:
# HTTP/1.1 200 OK
# Content-Type: application/json
# {
#   "message": "Welcome to Simple Express API",
#   "version": "1.0.0",
#   "timestamp": "2025-01-22T10:00:00.000Z"
# }
```

### Test 2: Health Endpoint
```bash
curl -i http://localhost:3000/health

# Expected:
# HTTP/1.1 200 OK
# Content-Type: application/json
# {
#   "status": "ok",
#   "uptime": 123.456,
#   "timestamp": "2025-01-22T10:00:00.000Z"
# }
```

### Test 3: Version Accuracy
```bash
# Modify version in package.json to "2.0.0"
# Restart server
curl http://localhost:3000/ | jq .version
# Should return "2.0.0"
```

### Test 4: Uptime Accuracy
```bash
# Start server and note time
# Wait 60 seconds
curl http://localhost:3000/health | jq .uptime
# Should be approximately 60
```

### Test 5: Response Time
```bash
time curl http://localhost:3000/health
# Should complete in under 100ms
```

## Performance Requirements
- [ ] Endpoints respond in < 100ms
- [ ] No blocking operations
- [ ] Efficient JSON serialization
- [ ] Minimal memory allocation

## Security Considerations
- [ ] No sensitive information exposed
- [ ] No internal paths revealed
- [ ] Proper error handling prevents stack traces
- [ ] Version info doesn't reveal vulnerabilities

## Error Handling
- [ ] Package.json read errors handled gracefully
- [ ] Missing package.json doesn't crash server
- [ ] Malformed requests return appropriate errors
- [ ] No unhandled promise rejections

## Definition of Done
- All endpoints return correct data
- Response format matches specification exactly
- Code is modular and maintainable
- Tests pass consistently
- No console errors or warnings
- Ready for additional endpoints