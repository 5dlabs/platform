# Task 5: Real-time Communication with Socket.io - Toolman Usage Guide

## Overview
This guide explains how to use the selected Toolman tools to implement real-time communication features with Socket.io. The tools focus on file operations for creating WebSocket handlers and researching best practices for scalable real-time systems.

## Core Tools

### 1. brave_web_search
**Purpose**: Research Socket.io best practices and scaling patterns
**When to use**: 
- Before implementing socket architecture
- When designing event patterns
- For Redis adapter configuration
- To find performance optimization techniques

**How to use**:
```json
{
  "tool": "brave_web_search",
  "query": "Socket.io Redis adapter horizontal scaling 2024",
  "freshness": "year"
}
```

**Key research topics**:
- "Socket.io authentication JWT best practices"
- "Socket.io Redis adapter cluster configuration"
- "WebSocket connection management at scale"
- "Socket.io typing indicators implementation"
- "Socket.io reconnection strategies production"

### 2. create_directory
**Purpose**: Organize socket-related code structure
**When to use**:
- Creating socket handler directories
- Setting up service folders
- Organizing event handlers

**How to use**:
```json
{
  "tool": "create_directory",
  "path": "/chat-application/backend/src/socket/handlers"
}
```

**Directory structure**:
```
/backend/src/
├── socket/
│   ├── socketServer.ts
│   ├── handlers/
│   │   ├── index.ts
│   │   ├── messageHandlers.ts
│   │   ├── roomHandlers.ts
│   │   └── presenceHandlers.ts
│   └── middleware/
│       └── socketAuth.ts
├── services/
│   ├── typingService.ts
│   └── presenceService.ts
```

### 3. write_file
**Purpose**: Create socket implementation files
**When to use**:
- Writing socket server configuration
- Creating event handlers
- Implementing services
- Setting up client hooks

**How to use**:
```json
{
  "tool": "write_file",
  "path": "/chat-application/backend/src/socket/socketServer.ts",
  "content": "// Socket.io server implementation"
}
```

### 4. edit_file
**Purpose**: Update existing files with socket integration
**When to use**:
- Adding socket server to main app
- Updating types for socket events
- Modifying services for real-time features
- Integrating with existing APIs

**How to use**:
```json
{
  "tool": "edit_file",
  "path": "/chat-application/backend/src/app.ts",
  "old_string": "const server = http.createServer(app);",
  "new_string": "const server = http.createServer(app);\nconst socketServer = new SocketServer(server);"
}
```

### 5. read_file
**Purpose**: Review existing code before socket integration
**When to use**:
- Before modifying server setup
- To understand current message flow
- To check authentication implementation
- Before updating type definitions

## Implementation Flow

### Phase 1: Research Best Practices (15 minutes)
1. **Socket.io patterns**:
   ```json
   {
     "tool": "brave_web_search",
     "query": "Socket.io event naming conventions best practices"
   }
   ```

2. **Scaling strategies**:
   ```json
   {
     "tool": "brave_web_search",
     "query": "Socket.io Redis adapter sticky sessions load balancing"
   }
   ```

3. **Performance optimization**:
   ```json
   {
     "tool": "brave_web_search",
     "query": "Socket.io performance optimization large scale"
   }
   ```

### Phase 2: Create Socket Structure (20 minutes)
1. **Create directories**:
   ```json
   {
     "tool": "create_directory",
     "path": "/chat-application/backend/src/socket/handlers"
   }
   ```

2. **Write socket server**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/socket/socketServer.ts",
     "content": "// Complete Socket.io server setup with Redis adapter"
   }
   ```

3. **Create authentication middleware**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/socket/middleware/socketAuth.ts",
     "content": "// JWT authentication for socket connections"
   }
   ```

### Phase 3: Implement Event Handlers (25 minutes)
1. **Message handlers**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/socket/handlers/messageHandlers.ts",
     "content": "// Real-time message handling"
   }
   ```

2. **Room handlers**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/socket/handlers/roomHandlers.ts",
     "content": "// Room join/leave logic"
   }
   ```

3. **Presence handlers**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/socket/handlers/presenceHandlers.ts",
     "content": "// User presence tracking"
   }
   ```

### Phase 4: Service Implementation (20 minutes)
1. **Typing service**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/services/typingService.ts",
     "content": "// Redis-based typing indicator management"
   }
   ```

2. **Presence service**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/services/presenceService.ts",
     "content": "// Online/offline status tracking"
   }
   ```

### Phase 5: Client Integration (15 minutes)
1. **Create socket hook**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/frontend/src/hooks/useSocket.ts",
     "content": "// React hook for socket management"
   }
   ```

2. **Update app initialization**:
   ```json
   {
     "tool": "read_file",
     "path": "/chat-application/backend/src/server.ts"
   }
   ```
   Then:
   ```json
   {
     "tool": "edit_file",
     "path": "/chat-application/backend/src/server.ts",
     "old_string": "server.listen(PORT",
     "new_string": "const socketServer = new SocketServer(server);\n\nserver.listen(PORT"
   }
   ```

## Best Practices

### Event Naming Conventions
```typescript
// Use clear, action-based names
socket.emit('send-message');     // Good
socket.emit('message');          // Ambiguous

// Namespace related events
socket.emit('room:join');        // Good
socket.emit('typing:start');     // Good

// Use past tense for confirmations
socket.on('message-sent');       // Confirmation
socket.on('user-joined');        // Broadcast
```

### Error Handling Pattern
```typescript
// Always use callbacks for critical events
socket.emit('send-message', data, (response) => {
  if (response.success) {
    // Handle success
  } else {
    // Handle error
    console.error(response.error);
  }
});
```

### Performance Patterns
```typescript
// Batch updates when possible
const updates = [];
// Collect updates...
socket.emit('batch-update', updates);

// Throttle frequent events
const throttledTyping = throttle(() => {
  socket.emit('typing-start', roomId);
}, 1000);
```

## Common Patterns

### Research → Implement → Test
```javascript
// 1. Research scaling patterns
const patterns = await brave_web_search("Socket.io horizontal scaling production");

// 2. Implement based on findings
await write_file("socket/config.ts", scalingConfig);

// 3. Create load tests
await write_file("tests/socket.load.test.ts", loadTests);
```

### Incremental Integration
```javascript
// 1. Create basic socket server
await write_file("socket/server.ts", basicServer);

// 2. Add authentication
await edit_file("socket/server.ts", 
  "io.on('connection'",
  "io.use(authenticate);\n\nio.on('connection'"
);

// 3. Add Redis adapter
await edit_file("socket/server.ts",
  "const io = new Server",
  "const io = new Server(server);\nio.adapter(redisAdapter);"
);
```

## Socket.io Specific Patterns

### Room Management
```typescript
// Join multiple rooms efficiently
socket.on('join-rooms', async (roomIds) => {
  const allowedRooms = await filterAllowedRooms(roomIds, socket.userId);
  allowedRooms.forEach(roomId => {
    socket.join(`room:${roomId}`);
  });
});
```

### Acknowledgment Pattern
```typescript
// Server acknowledges every critical event
socket.on('send-message', async (data, callback) => {
  try {
    const result = await processMessage(data);
    callback({ success: true, data: result });
  } catch (error) {
    callback({ success: false, error: error.message });
  }
});
```

### Presence Management
```typescript
// Track presence with Redis
const PRESENCE_KEY = 'presence:';
const PRESENCE_TTL = 300; // 5 minutes

async function updatePresence(userId: string) {
  await redis.setex(
    `${PRESENCE_KEY}${userId}`,
    PRESENCE_TTL,
    JSON.stringify({ online: true, lastSeen: Date.now() })
  );
}
```

## Troubleshooting

### Issue: Connection drops frequently
**Solution**: Adjust ping timeout, check proxy settings, ensure WebSocket transport

### Issue: Messages not syncing across instances
**Solution**: Verify Redis adapter configuration, check Redis connectivity

### Issue: Memory usage growing
**Solution**: Implement room cleanup, limit event listeners, check for memory leaks

### Issue: Typing indicators stuck
**Solution**: Implement auto-timeout, clear on disconnect, use Redis TTL

## Performance Optimization

### Connection Management
```typescript
// Limit rooms per socket
const MAX_ROOMS_PER_SOCKET = 100;

// Clean up on disconnect
socket.on('disconnect', async () => {
  const rooms = Array.from(socket.rooms);
  // Clean up presence, typing, etc.
});
```

### Event Optimization
```typescript
// Compress large payloads
io.engine.opts.compression = true;

// Batch message delivery
const messageQueue = new Map();
setInterval(flushMessageQueue, 100);
```

## Task Completion Checklist
- [ ] Socket server configured with authentication
- [ ] Redis adapter integrated
- [ ] All event handlers implemented
- [ ] Typing indicators working
- [ ] Presence tracking active
- [ ] Read receipts functional
- [ ] Client hook created
- [ ] Error handling complete
- [ ] Performance optimized
- [ ] Load tested successfully

This systematic approach ensures a robust, scalable real-time communication system ready for production use.