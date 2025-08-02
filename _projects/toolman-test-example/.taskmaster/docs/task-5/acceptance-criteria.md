# Task 5: Real-time Communication with Socket.io - Acceptance Criteria

## Functional Requirements

### 1. Socket.io Server Setup ✓
- [ ] Socket.io server initialized with HTTP server
- [ ] CORS configured for frontend URL
- [ ] Transports include websocket and polling
- [ ] Ping timeout set to 60 seconds
- [ ] Ping interval set to 25 seconds
- [ ] Redis adapter configured and connected
- [ ] Error handling for Redis connection failures

### 2. Authentication ✓
- [ ] JWT validation on socket connection
- [ ] Token extracted from handshake.auth
- [ ] User context attached to socket instance
- [ ] Invalid tokens reject connection
- [ ] Missing tokens reject connection
- [ ] User not found rejects connection
- [ ] Authentication errors return clear messages

### 3. Room Management ✓
- [ ] Users can join rooms they're members of
- [ ] Users cannot join rooms they're not members of
- [ ] Socket joins appropriate room namespace
- [ ] Other room members notified of new user
- [ ] Users can leave rooms
- [ ] Disconnection removes user from all rooms
- [ ] Room state sent to joining user

### 4. Real-time Messaging ✓
- [ ] Messages delivered to all room members
- [ ] Sender receives acknowledgment
- [ ] Message includes user details
- [ ] Non-members cannot send messages
- [ ] Message saved to database
- [ ] Delivery latency < 100ms
- [ ] Messages maintain correct order

### 5. Typing Indicators ✓
- [ ] Typing start event broadcasts to room
- [ ] Typing stop event broadcasts to room
- [ ] Auto-timeout after 5 seconds
- [ ] Multiple users can type simultaneously
- [ ] Typing state cleared on message send
- [ ] Typing state cleared on disconnect
- [ ] No typing indicators from non-members

### 6. User Presence ✓
- [ ] Online status updated on connection
- [ ] Offline status updated on disconnect
- [ ] Presence stored in Redis with TTL
- [ ] Database updated with online status
- [ ] Room members notified of status changes
- [ ] Bulk online users list available
- [ ] Presence refreshed periodically

### 7. Read Receipts ✓
- [ ] Messages marked as read in database
- [ ] Read receipts broadcast to room
- [ ] Batch marking supported
- [ ] Only own unread messages can be marked
- [ ] Read receipts include timestamp
- [ ] Read status persists across sessions

## Technical Validation

### Connection Tests
```javascript
// Test 1: Successful connection with valid token
const socket = io(url, {
  auth: { token: validToken }
});
✓ Connected successfully
✓ socket.connected === true

// Test 2: Failed connection with invalid token
const socket = io(url, {
  auth: { token: invalidToken }
});
✓ Connection rejected
✓ Error: "Authentication failed"

// Test 3: Reconnection after disconnect
socket.disconnect();
await wait(1000);
✓ Automatically reconnects
✓ User state restored
```

### Event Emission Tests
```javascript
// Test 1: Send message
socket.emit('send-message', {
  roomId: 'room123',
  content: 'Hello world'
}, (response) => {
  ✓ response.success === true
  ✓ response.data.id exists
  ✓ response.data.content === 'Hello world'
});

// Test 2: Join room
socket.emit('join-room', 'room123', (response) => {
  ✓ response.success === true
  ✓ response.data includes room details
  ✓ Socket added to room namespace
});

// Test 3: Invalid room join
socket.emit('join-room', 'unauthorized-room', (response) => {
  ✓ response.success === false
  ✓ response.error === 'Cannot join this room'
});
```

### Broadcast Tests
```javascript
// Setup: Two clients in same room
const client1 = io(url, { auth: { token: token1 } });
const client2 = io(url, { auth: { token: token2 } });

// Test 1: Message broadcast
client2.on('new-message', (message) => {
  ✓ Message received from client1
  ✓ message.user.id === userId1
  ✓ message.content === 'Test message'
});

client1.emit('send-message', {
  roomId: 'room123',
  content: 'Test message'
});

// Test 2: Typing indicator broadcast
client2.on('user-typing', (data) => {
  ✓ data.userId === userId1
  ✓ data.roomId === 'room123'
  ✓ data.username === 'user1'
});

client1.emit('typing-start', 'room123');
```

## Performance Criteria

### Latency Requirements
- [ ] Message delivery: < 100ms (95th percentile)
- [ ] Typing indicators: < 50ms
- [ ] Presence updates: < 200ms
- [ ] Read receipts: < 150ms
- [ ] Room join: < 300ms

### Throughput Tests
```bash
# Load test with Artillery
artillery quick \
  --count 1000 \
  --num 10 \
  --target "ws://localhost:3000"

✓ 1000 concurrent connections handled
✓ No dropped connections
✓ Average latency < 100ms
✓ No memory leaks
```

### Scalability Tests
- [ ] Horizontal scaling with 3 instances
- [ ] Redis adapter synchronizes events
- [ ] Messages delivered across instances
- [ ] Presence consistent across instances
- [ ] No duplicate message delivery

## Redis Integration Tests

### Adapter Configuration
```javascript
// Test 1: Redis adapter active
✓ io.adapter() returns RedisAdapter instance
✓ Pub client connected
✓ Sub client connected

// Test 2: Cross-instance messaging
// Instance 1
socket1.emit('send-message', data);

// Instance 2
socket2.on('new-message', (message) => {
  ✓ Message received from instance 1
  ✓ No duplicate messages
});
```

### State Management
- [ ] Typing indicators stored in Redis
- [ ] Presence data stored with TTL
- [ ] Online users set maintained
- [ ] Data expires correctly
- [ ] Redis memory usage reasonable

## Error Handling Tests

### Connection Errors
- [ ] Invalid token shows clear error
- [ ] Network failure triggers reconnect
- [ ] Server restart handled gracefully
- [ ] Redis failure doesn't crash server
- [ ] Client notified of connection issues

### Event Errors
```javascript
// Test 1: Send message to non-member room
socket.emit('send-message', {
  roomId: 'not-a-member',
  content: 'Test'
}, (response) => {
  ✓ response.success === false
  ✓ response.error === 'Not a room member'
});

// Test 2: Invalid message format
socket.emit('send-message', {
  roomId: 'room123'
  // Missing content
}, (response) => {
  ✓ response.success === false
  ✓ response.error includes validation message
});
```

## Security Validation

### Authentication Security
- [ ] Tokens validated on every connection
- [ ] Expired tokens rejected
- [ ] Malformed tokens rejected
- [ ] User context verified
- [ ] No token = no connection

### Authorization Checks
- [ ] Room membership verified for all events
- [ ] Message sending requires membership
- [ ] Typing events require membership
- [ ] Read receipts validated
- [ ] Admin actions verified

### Input Validation
- [ ] Message content sanitized
- [ ] Message length limited (5000 chars)
- [ ] Room IDs validated as UUIDs
- [ ] Event payloads size-limited
- [ ] XSS attempts blocked

## Client Integration Tests

### React Hook Tests
```javascript
// Test useSocket hook
const { result } = renderHook(() => useSocket());

✓ Socket initialized with token
✓ Event listeners registered
✓ Methods available (joinRoom, sendMessage, etc.)
✓ Cleanup on unmount
```

### State Management
- [ ] Messages added to store on receipt
- [ ] Typing users tracked per room
- [ ] User presence updated
- [ ] Read receipts processed
- [ ] Optimistic updates work

### Reconnection Handling
- [ ] Auto-reconnect after disconnect
- [ ] Exponential backoff implemented
- [ ] State restored after reconnect
- [ ] Pending messages queued
- [ ] User notified of connection status

## End-to-End Tests

### Complete Message Flow
```javascript
// 1. User A connects
const socketA = connectWithAuth(tokenA);
✓ Connected successfully

// 2. User A joins room
socketA.emit('join-room', 'room123', callback);
✓ Joined successfully

// 3. User B connects and joins
const socketB = connectWithAuth(tokenB);
socketB.emit('join-room', 'room123', callback);
✓ User A notified of User B joining

// 4. User A starts typing
socketA.emit('typing-start', 'room123');
✓ User B sees typing indicator

// 5. User A sends message
socketA.emit('send-message', {
  roomId: 'room123',
  content: 'Hello!'
}, callback);
✓ User B receives message instantly
✓ Message saved to database

// 6. User B marks as read
socketB.emit('mark-read', [messageId], callback);
✓ User A sees read receipt

// 7. User A disconnects
socketA.disconnect();
✓ User B notified of offline status
✓ Typing indicator cleared
```

## Monitoring Requirements

### Metrics to Track
- [ ] Active socket connections count
- [ ] Messages per second
- [ ] Average message latency
- [ ] Socket errors per minute
- [ ] Redis memory usage
- [ ] Reconnection frequency

### Logging Requirements
- [ ] Connection events logged
- [ ] Authentication failures logged
- [ ] Message send errors logged
- [ ] Performance metrics logged
- [ ] Redis errors logged

## Documentation Completeness

### Code Documentation
- [ ] All socket events documented
- [ ] Event payload formats specified
- [ ] Error responses documented
- [ ] Client usage examples provided

### API Documentation
- [ ] Socket.io event reference
- [ ] Authentication flow documented
- [ ] Error codes listed
- [ ] Scaling guide included

## Final Checklist

### Core Functionality
- [ ] Real-time messaging works
- [ ] Typing indicators accurate
- [ ] Presence tracking reliable
- [ ] Read receipts functional
- [ ] Room management complete

### Performance
- [ ] Latency targets met
- [ ] Supports 1000+ concurrent users
- [ ] No memory leaks
- [ ] Efficient Redis usage

### Reliability
- [ ] Reconnection works
- [ ] Error handling comprehensive
- [ ] Graceful degradation
- [ ] No message loss

### Security
- [ ] Authentication enforced
- [ ] Authorization checked
- [ ] Input validated
- [ ] No vulnerabilities

**Task is complete when all real-time features work reliably, performance targets are met, and the system scales horizontally.**