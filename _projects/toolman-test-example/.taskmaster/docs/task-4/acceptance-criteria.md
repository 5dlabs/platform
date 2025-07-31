# Acceptance Criteria for Chat Room API Implementation

## Functional Requirements

### Room Management

#### 1. Room Creation
- [ ] Users can create new rooms with name and description
- [ ] Room names must be 1-100 characters
- [ ] Descriptions are optional, max 500 characters
- [ ] Created rooms have unique IDs
- [ ] Creator is automatically added as owner
- [ ] Private room flag is properly stored
- [ ] Creation timestamp is recorded

#### 2. Room Listing
- [ ] Authenticated users can list all public rooms
- [ ] Users see only their joined private rooms
- [ ] Pagination works with default 20 items per page
- [ ] Search functionality filters by room name
- [ ] Response includes total count and pages
- [ ] Results are sorted by activity or creation date

#### 3. Room Details
- [ ] Members can view room details
- [ ] Non-members cannot access private rooms
- [ ] Response includes member count
- [ ] Response includes member list with roles
- [ ] Last message preview is included
- [ ] Room metadata is complete and accurate

#### 4. Room Updates
- [ ] Only admins/owners can update rooms
- [ ] Room name and description are updatable
- [ ] Updates are validated before saving
- [ ] Updated timestamp is recorded
- [ ] Changes are immediately reflected

#### 5. Room Deletion
- [ ] Only owners can delete rooms
- [ ] Deletion is soft (deleted_at timestamp)
- [ ] Messages are preserved but inaccessible
- [ ] Members cannot access deleted rooms
- [ ] Room disappears from listings

#### 6. Join/Leave Operations
- [ ] Users can join public rooms
- [ ] Private rooms require invitation
- [ ] Room capacity limits are enforced
- [ ] Users cannot join twice
- [ ] Leaving removes membership
- [ ] Owner leaving triggers ownership transfer

### Message Management

#### 1. Message History
- [ ] Members can retrieve room messages
- [ ] Pagination returns 50 messages by default
- [ ] Messages are sorted newest first
- [ ] Cursor-based pagination works correctly
- [ ] Before/after filters work properly
- [ ] User details are included with messages

#### 2. Message Posting
- [ ] Members can post messages to joined rooms
- [ ] Message content is required
- [ ] Content length is limited to 1000 chars
- [ ] Reply-to threading works correctly
- [ ] Messages have unique IDs
- [ ] Timestamps are accurate

#### 3. Message Deletion
- [ ] Authors can delete their own messages
- [ ] Admins can delete any message
- [ ] Deletion is soft (shows [deleted])
- [ ] Message ID remains for thread continuity
- [ ] Deleted messages cannot be retrieved

### Authorization & Security

#### 1. Authentication
- [ ] All endpoints require valid JWT
- [ ] Invalid tokens return 401 error
- [ ] Expired tokens are rejected
- [ ] User context is properly extracted

#### 2. Role-Based Access
- [ ] Room roles are enforced (owner/admin/member)
- [ ] Permission checks prevent unauthorized actions
- [ ] Role changes are immediately effective
- [ ] Default member role is assigned on join

#### 3. Input Validation
- [ ] All inputs are validated before processing
- [ ] Invalid inputs return 400 with details
- [ ] SQL injection attempts are prevented
- [ ] XSS attempts are sanitized

## Performance Criteria

### Response Time Requirements
- [ ] Room list loads in < 200ms (1000 rooms)
- [ ] Message history loads in < 300ms (100 messages)
- [ ] Message posting completes in < 100ms
- [ ] Room creation completes in < 150ms

### Scalability Metrics
- [ ] API handles 100 concurrent requests
- [ ] Database queries use appropriate indexes
- [ ] Pagination prevents memory overload
- [ ] Connection pooling is implemented

### Rate Limiting
- [ ] Global rate limit: 100 requests/15 minutes
- [ ] Message posting: 30 messages/minute
- [ ] Rate limit headers are included
- [ ] 429 status returned when exceeded

## Security Test Cases

### 1. Authentication Tests
```javascript
describe('Authentication', () => {
  test('rejects requests without token', async () => {
    const response = await request(app)
      .get('/api/v1/rooms')
      .expect(401);
    expect(response.body.error.message).toBe('Authentication required');
  });

  test('rejects invalid tokens', async () => {
    const response = await request(app)
      .get('/api/v1/rooms')
      .set('Authorization', 'Bearer invalid-token')
      .expect(401);
  });

  test('accepts valid tokens', async () => {
    const token = generateValidToken(testUser);
    const response = await request(app)
      .get('/api/v1/rooms')
      .set('Authorization', `Bearer ${token}`)
      .expect(200);
  });
});
```

### 2. Authorization Tests
```javascript
describe('Room Authorization', () => {
  test('members can view room details', async () => {
    const response = await authenticatedRequest(memberUser)
      .get(`/api/v1/rooms/${roomId}`)
      .expect(200);
  });

  test('non-members cannot view private rooms', async () => {
    const response = await authenticatedRequest(outsiderUser)
      .get(`/api/v1/rooms/${privateRoomId}`)
      .expect(403);
  });

  test('only admins can update rooms', async () => {
    const response = await authenticatedRequest(memberUser)
      .put(`/api/v1/rooms/${roomId}`)
      .send({ name: 'New Name' })
      .expect(403);
  });

  test('only owners can delete rooms', async () => {
    const response = await authenticatedRequest(adminUser)
      .delete(`/api/v1/rooms/${roomId}`)
      .expect(403);
  });
});
```

### 3. Input Validation Tests
```javascript
describe('Input Validation', () => {
  test('rejects empty room name', async () => {
    const response = await authenticatedRequest(testUser)
      .post('/api/v1/rooms')
      .send({ name: '', description: 'Test' })
      .expect(400);
    expect(response.body.error.details).toContainEqual(
      expect.objectContaining({ field: 'name' })
    );
  });

  test('rejects oversized content', async () => {
    const longContent = 'x'.repeat(1001);
    const response = await authenticatedRequest(testUser)
      .post(`/api/v1/rooms/${roomId}/messages`)
      .send({ content: longContent })
      .expect(400);
  });

  test('sanitizes HTML in messages', async () => {
    const response = await authenticatedRequest(testUser)
      .post(`/api/v1/rooms/${roomId}/messages`)
      .send({ content: '<script>alert("xss")</script>Hello' })
      .expect(201);
    expect(response.body.data.content).toBe('Hello');
  });
});
```

## Integration Test Scenarios

### 1. Complete Room Flow
```javascript
test('complete room lifecycle', async () => {
  // Create room
  const createResponse = await authenticatedRequest(testUser)
    .post('/api/v1/rooms')
    .send({ name: 'Test Room', description: 'Integration test' })
    .expect(201);
  
  const roomId = createResponse.body.data.id;
  
  // List rooms (should include new room)
  const listResponse = await authenticatedRequest(testUser)
    .get('/api/v1/rooms')
    .expect(200);
  expect(listResponse.body.data).toContainEqual(
    expect.objectContaining({ id: roomId })
  );
  
  // Join room (another user)
  await authenticatedRequest(otherUser)
    .post(`/api/v1/rooms/${roomId}/join`)
    .expect(200);
  
  // Post message
  await authenticatedRequest(otherUser)
    .post(`/api/v1/rooms/${roomId}/messages`)
    .send({ content: 'Hello, room!' })
    .expect(201);
  
  // Get messages
  const messagesResponse = await authenticatedRequest(testUser)
    .get(`/api/v1/rooms/${roomId}/messages`)
    .expect(200);
  expect(messagesResponse.body.data).toHaveLength(1);
  
  // Leave room
  await authenticatedRequest(otherUser)
    .post(`/api/v1/rooms/${roomId}/leave`)
    .expect(200);
  
  // Delete room
  await authenticatedRequest(testUser)
    .delete(`/api/v1/rooms/${roomId}`)
    .expect(200);
});
```

### 2. Pagination Test
```javascript
test('message pagination works correctly', async () => {
  // Create 100 messages
  for (let i = 0; i < 100; i++) {
    await createMessage(roomId, `Message ${i}`);
  }
  
  // Get first page
  const page1 = await authenticatedRequest(testUser)
    .get(`/api/v1/rooms/${roomId}/messages?page=1&limit=20`)
    .expect(200);
  
  expect(page1.body.data).toHaveLength(20);
  expect(page1.body.meta.total).toBe(100);
  expect(page1.body.meta.totalPages).toBe(5);
  
  // Get second page
  const page2 = await authenticatedRequest(testUser)
    .get(`/api/v1/rooms/${roomId}/messages?page=2&limit=20`)
    .expect(200);
  
  expect(page2.body.data).toHaveLength(20);
  expect(page2.body.data[0].id).not.toBe(page1.body.data[0].id);
});
```

## Performance Benchmarks

### Load Testing Script
```javascript
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '30s', target: 100 }, // Ramp up to 100 users
    { duration: '1m', target: 100 },  // Stay at 100 users
    { duration: '30s', target: 0 },   // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<300'], // 95% of requests under 300ms
    http_req_failed: ['rate<0.1'],    // Error rate under 10%
  },
};

export default function() {
  const token = __ENV.AUTH_TOKEN;
  
  // Test room listing
  const listResponse = http.get('http://localhost:3000/api/v1/rooms', {
    headers: { 'Authorization': `Bearer ${token}` },
  });
  
  check(listResponse, {
    'room list status is 200': (r) => r.status === 200,
    'room list response time < 200ms': (r) => r.timings.duration < 200,
  });
  
  // Test message posting
  const messageResponse = http.post(
    `http://localhost:3000/api/v1/rooms/${__ENV.ROOM_ID}/messages`,
    JSON.stringify({ content: 'Load test message' }),
    {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
    }
  );
  
  check(messageResponse, {
    'message post status is 201': (r) => r.status === 201,
    'message post time < 100ms': (r) => r.timings.duration < 100,
  });
  
  sleep(1);
}
```

## API Documentation Validation

### 1. OpenAPI Specification
- [ ] All endpoints are documented
- [ ] Request/response schemas are defined
- [ ] Authentication requirements are specified
- [ ] Error responses are documented
- [ ] Examples are provided for each endpoint

### 2. Generated Documentation
- [ ] Swagger UI is accessible
- [ ] All endpoints can be tested via UI
- [ ] Authentication works in UI
- [ ] Response examples are accurate

## Deployment Readiness

### 1. Configuration
- [ ] All secrets use environment variables
- [ ] Database connection pooling is configured
- [ ] CORS settings are production-ready
- [ ] Logging is properly configured

### 2. Health Checks
- [ ] /health endpoint returns 200
- [ ] Database connectivity is verified
- [ ] Redis connectivity is verified (if used)
- [ ] Response includes version info

### 3. Kubernetes Manifests
- [ ] Service definition is complete
- [ ] Deployment has proper resource limits
- [ ] ConfigMaps for non-secret config
- [ ] Secrets for sensitive data
- [ ] Ingress rules are defined
- [ ] HPA is configured for scaling

## Code Quality Metrics

### 1. Test Coverage
- [ ] Unit test coverage > 80%
- [ ] Integration test coverage > 70%
- [ ] All critical paths are tested
- [ ] Error cases are covered

### 2. Code Standards
- [ ] ESLint passes with no errors
- [ ] TypeScript strict mode enabled
- [ ] No any types without justification
- [ ] Consistent naming conventions

### 3. Documentation
- [ ] All public methods have JSDoc
- [ ] Complex logic has inline comments
- [ ] README includes setup instructions
- [ ] Architecture decisions are documented

## Final Checklist

- [ ] All endpoints return correct HTTP status codes
- [ ] Error messages are user-friendly
- [ ] API follows RESTful conventions
- [ ] Database migrations are versioned
- [ ] Rollback procedures are documented
- [ ] Monitoring/logging is implemented
- [ ] Performance meets specified criteria
- [ ] Security best practices are followed
- [ ] API is ready for production deployment