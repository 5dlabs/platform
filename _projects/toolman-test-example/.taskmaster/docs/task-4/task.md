# Task 4: Chat Room API Implementation

## Overview

This task involves developing a comprehensive REST API for chat room functionality, including room management, messaging, and user participation features. The implementation requires secure authorization, efficient data persistence, pagination, and adherence to REST API best practices.

## Architecture Overview

### API Structure
```
/api/v1/
├── rooms/
│   ├── GET    /           (list all rooms)
│   ├── POST   /           (create new room)
│   ├── GET    /:id        (get room details)
│   ├── PUT    /:id        (update room)
│   ├── DELETE /:id        (delete room)
│   ├── POST   /:id/join   (join room)
│   ├── POST   /:id/leave  (leave room)
│   └── messages/
│       ├── GET    /       (get message history)
│       ├── POST   /       (post new message)
│       └── DELETE /:messageId (delete message)
```

## Technical Implementation Guide

### 1. API Module Structure

```typescript
src/
├── api/
│   ├── v1/
│   │   ├── routes/
│   │   │   ├── index.ts
│   │   │   ├── rooms.ts
│   │   │   └── messages.ts
│   │   ├── controllers/
│   │   │   ├── roomController.ts
│   │   │   └── messageController.ts
│   │   ├── middleware/
│   │   │   ├── auth.ts
│   │   │   ├── validation.ts
│   │   │   └── errorHandler.ts
│   │   └── validators/
│   │       ├── roomValidators.ts
│   │       └── messageValidators.ts
│   ├── models/
│   │   ├── Room.ts
│   │   ├── Message.ts
│   │   └── RoomUser.ts
│   └── repositories/
│       ├── roomRepository.ts
│       ├── messageRepository.ts
│       └── roomUserRepository.ts
```

### 2. Room Controller Implementation

```typescript
// src/api/v1/controllers/roomController.ts
import { Request, Response, NextFunction } from 'express';
import { roomRepository } from '../repositories/roomRepository';
import { roomUserRepository } from '../repositories/roomUserRepository';
import { AppError } from '../utils/errors';

export class RoomController {
  async listRooms(req: Request, res: Response, next: NextFunction) {
    try {
      const { page = 1, limit = 20, search } = req.query;
      const userId = req.user.id;
      
      const rooms = await roomRepository.findAll({
        page: Number(page),
        limit: Number(limit),
        search: search as string,
        userId // Optional: filter by user's rooms
      });
      
      res.json({
        data: rooms.data,
        meta: {
          page: rooms.page,
          limit: rooms.limit,
          total: rooms.total,
          totalPages: rooms.totalPages
        }
      });
    } catch (error) {
      next(error);
    }
  }

  async createRoom(req: Request, res: Response, next: NextFunction) {
    try {
      const { name, description, isPrivate = false } = req.body;
      const userId = req.user.id;
      
      // Begin transaction
      const room = await roomRepository.transaction(async (trx) => {
        // Create room
        const newRoom = await roomRepository.create({
          name,
          description,
          isPrivate,
          createdBy: userId
        }, trx);
        
        // Add creator as member
        await roomUserRepository.addUserToRoom(
          newRoom.id, 
          userId, 
          'owner',
          trx
        );
        
        return newRoom;
      });
      
      res.status(201).json({ data: room });
    } catch (error) {
      next(error);
    }
  }

  async getRoom(req: Request, res: Response, next: NextFunction) {
    try {
      const { id } = req.params;
      const userId = req.user.id;
      
      // Check if user has access to room
      const isMember = await roomUserRepository.isMember(id, userId);
      if (!isMember) {
        throw new AppError('Access denied', 403);
      }
      
      const room = await roomRepository.findById(id);
      if (!room) {
        throw new AppError('Room not found', 404);
      }
      
      res.json({ data: room });
    } catch (error) {
      next(error);
    }
  }
}
```

### 3. Message Controller Implementation

```typescript
// src/api/v1/controllers/messageController.ts
export class MessageController {
  async getMessages(req: Request, res: Response, next: NextFunction) {
    try {
      const { id: roomId } = req.params;
      const { page = 1, limit = 50, before, after } = req.query;
      const userId = req.user.id;
      
      // Verify user access
      await this.verifyRoomAccess(roomId, userId);
      
      const messages = await messageRepository.findByRoom(roomId, {
        page: Number(page),
        limit: Number(limit),
        before: before as string,
        after: after as string
      });
      
      res.json({
        data: messages.data,
        meta: {
          page: messages.page,
          limit: messages.limit,
          total: messages.total,
          hasMore: messages.hasMore
        }
      });
    } catch (error) {
      next(error);
    }
  }

  async postMessage(req: Request, res: Response, next: NextFunction) {
    try {
      const { id: roomId } = req.params;
      const { content, replyTo } = req.body;
      const userId = req.user.id;
      
      await this.verifyRoomAccess(roomId, userId);
      
      const message = await messageRepository.create({
        roomId,
        userId,
        content,
        replyTo
      });
      
      // Emit real-time event
      io.to(`room:${roomId}`).emit('message:new', message);
      
      res.status(201).json({ data: message });
    } catch (error) {
      next(error);
    }
  }
}
```

### 4. Authorization Middleware

```typescript
// src/api/v1/middleware/auth.ts
export const requireAuth = async (req: Request, res: Response, next: NextFunction) => {
  try {
    const token = req.headers.authorization?.replace('Bearer ', '');
    if (!token) {
      throw new AppError('Authentication required', 401);
    }
    
    const decoded = jwt.verify(token, process.env.JWT_SECRET);
    req.user = await userRepository.findById(decoded.id);
    
    if (!req.user) {
      throw new AppError('User not found', 401);
    }
    
    next();
  } catch (error) {
    next(new AppError('Invalid token', 401));
  }
};

export const requireRoomRole = (role: string) => {
  return async (req: Request, res: Response, next: NextFunction) => {
    try {
      const roomId = req.params.id;
      const userId = req.user.id;
      
      const userRole = await roomUserRepository.getUserRole(roomId, userId);
      
      if (!hasPermission(userRole, role)) {
        throw new AppError('Insufficient permissions', 403);
      }
      
      next();
    } catch (error) {
      next(error);
    }
  };
};
```

### 5. PostgreSQL Persistence

```typescript
// src/api/v1/repositories/messageRepository.ts
export class MessageRepository {
  async findByRoom(roomId: string, options: PaginationOptions) {
    const query = db('messages')
      .where({ room_id: roomId })
      .orderBy('created_at', 'desc')
      .limit(options.limit)
      .offset((options.page - 1) * options.limit);
    
    if (options.before) {
      query.where('created_at', '<', options.before);
    }
    
    if (options.after) {
      query.where('created_at', '>', options.after);
    }
    
    const [messages, countResult] = await Promise.all([
      query,
      db('messages').where({ room_id: roomId }).count('* as total')
    ]);
    
    return {
      data: messages,
      page: options.page,
      limit: options.limit,
      total: countResult[0].total,
      hasMore: messages.length === options.limit
    };
  }
  
  async create(data: CreateMessageDto) {
    const [message] = await db('messages')
      .insert({
        id: uuid(),
        room_id: data.roomId,
        user_id: data.userId,
        content: data.content,
        reply_to: data.replyTo,
        created_at: new Date()
      })
      .returning('*');
    
    return message;
  }
}
```

### 6. Validation Schemas

```typescript
// src/api/v1/validators/roomValidators.ts
import Joi from 'joi';

export const createRoomSchema = Joi.object({
  name: Joi.string().min(1).max(100).required(),
  description: Joi.string().max(500).optional(),
  isPrivate: Joi.boolean().optional()
});

export const updateRoomSchema = Joi.object({
  name: Joi.string().min(1).max(100).optional(),
  description: Joi.string().max(500).optional()
});

export const paginationSchema = Joi.object({
  page: Joi.number().integer().min(1).optional(),
  limit: Joi.number().integer().min(1).max(100).optional(),
  search: Joi.string().optional()
});
```

### 7. Error Handling

```typescript
// src/api/v1/middleware/errorHandler.ts
export const errorHandler = (err: Error, req: Request, res: Response, next: NextFunction) => {
  if (err instanceof AppError) {
    return res.status(err.statusCode).json({
      error: {
        message: err.message,
        code: err.code
      }
    });
  }
  
  if (err.name === 'ValidationError') {
    return res.status(400).json({
      error: {
        message: 'Validation failed',
        details: err.details
      }
    });
  }
  
  // Log unexpected errors
  logger.error('Unhandled error:', err);
  
  res.status(500).json({
    error: {
      message: 'Internal server error'
    }
  });
};
```

## Security Considerations

### 1. Authentication & Authorization
- JWT token validation on all protected endpoints
- Role-based access control for room operations
- User verification for message operations

### 2. Input Validation
- Strict schema validation using Joi
- SQL injection prevention through parameterized queries
- XSS prevention through input sanitization

### 3. Rate Limiting
```typescript
const rateLimiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100, // limit each IP to 100 requests per windowMs
  message: 'Too many requests'
});

// Message posting specific limit
const messageRateLimit = rateLimit({
  windowMs: 60 * 1000, // 1 minute
  max: 30, // 30 messages per minute
  keyGenerator: (req) => `${req.user.id}:${req.params.id}`
});
```

### 4. CORS Configuration
```typescript
const corsOptions = {
  origin: process.env.ALLOWED_ORIGINS?.split(',') || ['http://localhost:3000'],
  credentials: true,
  optionsSuccessStatus: 200
};
```

## API Versioning Strategy

### 1. URL Path Versioning
- Version in URL path: `/api/v1/rooms`
- Clear version visibility
- Easy to route different versions

### 2. Version Migration
```typescript
// Support multiple versions
app.use('/api/v1', v1Routes);
app.use('/api/v2', v2Routes);

// Deprecation headers
app.use('/api/v1', (req, res, next) => {
  res.setHeader('X-API-Deprecation-Date', '2024-12-31');
  res.setHeader('X-API-Deprecation-Info', 'https://api.example.com/migration');
  next();
});
```

## Performance Optimization

### 1. Database Indexing
```sql
-- Optimize message queries
CREATE INDEX idx_messages_room_created ON messages(room_id, created_at DESC);
CREATE INDEX idx_messages_user ON messages(user_id);

-- Optimize room queries
CREATE INDEX idx_room_users_composite ON room_users(room_id, user_id);
```

### 2. Response Caching
```typescript
// Cache room lists
app.get('/api/v1/rooms', 
  cache('5 minutes'),
  roomController.listRooms
);
```

### 3. Query Optimization
- Use database views for complex joins
- Implement cursor-based pagination for large datasets
- Batch operations where possible

## Testing Strategy

### Unit Tests
- Test controllers with mocked repositories
- Test validators with various inputs
- Test middleware logic independently

### Integration Tests
- Test complete request/response cycles
- Test database transactions
- Test authorization flows

### Performance Tests
- Load testing with k6 or Artillery
- Measure response times under load
- Test pagination with large datasets