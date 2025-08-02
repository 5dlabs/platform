# Task 5: Real-time Communication with Socket.io - AI Agent Prompt

You are a senior full-stack engineer tasked with implementing real-time communication features using Socket.io. Your implementation must provide instant messaging, typing indicators, user presence tracking, and read receipts while maintaining security and scalability.

## Primary Objectives

1. **Socket.io Server Setup**: Configure Socket.io with proper CORS, authentication, and Redis adapter for horizontal scaling.

2. **Authentication System**: Implement JWT-based socket authentication with proper user context management.

3. **Event Handlers**: Create handlers for room management, messaging, typing indicators, presence updates, and read receipts.

4. **Redis Integration**: Use Redis adapter for multi-instance support and state management.

5. **Client Integration**: Provide a robust client-side implementation with reconnection logic and error handling.

## Required Actions

### Phase 1: Server Setup (20 minutes)
1. Install Socket.io dependencies:
   ```bash
   npm install socket.io @socket.io/redis-adapter
   npm install -D @types/socket.io
   ```

2. Create socket server configuration:
   - Initialize Socket.io with HTTP server
   - Configure CORS settings
   - Set up transport options
   - Configure ping/pong timeouts

3. Implement Redis adapter:
   - Connect pub/sub clients
   - Configure adapter
   - Handle connection errors

4. Set up authentication middleware:
   - Extract JWT from handshake
   - Verify token validity
   - Attach user context to socket

### Phase 2: Core Event Handlers (30 minutes)
1. **Connection Management**:
   - Handle user connection
   - Join user's existing rooms
   - Update presence status
   - Send initial state

2. **Room Events**:
   ```typescript
   - join-room: Join a chat room
   - leave-room: Leave a chat room
   - get-room-users: Get online users in room
   ```

3. **Message Events**:
   ```typescript
   - send-message: Send a new message
   - edit-message: Edit existing message
   - delete-message: Delete a message
   - mark-read: Mark messages as read
   ```

4. **Real-time Features**:
   ```typescript
   - typing-start: User started typing
   - typing-stop: User stopped typing
   - update-presence: Update online status
   ```

### Phase 3: Service Implementation (20 minutes)
1. **Typing Service**:
   - Track typing users per room
   - Auto-expire typing status
   - Broadcast updates efficiently

2. **Presence Service**:
   - Track online/offline status
   - Store in Redis with TTL
   - Update database periodically
   - Handle connection drops

3. **Message Service Integration**:
   - Real-time message delivery
   - Read receipt tracking
   - Message history sync

### Phase 4: Client Implementation (15 minutes)
1. Create Socket.io client hook:
   - Connection management
   - Event listeners
   - Emit methods
   - Reconnection handling

2. Implement state management:
   - Message updates
   - Typing indicators
   - Presence tracking
   - Read receipts

3. Error handling:
   - Connection failures
   - Authentication errors
   - Network issues
   - Fallback mechanisms

### Phase 5: Testing & Optimization (15 minutes)
1. Test real-time features:
   - Multiple client connections
   - Message delivery speed
   - Typing indicator accuracy
   - Presence updates

2. Optimize performance:
   - Reduce event payload size
   - Implement event throttling
   - Batch updates
   - Connection pooling

## Implementation Details

### Socket Event Flow
```typescript
// Client → Server → Other Clients
Client A: emit('send-message', data)
    ↓
Server: validate → save → broadcast
    ↓
Client B,C,D: on('new-message', data)
```

### Authentication Flow
```typescript
// Socket authentication middleware
io.use(async (socket, next) => {
  const token = socket.handshake.auth.token;
  
  try {
    const user = await validateToken(token);
    socket.userId = user.id;
    socket.user = user;
    next();
  } catch (error) {
    next(new Error('Authentication failed'));
  }
});
```

### Room Management
```typescript
// User joins room
socket.on('join-room', async (roomId, callback) => {
  // 1. Verify membership
  // 2. Join socket room
  // 3. Notify other members
  // 4. Send room state
  // 5. Acknowledge with callback
});
```

### Typing Indicators
```typescript
// Typing with auto-timeout
const typingTimers = new Map();

socket.on('typing-start', (roomId) => {
  // Clear existing timer
  if (typingTimers.has(socket.userId)) {
    clearTimeout(typingTimers.get(socket.userId));
  }
  
  // Broadcast typing
  socket.to(`room:${roomId}`).emit('user-typing', {
    userId: socket.userId,
    username: socket.user.username
  });
  
  // Set auto-stop timer
  const timer = setTimeout(() => {
    socket.emit('typing-stop', roomId);
  }, 3000);
  
  typingTimers.set(socket.userId, timer);
});
```

## Event Documentation

### Client → Server Events
| Event | Payload | Response | Description |
|-------|---------|----------|-------------|
| join-room | `{roomId: string}` | `{success, room}` | Join a chat room |
| leave-room | `{roomId: string}` | `{success}` | Leave a chat room |
| send-message | `{roomId, content, type}` | `{success, message}` | Send message |
| typing-start | `roomId: string` | - | Start typing indicator |
| typing-stop | `roomId: string` | - | Stop typing indicator |
| mark-read | `messageIds: string[]` | `{success}` | Mark messages read |

### Server → Client Events
| Event | Payload | Description |
|-------|---------|-------------|
| new-message | `{message, user}` | New message received |
| user-joined | `{roomId, user}` | User joined room |
| user-left | `{roomId, userId}` | User left room |
| user-typing | `{roomId, userId, username}` | User is typing |
| user-stopped-typing | `{roomId, userId}` | User stopped typing |
| messages-read | `{userId, messageIds}` | Messages marked as read |
| user-online | `{userId}` | User came online |
| user-offline | `{userId}` | User went offline |

## Performance Requirements

### Latency Targets
- Message delivery: < 100ms
- Typing indicators: < 50ms
- Presence updates: < 200ms
- Read receipts: < 150ms

### Scalability
- Support 10,000+ concurrent connections
- Handle 1,000 messages/second
- Horizontal scaling with Redis
- Graceful degradation

## Security Checklist

### Authentication
- [ ] JWT validation on connection
- [ ] Token expiry handling
- [ ] User context verification
- [ ] Automatic disconnection on failure

### Authorization
- [ ] Room membership checks
- [ ] Message sending permissions
- [ ] Admin action validation
- [ ] Rate limiting per user

### Input Validation
- [ ] Sanitize message content
- [ ] Validate room IDs
- [ ] Check payload sizes
- [ ] Prevent injection attacks

## Error Handling

### Connection Errors
```typescript
socket.on('connect_error', (error) => {
  if (error.message === 'Authentication failed') {
    // Refresh token and retry
  } else {
    // Show connection error to user
  }
});
```

### Event Errors
```typescript
socket.emit('send-message', data, (response) => {
  if (!response.success) {
    // Handle error
    console.error('Failed to send:', response.error);
    // Retry or show error
  }
});
```

## Testing Strategy

### Unit Tests
```typescript
describe('Socket Handlers', () => {
  test('authenticates valid token');
  test('rejects invalid token');
  test('joins room successfully');
  test('prevents non-member from sending');
  test('broadcasts message to room');
  test('updates typing indicators');
});
```

### Integration Tests
- Multiple clients messaging
- Reconnection scenarios
- Redis adapter failover
- Load testing

## Monitoring & Debugging

### Metrics to Track
- Active connections
- Messages per second
- Error rates
- Reconnection frequency
- Redis adapter latency

### Debug Mode
```typescript
// Enable Socket.io debug
DEBUG=socket.io* node server.js

// Custom debug events
io.on('connection', (socket) => {
  socket.emit('debug', {
    socketId: socket.id,
    userId: socket.userId,
    rooms: Array.from(socket.rooms)
  });
});
```

## Deployment Considerations

### Load Balancing
- Use sticky sessions
- Configure session affinity
- Handle server restarts
- Implement health checks

### Redis Configuration
- Use Redis Cluster for HA
- Configure proper memory limits
- Set up persistence
- Monitor memory usage

Execute this task systematically, ensuring real-time features work reliably at scale. The implementation should provide a seamless real-time experience while maintaining security and performance.