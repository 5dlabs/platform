# Task 5: Real-time Communication with Socket.io

## Overview
Implement WebSocket-based real-time communication using Socket.io to enable instant messaging, typing indicators, user presence tracking, and read receipts. This task transforms the chat application from a REST-only system to a fully real-time communication platform.

## Technical Architecture

### Real-time Stack
- **WebSocket Server**: Socket.io with TypeScript
- **Authentication**: JWT-based socket authentication
- **Scaling**: Redis adapter for multi-instance support
- **State Management**: Redis for presence and typing indicators
- **Protocol**: Socket.io v4 with acknowledgments

### Event Architecture
```
Client Events (Emit)          Server Events (Broadcast)
├── join-room          →      ├── user-joined
├── leave-room         →      ├── user-left
├── send-message       →      ├── new-message
├── typing-start       →      ├── user-typing
├── typing-stop        →      ├── user-stopped-typing
├── mark-read          →      ├── message-read
└── update-presence    →      └── presence-update
```

## Implementation Details

### 1. Socket.io Server Setup

```typescript
// backend/src/socket/socketServer.ts
import { Server as HttpServer } from 'http';
import { Server, Socket } from 'socket.io';
import { createAdapter } from '@socket.io/redis-adapter';
import { pubClient, subClient } from '../config/redis';
import { TokenService } from '../services/tokenService';
import { SocketHandlers } from './handlers';
import { AuthenticatedSocket } from '../types/socket';

export class SocketServer {
  private io: Server;
  private tokenService = new TokenService();
  private handlers = new SocketHandlers();

  constructor(httpServer: HttpServer) {
    this.io = new Server(httpServer, {
      cors: {
        origin: process.env.FRONTEND_URL,
        credentials: true
      },
      transports: ['websocket', 'polling'],
      pingTimeout: 60000,
      pingInterval: 25000
    });

    this.setupRedisAdapter();
    this.setupAuthentication();
    this.setupEventHandlers();
  }

  private setupRedisAdapter(): void {
    this.io.adapter(createAdapter(pubClient, subClient));
    console.log('Redis adapter configured for Socket.io');
  }

  private setupAuthentication(): void {
    this.io.use(async (socket: Socket, next) => {
      try {
        const token = socket.handshake.auth.token;
        if (!token) {
          return next(new Error('No token provided'));
        }

        const payload = this.tokenService.verifyAccessToken(token);
        const user = await userRepository.findById(payload.userId);
        
        if (!user) {
          return next(new Error('User not found'));
        }

        (socket as AuthenticatedSocket).userId = user.id;
        (socket as AuthenticatedSocket).user = user;
        
        next();
      } catch (error) {
        next(new Error('Authentication failed'));
      }
    });
  }

  private setupEventHandlers(): void {
    this.io.on('connection', (socket: AuthenticatedSocket) => {
      console.log(`User connected: ${socket.userId}`);
      
      // Update user presence
      this.handlers.handleUserConnect(socket);

      // Room management
      socket.on('join-room', (roomId: string, callback) => 
        this.handlers.handleJoinRoom(socket, roomId, callback)
      );
      
      socket.on('leave-room', (roomId: string, callback) =>
        this.handlers.handleLeaveRoom(socket, roomId, callback)
      );

      // Messaging
      socket.on('send-message', (data, callback) =>
        this.handlers.handleSendMessage(socket, data, callback)
      );

      // Typing indicators
      socket.on('typing-start', (roomId: string) =>
        this.handlers.handleTypingStart(socket, roomId)
      );
      
      socket.on('typing-stop', (roomId: string) =>
        this.handlers.handleTypingStop(socket, roomId)
      );

      // Read receipts
      socket.on('mark-read', (messageIds: string[], callback) =>
        this.handlers.handleMarkRead(socket, messageIds, callback)
      );

      // Disconnection
      socket.on('disconnect', () =>
        this.handlers.handleDisconnect(socket)
      );

      // Error handling
      socket.on('error', (error) => {
        console.error(`Socket error for user ${socket.userId}:`, error);
      });
    });
  }

  public getIO(): Server {
    return this.io;
  }
}
```

### 2. Socket Event Handlers

```typescript
// backend/src/socket/handlers/index.ts
import { AuthenticatedSocket } from '../../types/socket';
import { RoomService } from '../../services/roomService';
import { MessageService } from '../../services/messageService';
import { PresenceService } from '../../services/presenceService';
import { TypingService } from '../../services/typingService';

export class SocketHandlers {
  private roomService = new RoomService();
  private messageService = new MessageService();
  private presenceService = new PresenceService();
  private typingService = new TypingService();

  async handleUserConnect(socket: AuthenticatedSocket): Promise<void> {
    // Update user online status
    await this.presenceService.setUserOnline(socket.userId);
    
    // Join user's rooms
    const userRooms = await this.roomService.getUserRooms(socket.userId);
    for (const room of userRooms) {
      socket.join(`room:${room.id}`);
      
      // Notify room members
      socket.to(`room:${room.id}`).emit('user-online', {
        userId: socket.userId,
        username: socket.user.username
      });
    }

    // Send current online users
    const onlineUsers = await this.presenceService.getOnlineUsers();
    socket.emit('online-users', onlineUsers);
  }

  async handleJoinRoom(
    socket: AuthenticatedSocket,
    roomId: string,
    callback: (response: SocketResponse) => void
  ): Promise<void> {
    try {
      // Verify user can join room
      const canJoin = await this.roomService.canUserJoinRoom(socket.userId, roomId);
      if (!canJoin) {
        return callback({ success: false, error: 'Cannot join this room' });
      }

      // Join socket room
      socket.join(`room:${roomId}`);

      // Add user to room in database
      await this.roomService.addUserToRoom(roomId, socket.userId);

      // Get room details
      const room = await this.roomService.getRoomWithMembers(roomId);

      // Notify other room members
      socket.to(`room:${roomId}`).emit('user-joined', {
        roomId,
        user: {
          id: socket.userId,
          username: socket.user.username,
          avatarUrl: socket.user.avatarUrl
        }
      });

      callback({ success: true, data: room });
    } catch (error) {
      console.error('Join room error:', error);
      callback({ success: false, error: 'Failed to join room' });
    }
  }

  async handleSendMessage(
    socket: AuthenticatedSocket,
    data: { roomId: string; content: string; messageType?: string },
    callback: (response: SocketResponse) => void
  ): Promise<void> {
    try {
      // Verify user is room member
      const isMember = await this.roomService.isUserInRoom(socket.userId, data.roomId);
      if (!isMember) {
        return callback({ success: false, error: 'Not a room member' });
      }

      // Create message
      const message = await this.messageService.createMessage({
        roomId: data.roomId,
        userId: socket.userId,
        content: data.content,
        messageType: data.messageType || 'text'
      });

      // Populate user data
      const messageWithUser = {
        ...message,
        user: {
          id: socket.userId,
          username: socket.user.username,
          avatarUrl: socket.user.avatarUrl
        }
      };

      // Broadcast to room members
      socket.to(`room:${data.roomId}`).emit('new-message', messageWithUser);

      // Clear typing indicator
      await this.typingService.removeTypingUser(data.roomId, socket.userId);
      socket.to(`room:${data.roomId}`).emit('user-stopped-typing', {
        roomId: data.roomId,
        userId: socket.userId
      });

      callback({ success: true, data: messageWithUser });
    } catch (error) {
      console.error('Send message error:', error);
      callback({ success: false, error: 'Failed to send message' });
    }
  }

  async handleTypingStart(socket: AuthenticatedSocket, roomId: string): Promise<void> {
    // Add user to typing list
    await this.typingService.addTypingUser(roomId, socket.userId);

    // Broadcast to room members
    socket.to(`room:${roomId}`).emit('user-typing', {
      roomId,
      userId: socket.userId,
      username: socket.user.username
    });

    // Auto-remove after timeout
    setTimeout(async () => {
      await this.handleTypingStop(socket, roomId);
    }, 5000);
  }

  async handleTypingStop(socket: AuthenticatedSocket, roomId: string): Promise<void> {
    // Remove user from typing list
    await this.typingService.removeTypingUser(roomId, socket.userId);

    // Broadcast to room members
    socket.to(`room:${roomId}`).emit('user-stopped-typing', {
      roomId,
      userId: socket.userId
    });
  }

  async handleMarkRead(
    socket: AuthenticatedSocket,
    messageIds: string[],
    callback: (response: SocketResponse) => void
  ): Promise<void> {
    try {
      // Mark messages as read
      const readReceipts = await this.messageService.markMessagesAsRead(
        messageIds,
        socket.userId
      );

      // Get room IDs for broadcasting
      const roomIds = [...new Set(readReceipts.map(r => r.roomId))];

      // Broadcast read receipts to relevant rooms
      for (const roomId of roomIds) {
        const roomReceipts = readReceipts.filter(r => r.roomId === roomId);
        socket.to(`room:${roomId}`).emit('messages-read', {
          userId: socket.userId,
          messageIds: roomReceipts.map(r => r.messageId)
        });
      }

      callback({ success: true, data: readReceipts });
    } catch (error) {
      console.error('Mark read error:', error);
      callback({ success: false, error: 'Failed to mark messages as read' });
    }
  }

  async handleDisconnect(socket: AuthenticatedSocket): Promise<void> {
    console.log(`User disconnected: ${socket.userId}`);

    // Update user offline status
    await this.presenceService.setUserOffline(socket.userId);

    // Get user's rooms
    const userRooms = await this.roomService.getUserRooms(socket.userId);

    // Clear typing indicators
    for (const room of userRooms) {
      await this.typingService.removeTypingUser(room.id, socket.userId);
      socket.to(`room:${room.id}`).emit('user-stopped-typing', {
        roomId: room.id,
        userId: socket.userId
      });

      // Notify room members of offline status
      socket.to(`room:${room.id}`).emit('user-offline', {
        userId: socket.userId
      });
    }
  }
}
```

### 3. Typing Indicator Service

```typescript
// backend/src/services/typingService.ts
import { redis } from '../config/redis';

export class TypingService {
  private readonly TYPING_PREFIX = 'typing:';
  private readonly TYPING_TTL = 5; // seconds

  async addTypingUser(roomId: string, userId: string): Promise<void> {
    const key = `${this.TYPING_PREFIX}${roomId}`;
    await redis.sadd(key, userId);
    await redis.expire(key, this.TYPING_TTL);
  }

  async removeTypingUser(roomId: string, userId: string): Promise<void> {
    const key = `${this.TYPING_PREFIX}${roomId}`;
    await redis.srem(key, userId);
  }

  async getTypingUsers(roomId: string): Promise<string[]> {
    const key = `${this.TYPING_PREFIX}${roomId}`;
    return redis.smembers(key);
  }

  async clearRoomTyping(roomId: string): Promise<void> {
    const key = `${this.TYPING_PREFIX}${roomId}`;
    await redis.del(key);
  }
}
```

### 4. Presence Service

```typescript
// backend/src/services/presenceService.ts
import { redis } from '../config/redis';
import { UserRepository } from '../repositories/userRepository';

export class PresenceService {
  private readonly PRESENCE_PREFIX = 'presence:';
  private readonly ONLINE_USERS_SET = 'online_users';
  private readonly PRESENCE_TTL = 300; // 5 minutes
  private userRepository = new UserRepository();

  async setUserOnline(userId: string): Promise<void> {
    // Update Redis
    const key = `${this.PRESENCE_PREFIX}${userId}`;
    await redis.setex(key, this.PRESENCE_TTL, JSON.stringify({
      status: 'online',
      lastSeen: new Date().toISOString()
    }));
    
    await redis.sadd(this.ONLINE_USERS_SET, userId);

    // Update database
    await this.userRepository.updateOnlineStatus(userId, true);
  }

  async setUserOffline(userId: string): Promise<void> {
    // Update Redis
    const key = `${this.PRESENCE_PREFIX}${userId}`;
    await redis.del(key);
    await redis.srem(this.ONLINE_USERS_SET, userId);

    // Update database
    await this.userRepository.updateOnlineStatus(userId, false);
  }

  async refreshUserPresence(userId: string): Promise<void> {
    const key = `${this.PRESENCE_PREFIX}${userId}`;
    await redis.expire(key, this.PRESENCE_TTL);
  }

  async getOnlineUsers(): Promise<string[]> {
    return redis.smembers(this.ONLINE_USERS_SET);
  }

  async isUserOnline(userId: string): Promise<boolean> {
    return redis.sismember(this.ONLINE_USERS_SET, userId);
  }

  async getUserPresence(userId: string): Promise<any> {
    const key = `${this.PRESENCE_PREFIX}${userId}`;
    const data = await redis.get(key);
    return data ? JSON.parse(data) : null;
  }
}
```

### 5. Client Socket Integration

```typescript
// frontend/src/hooks/useSocket.ts
import { useEffect, useRef, useCallback } from 'react';
import io, { Socket } from 'socket.io-client';
import { useAuth } from './useAuth';
import { useAppDispatch } from '../store/hooks';
import { addMessage, updateTypingUsers, updateUserPresence } from '../store/slices/chatSlice';

export const useSocket = () => {
  const socketRef = useRef<Socket | null>(null);
  const { token } = useAuth();
  const dispatch = useAppDispatch();

  useEffect(() => {
    if (!token) return;

    // Initialize socket connection
    socketRef.current = io(process.env.REACT_APP_API_URL, {
      auth: { token },
      transports: ['websocket'],
      reconnection: true,
      reconnectionDelay: 1000,
      reconnectionAttempts: 5
    });

    const socket = socketRef.current;

    // Connection events
    socket.on('connect', () => {
      console.log('Connected to server');
    });

    socket.on('disconnect', (reason) => {
      console.log('Disconnected:', reason);
    });

    socket.on('connect_error', (error) => {
      console.error('Connection error:', error.message);
    });

    // Message events
    socket.on('new-message', (message) => {
      dispatch(addMessage(message));
    });

    // Typing events
    socket.on('user-typing', ({ roomId, userId, username }) => {
      dispatch(updateTypingUsers({ roomId, userId, username, isTyping: true }));
    });

    socket.on('user-stopped-typing', ({ roomId, userId }) => {
      dispatch(updateTypingUsers({ roomId, userId, isTyping: false }));
    });

    // Presence events
    socket.on('user-online', ({ userId }) => {
      dispatch(updateUserPresence({ userId, isOnline: true }));
    });

    socket.on('user-offline', ({ userId }) => {
      dispatch(updateUserPresence({ userId, isOnline: false }));
    });

    // Read receipt events
    socket.on('messages-read', ({ userId, messageIds }) => {
      dispatch(markMessagesAsRead({ userId, messageIds }));
    });

    return () => {
      socket.disconnect();
    };
  }, [token, dispatch]);

  const joinRoom = useCallback((roomId: string) => {
    socketRef.current?.emit('join-room', roomId, (response) => {
      if (!response.success) {
        console.error('Failed to join room:', response.error);
      }
    });
  }, []);

  const sendMessage = useCallback((roomId: string, content: string) => {
    socketRef.current?.emit('send-message', { roomId, content }, (response) => {
      if (!response.success) {
        console.error('Failed to send message:', response.error);
      }
    });
  }, []);

  const startTyping = useCallback((roomId: string) => {
    socketRef.current?.emit('typing-start', roomId);
  }, []);

  const stopTyping = useCallback((roomId: string) => {
    socketRef.current?.emit('typing-stop', roomId);
  }, []);

  const markAsRead = useCallback((messageIds: string[]) => {
    socketRef.current?.emit('mark-read', messageIds, (response) => {
      if (!response.success) {
        console.error('Failed to mark as read:', response.error);
      }
    });
  }, []);

  return {
    socket: socketRef.current,
    joinRoom,
    sendMessage,
    startTyping,
    stopTyping,
    markAsRead
  };
};
```

## Performance Optimizations

### Message Delivery Optimization
- Direct room broadcasting without database round-trip
- Message creation happens asynchronously
- Redis pub/sub for multi-instance communication
- Batch read receipt updates

### Connection Management
- Automatic reconnection with exponential backoff
- Connection state recovery
- Heartbeat mechanism (ping/pong)
- Graceful disconnection handling

### Scaling Considerations
- Redis adapter for horizontal scaling
- Sticky sessions for load balancing
- Room-based event isolation
- Efficient presence tracking

## Security Implementation

### Authentication
- JWT validation on connection
- Token refresh handling
- User context attached to socket
- Automatic disconnection on auth failure

### Authorization
- Room membership verification
- Message sending authorization
- Read receipt validation
- Admin action verification

### Input Validation
- Message content sanitization
- Room ID validation
- Event payload validation
- Rate limiting per socket

## Error Handling

### Client-Side
- Automatic reconnection
- Offline message queue
- Connection state UI feedback
- Error event handling

### Server-Side
- Graceful error responses
- Event acknowledgments
- Error logging
- Fallback mechanisms

## Monitoring and Debugging

### Metrics to Track
- Active connections count
- Message delivery latency
- Reconnection frequency
- Error rates by event type
- Redis adapter performance

### Debug Tools
- Socket.io admin UI
- Custom debug events
- Connection logging
- Performance profiling