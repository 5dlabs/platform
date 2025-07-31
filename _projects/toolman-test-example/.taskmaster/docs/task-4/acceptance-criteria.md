# Acceptance Criteria: Chat Room API Implementation

## Overview
This document defines the acceptance criteria for the chat room API implementation with message handling.

## Research Completion Criteria

### ✅ Pattern Research
- [ ] Document 3+ Rust HTTP server patterns identified
- [ ] List Express.js REST API best practices applied
- [ ] Document Kubernetes deployment patterns researched
- [ ] Create summary of patterns incorporated

## Room Management API Criteria

### ✅ Room Endpoints
- [ ] `GET /api/rooms` returns paginated room list
- [ ] Search parameter filters rooms by name
- [ ] Shows join status for authenticated user
- [ ] `POST /api/rooms` creates new room
- [ ] Creator automatically becomes admin
- [ ] `GET /api/rooms/:id` returns room details
- [ ] Private rooms require membership to view
- [ ] `PUT /api/rooms/:id` restricted to admins
- [ ] `DELETE /api/rooms/:id` restricted to admins

### ✅ Response Format
- [ ] Consistent success response with data field
- [ ] Pagination metadata included
- [ ] Error responses follow Rust-inspired format
- [ ] HTTP status codes appropriate

## Room Membership Criteria

### ✅ Membership Endpoints
- [ ] `POST /api/rooms/:id/join` adds user to room
- [ ] Returns 409 if already member
- [ ] `POST /api/rooms/:id/leave` removes user
- [ ] Prevents last admin from leaving
- [ ] `GET /api/rooms/:id/members` lists members
- [ ] Shows role and join date

### ✅ Authorization Rules
- [ ] Non-members cannot access private rooms
- [ ] Only admins can update room details
- [ ] Member status verified for all operations
- [ ] Proper 403 responses for unauthorized access

## Message API Criteria

### ✅ Message Endpoints
- [ ] `GET /api/rooms/:id/messages` returns messages
- [ ] Cursor-based pagination with before/after
- [ ] Maximum 100 messages per request
- [ ] Messages include user information
- [ ] `POST /api/rooms/:id/messages` sends message
- [ ] Content validation (1-1000 chars)
- [ ] Only room members can send
- [ ] `DELETE /api/messages/:id` soft deletes
- [ ] Only owner or admin can delete

### ✅ Pagination Implementation
- [ ] Messages use cursor-based pagination
- [ ] Rooms use page-based pagination
- [ ] Pagination metadata in responses
- [ ] Efficient queries with proper indexes

## Data Access Layer Criteria

### ✅ Repository Pattern
- [ ] RoomRepository with CRUD methods
- [ ] RoomUserRepository for memberships
- [ ] MessageRepository with pagination
- [ ] All methods return TypeScript types
- [ ] No raw SQL in controllers

### ✅ Query Performance
- [ ] Room list query < 50ms
- [ ] Message pagination < 100ms
- [ ] No N+1 query problems
- [ ] Proper JOIN usage
- [ ] Indexes utilized effectively

## Input Validation Criteria

### ✅ Request Validation
- [ ] Schema validation middleware implemented
- [ ] Room name: 3-100 characters validated
- [ ] Message content: 1-1000 characters
- [ ] UUID format validated for IDs
- [ ] Clear validation error messages

## Error Handling Criteria

### ✅ Error Responses
- [ ] Consistent error format across API
- [ ] Appropriate HTTP status codes
- [ ] No sensitive data in errors
- [ ] Errors logged server-side
- [ ] Database errors handled gracefully

## Kubernetes Deployment Criteria

### ✅ Deployment Configuration
- [ ] Backend deployment with 3 replicas
- [ ] Resource requests and limits set
- [ ] Health check endpoints implemented
- [ ] Readiness probe configured
- [ ] Environment variables from secrets

### ✅ Service Configuration
- [ ] ClusterIP service for internal access
- [ ] Ingress for external routing
- [ ] Rate limiting configured
- [ ] Proper service discovery

## Testing Checklist

### Integration Tests
```javascript
describe('Room API', () => {
  it('creates room with valid data');
  it('lists rooms with pagination');
  it('enforces private room access');
  it('allows joining public rooms');
  it('prevents leaving as last admin');
});

describe('Message API', () => {
  it('sends messages to joined rooms');
  it('paginates messages correctly');
  it('enforces message deletion rules');
  it('validates message content');
});
```

### API Testing Commands
```bash
# Create room
curl -X POST http://localhost:3001/api/rooms \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"General Chat","description":"Main chat room"}'

# List rooms
curl -X GET "http://localhost:3001/api/rooms?page=1&limit=10" \
  -H "Authorization: Bearer $TOKEN"

# Send message
curl -X POST http://localhost:3001/api/rooms/$ROOM_ID/messages \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"content":"Hello, world!"}'

# Get messages with pagination
curl -X GET "http://localhost:3001/api/rooms/$ROOM_ID/messages?limit=50" \
  -H "Authorization: Bearer $TOKEN"
```

## Definition of Done

The task is complete when:
1. All room management endpoints functional
2. Membership operations work correctly
3. Message API with pagination implemented
4. Authorization enforced on all endpoints
5. Input validation prevents bad data
6. Error handling is consistent
7. Kubernetes deployment files created
8. Performance targets met
9. All tests passing

## Common Issues to Avoid

- ❌ Missing authorization checks
- ❌ N+1 queries in room listings
- ❌ Inefficient message pagination
- ❌ No input validation
- ❌ Exposing internal errors
- ❌ Missing database indexes
- ❌ Hard-coded configuration
- ❌ No health check endpoints

## Performance Verification

```bash
# Load test room listing
ab -n 1000 -c 10 -H "Authorization: Bearer $TOKEN" \
  http://localhost:3001/api/rooms

# Test message pagination
time curl -X GET \
  "http://localhost:3001/api/rooms/$ROOM_ID/messages?limit=100" \
  -H "Authorization: Bearer $TOKEN"

# Verify Kubernetes deployment
kubectl apply -f kubernetes/
kubectl get pods -l app=chat-api
kubectl logs -f deployment/chat-api
```