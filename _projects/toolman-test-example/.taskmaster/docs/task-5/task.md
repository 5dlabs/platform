# Task 5: Real-time Communication with Socket.io

## Overview
Implement Socket.io for real-time messaging capabilities including live message delivery, typing indicators, user presence tracking, and read receipts. Configure Redis adapter for horizontal scaling support.

## Technical Implementation Guide

### Phase 1: Socket.io Server Setup

#### Core Socket Server Configuration
```typescript
// backend/src/socket/socketServer.ts
import { Server as HTTPServer } from 'http';
import { Server, Socket } from 'socket.io';
import { createAdapter } from '@socket.io/redis-adapter';
import { createClient } from 'redis';
import { verifyAccessToken } from '../utils/jwt';
import { SocketHandler } from './socketHandler';

export interface AuthenticatedSocket extends Socket {
  userId: string;
  username: string;
}

export class SocketServer {
  private io: Server;
  private socketHandler: SocketHandler;

  constructor(httpServer: HTTPServer) {
    this.io = new Server(httpServer, {
      cors: {
        origin: process.env.FRONTEND_URL || 'http://localhost:3000',
        credentials: true
      },
      transports: ['websocket', 'polling'],
      pingTimeout: 60000,
      pingInterval: 25000
    });

    this.setupRedisAdapter();
    this.setupAuthentication();
    this.socketHandler = new SocketHandler(this.io);
  }

  private async setupRedisAdapter() {
    const pubClient = createClient({ 
      url: process.env.REDIS_URL,
      socket: {
        reconnectStrategy: (retries) => Math.min(retries * 50, 500)
      }
    });
    
    const subClient = pubClient.duplicate();
    
    await Promise.all([
      pubClient.connect(),
      subClient.connect()
    ]);

    this.io.adapter(createAdapter(pubClient, subClient));
    
    console.log('Socket.io Redis adapter configured');
  }

  private setupAuthentication() {
    this.io.use(async (socket: Socket, next) => {
      try {
        const token = socket.handshake.auth.token || socket.handshake.headers.authorization?.split(' ')[1];
        
        if (!token) {
          return next(new Error('No token provided'));
        }

        const payload = verifyAccessToken(token);
        const user = await userRepository.findById(payload.userId);
        
        if (!user) {
          return next(new Error('User not found'));
        }

        // Attach user info to socket
        (socket as AuthenticatedSocket).userId = user.id;
        (socket as AuthenticatedSocket).username = user.username;
        
        next();
      } catch (error) {
        next(new Error('Authentication failed'));
      }
    });
  }

  public start() {
    this.io.on('connection', (socket: Socket) => {
      this.socketHandler.handleConnection(socket as AuthenticatedSocket);
    });
  }
}
```

### Phase 2: Socket Event Handlers

#### Main Socket Handler
```typescript
// backend/src/socket/socketHandler.ts
import { Server } from 'socket.io';
import { AuthenticatedSocket } from './socketServer';
import { MessageHandler } from './handlers/messageHandler';
import { RoomHandler } from './handlers/roomHandler';
import { PresenceHandler } from './handlers/presenceHandler';
import { TypingHandler } from './handlers/typingHandler';

export class SocketHandler {
  private messageHandler: MessageHandler;
  private roomHandler: RoomHandler;
  private presenceHandler: PresenceHandler;
  private typingHandler: TypingHandler;

  constructor(private io: Server) {
    this.messageHandler = new MessageHandler(io);
    this.roomHandler = new RoomHandler(io);
    this.presenceHandler = new PresenceHandler(io);
    this.typingHandler = new TypingHandler(io);
  }

  handleConnection(socket: AuthenticatedSocket) {
    console.log(`User connected: ${socket.userId} (${socket.username})`);

    // Update user presence
    this.presenceHandler.handleUserConnect(socket);

    // Join user to their personal notification room
    socket.join(`user:${socket.userId}`);

    // Register event handlers
    this.registerEventHandlers(socket);

    // Handle disconnect
    socket.on('disconnect', () => {
      console.log(`User disconnected: ${socket.userId}`);
      this.presenceHandler.handleUserDisconnect(socket);
    });
  }

  private registerEventHandlers(socket: AuthenticatedSocket) {
    // Room events
    socket.on('join-room', (roomId: string) => 
      this.roomHandler.handleJoinRoom(socket, roomId)
    );
    
    socket.on('leave-room', (roomId: string) => 
      this.roomHandler.handleLeaveRoom(socket, roomId)
    );

    // Message events
    socket.on('send-message', (data: any) => 
      this.messageHandler.handleSendMessage(socket, data)
    );
    
    socket.on('edit-message', (data: any) => 
      this.messageHandler.handleEditMessage(socket, data)
    );
    
    socket.on('delete-message', (data: any) => 
      this.messageHandler.handleDeleteMessage(socket, data)
    );

    // Typing events
    socket.on('typing-start', (roomId: string) => 
      this.typingHandler.handleTypingStart(socket, roomId)
    );
    
    socket.on('typing-stop', (roomId: string) => 
      this.typingHandler.handleTypingStop(socket, roomId)
    );

    // Read receipt events
    socket.on('mark-read', (data: any) => 
      this.messageHandler.handleMarkRead(socket, data)
    );

    // Error handling
    socket.on('error', (error) => {
      console.error(`Socket error for user ${socket.userId}:`, error);
    });
  }
}
```

### Phase 3: Message Handler Implementation

```typescript
// backend/src/socket/handlers/messageHandler.ts
import { Server } from 'socket.io';
import { AuthenticatedSocket } from '../socketServer';
import { messageRepository } from '../../repositories/messageRepository';
import { roomUserRepository } from '../../repositories/roomUserRepository';
import redis from '../../config/redis';

export class MessageHandler {
  constructor(private io: Server) {}

  async handleSendMessage(socket: AuthenticatedSocket, data: {
    roomId: string;
    content: string;
    messageType?: string;
  }) {
    try {
      const { roomId, content, messageType = 'text' } = data;

      // Validate user is room member
      const isMember = await roomUserRepository.isUserInRoom(roomId, socket.userId);
      if (!isMember) {
        socket.emit('error', { message: 'Not a member of this room' });
        return;
      }

      // Create message
      const message = await messageRepository.create({
        roomId,
        userId: socket.userId,
        content: content.trim(),
        messageType
      });

      // Get user info for message
      const user = await userRepository.findById(socket.userId);

      const messageData = {
        ...message,
        user: {
          id: user!.id,
          username: user!.username,
          avatarUrl: user!.avatarUrl
        }
      };

      // Emit to all room members
      this.io.to(`room:${roomId}`).emit('message-received', messageData);

      // Clear typing indicator
      await this.clearTypingIndicator(socket, roomId);

    } catch (error) {
      console.error('Send message error:', error);
      socket.emit('error', { message: 'Failed to send message' });
    }
  }

  async handleMarkRead(socket: AuthenticatedSocket, data: {
    roomId: string;
    messageId: string;
  }) {
    try {
      const { roomId, messageId } = data;

      // Mark message as read
      await messageRepository.markAsRead(roomId, socket.userId, messageId);

      // Store in Redis for quick access
      await redis.sadd(`read_receipts:${messageId}`, socket.userId);
      await redis.expire(`read_receipts:${messageId}`, 86400); // 24 hours

      // Emit read receipt to room members
      this.io.to(`room:${roomId}`).emit('message-read', {
        messageId,
        userId: socket.userId,
        readAt: new Date()
      });

    } catch (error) {
      console.error('Mark read error:', error);
    }
  }

  private async clearTypingIndicator(socket: AuthenticatedSocket, roomId: string) {
    await redis.srem(`typing:${roomId}`, socket.userId);
  }
}
```

### Phase 4: Room Handler Implementation

```typescript
// backend/src/socket/handlers/roomHandler.ts
export class RoomHandler {
  constructor(private io: Server) {}

  async handleJoinRoom(socket: AuthenticatedSocket, roomId: string) {
    try {
      // Verify membership
      const isMember = await roomUserRepository.isUserInRoom(roomId, socket.userId);
      if (!isMember) {
        socket.emit('error', { message: 'Not a member of this room' });
        return;
      }

      // Join socket room
      socket.join(`room:${roomId}`);

      // Get room info
      const room = await roomRepository.findById(roomId);
      const onlineUsers = await this.getOnlineUsersInRoom(roomId);

      // Notify user of successful join
      socket.emit('room-joined', {
        roomId,
        room,
        onlineUsers
      });

      // Notify other room members
      socket.to(`room:${roomId}`).emit('user-joined-room', {
        roomId,
        user: {
          id: socket.userId,
          username: socket.username
        }
      });

      // Update user's active room in Redis
      await redis.set(`active_room:${socket.userId}`, roomId, 'EX', 3600);

    } catch (error) {
      console.error('Join room error:', error);
      socket.emit('error', { message: 'Failed to join room' });
    }
  }

  async handleLeaveRoom(socket: AuthenticatedSocket, roomId: string) {
    try {
      // Leave socket room
      socket.leave(`room:${roomId}`);

      // Clear typing indicator
      await redis.srem(`typing:${roomId}`, socket.userId);

      // Notify other room members
      socket.to(`room:${roomId}`).emit('user-left-room', {
        roomId,
        userId: socket.userId
      });

      // Clear active room
      await redis.del(`active_room:${socket.userId}`);

      socket.emit('room-left', { roomId });

    } catch (error) {
      console.error('Leave room error:', error);
    }
  }

  private async getOnlineUsersInRoom(roomId: string): Promise<string[]> {
    const sockets = await this.io.in(`room:${roomId}`).fetchSockets();
    return sockets.map(s => (s as any).userId);
  }
}
```

### Phase 5: Typing Indicator Handler

```typescript
// backend/src/socket/handlers/typingHandler.ts
export class TypingHandler {
  private typingTimers: Map<string, NodeJS.Timeout> = new Map();

  constructor(private io: Server) {}

  async handleTypingStart(socket: AuthenticatedSocket, roomId: string) {
    try {
      // Add to typing set in Redis
      await redis.sadd(`typing:${roomId}`, socket.userId);
      await redis.expire(`typing:${roomId}`, 10);

      // Clear existing timer if any
      const timerKey = `${socket.userId}:${roomId}`;
      if (this.typingTimers.has(timerKey)) {
        clearTimeout(this.typingTimers.get(timerKey)!);
      }

      // Auto-stop typing after 5 seconds
      const timer = setTimeout(() => {
        this.handleTypingStop(socket, roomId);
      }, 5000);
      this.typingTimers.set(timerKey, timer);

      // Notify room members
      socket.to(`room:${roomId}`).emit('user-typing', {
        roomId,
        userId: socket.userId,
        username: socket.username
      });

    } catch (error) {
      console.error('Typing start error:', error);
    }
  }

  async handleTypingStop(socket: AuthenticatedSocket, roomId: string) {
    try {
      // Remove from typing set
      await redis.srem(`typing:${roomId}`, socket.userId);

      // Clear timer
      const timerKey = `${socket.userId}:${roomId}`;
      if (this.typingTimers.has(timerKey)) {
        clearTimeout(this.typingTimers.get(timerKey)!);
        this.typingTimers.delete(timerKey);
      }

      // Notify room members
      socket.to(`room:${roomId}`).emit('user-stopped-typing', {
        roomId,
        userId: socket.userId
      });

    } catch (error) {
      console.error('Typing stop error:', error);
    }
  }
}
```

### Phase 6: Presence Handler Implementation

```typescript
// backend/src/socket/handlers/presenceHandler.ts
export class PresenceHandler {
  constructor(private io: Server) {}

  async handleUserConnect(socket: AuthenticatedSocket) {
    try {
      // Update user online status
      await userRepository.updateOnlineStatus(socket.userId, true);

      // Store socket ID in Redis
      await redis.sadd(`user_sockets:${socket.userId}`, socket.id);
      
      // Set presence with TTL
      await redis.setex(`presence:${socket.userId}`, 300, Date.now().toString());

      // Get user's rooms
      const userRooms = await roomUserRepository.getUserRooms(socket.userId);

      // Notify all user's rooms about online status
      for (const room of userRooms) {
        socket.to(`room:${room.id}`).emit('user-online', {
          userId: socket.userId,
          username: socket.username
        });
      }

      // Subscribe to user's notification channel
      await this.subscribeToNotifications(socket);

    } catch (error) {
      console.error('User connect error:', error);
    }
  }

  async handleUserDisconnect(socket: AuthenticatedSocket) {
    try {
      // Remove socket from Redis
      await redis.srem(`user_sockets:${socket.userId}`, socket.id);

      // Check if user has other active connections
      const socketCount = await redis.scard(`user_sockets:${socket.userId}`);
      
      if (socketCount === 0) {
        // No more connections, mark as offline
        await userRepository.updateOnlineStatus(socket.userId, false);
        await redis.del(`presence:${socket.userId}`);

        // Get user's rooms
        const userRooms = await roomUserRepository.getUserRooms(socket.userId);

        // Notify all rooms about offline status
        for (const room of userRooms) {
          this.io.to(`room:${room.id}`).emit('user-offline', {
            userId: socket.userId,
            lastSeen: new Date()
          });
        }
      }

      // Clear any typing indicators
      const activeRoom = await redis.get(`active_room:${socket.userId}`);
      if (activeRoom) {
        await redis.srem(`typing:${activeRoom}`, socket.userId);
      }

    } catch (error) {
      console.error('User disconnect error:', error);
    }
  }

  private async subscribeToNotifications(socket: AuthenticatedSocket) {
    // Implementation for personal notifications
    // This could include mentions, DMs, etc.
  }
}
```

### Phase 7: Client Connection Setup

```typescript
// Example client connection (for reference)
// frontend/src/services/socketService.ts
import { io, Socket } from 'socket.io-client';

class SocketService {
  private socket: Socket | null = null;

  connect(token: string) {
    this.socket = io(process.env.REACT_APP_SOCKET_URL || 'http://localhost:3001', {
      auth: { token },
      transports: ['websocket', 'polling'],
      reconnection: true,
      reconnectionDelay: 1000,
      reconnectionDelayMax: 5000,
      reconnectionAttempts: 5
    });

    this.setupEventListeners();
  }

  private setupEventListeners() {
    if (!this.socket) return;

    this.socket.on('connect', () => {
      console.log('Connected to socket server');
    });

    this.socket.on('disconnect', (reason) => {
      console.log('Disconnected:', reason);
    });

    this.socket.on('error', (error) => {
      console.error('Socket error:', error);
    });
  }

  joinRoom(roomId: string) {
    this.socket?.emit('join-room', roomId);
  }

  sendMessage(roomId: string, content: string) {
    this.socket?.emit('send-message', { roomId, content });
  }

  startTyping(roomId: string) {
    this.socket?.emit('typing-start', roomId);
  }

  stopTyping(roomId: string) {
    this.socket?.emit('typing-stop', roomId);
  }
}
```

### Phase 8: Error Handling and Monitoring

```typescript
// backend/src/socket/middleware/errorHandler.ts
export const socketErrorHandler = (socket: AuthenticatedSocket) => {
  return (error: Error) => {
    console.error(`Socket error for user ${socket.userId}:`, error);
    
    // Send error to client
    socket.emit('error', {
      message: 'An error occurred',
      code: error.name,
      timestamp: new Date()
    });

    // Log to monitoring service
    if (process.env.NODE_ENV === 'production') {
      // Send to error tracking service
    }
  };
};

// Rate limiting for socket events
export const rateLimiter = (eventName: string, maxRequests: number = 10, windowMs: number = 1000) => {
  const requests = new Map<string, number[]>();

  return (socket: AuthenticatedSocket, next: Function) => {
    const key = `${socket.userId}:${eventName}`;
    const now = Date.now();
    const userRequests = requests.get(key) || [];
    
    // Remove old requests outside window
    const validRequests = userRequests.filter(time => now - time < windowMs);
    
    if (validRequests.length >= maxRequests) {
      socket.emit('error', { message: 'Rate limit exceeded' });
      return;
    }
    
    validRequests.push(now);
    requests.set(key, validRequests);
    next();
  };
};
```

## Performance Optimizations

1. **Redis Pub/Sub**: Use Redis adapter for multi-instance support
2. **Message Batching**: Batch multiple messages for bulk delivery
3. **Compression**: Enable compression for large payloads
4. **Connection Pooling**: Reuse Redis connections
5. **Event Debouncing**: Debounce typing indicators client-side

## Success Metrics

- Message delivery latency < 100ms
- Support 10,000+ concurrent connections
- Typing indicators update in real-time
- Presence status accurate within 5 seconds
- Read receipts delivered immediately
- Horizontal scaling with Redis adapter
- Graceful reconnection handling