# Autonomous Agent Prompt: Real-time Communication with Socket.io

You are tasked with implementing a comprehensive real-time communication system using Socket.io for a chat application, including message delivery, typing indicators, presence tracking, and read receipts.

## Objective
Build a scalable Socket.io server with Redis adapter for horizontal scaling, implementing real-time features while maintaining sub-100ms message delivery latency.

## Detailed Requirements

### 1. Socket.io Server Setup
Configure Socket.io server with:
- CORS settings for frontend connection
- JWT-based authentication middleware
- Redis adapter for multi-instance support
- Proper transport configuration (WebSocket preferred)
- Connection timeout and heartbeat settings

### 2. Authentication Middleware
Implement socket authentication:
- Extract JWT token from handshake
- Verify token and retrieve user info
- Attach userId and username to socket instance
- Reject connections with invalid tokens
- Handle token expiration gracefully

### 3. Redis Adapter Configuration
Set up Redis adapter for scaling:
- Configure pub/sub clients
- Handle connection retries
- Enable sticky sessions support
- Monitor adapter health
- Test multi-instance message delivery

### 4. Room Management
Implement room functionality:
- Join room with membership verification
- Leave room and cleanup
- Track active users per room
- Personal notification rooms (user:{id})
- Emit user join/leave events

### 5. Message Handling
Create message event handlers:
- `send-message`: Validate, save, and broadcast
- `edit-message`: Update and notify
- `delete-message`: Soft delete with permissions
- Include user info in message payload
- Ensure delivery to all room members

### 6. Typing Indicators
Implement typing system:
- `typing-start`: Add to Redis set with TTL
- `typing-stop`: Remove from set
- Auto-expire after 5 seconds
- Broadcast to room members
- Clear on message send or disconnect

### 7. User Presence
Track online/offline status:
- Update database on connect/disconnect
- Store socket IDs in Redis
- Handle multiple connections per user
- Broadcast status to user's rooms
- Set last seen timestamp

### 8. Read Receipts
Implement read tracking:
- `mark-read`: Update database and Redis
- Store receipts with TTL
- Broadcast to message sender
- Batch updates for performance
- Track per room/user

### 9. Error Handling
Comprehensive error management:
- Catch and log all errors
- Send user-friendly error messages
- Handle disconnection scenarios
- Implement reconnection logic
- Rate limit socket events

### 10. Performance Optimizations
- Use Redis pipelining
- Implement message batching
- Enable compression for large data
- Debounce client events
- Monitor event processing time

## Expected Deliverables

1. Socket server configuration file
2. Authentication middleware
3. Event handler classes:
   - MessageHandler
   - RoomHandler
   - TypingHandler
   - PresenceHandler
4. Redis adapter setup
5. Error handling middleware
6. Rate limiting implementation
7. Client connection example
8. Performance monitoring

## Event Structure

### Client → Server Events
```typescript
// Room events
'join-room': (roomId: string) => void
'leave-room': (roomId: string) => void

// Message events
'send-message': (data: { roomId: string, content: string, messageType?: string }) => void
'edit-message': (data: { messageId: string, content: string }) => void
'delete-message': (data: { messageId: string }) => void

// Typing events
'typing-start': (roomId: string) => void
'typing-stop': (roomId: string) => void

// Read receipts
'mark-read': (data: { roomId: string, messageId: string }) => void
```

### Server → Client Events
```typescript
// Room events
'room-joined': (data: { roomId: string, room: Room, onlineUsers: string[] }) => void
'user-joined-room': (data: { roomId: string, user: User }) => void
'user-left-room': (data: { roomId: string, userId: string }) => void

// Message events
'message-received': (message: Message & { user: User }) => void
'message-edited': (data: { messageId: string, content: string }) => void
'message-deleted': (data: { messageId: string }) => void

// Typing events
'user-typing': (data: { roomId: string, userId: string, username: string }) => void
'user-stopped-typing': (data: { roomId: string, userId: string }) => void

// Presence events
'user-online': (data: { userId: string, username: string }) => void
'user-offline': (data: { userId: string, lastSeen: Date }) => void

// Read receipts
'message-read': (data: { messageId: string, userId: string, readAt: Date }) => void

// Error events
'error': (data: { message: string, code?: string }) => void
```

## Redis Key Patterns

- User sockets: `user_sockets:{userId}` (Set)
- Typing indicators: `typing:{roomId}` (Set with TTL)
- Active room: `active_room:{userId}` (String)
- Presence: `presence:{userId}` (String with TTL)
- Read receipts: `read_receipts:{messageId}` (Set)

## Testing Requirements

1. Unit test all event handlers
2. Test authentication middleware
3. Verify Redis adapter functionality
4. Load test with 1000+ connections
5. Test message delivery latency
6. Verify typing indicator timing
7. Test multi-instance setup
8. Check memory usage under load

## Performance Targets

- Message delivery: < 100ms
- Typing indicator update: < 50ms
- Presence update: < 5 seconds
- Support 10,000 concurrent connections
- Memory usage: < 100MB per 1000 connections

## Security Considerations

- Validate all input data
- Verify room membership for all operations
- Rate limit events per user
- Sanitize message content
- Log suspicious activity
- Implement connection limits

Begin by setting up the Socket.io server with Redis adapter, then implement authentication, followed by the various event handlers for real-time functionality.