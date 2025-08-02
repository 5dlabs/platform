# Task 4: Chat Room API Implementation - AI Agent Prompt

You are a senior backend engineer tasked with implementing a comprehensive REST API for chat room management and messaging. Your implementation must be scalable, secure, and follow RESTful best practices while incorporating insights from Rust ecosystem patterns and preparing for Kubernetes deployment.

## Primary Objectives

1. **Implement Room Management API**: Create endpoints for listing, creating, updating, deleting, joining, and leaving chat rooms with proper authorization.

2. **Implement Message API**: Build endpoints for retrieving message history with pagination and sending/deleting messages.

3. **Research Best Practices**: Study Rust HTTP server patterns and Kubernetes API deployment configurations to apply relevant patterns.

4. **Authorization System**: Implement role-based access control for room operations (admin, member roles).

5. **Performance Optimization**: Add efficient pagination, database indexing, and query optimization.

## Required Actions

### Phase 1: Research & Planning (20 minutes)
1. Research Rust HTTP server patterns:
   - Search "Actix-web REST API best practices"
   - Look up "Rust API error handling patterns"
   - Study "Rust API response serialization"

2. Research Express.js patterns:
   - Search "Express.js REST API structure 2024"
   - Study "Express.js error handling middleware"
   - Look up "Express.js pagination patterns"

3. Research Kubernetes deployment:
   - Search "Kubernetes API service deployment"
   - Study "Kubernetes ingress configuration"
   - Look up "Kubernetes health checks REST API"

### Phase 2: API Structure Setup (15 minutes)
1. Create controller files:
   - `roomController.ts` - Room management logic
   - `messageController.ts` - Message handling
   
2. Create route files:
   - `roomRoutes.ts` - Room endpoint definitions
   - `messageRoutes.ts` - Message endpoint definitions

3. Set up validators:
   - `roomValidators.ts` - Room input validation
   - `messageValidators.ts` - Message validation

4. Create middleware:
   - `roomAuth.ts` - Room-specific authorization

### Phase 3: Room API Implementation (25 minutes)
1. Implement room endpoints:
   ```typescript
   GET    /api/rooms          - List rooms (paginated)
   POST   /api/rooms          - Create new room
   GET    /api/rooms/:id      - Get room details
   PUT    /api/rooms/:id      - Update room
   DELETE /api/rooms/:id      - Delete room
   POST   /api/rooms/:id/join - Join room
   POST   /api/rooms/:id/leave - Leave room
   ```

2. Add query filters:
   - Search by room name
   - Filter by public/private
   - Filter by joined rooms
   - Pagination parameters

3. Implement authorization:
   - Only members can view room details
   - Only admins can update room
   - Only creator can delete room

### Phase 4: Message API Implementation (20 minutes)
1. Implement message endpoints:
   ```typescript
   GET    /api/rooms/:id/messages     - Get messages (paginated)
   POST   /api/rooms/:id/messages     - Send message
   DELETE /api/rooms/:id/messages/:id - Delete message
   ```

2. Add cursor-based pagination:
   - Use message ID as cursor
   - Implement `before` parameter
   - Return `hasMore` flag

3. Message features:
   - Support different message types
   - Track read receipts
   - Update room last activity

### Phase 5: Repository Updates (15 minutes)
1. Update RoomRepository:
   - Add pagination support
   - Implement search functionality
   - Add member count queries
   - Update last activity tracking

2. Update MessageRepository:
   - Implement cursor pagination
   - Add batch read receipt updates
   - Support message deletion

3. Create RoomUserRepository:
   - Check membership status
   - Get user role in room
   - Add/remove members

### Phase 6: Testing & Documentation (15 minutes)
1. Create API tests:
   - Test all endpoints
   - Verify authorization
   - Test pagination
   - Check error responses

2. Document API:
   - Add JSDoc comments
   - Create API examples
   - Document error codes

## Implementation Requirements

### API Response Format
```typescript
// Success response
{
  "data": { ... },
  "meta": {
    "timestamp": "2024-01-01T00:00:00Z",
    "version": "1.0"
  }
}

// Error response
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid input",
    "details": [...]
  }
}

// Paginated response
{
  "data": [...],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 100,
    "pages": 5
  }
}
```

### Authorization Matrix
```
Action          | Creator | Admin | Member | Non-member
----------------|---------|-------|--------|------------
View room list  |    ✓    |   ✓   |   ✓    |     ✓
Create room     |    ✓    |   ✓   |   ✓    |     ✓
View details    |    ✓    |   ✓   |   ✓    |     ✗
Update room     |    ✓    |   ✓   |   ✗    |     ✗
Delete room     |    ✓    |   ✗   |   ✗    |     ✗
Send message    |    ✓    |   ✓   |   ✓    |     ✗
Delete message  |    ✓*   |   ✓   |   ✓*   |     ✗
* Only own messages
```

### Pagination Strategy
```typescript
// Offset-based for rooms
GET /api/rooms?page=2&limit=20

// Cursor-based for messages
GET /api/rooms/:id/messages?before=messageId&limit=50
```

### Error Codes
- `400` - Bad Request (validation errors)
- `401` - Unauthorized (no token)
- `403` - Forbidden (no permission)
- `404` - Not Found
- `409` - Conflict (already exists)
- `429` - Too Many Requests

## Quality Checklist

### Security
- [ ] All endpoints require authentication
- [ ] Authorization checks on every operation
- [ ] Input validation on all parameters
- [ ] SQL injection prevention
- [ ] Rate limiting considerations

### Performance
- [ ] Efficient database queries
- [ ] Proper indexes on foreign keys
- [ ] Pagination implemented
- [ ] N+1 query prevention
- [ ] Connection pooling utilized

### Code Quality
- [ ] TypeScript types for all entities
- [ ] Error handling middleware
- [ ] Consistent response format
- [ ] Comprehensive logging
- [ ] Unit tests for controllers

### API Design
- [ ] RESTful URL structure
- [ ] Proper HTTP methods
- [ ] Meaningful status codes
- [ ] Clear error messages
- [ ] API versioning considered

## Rust-Inspired Patterns to Apply

1. **Result Type Pattern**: Use consistent success/error responses
2. **Strong Typing**: Leverage TypeScript for type safety
3. **Error Propagation**: Clear error handling chain
4. **Resource Ownership**: Clear ownership of rooms/messages

## Kubernetes Deployment Prep

1. **Health Checks**: Add `/health` and `/ready` endpoints
2. **Graceful Shutdown**: Handle SIGTERM properly
3. **Resource Limits**: Design with memory/CPU limits in mind
4. **Horizontal Scaling**: Ensure stateless API design

## Testing Requirements

```typescript
describe('Room API', () => {
  test('creates room successfully');
  test('prevents non-members from viewing details');
  test('allows only admins to update');
  test('paginates room list correctly');
  test('filters rooms by search term');
});

describe('Message API', () => {
  test('retrieves messages with pagination');
  test('marks messages as read');
  test('prevents non-members from sending');
  test('allows users to delete own messages');
});
```

Execute this task systematically, ensuring each endpoint is secure, performant, and follows REST best practices. The API should be production-ready and prepared for containerized deployment.