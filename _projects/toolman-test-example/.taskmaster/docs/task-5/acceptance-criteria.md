# Acceptance Criteria: Real-time Communication with Socket.io

## Overview
This document defines the acceptance criteria for implementing real-time communication features using Socket.io.

## Socket.io Server Setup Criteria

### ✅ Server Configuration
- [ ] Socket.io server initialized with HTTP server
- [ ] CORS configured for frontend URL
- [ ] WebSocket transport prioritized
- [ ] Polling fallback available
- [ ] Heartbeat interval set (25 seconds)
- [ ] Connection timeout configured (60 seconds)

### ✅ Redis Adapter
- [ ] Redis adapter connected successfully
- [ ] Pub/sub clients configured
- [ ] Multi-instance message delivery works
- [ ] Connection retry logic implemented
- [ ] Adapter errors handled gracefully

## Authentication Criteria

### ✅ Socket Authentication
- [ ] JWT token extracted from handshake
- [ ] Token validation implemented
- [ ] User info attached to socket
- [ ] Invalid tokens rejected with error
- [ ] Authentication errors sent to client
- [ ] Expired tokens handled properly

### ✅ User Context
- [ ] userId available on all events
- [ ] username stored on socket
- [ ] User verified in database
- [ ] Socket ID tracked in Redis

## Room Management Criteria

### ✅ Join/Leave Operations
- [ ] Room membership verified on join
- [ ] Socket joins room namespace
- [ ] User notification room created
- [ ] Join event broadcast to room
- [ ] Leave cleans up typing state
- [ ] Active room tracked in Redis

### ✅ Room Events
- [ ] `room-joined` includes room data
- [ ] Online users list provided
- [ ] `user-joined-room` broadcast
- [ ] `user-left-room` broadcast
- [ ] Error on unauthorized access

## Message Handling Criteria

### ✅ Send Message
- [ ] Content validated (1-1000 chars)
- [ ] Room membership verified
- [ ] Message saved to database
- [ ] Broadcast to all room members
- [ ] User info included in payload
- [ ] Typing indicator cleared

### ✅ Message Operations
- [ ] Edit restricted to owner
- [ ] Delete checks permissions
- [ ] Soft delete implemented
- [ ] Events broadcast to room
- [ ] Database updated correctly

### ✅ Message Delivery
- [ ] Latency < 100ms (95th percentile)
- [ ] Messages arrive in order
- [ ] No message loss
- [ ] Works across instances

## Typing Indicators Criteria

### ✅ Typing Events
- [ ] `typing-start` adds to Redis set
- [ ] TTL set to 10 seconds
- [ ] `typing-stop` removes from set
- [ ] Auto-stop after 5 seconds
- [ ] Cleared on message send
- [ ] Cleared on disconnect

### ✅ Typing Broadcast
- [ ] Events sent to room members
- [ ] Username included in event
- [ ] No self-notification
- [ ] Updates in real-time

## User Presence Criteria

### ✅ Online Status
- [ ] Database updated on connect
- [ ] Status set offline on disconnect
- [ ] Multiple connections handled
- [ ] Last seen timestamp set
- [ ] Presence key in Redis with TTL

### ✅ Presence Events
- [ ] `user-online` broadcast to rooms
- [ ] `user-offline` sent on disconnect
- [ ] Only sent when status changes
- [ ] Includes username in payload

## Read Receipts Criteria

### ✅ Read Tracking
- [ ] Messages marked as read in DB
- [ ] Read receipts stored in Redis
- [ ] TTL set on Redis keys (24h)
- [ ] Broadcast to message sender
- [ ] Includes timestamp

## Error Handling Criteria

### ✅ Error Management
- [ ] All handlers wrapped in try-catch
- [ ] Errors logged server-side
- [ ] User-friendly error messages
- [ ] Error event sent to client
- [ ] No sensitive data exposed

### ✅ Rate Limiting
- [ ] Event rate limiting implemented
- [ ] Per-user limits enforced
- [ ] Rate limit error sent
- [ ] Limits configurable
- [ ] Memory efficient tracking

## Performance Criteria

### ✅ Latency Requirements
- [ ] Message delivery < 100ms
- [ ] Typing updates < 50ms
- [ ] Presence updates < 5s
- [ ] Connection time < 1s

### ✅ Scalability
- [ ] Supports 10,000+ connections
- [ ] Memory < 100MB per 1000 users
- [ ] CPU usage reasonable
- [ ] Horizontal scaling works

## Testing Checklist

### Unit Tests
```javascript
describe('Socket.io Server', () => {
  it('authenticates valid JWT tokens');
  it('rejects invalid tokens');
  it('handles room join with permission');
  it('broadcasts messages to room');
  it('updates typing indicators');
  it('tracks user presence');
  it('handles read receipts');
});
```

### Integration Tests
```javascript
// Test multi-client scenarios
it('delivers messages between clients');
it('shows typing to other users');
it('updates presence across rooms');
it('handles reconnection properly');
```

### Load Testing
```bash
# Artillery.io configuration
config:
  target: "http://localhost:3001"
  phases:
    - duration: 60
      arrivalRate: 100
  socketio:
    transports: ["websocket"]

scenarios:
  - name: "Chat User"
    engine: "socketio"
    flow:
      - emit:
          channel: "join-room"
          data: "room-123"
      - emit:
          channel: "send-message"
          data:
            roomId: "room-123"
            content: "Test message"
      - think: 5
```

## Definition of Done

The task is complete when:
1. Socket.io server configured with Redis
2. JWT authentication working
3. All real-time events functional
4. Message delivery < 100ms
5. Typing indicators update live
6. Presence tracking accurate
7. Read receipts delivered
8. Multi-instance setup tested
9. All tests passing

## Common Issues to Avoid

- ❌ Not verifying room membership
- ❌ Missing error handlers
- ❌ Memory leaks from event listeners
- ❌ Not cleaning up on disconnect
- ❌ Forgetting Redis TTLs
- ❌ Broadcasting to wrong rooms
- ❌ Not handling reconnection
- ❌ Missing rate limiting

## Verification Steps

```bash
# Start Socket.io server
npm run dev

# Monitor Redis
redis-cli monitor | grep -E "(typing|presence|socket)"

# Test with multiple clients
# Open multiple browser tabs and verify:
# 1. Messages appear instantly
# 2. Typing shows/hides properly
# 3. Online status updates
# 4. Read receipts work

# Check performance
# Monitor server logs for message delivery time
# Verify < 100ms latency

# Test scaling
# Start multiple server instances
# Verify messages work across instances
```