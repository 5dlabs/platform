# Task 4: Chat Room API Implementation - Acceptance Criteria

## Functional Requirements

### 1. Room Management Endpoints ✓

#### List Rooms
- [ ] GET /api/rooms returns paginated room list
- [ ] Supports page and limit query parameters
- [ ] Search functionality works with room names
- [ ] Filter by public/private rooms
- [ ] Filter by joined rooms for authenticated user
- [ ] Returns member count for each room
- [ ] Shows last activity timestamp

#### Create Room
- [ ] POST /api/rooms creates new room
- [ ] Validates room name (1-100 chars)
- [ ] Optional description (max 500 chars)
- [ ] Creator automatically added as admin
- [ ] Returns 201 with room details
- [ ] Prevents duplicate room names

#### Get Room Details
- [ ] GET /api/rooms/:id returns full room info
- [ ] Only accessible to room members
- [ ] Returns member list with roles
- [ ] Shows room creator information
- [ ] Returns 403 for non-members
- [ ] Returns 404 for non-existent rooms

#### Update Room
- [ ] PUT /api/rooms/:id updates room details
- [ ] Only admins can update
- [ ] Can update name and description
- [ ] Returns updated room data
- [ ] Validates input parameters

#### Delete Room
- [ ] DELETE /api/rooms/:id removes room
- [ ] Only room creator can delete
- [ ] Cascades delete to messages
- [ ] Removes all member associations
- [ ] Returns 204 on success

#### Join/Leave Room
- [ ] POST /api/rooms/:id/join adds user to room
- [ ] POST /api/rooms/:id/leave removes user
- [ ] Prevents duplicate joins
- [ ] Updates member count
- [ ] Returns appropriate success message

### 2. Message Management Endpoints ✓

#### Get Messages
- [ ] GET /api/rooms/:id/messages returns message history
- [ ] Cursor-based pagination with 'before' parameter
- [ ] Default limit 50, max 100
- [ ] Only room members can access
- [ ] Messages include user details
- [ ] Marks retrieved messages as read
- [ ] Returns in reverse chronological order

#### Send Message
- [ ] POST /api/rooms/:id/messages creates message
- [ ] Validates message content (required, max 5000 chars)
- [ ] Supports message types (text, image, file)
- [ ] Only room members can send
- [ ] Updates room last activity
- [ ] Returns created message with ID

#### Delete Message
- [ ] DELETE /api/rooms/:id/messages/:messageId removes message
- [ ] Users can delete own messages
- [ ] Room admins can delete any message
- [ ] Returns 404 if message not found
- [ ] Returns 403 if unauthorized

## API Response Validation

### Success Responses
```bash
# Room list response
GET /api/rooms?page=1&limit=10
Response 200:
{
  "rooms": [...],
  "pagination": {
    "page": 1,
    "limit": 10,
    "total": 25,
    "pages": 3
  }
}

# Create room response
POST /api/rooms
Response 201:
{
  "id": "uuid",
  "name": "New Room",
  "description": "...",
  "createdBy": "userId",
  "members": ["userId"],
  "createdAt": "2024-01-01T00:00:00Z"
}

# Message pagination response
GET /api/rooms/:id/messages?before=cursor&limit=50
Response 200:
{
  "messages": [...],
  "pagination": {
    "hasMore": true,
    "nextCursor": "messageId"
  }
}
```

### Error Responses
```bash
# Validation error
Response 400:
{
  "errors": [
    {
      "field": "name",
      "message": "Room name is required"
    }
  ]
}

# Authorization error
Response 403:
{
  "error": "You must be a room member"
}

# Not found error
Response 404:
{
  "error": "Room not found"
}
```

## Authorization Tests

### Room Authorization Matrix
| Endpoint | Creator | Admin | Member | Non-member |
|----------|---------|-------|--------|------------|
| List rooms | ✓ | ✓ | ✓ | ✓ |
| Create room | ✓ | ✓ | ✓ | ✓ |
| View details | ✓ | ✓ | ✓ | ✗ (403) |
| Update room | ✓ | ✓ | ✗ (403) | ✗ (403) |
| Delete room | ✓ | ✗ (403) | ✗ (403) | ✗ (403) |
| Join room | ✓ | ✓ | ✓ | ✓ |
| Leave room | ✓ | ✓ | ✓ | ✗ (403) |
| Send message | ✓ | ✓ | ✓ | ✗ (403) |
| Delete own msg | ✓ | ✓ | ✓ | ✗ (403) |
| Delete any msg | ✓ | ✓ | ✗ (403) | ✗ (403) |

### Authorization Test Cases
```javascript
// Test 1: Non-member cannot view room details
await request(app)
  .get('/api/rooms/roomId')
  .set('Authorization', 'Bearer nonMemberToken')
  .expect(403);

// Test 2: Regular member cannot update room
await request(app)
  .put('/api/rooms/roomId')
  .set('Authorization', 'Bearer memberToken')
  .send({ name: 'New Name' })
  .expect(403);

// Test 3: Only creator can delete room
await request(app)
  .delete('/api/rooms/roomId')
  .set('Authorization', 'Bearer adminToken')
  .expect(403); // Admin but not creator
```

## Pagination Tests

### Room Pagination
- [ ] Default page size is 20
- [ ] Respects limit parameter (max 100)
- [ ] Page parameter works correctly
- [ ] Returns total count and pages
- [ ] Handles out-of-range pages gracefully

### Message Pagination
- [ ] Cursor-based pagination works
- [ ] 'before' parameter returns older messages
- [ ] Default limit is 50
- [ ] Maximum limit enforced (100)
- [ ] 'hasMore' flag accurate
- [ ] 'nextCursor' provided when applicable

## Input Validation Tests

### Room Validation
- [ ] Name required and trimmed
- [ ] Name length 1-100 characters
- [ ] Description optional, max 500 chars
- [ ] isPrivate must be boolean
- [ ] Invalid UUID returns 400

### Message Validation
- [ ] Content required and trimmed
- [ ] Content max 5000 characters
- [ ] messageType validates against enum
- [ ] Empty content rejected
- [ ] XSS attempts sanitized

## Performance Criteria

### Response Times
- [ ] Room list < 200ms
- [ ] Room creation < 100ms
- [ ] Message history < 150ms
- [ ] Message sending < 50ms

### Database Queries
- [ ] Room list uses single query with joins
- [ ] No N+1 queries in any endpoint
- [ ] Proper indexes on:
  - [ ] room_users (room_id, user_id)
  - [ ] messages (room_id, created_at)
  - [ ] messages (user_id)

### Concurrent Operations
- [ ] Handle 100 concurrent room creations
- [ ] Handle 1000 concurrent message sends
- [ ] No race conditions in member counts
- [ ] Proper transaction handling

## Integration Tests

### Complete Room Flow
```javascript
// 1. Create room
const room = await createRoom({ name: 'Test Room' });
✓ Room created with ID

// 2. List rooms
const rooms = await listRooms();
✓ New room appears in list

// 3. Join room (as different user)
await joinRoom(room.id);
✓ Successfully joined

// 4. Send message
const message = await sendMessage(room.id, 'Hello');
✓ Message created

// 5. Get messages
const messages = await getMessages(room.id);
✓ Message retrieved
✓ Marked as read

// 6. Leave room
await leaveRoom(room.id);
✓ Successfully left

// 7. Try to send message
await sendMessage(room.id, 'Fail');
✓ Returns 403 Forbidden
```

## Repository Method Tests

### RoomRepository
- [ ] `findAll()` supports all filters
- [ ] `create()` returns complete room
- [ ] `update()` only updates provided fields
- [ ] `delete()` cascades properly
- [ ] `updateLastActivity()` works

### MessageRepository
- [ ] `findByRoomWithPagination()` returns correct order
- [ ] `create()` sets timestamps
- [ ] `markMessagesAsRead()` batch updates
- [ ] `delete()` removes message
- [ ] Pagination cursor works correctly

### RoomUserRepository
- [ ] `isUserInRoom()` returns boolean
- [ ] `getUserRole()` returns correct role
- [ ] `addUserToRoom()` prevents duplicates
- [ ] `removeUserFromRoom()` works
- [ ] Member count queries accurate

## Error Handling

### HTTP Status Codes
- [ ] 200 - Successful GET
- [ ] 201 - Successful POST
- [ ] 204 - Successful DELETE
- [ ] 400 - Validation errors
- [ ] 401 - No authentication
- [ ] 403 - No authorization
- [ ] 404 - Resource not found
- [ ] 409 - Conflict (duplicate)

### Error Message Format
- [ ] Consistent error structure
- [ ] Meaningful error messages
- [ ] Field-level validation errors
- [ ] No sensitive data leaked
- [ ] Proper error logging

## Security Validation

### Authentication
- [ ] All endpoints require valid JWT
- [ ] Token validation on every request
- [ ] Proper 401 for invalid tokens

### Authorization
- [ ] Role checks implemented
- [ ] Member checks enforced
- [ ] No privilege escalation possible

### Input Security
- [ ] SQL injection prevented
- [ ] XSS protection active
- [ ] Path traversal blocked
- [ ] Rate limiting ready

## Documentation Requirements

### Code Documentation
- [ ] All endpoints have JSDoc comments
- [ ] Parameter types documented
- [ ] Return types specified
- [ ] Error scenarios documented

### API Examples
- [ ] cURL examples for each endpoint
- [ ] Request/response examples
- [ ] Error response examples
- [ ] Authentication examples

## Final Checklist

### Functionality
- [ ] All 10 endpoints implemented
- [ ] Authorization working correctly
- [ ] Pagination functioning
- [ ] Input validation complete
- [ ] Error handling comprehensive

### Quality
- [ ] TypeScript types complete
- [ ] No any types used
- [ ] Tests passing (90%+ coverage)
- [ ] No console.log statements
- [ ] Proper logging implemented

### Performance
- [ ] Response times meet targets
- [ ] Database queries optimized
- [ ] Indexes created
- [ ] Connection pooling used

**Task is complete when all endpoints work correctly, authorization is enforced, and performance targets are met.**