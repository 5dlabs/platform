# Acceptance Criteria for Task 10: Implement Health Check Endpoint

## Required Outcomes

### 1. Route Implementation
- [ ] GET route handler exists for path '/health'
- [ ] Route is defined in src/index.js
- [ ] Route is placed after other routes but before app.listen()
- [ ] Route follows same pattern as root endpoint

### 2. Response Format
- [ ] Returns JSON response
- [ ] Response contains "status" property
- [ ] Status value is "healthy"
- [ ] Response contains "timestamp" property
- [ ] Timestamp is in ISO 8601 format
- [ ] Timestamp includes milliseconds and 'Z' suffix

### 3. HTTP Standards
- [ ] HTTP status code is 200
- [ ] Content-Type header is "application/json"
- [ ] Response is valid JSON
- [ ] No caching headers (health should be real-time)

### 4. Functional Requirements
- [ ] Timestamp updates on each request
- [ ] Endpoint responds quickly (< 50ms)
- [ ] No external dependencies for basic health
- [ ] Works independently of other endpoints

## Test Cases

### Test 1: Basic Health Check
```bash
curl http://localhost:3000/health
# Expected output format:
{"status":"healthy","timestamp":"2023-12-01T12:00:00.000Z"}
```

### Test 2: Response Headers
```bash
curl -I http://localhost:3000/health
# Expected headers:
# HTTP/1.1 200 OK
# Content-Type: application/json; charset=utf-8
```

### Test 3: JSON Structure Validation
```bash
curl -s http://localhost:3000/health | python3 -c "
import json, sys
data = json.load(sys.stdin)
assert 'status' in data, 'Missing status field'
assert 'timestamp' in data, 'Missing timestamp field'
assert data['status'] == 'healthy', 'Status not healthy'
print('JSON structure valid')
"
# Expected: JSON structure valid
```

### Test 4: Timestamp Format Validation
```bash
# Verify ISO 8601 format with Z suffix
curl -s http://localhost:3000/health | grep -E '"timestamp":"[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]{3}Z"'
echo $?
# Expected: 0 (pattern found)
```

### Test 5: Dynamic Timestamp Test
```bash
# Get two timestamps and verify they're different
TIMESTAMP1=$(curl -s http://localhost:3000/health | grep -o '"timestamp":"[^"]*"')
sleep 1
TIMESTAMP2=$(curl -s http://localhost:3000/health | grep -o '"timestamp":"[^"]*"')
[ "$TIMESTAMP1" != "$TIMESTAMP2" ] && echo "Timestamps are different" || echo "ERROR: Timestamps are same"
# Expected: Timestamps are different
```

### Test 6: Concurrent Request Handling
```bash
# Send 10 concurrent requests
for i in {1..10}; do
  curl -s http://localhost:3000/health &
done | wait
# Expected: All requests return valid responses
```

## Response Validation

### Required Response Fields
```json
{
  "status": "healthy",
  "timestamp": "2023-12-01T12:00:00.000Z"
}
```

### Field Requirements
- [ ] "status" is always present
- [ ] "status" value is exactly "healthy"
- [ ] "timestamp" is always present
- [ ] "timestamp" is valid ISO 8601 with milliseconds
- [ ] No additional required fields in basic implementation

### Optional Enhanced Fields
If implementing enhanced health check:
- [ ] "uptime" in seconds (number)
- [ ] "memory" object with free/total
- [ ] "cpu" count (number)
- [ ] "service" object with name/version

## Performance Criteria
- [ ] Response time < 50ms
- [ ] No memory leaks with repeated requests
- [ ] Handles 100+ requests per second
- [ ] No blocking operations

## Definition of Done
- GET /health endpoint returns correct JSON
- HTTP 200 status code
- Dynamic ISO 8601 timestamp
- All test cases pass
- Performance criteria met
- Ready for deployment monitoring

## Common Issues to Avoid
1. Hardcoded or static timestamp
2. Missing 'Z' suffix in ISO timestamp
3. Wrong status value or typos
4. Blocking health checks (DB queries, etc.)
5. Cached responses
6. Missing Content-Type header

## Monitoring Integration
- [ ] Response format compatible with common monitoring tools
- [ ] Quick response for frequent polling
- [ ] Clear success/failure indication
- [ ] Machine-readable timestamp format