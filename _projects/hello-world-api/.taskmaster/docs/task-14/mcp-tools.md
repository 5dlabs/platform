# MCP Tools for Task 14: Manual Testing of API Endpoints

## Tool Selection Reasoning
This task focuses on manual testing and documentation of test results. While the actual testing would use curl or Postman, the filesystem tool is needed for:
- Creating test documentation and reports
- Reading server files to understand expected behavior
- Saving test scripts for future use
- Creating Postman collection exports

No remote tools are required as all testing is performed against the local API instance.

## Selected Tools

### filesystem (Local Tool)
**Description**: File system operations for reading, writing, and managing files

**Why Selected**: Essential for creating test documentation, saving test results, and generating test reports. Also useful for creating reusable test scripts and Postman collections.

**Available Operations**:
- `write_file`: Create test reports and documentation
- `read_file`: Review implementation for expected behavior
- `create_directory`: Organize test artifacts
- `list_directory`: Verify project structure

**Task-Specific Usage Examples**:

1. **Create Test Report**:
```javascript
write_file({
  path: "hello-world-api/test-report.md",
  content: `# API Test Report

**Date:** ${new Date().toISOString().split('T')[0]}
**Environment:** Local Development
**Base URL:** http://localhost:3000

## Test Summary

| Endpoint | Method | Expected Status | Actual Status | Result |
|----------|---------|----------------|---------------|---------|
| / | GET | 200 | 200 | ✅ PASS |
| /health | GET | 200 | 200 | ✅ PASS |
| /undefined | GET | 404 | 404 | ✅ PASS |

## Detailed Test Results

### 1. Root Endpoint (GET /)
[... detailed results ...]

### 2. Health Endpoint (GET /health)
[... detailed results ...]

### 3. Error Handling
[... detailed results ...]

## Conclusion
All tests passed successfully. The API is functioning according to specifications.`
})
```

2. **Create Test Scripts**:
```javascript
// Create directory for test scripts
create_directory({ path: "hello-world-api/tests" })

// Create bash test script
write_file({
  path: "hello-world-api/tests/api-tests.sh",
  content: `#!/bin/bash

# API Test Script
BASE_URL="http://localhost:3000"

echo "Testing Hello World API..."

# Test root endpoint
echo -n "Testing GET /: "
response=$(curl -s -w "\\n%{http_code}" $BASE_URL/)
status_code=$(echo "$response" | tail -n 1)
body=$(echo "$response" | head -n -1)
if [ "$status_code" = "200" ] && [ "$body" = '{"message":"Hello, World!"}' ]; then
  echo "✅ PASS"
else
  echo "❌ FAIL (Status: $status_code, Body: $body)"
fi

# Test health endpoint
echo -n "Testing GET /health: "
response=$(curl -s -w "\\n%{http_code}" $BASE_URL/health)
status_code=$(echo "$response" | tail -n 1)
if [ "$status_code" = "200" ] && echo "$response" | grep -q '"status":"healthy"'; then
  echo "✅ PASS"
else
  echo "❌ FAIL"
fi

# Test 404 handling
echo -n "Testing 404 handling: "
response=$(curl -s -w "\\n%{http_code}" $BASE_URL/undefined)
status_code=$(echo "$response" | tail -n 1)
body=$(echo "$response" | head -n -1)
if [ "$status_code" = "404" ] && [ "$body" = '{"error":"Not Found"}' ]; then
  echo "✅ PASS"
else
  echo "❌ FAIL"
fi

echo "Testing complete!"`
})
```

3. **Create Postman Collection**:
```javascript
write_file({
  path: "hello-world-api/tests/postman-collection.json",
  content: JSON.stringify({
    "info": {
      "name": "Hello World API Tests",
      "description": "Complete test collection for Hello World API",
      "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
    },
    "variable": [
      {
        "key": "baseUrl",
        "value": "http://localhost:3000",
        "type": "string"
      }
    ],
    "item": [
      {
        "name": "Successful Requests",
        "item": [
          {
            "name": "Get Hello World",
            "request": {
              "method": "GET",
              "url": "{{baseUrl}}/",
              "description": "Should return Hello World message"
            },
            "response": []
          },
          {
            "name": "Health Check",
            "request": {
              "method": "GET",
              "url": "{{baseUrl}}/health",
              "description": "Should return health status"
            },
            "response": []
          }
        ]
      },
      {
        "name": "Error Scenarios",
        "item": [
          {
            "name": "404 Not Found",
            "request": {
              "method": "GET",
              "url": "{{baseUrl}}/nonexistent",
              "description": "Should return 404 error"
            },
            "response": []
          }
        ]
      }
    ]
  }, null, 2)
})
```

4. **Document Test Evidence**:
```javascript
// Create test evidence directory
create_directory({ path: "hello-world-api/tests/evidence" })

// Save test outputs
write_file({
  path: "hello-world-api/tests/evidence/root-endpoint-output.json",
  content: JSON.stringify({ message: "Hello, World!" }, null, 2)
})

write_file({
  path: "hello-world-api/tests/evidence/health-endpoint-output.json",
  content: JSON.stringify({
    status: "healthy",
    timestamp: new Date().toISOString()
  }, null, 2)
})

// Save console logs
write_file({
  path: "hello-world-api/tests/evidence/console-logs.txt",
  content: `2024-01-15T10:30:00.000Z - GET /
2024-01-15T10:30:01.000Z - GET /health
2024-01-15T10:30:02.000Z - GET /nonexistent
404 Not Found: GET /nonexistent`
})
```

5. **Create Test Checklist**:
```javascript
write_file({
  path: "hello-world-api/tests/test-checklist.md",
  content: `# API Testing Checklist

## Pre-Test Setup
- [ ] Server started successfully
- [ ] Port 3000 is available
- [ ] Testing tools ready (curl/Postman)

## Endpoint Tests
- [ ] GET / returns 200 with Hello World message
- [ ] GET /health returns 200 with health status
- [ ] GET /undefined returns 404 with error message
- [ ] POST / returns 404
- [ ] PUT /health returns 404

## Response Validation
- [ ] JSON format is valid
- [ ] Content-Type headers are correct
- [ ] Status codes match specification
- [ ] Timestamps are ISO 8601 format

## Performance Tests
- [ ] Response time < 100ms
- [ ] Handles 10 concurrent requests
- [ ] No memory leaks observed

## Logging Tests
- [ ] All requests logged to console
- [ ] Timestamps included in logs
- [ ] Error requests logged appropriately

## Documentation
- [ ] Test report created
- [ ] Evidence collected
- [ ] Issues documented
- [ ] Recommendations provided`
})
```

## Tool Usage Guidelines for This Task

### Documentation Strategy
1. **Before Testing**: Create test plan and checklist
2. **During Testing**: Document results in real-time
3. **After Testing**: Compile comprehensive report
4. **Archive**: Save scripts and evidence for future use

### Best Practices
1. **Structured Documentation**: Use consistent format for all test results
2. **Evidence Collection**: Save actual outputs and responses
3. **Reproducibility**: Create scripts that can be re-run
4. **Clear Reporting**: Make results easy to understand

### Testing Workflow Pattern
1. Create test structure → Run tests → Document results → Generate report
2. Save test scripts → Create evidence → Build test suite
3. Document issues → Provide recommendations → Archive artifacts

## Integration Considerations
- Test documentation should align with README.md
- Test scripts should be executable and reusable
- Postman collections should be importable
- Reports should be stakeholder-friendly

## Quality Assurance
1. **Accurate Documentation**: Record actual results, not assumed
2. **Complete Coverage**: Test all endpoints and scenarios
3. **Clear Evidence**: Include screenshots or output logs
4. **Actionable Findings**: Provide specific recommendations
5. **Professional Format**: Use proper markdown and formatting

## Common Patterns
1. Run test → Capture output → Document result → Move to next test
2. Identify issue → Document details → Suggest fix → Verify resolution
3. Create script → Test script → Save for regression testing
4. Build collection → Export → Share with team