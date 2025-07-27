# Task 14: Manual Testing of API Endpoints - Acceptance Criteria

## Definition of Done
Manual testing is successfully completed when all the following criteria are met:

## Required Deliverables

### 1. Test Execution
- [ ] All endpoints tested with curl
- [ ] Multiple HTTP methods tested
- [ ] Error scenarios tested
- [ ] Edge cases covered
- [ ] Request logging verified

### 2. Root Endpoint Testing
- [ ] GET / returns 200 status
- [ ] Response is {"message": "Hello, World!"}
- [ ] Content-Type is application/json
- [ ] POST, PUT, DELETE return 404
- [ ] Response time is reasonable

### 3. Health Endpoint Testing
- [ ] GET /health returns 200 status
- [ ] Response contains "status": "healthy"
- [ ] Timestamp is valid ISO format
- [ ] Timestamp updates between requests
- [ ] Non-GET methods return 404

### 4. Error Handling Testing
- [ ] Undefined routes return 404
- [ ] Error response format is consistent
- [ ] No stack traces exposed
- [ ] Special characters handled safely
- [ ] Case-sensitive routing verified

### 5. Logging Verification
- [ ] All requests appear in console
- [ ] Log format includes timestamp
- [ ] HTTP method is logged
- [ ] Request path is logged
- [ ] Logs are readable and consistent

### 6. Documentation
- [ ] Test results documented
- [ ] Issues list created (if any)
- [ ] Recommendations documented
- [ ] Test report is complete
- [ ] Evidence/screenshots included

### 7. Postman Collection
- [ ] Collection created
- [ ] Environment variables set
- [ ] All endpoints included
- [ ] Tests organized by category
- [ ] Collection is exportable

## Verification Tests

### Test 1: Root Endpoint Verification
```bash
# Test successful GET
RESPONSE=$(curl -s -w "\nSTATUS:%{http_code}" http://localhost:3000/)
BODY=$(echo "$RESPONSE" | head -n -1)
STATUS=$(echo "$RESPONSE" | tail -n 1 | cut -d: -f2)

if [ "$STATUS" = "200" ] && [ "$BODY" = '{"message":"Hello, World!"}' ]; then
  echo "✓ Root endpoint test passed"
else
  echo "✗ Root endpoint test failed"
  echo "Status: $STATUS, Body: $BODY"
fi
```

### Test 2: Health Endpoint Verification
```bash
# Test health endpoint
HEALTH=$(curl -s http://localhost:3000/health)

# Check for required fields
if echo "$HEALTH" | grep -q '"status":"healthy"' && \
   echo "$HEALTH" | grep -q '"timestamp":'; then
  echo "✓ Health endpoint test passed"
else
  echo "✗ Health endpoint test failed"
fi

# Test timestamp freshness
TIME1=$(curl -s http://localhost:3000/health | grep -o '"timestamp":"[^"]*"' | cut -d'"' -f4)
sleep 1
TIME2=$(curl -s http://localhost:3000/health | grep -o '"timestamp":"[^"]*"' | cut -d'"' -f4)

if [ "$TIME1" != "$TIME2" ]; then
  echo "✓ Timestamp updates correctly"
else
  echo "✗ Timestamp not updating"
fi
```

### Test 3: Error Handling Verification
```bash
# Test 404 responses
404_TESTS=("/api" "/users" "/test" "/Health")

for path in "${404_TESTS[@]}"; do
  RESPONSE=$(curl -s -w "\n%{http_code}" "http://localhost:3000$path")
  STATUS=$(echo "$RESPONSE" | tail -n 1)
  BODY=$(echo "$RESPONSE" | head -n -1)
  
  if [ "$STATUS" = "404" ] && echo "$BODY" | grep -q '"error":"Not Found"'; then
    echo "✓ 404 test passed for $path"
  else
    echo "✗ 404 test failed for $path"
  fi
done
```

### Test 4: Method Testing
```bash
# Test non-GET methods on defined routes
METHODS=("POST" "PUT" "DELETE" "PATCH")
PATHS=("/" "/health")

for method in "${METHODS[@]}"; do
  for path in "${PATHS[@]}"; do
    STATUS=$(curl -s -X $method -o /dev/null -w "%{http_code}" "http://localhost:3000$path")
    if [ "$STATUS" = "404" ]; then
      echo "✓ $method $path correctly returns 404"
    else
      echo "✗ $method $path returned $STATUS (expected 404)"
    fi
  done
done
```

### Test 5: Logging Verification
```bash
# This would need to be checked manually by viewing server console
echo "Manual Check Required:"
echo "1. Verify server console shows request logs"
echo "2. Format should be: TIMESTAMP - METHOD PATH"
echo "3. All test requests should appear"
```

## Test Report Requirements

### Minimum Content
1. **Summary Section**
   - Total tests executed
   - Pass/fail counts
   - Critical issues found

2. **Detailed Results**
   - Each endpoint tested
   - Methods tested per endpoint
   - Actual vs expected results

3. **Issues Section**
   - Any failures documented
   - Severity assessment
   - Reproduction steps

4. **Recommendations**
   - Improvements identified
   - Security observations
   - Performance notes

## Common Failure Modes

1. **Incomplete Testing**
   - Not testing all HTTP methods
   - Missing error scenarios
   - Skipping edge cases

2. **Incorrect Validation**
   - Not checking exact response format
   - Ignoring status codes
   - Missing header validation

3. **Poor Documentation**
   - Results not recorded
   - Missing test evidence
   - Incomplete issue descriptions

4. **Postman Issues**
   - Collection not created
   - Missing test cases
   - No environment setup

## Final Validation Script
```bash
#!/bin/bash
echo "Running API Endpoint Tests..."

# Check if server is running
if ! curl -s http://localhost:3000 > /dev/null; then
  echo "✗ Server not running on port 3000"
  exit 1
fi

TESTS_PASSED=0
TESTS_FAILED=0

# Test 1: Root endpoint
if curl -s http://localhost:3000/ | grep -q '"message":"Hello, World!"'; then
  echo "✓ Root endpoint working"
  ((TESTS_PASSED++))
else
  echo "✗ Root endpoint failed"
  ((TESTS_FAILED++))
fi

# Test 2: Health endpoint
if curl -s http://localhost:3000/health | grep -q '"status":"healthy"'; then
  echo "✓ Health endpoint working"
  ((TESTS_PASSED++))
else
  echo "✗ Health endpoint failed"
  ((TESTS_FAILED++))
fi

# Test 3: 404 handling
if curl -s http://localhost:3000/nonexistent | grep -q '"error":"Not Found"'; then
  echo "✓ 404 handling working"
  ((TESTS_PASSED++))
else
  echo "✗ 404 handling failed"
  ((TESTS_FAILED++))
fi

# Test 4: Wrong method handling
STATUS=$(curl -s -X POST -o /dev/null -w "%{http_code}" http://localhost:3000/)
if [ "$STATUS" = "404" ]; then
  echo "✓ Method handling working"
  ((TESTS_PASSED++))
else
  echo "✗ Method handling failed"
  ((TESTS_FAILED++))
fi

# Summary
echo "\n==== TEST SUMMARY ===="
echo "Passed: $TESTS_PASSED"
echo "Failed: $TESTS_FAILED"

if [ $TESTS_FAILED -eq 0 ]; then
  echo "✅ All tests passed!"
  exit 0
else
  echo "❌ Some tests failed"
  exit 1
fi
```

## Success Metrics
- 100% of endpoints tested
- All test cases documented
- Zero unhandled errors
- Complete test report delivered
- Postman collection functional
- No security issues found