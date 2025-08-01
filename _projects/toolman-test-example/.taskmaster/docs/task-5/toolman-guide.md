# Toolman Guide for Task 5: Real-time Communication with Socket.io

## Overview

This guide provides comprehensive instructions for using the selected tools to implement Task 5, which focuses on integrating Socket.io for real-time messaging, typing indicators, user presence management, and read receipts with Redis adapter support for horizontal scaling.

## Core Tools

### 1. **create_directory** (Local - filesystem)
**Purpose**: Create the Socket.io implementation directory structure

**When to Use**: 
- At the beginning to organize Socket.io components
- When separating event handlers and middleware
- For organizing socket utilities and adapters

**How to Use**:
```
# Create Socket.io structure
create_directory /chat-application/backend/src/socket
create_directory /chat-application/backend/src/socket/handlers
create_directory /chat-application/backend/src/socket/middleware
create_directory /chat-application/backend/src/socket/utils
create_directory /chat-application/backend/src/socket/events
create_directory /chat-application/backend/src/socket/adapters
```

**Parameters**:
- `path`: Directory path to create

### 2. **write_file** (Local - filesystem)
**Purpose**: Create Socket.io server configuration, event handlers, and middleware

**When to Use**: 
- To create Socket.io server setup
- To implement event handlers
- To create authentication middleware
- To configure Redis adapter

**How to Use**:
```
# Create Socket.io server configuration
write_file /chat-application/backend/src/socket/socketServer.ts <server-content>

# Create message handler
write_file /chat-application/backend/src/socket/handlers/messageHandler.ts <handler-content>

# Create authentication middleware
write_file /chat-application/backend/src/socket/middleware/socketAuth.ts <auth-content>

# Create Redis adapter configuration
write_file /chat-application/backend/src/socket/adapters/redisAdapter.ts <adapter-content>
```

**Parameters**:
- `path`: File path to write
- `content`: Complete file content

### 3. **read_file** (Local - filesystem)
**Purpose**: Review existing authentication and Redis configurations

**When to Use**: 
- To check JWT implementation from Task 3
- To review Redis configuration from Task 2
- To understand room models from Task 4

**How to Use**:
```
# Read authentication middleware
read_file /chat-application/backend/src/auth/middleware/authMiddleware.ts

# Check Redis configuration
read_file /chat-application/backend/src/database/config/redis.config.ts

# Review room controller
read_file /chat-application/backend/src/api/controllers/roomController.ts
```

**Parameters**:
- `path`: File to read
- `head`/`tail`: Optional line limits

### 4. **edit_file** (Local - filesystem)
**Purpose**: Update server files to integrate Socket.io

**When to Use**: 
- To add Socket.io to main server
- To update package.json with Socket.io dependencies
- To modify environment variables

**How to Use**:
```
# Add Socket.io dependencies
edit_file /chat-application/backend/package.json
# Add: socket.io, @socket.io/redis-adapter, socket.io-client (for testing)

# Update main server file
edit_file /chat-application/backend/src/index.ts
# Integrate Socket.io server with Express

# Update environment variables
edit_file /chat-application/backend/.env.example
# Add Socket.io and Redis configuration
```

**Parameters**:
- `old_string`: Exact text to replace
- `new_string`: New text
- `path`: File to edit

### 5. **list_directory** (Local - filesystem)
**Purpose**: Verify Socket.io structure and files

**When to Use**: 
- After creating directory structure
- To confirm all handlers are in place
- Before testing implementation

**How to Use**:
```
# Verify socket structure
list_directory /chat-application/backend/src/socket

# Check handlers
list_directory /chat-application/backend/src/socket/handlers
```

**Parameters**:
- `path`: Directory to list

## Implementation Flow

1. **Directory Setup Phase**
   - Use `create_directory` to build Socket.io structure
   - Organize by handlers, middleware, utils, events

2. **Server Configuration Phase**
   - Use `write_file` to create socketServer.ts
   - Configure CORS settings
   - Set up Redis adapter for scaling
   - Integrate with Express server

3. **Authentication Middleware Phase**
   - Use `write_file` to create socket authentication
   - Implement JWT verification for socket connections
   - Handle authentication errors

4. **Event Handlers Implementation**
   - Use `write_file` to create handlers for:
     - **join-room**: Room joining logic
     - **send-message**: Message broadcasting
     - **typing**: Typing indicator management
     - **read-message**: Read receipt handling
     - **disconnect**: Presence management

5. **Redis Adapter Configuration**
   - Use `write_file` to configure Redis adapter
   - Enable pub/sub for multi-server support
   - Set up sticky sessions if needed

6. **Integration Phase**
   - Use `edit_file` to update main server
   - Attach Socket.io to HTTP server
   - Configure middleware stack

## Best Practices

1. **Authentication**: Verify JWT on every connection
2. **Room Management**: Validate room membership before operations
3. **Error Handling**: Emit error events to clients
4. **Performance**: Use Redis adapter for scaling
5. **Reconnection**: Implement automatic reconnection logic
6. **Message Delivery**: Ensure delivery with acknowledgments

## Task-Specific Implementation Details

### Socket.io Server Setup Pattern
```typescript
// socketServer.ts
import { Server } from 'socket.io';
import { createAdapter } from '@socket.io/redis-adapter';
import { authMiddleware } from './middleware/socketAuth';

export const initializeSocket = (httpServer) => {
  const io = new Server(httpServer, {
    cors: {
      origin: process.env.CLIENT_URL,
      credentials: true
    }
  });

  // Redis adapter setup
  const pubClient = redisClient.duplicate();
  const subClient = redisClient.duplicate();
  io.adapter(createAdapter(pubClient, subClient));

  // Authentication middleware
  io.use(authMiddleware);

  return io;
};
```

### Event Handler Pattern
```typescript
// messageHandler.ts
export const handleSendMessage = (io, socket) => {
  socket.on('send-message', async (data, callback) => {
    try {
      const { roomId, content } = data;
      const userId = socket.userId;

      // Validate room membership
      const isMember = await roomService.checkMembership(roomId, userId);
      if (!isMember) {
        return callback({ error: 'Not a room member' });
      }

      // Save to database
      const message = await messageService.create({
        room_id: roomId,
        user_id: userId,
        content
      });

      // Broadcast to room
      io.to(roomId).emit('new-message', message);
      
      // Acknowledge
      callback({ success: true, messageId: message.id });
    } catch (error) {
      callback({ error: error.message });
    }
  });
};
```

### Typing Indicator Pattern
```typescript
// typingHandler.ts
export const handleTyping = (io, socket, redisClient) => {
  socket.on('typing', async ({ roomId, isTyping }) => {
    const userId = socket.userId;
    const key = `typing:${roomId}:${userId}`;

    if (isTyping) {
      // Set with 5 second expiry
      await redisClient.setex(key, 5, 'true');
      socket.to(roomId).emit('user-typing', { userId, isTyping: true });
    } else {
      await redisClient.del(key);
      socket.to(roomId).emit('user-typing', { userId, isTyping: false });
    }
  });
};
```

### Presence Management Pattern
```typescript
// presenceHandler.ts
export const handlePresence = (io, socket, redisClient) => {
  // On connection
  socket.on('connect', async () => {
    const userId = socket.userId;
    await redisClient.setex(`presence:${userId}`, 30, 'online');
    io.emit('user-online', { userId });
  });

  // On disconnect
  socket.on('disconnect', async () => {
    const userId = socket.userId;
    await redisClient.del(`presence:${userId}`);
    io.emit('user-offline', { userId });
  });
};
```

## Troubleshooting

- **Connection Issues**: Check CORS configuration
- **Authentication Failures**: Verify JWT token format
- **Scaling Issues**: Ensure Redis adapter is properly configured
- **Message Loss**: Implement acknowledgment callbacks
- **Memory Leaks**: Properly clean up event listeners

## Testing Approach

1. **Unit Tests**:
   - Test individual event handlers
   - Test authentication middleware
   - Mock Socket.io for testing

2. **Integration Tests**:
   - Test multi-client scenarios
   - Test message delivery
   - Test reconnection handling

3. **Performance Tests**:
   - Measure message latency (<100ms requirement)
   - Test with multiple concurrent connections
   - Test horizontal scaling with Redis adapter