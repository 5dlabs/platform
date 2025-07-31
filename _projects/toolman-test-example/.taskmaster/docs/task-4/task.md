# Task 4: Chat Room API Implementation

## Overview
Develop a comprehensive REST API for chat room management, including room creation, membership management, and message handling with pagination. Incorporate best practices from Rust HTTP server patterns and prepare for Kubernetes deployment.

## Technical Implementation Guide

### Phase 1: Room Controller Implementation

#### Room Management Endpoints
```typescript
// backend/src/controllers/roomController.ts
import { Request, Response } from 'express';
import { AuthRequest } from '../middleware/auth';
import { RoomRepository } from '../repositories/roomRepository';
import { RoomUserRepository } from '../repositories/roomUserRepository';

// GET /api/rooms - List all rooms
export const listRooms = async (req: AuthRequest, res: Response) => {
  try {
    const { page = 1, limit = 20, search } = req.query;
    
    const rooms = await roomRepository.findAll({
      page: Number(page),
      limit: Number(limit),
      search: search as string,
      userId: req.userId // For showing joined status
    });
    
    res.json({
      data: rooms.data,
      pagination: {
        page: Number(page),
        limit: Number(limit),
        total: rooms.total,
        totalPages: Math.ceil(rooms.total / Number(limit))
      }
    });
  } catch (error) {
    console.error('List rooms error:', error);
    res.status(500).json({ error: 'Failed to fetch rooms' });
  }
};

// POST /api/rooms - Create new room
export const createRoom = async (req: AuthRequest, res: Response) => {
  try {
    const { name, description, isPrivate = false } = req.body;
    
    // Validation
    if (!name || name.trim().length < 3) {
      return res.status(400).json({ 
        error: 'Room name must be at least 3 characters' 
      });
    }
    
    // Create room
    const room = await roomRepository.create({
      name: name.trim(),
      description: description?.trim(),
      isPrivate,
      createdBy: req.userId!
    });
    
    // Add creator as room member
    await roomUserRepository.addUserToRoom(room.id, req.userId!, 'admin');
    
    res.status(201).json({
      id: room.id,
      name: room.name,
      description: room.description,
      isPrivate: room.isPrivate,
      createdBy: room.createdBy,
      createdAt: room.createdAt,
      memberCount: 1,
      isJoined: true,
      role: 'admin'
    });
  } catch (error) {
    console.error('Create room error:', error);
    res.status(500).json({ error: 'Failed to create room' });
  }
};

// GET /api/rooms/:id - Get room details
export const getRoomDetails = async (req: AuthRequest, res: Response) => {
  try {
    const { id } = req.params;
    
    const room = await roomRepository.findById(id);
    if (!room) {
      return res.status(404).json({ error: 'Room not found' });
    }
    
    // Check if user is member for private rooms
    if (room.isPrivate) {
      const isMember = await roomUserRepository.isUserInRoom(id, req.userId!);
      if (!isMember) {
        return res.status(403).json({ error: 'Access denied' });
      }
    }
    
    // Get member info
    const memberInfo = await roomUserRepository.getUserRoomInfo(id, req.userId!);
    const memberCount = await roomUserRepository.getRoomMemberCount(id);
    
    res.json({
      ...room,
      memberCount,
      isJoined: !!memberInfo,
      role: memberInfo?.role,
      joinedAt: memberInfo?.joinedAt
    });
  } catch (error) {
    console.error('Get room details error:', error);
    res.status(500).json({ error: 'Failed to fetch room details' });
  }
};
```

#### Room Membership Endpoints
```typescript
// POST /api/rooms/:id/join - Join room
export const joinRoom = async (req: AuthRequest, res: Response) => {
  try {
    const { id } = req.params;
    
    const room = await roomRepository.findById(id);
    if (!room) {
      return res.status(404).json({ error: 'Room not found' });
    }
    
    // Check if already member
    const existingMembership = await roomUserRepository.getUserRoomInfo(id, req.userId!);
    if (existingMembership) {
      return res.status(409).json({ error: 'Already a member of this room' });
    }
    
    // Add user to room
    await roomUserRepository.addUserToRoom(id, req.userId!, 'member');
    
    res.json({ 
      message: 'Successfully joined room',
      roomId: id
    });
  } catch (error) {
    console.error('Join room error:', error);
    res.status(500).json({ error: 'Failed to join room' });
  }
};

// POST /api/rooms/:id/leave - Leave room
export const leaveRoom = async (req: AuthRequest, res: Response) => {
  try {
    const { id } = req.params;
    
    // Check membership
    const membership = await roomUserRepository.getUserRoomInfo(id, req.userId!);
    if (!membership) {
      return res.status(404).json({ error: 'Not a member of this room' });
    }
    
    // Don't allow admin to leave if they're the only admin
    if (membership.role === 'admin') {
      const adminCount = await roomUserRepository.getRoomAdminCount(id);
      if (adminCount === 1) {
        return res.status(400).json({ 
          error: 'Cannot leave room as the only admin. Transfer ownership first.' 
        });
      }
    }
    
    // Remove user from room
    await roomUserRepository.removeUserFromRoom(id, req.userId!);
    
    res.json({ 
      message: 'Successfully left room',
      roomId: id
    });
  } catch (error) {
    console.error('Leave room error:', error);
    res.status(500).json({ error: 'Failed to leave room' });
  }
};
```

### Phase 2: Message Controller Implementation

#### Message Endpoints
```typescript
// backend/src/controllers/messageController.ts
import { MessageRepository } from '../repositories/messageRepository';

// GET /api/rooms/:id/messages - Get message history
export const getMessages = async (req: AuthRequest, res: Response) => {
  try {
    const { id: roomId } = req.params;
    const { 
      limit = 50, 
      before, // Message ID to get messages before
      after   // Message ID to get messages after
    } = req.query;
    
    // Check room membership
    const isMember = await roomUserRepository.isUserInRoom(roomId, req.userId!);
    if (!isMember) {
      return res.status(403).json({ error: 'Not a member of this room' });
    }
    
    // Fetch messages with pagination
    const messages = await messageRepository.findByRoom(roomId, {
      limit: Math.min(Number(limit), 100), // Max 100 messages
      before: before as string,
      after: after as string
    });
    
    // Mark messages as read
    if (messages.length > 0) {
      await messageRepository.markAsRead(
        roomId, 
        req.userId!, 
        messages[0].id
      );
    }
    
    res.json({
      data: messages,
      pagination: {
        limit: Number(limit),
        hasMore: messages.length === Number(limit)
      }
    });
  } catch (error) {
    console.error('Get messages error:', error);
    res.status(500).json({ error: 'Failed to fetch messages' });
  }
};

// POST /api/rooms/:id/messages - Send message
export const sendMessage = async (req: AuthRequest, res: Response) => {
  try {
    const { id: roomId } = req.params;
    const { content, messageType = 'text' } = req.body;
    
    // Validation
    if (!content || content.trim().length === 0) {
      return res.status(400).json({ error: 'Message content required' });
    }
    
    if (content.length > 1000) {
      return res.status(400).json({ 
        error: 'Message too long (max 1000 characters)' 
      });
    }
    
    // Check membership
    const isMember = await roomUserRepository.isUserInRoom(roomId, req.userId!);
    if (!isMember) {
      return res.status(403).json({ error: 'Not a member of this room' });
    }
    
    // Create message
    const message = await messageRepository.create({
      roomId,
      userId: req.userId!,
      content: content.trim(),
      messageType
    });
    
    // Get user info for response
    const user = await userRepository.findById(req.userId!);
    
    res.status(201).json({
      ...message,
      user: {
        id: user!.id,
        username: user!.username,
        avatarUrl: user!.avatarUrl
      }
    });
  } catch (error) {
    console.error('Send message error:', error);
    res.status(500).json({ error: 'Failed to send message' });
  }
};

// DELETE /api/rooms/:id/messages/:messageId - Delete message
export const deleteMessage = async (req: AuthRequest, res: Response) => {
  try {
    const { id: roomId, messageId } = req.params;
    
    // Get message
    const message = await messageRepository.findById(messageId);
    if (!message || message.roomId !== roomId) {
      return res.status(404).json({ error: 'Message not found' });
    }
    
    // Check permissions (owner or room admin)
    const canDelete = message.userId === req.userId!;
    if (!canDelete) {
      const membership = await roomUserRepository.getUserRoomInfo(roomId, req.userId!);
      if (!membership || membership.role !== 'admin') {
        return res.status(403).json({ error: 'Permission denied' });
      }
    }
    
    // Soft delete (keep record but mark as deleted)
    await messageRepository.softDelete(messageId);
    
    res.json({ message: 'Message deleted successfully' });
  } catch (error) {
    console.error('Delete message error:', error);
    res.status(500).json({ error: 'Failed to delete message' });
  }
};
```

### Phase 3: Repository Implementation

#### Room Repository
```typescript
// backend/src/repositories/roomRepository.ts
export class RoomRepository {
  async findAll(options: {
    page: number;
    limit: number;
    search?: string;
    userId?: string;
  }) {
    const offset = (options.page - 1) * options.limit;
    
    let query = `
      SELECT 
        r.*,
        COUNT(DISTINCT ru.user_id) as member_count,
        ${options.userId ? 'ru_user.user_id IS NOT NULL as is_joined,' : 'false as is_joined,'}
        ${options.userId ? 'ru_user.role' : 'NULL as role'}
      FROM rooms r
      LEFT JOIN room_users ru ON r.id = ru.room_id
      ${options.userId ? 'LEFT JOIN room_users ru_user ON r.id = ru_user.room_id AND ru_user.user_id = $3' : ''}
    `;
    
    const params: any[] = [options.limit, offset];
    if (options.userId) params.push(options.userId);
    
    if (options.search) {
      query += ` WHERE r.name ILIKE $${params.length + 1}`;
      params.push(`%${options.search}%`);
    }
    
    query += `
      GROUP BY r.id${options.userId ? ', ru_user.user_id, ru_user.role' : ''}
      ORDER BY r.created_at DESC
      LIMIT $1 OFFSET $2
    `;
    
    const result = await pool.query(query, params);
    
    // Get total count
    const countQuery = options.search
      ? 'SELECT COUNT(*) FROM rooms WHERE name ILIKE $1'
      : 'SELECT COUNT(*) FROM rooms';
    
    const countResult = await pool.query(
      countQuery,
      options.search ? [`%${options.search}%`] : []
    );
    
    return {
      data: result.rows,
      total: parseInt(countResult.rows[0].count)
    };
  }
}
```

#### Message Repository with Pagination
```typescript
// backend/src/repositories/messageRepository.ts
export class MessageRepository {
  async findByRoom(roomId: string, options: {
    limit: number;
    before?: string;
    after?: string;
  }) {
    let query = `
      SELECT 
        m.*,
        u.username,
        u.avatar_url
      FROM messages m
      JOIN users u ON m.user_id = u.id
      WHERE m.room_id = $1
    `;
    
    const params: any[] = [roomId];
    
    // Cursor-based pagination
    if (options.before) {
      query += ` AND m.created_at < (SELECT created_at FROM messages WHERE id = $${params.length + 1})`;
      params.push(options.before);
    } else if (options.after) {
      query += ` AND m.created_at > (SELECT created_at FROM messages WHERE id = $${params.length + 1})`;
      params.push(options.after);
    }
    
    query += `
      ORDER BY m.created_at ${options.after ? 'ASC' : 'DESC'}
      LIMIT $${params.length + 1}
    `;
    params.push(options.limit);
    
    const result = await pool.query(query, params);
    
    // Reverse if fetching older messages (default behavior)
    const messages = options.after ? result.rows : result.rows.reverse();
    
    return messages.map(row => ({
      id: row.id,
      roomId: row.room_id,
      userId: row.user_id,
      content: row.content,
      messageType: row.message_type,
      isEdited: row.is_edited,
      createdAt: row.created_at,
      user: {
        id: row.user_id,
        username: row.username,
        avatarUrl: row.avatar_url
      }
    }));
  }
}
```

### Phase 4: API Route Configuration

```typescript
// backend/src/routes/rooms.ts
import { Router } from 'express';
import { authenticate } from '../middleware/auth';
import * as roomController from '../controllers/roomController';
import * as messageController from '../controllers/messageController';

const router = Router();

// All room routes require authentication
router.use(authenticate);

// Room management
router.get('/', roomController.listRooms);
router.post('/', roomController.createRoom);
router.get('/:id', roomController.getRoomDetails);
router.put('/:id', roomController.updateRoom);
router.delete('/:id', roomController.deleteRoom);

// Room membership
router.post('/:id/join', roomController.joinRoom);
router.post('/:id/leave', roomController.leaveRoom);
router.get('/:id/members', roomController.listMembers);

// Messages
router.get('/:id/messages', messageController.getMessages);
router.post('/:id/messages', messageController.sendMessage);
router.delete('/:id/messages/:messageId', messageController.deleteMessage);

export default router;
```

### Phase 5: Rust-Inspired API Patterns

#### Error Handling Pattern
```typescript
// Inspired by Rust's Result type
type ApiResult<T> = {
  ok: true;
  data: T;
} | {
  ok: false;
  error: string;
  code?: string;
};

// Middleware for consistent error responses
export const errorHandler = (
  err: Error,
  req: Request,
  res: Response,
  next: NextFunction
) => {
  const response: ApiResult<never> = {
    ok: false,
    error: err.message || 'Internal server error',
    code: err.name
  };
  
  res.status(500).json(response);
};
```

#### Request Validation Middleware
```typescript
// Inspired by Rust's strong typing
import { z } from 'zod';

export const validateRequest = (schema: z.ZodSchema) => {
  return (req: Request, res: Response, next: NextFunction) => {
    try {
      schema.parse({
        body: req.body,
        query: req.query,
        params: req.params
      });
      next();
    } catch (error) {
      if (error instanceof z.ZodError) {
        res.status(400).json({
          ok: false,
          error: 'Validation failed',
          details: error.errors
        });
      } else {
        next(error);
      }
    }
  };
};

// Usage
const createRoomSchema = z.object({
  body: z.object({
    name: z.string().min(3).max(100),
    description: z.string().max(500).optional(),
    isPrivate: z.boolean().optional()
  })
});

router.post('/', validateRequest(createRoomSchema), roomController.createRoom);
```

### Phase 6: Kubernetes Deployment Configuration

#### API Service Deployment
```yaml
# kubernetes/backend-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: chat-api
  labels:
    app: chat-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: chat-api
  template:
    metadata:
      labels:
        app: chat-api
    spec:
      containers:
      - name: api
        image: chat-app/backend:latest
        ports:
        - containerPort: 3001
        env:
        - name: NODE_ENV
          value: "production"
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: chat-secrets
              key: database-url
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3001
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 3001
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: chat-api-service
spec:
  selector:
    app: chat-api
  ports:
  - protocol: TCP
    port: 80
    targetPort: 3001
  type: ClusterIP
```

#### Ingress Configuration
```yaml
# kubernetes/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: chat-app-ingress
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
    nginx.ingress.kubernetes.io/rate-limit: "100"
spec:
  rules:
  - host: api.chatapp.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: chat-api-service
            port:
              number: 80
```

## Success Metrics

- All API endpoints respond within 200ms
- Pagination handles 1000+ messages efficiently
- Authorization checks prevent unauthorized access
- Message delivery completes in <100ms
- API handles 1000+ concurrent requests
- Kubernetes deployment scales automatically
- Zero downtime deployments achieved