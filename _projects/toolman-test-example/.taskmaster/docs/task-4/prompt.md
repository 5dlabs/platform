# Autonomous Agent Prompt: Chat Room API Implementation

You are tasked with implementing a comprehensive REST API for chat room management with message handling, incorporating best practices from Rust HTTP server patterns and preparing for Kubernetes deployment.

## Objective
Build a scalable REST API with endpoints for room management, membership control, and message handling with efficient pagination. Research and apply patterns from Rust ecosystem and prepare Kubernetes deployment configurations.

## Detailed Requirements

### 1. Research Phase
Before implementation:
- Research Rust HTTP server patterns (error handling, type safety, performance)
- Study Express.js REST API best practices and security patterns
- Investigate Kubernetes API service deployment and ingress configurations
- Document findings and patterns to incorporate

### 2. Room Management Endpoints
Implement the following endpoints:
- `GET /api/rooms` - List all rooms with pagination and search
- `POST /api/rooms` - Create new room
- `GET /api/rooms/:id` - Get room details
- `PUT /api/rooms/:id` - Update room (admin only)
- `DELETE /api/rooms/:id` - Delete room (admin only)

Room features:
- Support public and private rooms
- Track member count and join status
- Include user role information
- Implement proper authorization

### 3. Room Membership Endpoints
- `POST /api/rooms/:id/join` - Join a room
- `POST /api/rooms/:id/leave` - Leave a room
- `GET /api/rooms/:id/members` - List room members

Membership rules:
- Room creator becomes admin
- Prevent last admin from leaving
- Private rooms require invitation (future feature)
- Track join timestamps

### 4. Message Endpoints
- `GET /api/rooms/:id/messages` - Get messages with pagination
- `POST /api/rooms/:id/messages` - Send a message
- `DELETE /api/rooms/:id/messages/:messageId` - Delete message

Message features:
- Cursor-based pagination (before/after)
- Support different message types (text, image, file)
- Soft delete for audit trail
- Auto-mark messages as read

### 5. Authorization Implementation
Implement authorization checks:
- Only room members can view messages
- Only message owner or room admin can delete messages
- Private rooms require membership for access
- Room updates limited to admins

### 6. Pagination Strategy
Implement efficient pagination:
- Cursor-based for messages (using timestamps)
- Page-based for room listings
- Configurable limits with maximums
- Include pagination metadata in responses

### 7. Response Format
Consistent API responses:
```typescript
// Success
{
  "data": {...} | [...],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 100,
    "hasMore": true
  }
}

// Error (Rust-inspired)
{
  "ok": false,
  "error": "Error message",
  "code": "ERROR_CODE"
}
```

### 8. Repository Pattern
Create repositories for:
- RoomRepository: CRUD operations for rooms
- RoomUserRepository: Manage room memberships
- MessageRepository: Message operations with pagination

Include methods like:
- `findAll()` with filtering and pagination
- `isUserInRoom()` for authorization
- `markAsRead()` for read receipts

### 9. Input Validation
Use schema validation (e.g., Zod):
- Room name: 3-100 characters
- Description: max 500 characters
- Message content: 1-1000 characters
- Validate UUIDs for IDs

### 10. Performance Optimizations
- Use database indexes on foreign keys
- Implement connection pooling
- Cache room membership checks
- Optimize queries with proper JOINs

### 11. Kubernetes Configuration
Create deployment files:
- Backend deployment with 3 replicas
- Service configuration for internal networking
- Ingress for external access
- Health check endpoints (/health, /ready)
- Resource limits and requests

### 12. Error Handling
Implement comprehensive error handling:
- Consistent error response format
- Proper HTTP status codes
- Log errors without exposing details
- Handle database connection failures

## Expected Deliverables

1. Room controller with all endpoints
2. Message controller with pagination
3. Authorization middleware for rooms
4. Repository implementations
5. Input validation schemas
6. Route configuration files
7. Kubernetes deployment manifests
8. API documentation

## Quality Standards

- Response time < 200ms for all endpoints
- Pagination handles large datasets efficiently
- No N+1 query problems
- Proper error handling and logging
- Type-safe implementations
- Security best practices applied

## Testing Requirements

Write tests for:
1. Room CRUD operations
2. Membership management
3. Message sending and retrieval
4. Pagination edge cases
5. Authorization scenarios
6. Input validation
7. Error handling

## Verification Steps

1. Test room creation and listing
2. Join/leave rooms as different users
3. Send messages and verify pagination
4. Test authorization (access private rooms, delete messages)
5. Verify Kubernetes deployment works
6. Load test with 100+ concurrent users
7. Check database query performance

Begin by researching best practices from Rust and Kubernetes ecosystems, then implement the API endpoints with proper structure and error handling.