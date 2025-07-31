# Acceptance Criteria for Task 5: Real-time Communication with Socket.io

## Overview

This document defines the acceptance criteria and test cases for the Socket.io real-time communication implementation. All criteria must be met for the task to be considered complete.

## Functional Requirements

### 1. Socket.io Server Setup

#### Acceptance Criteria
- [ ] Socket.io server starts successfully on configured port
- [ ] Server accepts both WebSocket and polling connections
- [ ] CORS is properly configured for frontend URL
- [ ] Server handles graceful shutdown without dropping active connections

#### Test Cases
```typescript
// Test: Server initialization
it('should start Socket.io server on configured port', async () => {
  const server = new SocketManager(httpServer);
  expect(server.getIO()).toBeDefined();
  expect(httpServer.listening).toBe(true);
});

// Test: Transport support
it('should accept both websocket and polling connections', async () => {
  const wsClient = io(serverUrl, { transports: ['websocket'] });
  const pollingClient = io(serverUrl, { transports: ['polling'] });
  
  await Promise.all([
    new Promise(resolve => wsClient.on('connect', resolve)),
    new Promise(resolve => pollingClient.on('connect', resolve))
  ]);
  
  expect(wsClient.connected).toBe(true);
  expect(pollingClient.connected).toBe(true);
});
```

### 2. JWT Authentication

#### Acceptance Criteria
- [ ] Valid JWT tokens are accepted and user info extracted
- [ ] Invalid or expired tokens are rejected with appropriate error
- [ ] Missing tokens result in authentication error
- [ ] User is automatically joined to personal notification room
- [ ] Inactive users are denied access

#### Test Cases
```typescript
// Test: Valid authentication
it('should authenticate user with valid JWT', (done) => {
  const validToken = jwt.sign({ userId: 'user123' }, JWT_SECRET);
  const client = io(serverUrl, { auth: { token: validToken } });
  
  client.on('connect', () => {
    expect(client.connected).toBe(true);
    done();
  });
});

// Test: Invalid authentication
it('should reject invalid JWT', (done) => {
  const client = io(serverUrl, { auth: { token: 'invalid-token' } });
  
  client.on('connect_error', (error) => {
    expect(error.message).toBe('Authentication failed');
    done();
  });
});

// Test: Missing token
it('should reject connection without token', (done) => {
  const client = io(serverUrl);
  
  client.on('connect_error', (error) => {
    expect(error.message).toBe('Authentication required');
    done();
  });
});
```

### 3. Real-time Messaging

#### Acceptance Criteria
- [ ] Messages are delivered to all room members in real-time
- [ ] Message includes sender info, content, timestamp, and room ID
- [ ] Large messages (up to 1MB) are handled correctly
- [ ] Messages with attachments are properly transmitted
- [ ] Invalid messages are rejected with error response
- [ ] Rate limiting prevents message spam (30 messages/minute)

#### Test Cases
```typescript
// Test: Message delivery
it('should deliver messages to all room members', async () => {
  const room = 'test-room';
  const message = { roomId: room, content: 'Hello, world!' };
  
  // Setup two clients in same room
  await client1.emit('join-room', room);
  await client2.emit('join-room', room);
  
  const messageReceived = new Promise((resolve) => {
    client2.on('new-message', (msg) => {
      expect(msg.content).toBe(message.content);
      expect(msg.room).toBe(room);
      resolve(msg);
    });
  });
  
  client1.emit('send-message', message);
  await messageReceived;
});

// Test: Rate limiting
it('should enforce rate limiting', async () => {
  let errorCount = 0;
  
  client.on('error', (error) => {
    if (error.message === 'Rate limit exceeded') errorCount++;
  });
  
  // Send 31 messages (exceeding 30/min limit)
  for (let i = 0; i < 31; i++) {
    client.emit('send-message', { roomId: 'test', content: `Message ${i}` });
  }
  
  await new Promise(resolve => setTimeout(resolve, 100));
  expect(errorCount).toBeGreaterThan(0);
});
```

### 4. Typing Indicators

#### Acceptance Criteria
- [ ] Typing start event is broadcast to room members
- [ ] Typing stop event is broadcast when user stops
- [ ] Auto-stop after 3 seconds of no typing activity
- [ ] Multiple users can type simultaneously
- [ ] Typing events include user info

#### Test Cases
```typescript
// Test: Typing indicators
it('should broadcast typing indicators', async () => {
  const room = 'test-room';
  
  const typingReceived = new Promise((resolve) => {
    client2.on('user-typing', (data) => {
      expect(data.roomId).toBe(room);
      expect(data.isTyping).toBe(true);
      expect(data.userId).toBeDefined();
      resolve(data);
    });
  });
  
  client1.emit('typing-start', room);
  await typingReceived;
});

// Test: Auto-stop typing
it('should auto-stop typing after 3 seconds', async () => {
  const room = 'test-room';
  let typingStopReceived = false;
  
  client2.on('user-typing', (data) => {
    if (!data.isTyping) typingStopReceived = true;
  });
  
  client1.emit('typing-start', room);
  await new Promise(resolve => setTimeout(resolve, 3500));
  
  expect(typingStopReceived).toBe(true);
});
```

### 5. User Presence

#### Acceptance Criteria
- [ ] User status updates are broadcast to relevant rooms
- [ ] Offline status is set on disconnect
- [ ] Presence includes last seen timestamp
- [ ] Room presence list can be requested
- [ ] Presence updates are persisted in Redis

#### Test Cases
```typescript
// Test: Presence updates
it('should update user presence', async () => {
  const presenceUpdate = new Promise((resolve) => {
    client2.on('presence-update', (data) => {
      expect(data.userId).toBe(user1Id);
      expect(data.status).toBe('away');
      resolve(data);
    });
  });
  
  client1.emit('update-presence', 'away');
  await presenceUpdate;
});

// Test: Disconnect presence
it('should set user offline on disconnect', async () => {
  const offlineUpdate = new Promise((resolve) => {
    client2.on('presence-update', (data) => {
      if (data.status === 'offline') resolve(data);
    });
  });
  
  client1.disconnect();
  await offlineUpdate;
});
```

### 6. Read Receipts

#### Acceptance Criteria
- [ ] Read receipts are sent when messages are viewed
- [ ] Multiple messages can be marked as read in batch
- [ ] Read receipts include timestamp and user info
- [ ] Read status is persisted in database
- [ ] Only unread messages are marked (no duplicates)

#### Test Cases
```typescript
// Test: Read receipts
it('should handle read receipts', async () => {
  const messageIds = ['msg1', 'msg2', 'msg3'];
  const room = 'test-room';
  
  const readReceived = new Promise((resolve) => {
    client1.on('messages-read', (data) => {
      expect(data.messageIds).toEqual(messageIds);
      expect(data.userId).toBe(user2Id);
      expect(data.readAt).toBeDefined();
      resolve(data);
    });
  });
  
  client2.emit('mark-as-read', { messageIds, roomId: room });
  await readReceived;
});
```

## Performance Requirements

### 7. Latency Requirements

#### Acceptance Criteria
- [ ] Message delivery latency < 100ms (95th percentile)
- [ ] Typing indicator latency < 50ms
- [ ] Presence update latency < 200ms
- [ ] Connection establishment < 2 seconds

#### Test Cases
```typescript
// Test: Message latency
it('should deliver messages within 100ms', async () => {
  const latencies = [];
  
  for (let i = 0; i < 100; i++) {
    const start = Date.now();
    
    const received = new Promise((resolve) => {
      client2.once('new-message', () => {
        const latency = Date.now() - start;
        latencies.push(latency);
        resolve(latency);
      });
    });
    
    client1.emit('send-message', { roomId: 'test', content: 'Test' });
    await received;
  }
  
  const p95 = latencies.sort((a, b) => a - b)[94];
  expect(p95).toBeLessThan(100);
});
```

### 8. Scalability Requirements

#### Acceptance Criteria
- [ ] Redis adapter successfully connects and initializes
- [ ] Messages are delivered across multiple server instances
- [ ] Presence updates work across instances
- [ ] No message loss during Redis reconnection
- [ ] Handles 1000+ concurrent connections per instance

#### Test Cases
```typescript
// Test: Cross-instance messaging
it('should deliver messages across server instances', async () => {
  // Connect to different server instances
  const client1 = io('http://server1:3001', { auth: { token } });
  const client2 = io('http://server2:3001', { auth: { token } });
  
  await Promise.all([
    new Promise(resolve => client1.on('connect', resolve)),
    new Promise(resolve => client2.on('connect', resolve))
  ]);
  
  const messageReceived = new Promise((resolve) => {
    client2.on('new-message', (msg) => resolve(msg));
  });
  
  client1.emit('send-message', { roomId: 'test', content: 'Cross-instance' });
  const msg = await messageReceived;
  
  expect(msg.content).toBe('Cross-instance');
});
```

## Security Requirements

### 9. Security Measures

#### Acceptance Criteria
- [ ] Rate limiting prevents spam (per user and per IP)
- [ ] Input sanitization prevents XSS attacks
- [ ] CSRF protection validates socket connections
- [ ] IP blacklisting blocks abusive connections
- [ ] Message size limits are enforced (1MB max)

#### Test Cases
```typescript
// Test: XSS prevention
it('should sanitize user input', async () => {
  const maliciousContent = '<script>alert("XSS")</script>';
  
  const received = new Promise((resolve) => {
    client2.on('new-message', (msg) => resolve(msg));
  });
  
  client1.emit('send-message', { roomId: 'test', content: maliciousContent });
  const msg = await received;
  
  expect(msg.content).not.toContain('<script>');
  expect(msg.content).toContain('&lt;script&gt;');
});

// Test: Message size limit
it('should reject messages over 1MB', (done) => {
  const largeContent = 'x'.repeat(1024 * 1024 + 1); // 1MB + 1 byte
  
  client.on('error', (error) => {
    expect(error.message).toContain('size limit');
    done();
  });
  
  client.emit('send-message', { roomId: 'test', content: largeContent });
});
```

## Error Handling

### 10. Error Handling and Recovery

#### Acceptance Criteria
- [ ] Graceful handling of Redis connection failures
- [ ] Automatic reconnection with exponential backoff
- [ ] Clear error messages sent to clients
- [ ] No data loss during temporary disconnections
- [ ] Proper cleanup of resources on errors

#### Test Cases
```typescript
// Test: Reconnection handling
it('should reconnect after connection loss', async () => {
  let reconnectCount = 0;
  
  client.on('reconnect', () => reconnectCount++);
  
  // Simulate connection loss
  client.io.engine.close();
  
  await new Promise(resolve => setTimeout(resolve, 2000));
  
  expect(client.connected).toBe(true);
  expect(reconnectCount).toBeGreaterThan(0);
});

// Test: Error message format
it('should send structured error messages', (done) => {
  client.on('error', (error) => {
    expect(error).toHaveProperty('message');
    expect(error).toHaveProperty('timestamp');
    expect(error.code).toBeDefined();
    done();
  });
  
  // Trigger an error by sending invalid data
  client.emit('send-message', null);
});
```

## Integration Tests

### 11. End-to-End Scenarios

#### Test Case: Complete Chat Flow
```typescript
it('should handle complete chat flow', async () => {
  // 1. Users join room
  await client1.emit('join-room', 'chat-room');
  await client2.emit('join-room', 'chat-room');
  
  // 2. User 1 starts typing
  client1.emit('typing-start', 'chat-room');
  
  // 3. User 1 sends message
  const message = { roomId: 'chat-room', content: 'Hello!' };
  client1.emit('send-message', message);
  
  // 4. User 2 receives and reads message
  const msgId = await new Promise((resolve) => {
    client2.on('new-message', (msg) => {
      resolve(msg.id);
    });
  });
  
  client2.emit('mark-as-read', { messageIds: [msgId], roomId: 'chat-room' });
  
  // 5. Verify read receipt
  const readReceipt = await new Promise((resolve) => {
    client1.on('messages-read', (data) => resolve(data));
  });
  
  expect(readReceipt.messageIds).toContain(msgId);
});
```

## Monitoring and Metrics

### 12. Observability Requirements

#### Acceptance Criteria
- [ ] Socket connection count is tracked
- [ ] Message throughput is measured
- [ ] Error rates are monitored
- [ ] Latency metrics are collected
- [ ] Memory usage is within limits

#### Verification
- Check monitoring dashboard shows all metrics
- Verify alerts trigger on threshold breaches
- Confirm metrics are exported to monitoring system

## Documentation Requirements

### 13. Documentation Completeness

#### Acceptance Criteria
- [ ] All socket events are documented with examples
- [ ] Client integration guide is provided
- [ ] Deployment instructions include scaling considerations
- [ ] Troubleshooting guide covers common issues
- [ ] Performance tuning guide is available

## Final Checklist

Before marking this task as complete, ensure:

1. [ ] All unit tests pass with >80% coverage
2. [ ] Integration tests complete successfully
3. [ ] Performance benchmarks meet requirements
4. [ ] Security audit shows no vulnerabilities
5. [ ] Documentation is complete and accurate
6. [ ] Code review has been completed
7. [ ] Deployment to staging environment successful
8. [ ] Load testing confirms scalability targets
9. [ ] Monitoring and alerts are configured
10. [ ] Rollback procedure is documented and tested