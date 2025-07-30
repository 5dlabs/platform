# Task 4: Implement API Documentation with Swagger - Acceptance Criteria

## Acceptance Criteria Checklist

### 1. File Structure ✓
- [ ] File `docs/openapi.yaml` exists
- [ ] OpenAPI specification is valid YAML
- [ ] File contains OpenAPI 3.0.0 version
- [ ] Response schema is defined in components section

### 2. Swagger Integration in app.js ✓
- [ ] swagger-jsdoc import is present
- [ ] swagger-ui-express import is present
- [ ] swaggerOptions object is configured with:
  - [ ] OpenAPI 3.0.0 definition
  - [ ] API title, version, and description
  - [ ] Dynamic server URL using PORT environment variable
  - [ ] apis array includes route files and openapi.yaml
- [ ] Swagger UI middleware is mounted at /docs
- [ ] JSON endpoint is available at /docs.json
- [ ] Middleware is added in correct order (after core middleware, before routes)

### 3. Root Redirect ✓
- [ ] GET / redirects to /docs
- [ ] Redirect returns 302 status code
- [ ] Location header points to /docs
- [ ] Redirect is configured before main routes

### 4. Documentation Coverage ✓
- [ ] All endpoints appear in Swagger UI:
  - [ ] GET /health
  - [ ] GET /hello
  - [ ] GET /hello/:name
  - [ ] POST /echo
  - [ ] GET /info
- [ ] Each endpoint shows JSDoc-generated documentation
- [ ] Request/response schemas are displayed

### 5. Interactive Features ✓
- [ ] "Try it out" button works for all endpoints
- [ ] Parameters can be entered for /hello/:name
- [ ] Request body can be entered for /echo
- [ ] Execute button sends actual requests
- [ ] Responses are displayed in UI

## Test Cases

### Test Case 1: Root Redirect
```bash
curl -I http://localhost:3000/
```
**Expected Response:**
```
HTTP/1.1 302 Found
Location: /docs
```

### Test Case 2: Swagger UI Access
```bash
curl -I http://localhost:3000/docs
```
**Expected Response:**
```
HTTP/1.1 200 OK
Content-Type: text/html
```

### Test Case 3: JSON Specification
```bash
curl http://localhost:3000/docs.json | jq .
```
**Expected Response Structure:**
```json
{
  "openapi": "3.0.0",
  "info": {
    "title": "Hello World API",
    "version": "1.0.0",
    "description": "A simple REST API that serves as a \"Hello World\" example"
  },
  "servers": [...],
  "paths": {
    "/health": {...},
    "/hello": {...},
    "/hello/{name}": {...},
    "/echo": {...},
    "/info": {...}
  },
  "components": {...}
}
```

### Test Case 4: Verify OpenAPI Specification
```bash
# Check OpenAPI version
curl -s http://localhost:3000/docs.json | jq .openapi
# Expected: "3.0.0"

# Check all paths are documented
curl -s http://localhost:3000/docs.json | jq '.paths | keys'
# Expected: ["/echo", "/health", "/hello", "/hello/{name}", "/info"]
```

### Test Case 5: Schema Validation
```bash
# Check Response schema exists
curl -s http://localhost:3000/docs.json | jq .components.schemas.Response
```
**Expected: Response schema with status, message, data, timestamp properties**

### Test Case 6: Server Configuration
```bash
# Check server URL updates with PORT
PORT=4000 node src/server.js &
sleep 2
curl -s http://localhost:4000/docs.json | jq .servers[0].url
# Expected: "http://localhost:4000"
```

## Browser Testing Checklist

### Swagger UI Visual Verification
1. **Navigate to http://localhost:3000**
   - [ ] Automatically redirects to /docs
   - [ ] Swagger UI loads without errors
   - [ ] API title shows "Hello World API"
   - [ ] Version shows "1.0.0"

2. **Endpoint Display**
   - [ ] All 5 endpoints are listed
   - [ ] Endpoints are expandable
   - [ ] HTTP methods are color-coded (GET=blue, POST=green)

3. **Interactive Testing**
   - [ ] Click on GET /health
   - [ ] Click "Try it out"
   - [ ] Click "Execute"
   - [ ] Response shows in UI with status 200
   - [ ] Response body matches expected format

4. **Parameter Testing**
   - [ ] Expand GET /hello/{name}
   - [ ] Click "Try it out"
   - [ ] Enter "Test" in name field
   - [ ] Execute shows personalized response

5. **Request Body Testing**
   - [ ] Expand POST /echo
   - [ ] Click "Try it out"
   - [ ] Enter JSON in request body
   - [ ] Execute returns the same JSON

## Validation Commands

### Comprehensive Documentation Test
```bash
# Start server
npm run dev &
SERVER_PID=$!
sleep 2

# Test all documentation endpoints
echo "Testing root redirect..."
curl -s -I http://localhost:3000/ | grep Location

echo -e "\nTesting Swagger UI..."
curl -s http://localhost:3000/docs | grep -o "<title>.*</title>"

echo -e "\nTesting JSON spec..."
curl -s http://localhost:3000/docs.json | jq '{
  title: .info.title,
  version: .info.version,
  endpoints: [.paths | keys[] ]
}'

# Cleanup
kill $SERVER_PID
```

### OpenAPI Specification Validation
```bash
# If swagger-cli is installed globally
swagger-cli validate http://localhost:3000/docs.json

# Or use online validator
curl -s http://localhost:3000/docs.json > spec.json
# Upload spec.json to https://editor.swagger.io for validation
```

## Success Indicators
- ✅ Root URL redirects to documentation
- ✅ Swagger UI loads successfully
- ✅ All endpoints are documented
- ✅ Interactive testing works
- ✅ JSON specification is valid OpenAPI 3.0
- ✅ No console errors in browser
- ✅ Documentation is accurate and complete
- ✅ Server URL updates dynamically

## Common Issues and Solutions

### Issue 1: Swagger UI shows but no endpoints
**Solution:** Check that JSDoc comments exist in route files and apis array in swaggerOptions includes correct paths

### Issue 2: "Cannot GET /docs"
**Solution:** Ensure swagger-ui-express middleware is properly configured with both serve and setup

### Issue 3: Root doesn't redirect
**Solution:** Verify redirect is placed before `app.use('/', require('./routes'))`

### Issue 4: Paths show in JSON but not in UI
**Solution:** Check that JSDoc comments follow exact @swagger format

### Issue 5: Server URL incorrect in documentation
**Solution:** Use template literal with `process.env.PORT || 3000` in server configuration

## Performance Considerations
- Documentation UI should load in < 2 seconds
- JSON endpoint should respond in < 100ms
- Documentation should not impact API performance
- Swagger assets are cached by browser

## Documentation Quality Checks
- [ ] All endpoints have summaries
- [ ] All endpoints have descriptions
- [ ] Parameters are documented with types
- [ ] Response codes are documented
- [ ] Examples are provided where applicable
- [ ] Schema references work correctly