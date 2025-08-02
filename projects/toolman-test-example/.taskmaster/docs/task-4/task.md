# Task 4: Chat Room API Implementation

## Overview
Implement a comprehensive REST API for chat room management and messaging functionality. This task establishes the core business logic for room creation, membership management, and message persistence with proper authorization and pagination.

## Technical Architecture

### API Design Principles
- RESTful endpoints following HTTP standards
- Consistent JSON response format
- Proper HTTP status codes
- Pagination for large datasets
- Authorization at controller level
- Input validation on all endpoints

### Technology Stack
- **Framework**: Express.js with TypeScript
- **Validation**: express-validator
- **Authorization**: JWT middleware from Task 3
- **Database**: PostgreSQL with repositories
- **Documentation**: OpenAPI/Swagger

## API Endpoints Specification

### Room Management Endpoints

#### 1. List All Rooms
```
GET /api/rooms
Authorization: Bearer {token}

Query Parameters:
- page (default: 1)
- limit (default: 20)
- search (optional)
- filter: 'public' | 'private' | 'joined'

Response 200:
{
  "rooms": [
    {
      "id": "uuid",
      "name": "General Chat",
      "description": "Public discussion room",
      "isPrivate": false,
      "memberCount": 25,
      "createdBy": "userId",
      "createdAt": "2024-01-01T00:00:00Z",
      "lastActivity": "2024-01-01T12:00:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 45,
    "pages": 3
  }
}
```

#### 2. Create Room
```
POST /api/rooms
Authorization: Bearer {token}

Body:
{
  "name": "Project Discussion",
  "description": "Discuss project updates",
  "isPrivate": false
}

Response 201:
{
  "id": "uuid",
  "name": "Project Discussion",
  "description": "Discuss project updates",
  "isPrivate": false,
  "createdBy": "userId",
  "createdAt": "2024-01-01T00:00:00Z",
  "members": ["userId"]
}
```

#### 3. Get Room Details
```
GET /api/rooms/:id
Authorization: Bearer {token}

Response 200:
{
  "id": "uuid",
  "name": "Project Discussion",
  "description": "Discuss project updates",
  "isPrivate": false,
  "createdBy": {
    "id": "userId",
    "username": "john_doe",
    "avatarUrl": "https://..."
  },
  "memberCount": 12,
  "members": [
    {
      "id": "userId",
      "username": "john_doe",
      "role": "admin",
      "joinedAt": "2024-01-01T00:00:00Z"
    }
  ],
  "createdAt": "2024-01-01T00:00:00Z",
  "lastActivity": "2024-01-01T12:00:00Z"
}
```

#### 4. Update Room
```
PUT /api/rooms/:id
Authorization: Bearer {token}

Body:
{
  "name": "Updated Room Name",
  "description": "Updated description"
}

Response 200:
{
  "id": "uuid",
  "name": "Updated Room Name",
  "description": "Updated description",
  "updatedAt": "2024-01-01T12:00:00Z"
}
```

#### 5. Delete Room
```
DELETE /api/rooms/:id
Authorization: Bearer {token}

Response 204: No Content
```

#### 6. Join Room
```
POST /api/rooms/:id/join
Authorization: Bearer {token}

Response 200:
{
  "message": "Successfully joined room",
  "roomId": "uuid",
  "joinedAt": "2024-01-01T12:00:00Z"
}
```

#### 7. Leave Room
```
POST /api/rooms/:id/leave
Authorization: Bearer {token}

Response 200:
{
  "message": "Successfully left room",
  "roomId": "uuid"
}
```

### Message Management Endpoints

#### 1. Get Message History
```
GET /api/rooms/:id/messages
Authorization: Bearer {token}

Query Parameters:
- before (cursor for pagination)
- limit (default: 50, max: 100)

Response 200:
{
  "messages": [
    {
      "id": "uuid",
      "roomId": "roomId",
      "userId": "userId",
      "user": {
        "id": "userId",
        "username": "john_doe",
        "avatarUrl": "https://..."
      },
      "content": "Hello everyone!",
      "messageType": "text",
      "isEdited": false,
      "createdAt": "2024-01-01T12:00:00Z",
      "readBy": ["userId1", "userId2"]
    }
  ],
  "pagination": {
    "hasMore": true,
    "nextCursor": "messageId"
  }
}
```

#### 2. Send Message
```
POST /api/rooms/:id/messages
Authorization: Bearer {token}

Body:
{
  "content": "Hello everyone!",
  "messageType": "text"
}

Response 201:
{
  "id": "uuid",
  "roomId": "roomId",
  "userId": "userId",
  "content": "Hello everyone!",
  "messageType": "text",
  "createdAt": "2024-01-01T12:00:00Z"
}
```

#### 3. Delete Message
```
DELETE /api/rooms/:id/messages/:messageId
Authorization: Bearer {token}

Response 204: No Content
```

## Implementation Details

### 1. Room Controller

```typescript
// backend/src/controllers/roomController.ts
import { Request, Response } from 'express';
import { RoomRepository } from '../repositories/roomRepository';
import { RoomUserRepository } from '../repositories/roomUserRepository';
import { AuthRequest } from '../middleware/auth';
import { PaginationParams, RoomFilters } from '../types/api';

export class RoomController {
  private roomRepository = new RoomRepository();
  private roomUserRepository = new RoomUserRepository();

  listRooms = async (req: AuthRequest, res: Response): Promise<void> => {
    const { page = 1, limit = 20, search, filter } = req.query as PaginationParams & RoomFilters;
    const userId = req.userId!;

    const rooms = await this.roomRepository.findAll({
      page: Number(page),
      limit: Number(limit),
      search: search as string,
      filter: filter as 'public' | 'private' | 'joined',
      userId
    });

    res.json(rooms);
  };

  createRoom = async (req: AuthRequest, res: Response): Promise<void> => {
    const { name, description, isPrivate = false } = req.body;
    const userId = req.userId!;

    // Create room
    const room = await this.roomRepository.create({
      name,
      description,
      isPrivate,
      createdBy: userId
    });

    // Add creator as admin
    await this.roomUserRepository.addUserToRoom({
      roomId: room.id,
      userId,
      role: 'admin'
    });

    res.status(201).json(room);
  };

  getRoomDetails = async (req: AuthRequest, res: Response): Promise<void> => {
    const { id } = req.params;
    const userId = req.userId!;

    // Check if user is member
    const isMember = await this.roomUserRepository.isUserInRoom(id, userId);
    if (!isMember) {
      throw new ForbiddenError('You must be a member to view room details');
    }

    const room = await this.roomRepository.findByIdWithDetails(id);
    if (!room) {
      throw new NotFoundError('Room not found');
    }

    res.json(room);
  };

  updateRoom = async (req: AuthRequest, res: Response): Promise<void> => {
    const { id } = req.params;
    const { name, description } = req.body;
    const userId = req.userId!;

    // Check if user is admin
    const userRole = await this.roomUserRepository.getUserRole(id, userId);
    if (userRole !== 'admin') {
      throw new ForbiddenError('Only room admins can update room details');
    }

    const updatedRoom = await this.roomRepository.update(id, { name, description });
    res.json(updatedRoom);
  };

  deleteRoom = async (req: AuthRequest, res: Response): Promise<void> => {
    const { id } = req.params;
    const userId = req.userId!;

    // Check if user is room creator
    const room = await this.roomRepository.findById(id);
    if (!room || room.createdBy !== userId) {
      throw new ForbiddenError('Only room creator can delete the room');
    }

    await this.roomRepository.delete(id);
    res.status(204).send();
  };

  joinRoom = async (req: AuthRequest, res: Response): Promise<void> => {
    const { id } = req.params;
    const userId = req.userId!;

    // Check if already member
    const isMember = await this.roomUserRepository.isUserInRoom(id, userId);
    if (isMember) {
      throw new ValidationError('Already a member of this room');
    }

    await this.roomUserRepository.addUserToRoom({
      roomId: id,
      userId,
      role: 'member'
    });

    res.json({
      message: 'Successfully joined room',
      roomId: id,
      joinedAt: new Date().toISOString()
    });
  };

  leaveRoom = async (req: AuthRequest, res: Response): Promise<void> => {
    const { id } = req.params;
    const userId = req.userId!;

    await this.roomUserRepository.removeUserFromRoom(id, userId);

    res.json({
      message: 'Successfully left room',
      roomId: id
    });
  };
}
```

### 2. Message Controller

```typescript
// backend/src/controllers/messageController.ts
import { Request, Response } from 'express';
import { MessageRepository } from '../repositories/messageRepository';
import { RoomUserRepository } from '../repositories/roomUserRepository';
import { AuthRequest } from '../middleware/auth';
import { MessagePaginationParams } from '../types/api';

export class MessageController {
  private messageRepository = new MessageRepository();
  private roomUserRepository = new RoomUserRepository();

  getMessages = async (req: AuthRequest, res: Response): Promise<void> => {
    const { id: roomId } = req.params;
    const { before, limit = 50 } = req.query as MessagePaginationParams;
    const userId = req.userId!;

    // Verify user is room member
    const isMember = await this.roomUserRepository.isUserInRoom(roomId, userId);
    if (!isMember) {
      throw new ForbiddenError('You must be a member to view messages');
    }

    const messages = await this.messageRepository.findByRoomWithPagination({
      roomId,
      before: before as string,
      limit: Math.min(Number(limit), 100)
    });

    // Mark messages as read
    if (messages.messages.length > 0) {
      await this.messageRepository.markMessagesAsRead(
        messages.messages.map(m => m.id),
        userId
      );
    }

    res.json(messages);
  };

  sendMessage = async (req: AuthRequest, res: Response): Promise<void> => {
    const { id: roomId } = req.params;
    const { content, messageType = 'text' } = req.body;
    const userId = req.userId!;

    // Verify user is room member
    const isMember = await this.roomUserRepository.isUserInRoom(roomId, userId);
    if (!isMember) {
      throw new ForbiddenError('You must be a member to send messages');
    }

    const message = await this.messageRepository.create({
      roomId,
      userId,
      content,
      messageType
    });

    // Update room last activity
    await this.roomRepository.updateLastActivity(roomId);

    res.status(201).json(message);
  };

  deleteMessage = async (req: AuthRequest, res: Response): Promise<void> => {
    const { id: roomId, messageId } = req.params;
    const userId = req.userId!;

    // Get message
    const message = await this.messageRepository.findById(messageId);
    if (!message || message.roomId !== roomId) {
      throw new NotFoundError('Message not found');
    }

    // Check if user is message owner or room admin
    const userRole = await this.roomUserRepository.getUserRole(roomId, userId);
    if (message.userId !== userId && userRole !== 'admin') {
      throw new ForbiddenError('You can only delete your own messages');
    }

    await this.messageRepository.delete(messageId);
    res.status(204).send();
  };
}
```

### 3. Authorization Middleware

```typescript
// backend/src/middleware/roomAuth.ts
import { Response, NextFunction } from 'express';
import { AuthRequest } from './auth';
import { RoomUserRepository } from '../repositories/roomUserRepository';

const roomUserRepository = new RoomUserRepository();

export const requireRoomMember = async (
  req: AuthRequest,
  res: Response,
  next: NextFunction
): Promise<void> => {
  const roomId = req.params.id || req.params.roomId;
  const userId = req.userId!;

  const isMember = await roomUserRepository.isUserInRoom(roomId, userId);
  if (!isMember) {
    return res.status(403).json({ error: 'You must be a room member' });
  }

  next();
};

export const requireRoomAdmin = async (
  req: AuthRequest,
  res: Response,
  next: NextFunction
): Promise<void> => {
  const roomId = req.params.id || req.params.roomId;
  const userId = req.userId!;

  const role = await roomUserRepository.getUserRole(roomId, userId);
  if (role !== 'admin') {
    return res.status(403).json({ error: 'Admin access required' });
  }

  next();
};
```

### 4. Input Validation

```typescript
// backend/src/validators/roomValidators.ts
import { body, param, query } from 'express-validator';

export const roomValidators = {
  createRoom: [
    body('name')
      .trim()
      .isLength({ min: 1, max: 100 })
      .withMessage('Room name must be 1-100 characters'),
    body('description')
      .optional()
      .trim()
      .isLength({ max: 500 })
      .withMessage('Description must be less than 500 characters'),
    body('isPrivate')
      .optional()
      .isBoolean()
      .withMessage('isPrivate must be a boolean')
  ],

  updateRoom: [
    param('id').isUUID().withMessage('Invalid room ID'),
    body('name')
      .optional()
      .trim()
      .isLength({ min: 1, max: 100 }),
    body('description')
      .optional()
      .trim()
      .isLength({ max: 500 })
  ],

  listRooms: [
    query('page').optional().isInt({ min: 1 }),
    query('limit').optional().isInt({ min: 1, max: 100 }),
    query('filter')
      .optional()
      .isIn(['public', 'private', 'joined'])
  ],

  sendMessage: [
    param('id').isUUID().withMessage('Invalid room ID'),
    body('content')
      .trim()
      .notEmpty()
      .isLength({ max: 5000 })
      .withMessage('Message content is required and must be less than 5000 characters'),
    body('messageType')
      .optional()
      .isIn(['text', 'image', 'file'])
      .withMessage('Invalid message type')
  ]
};
```

### 5. Repository Implementation

```typescript
// backend/src/repositories/roomRepository.ts
import pool from '../config/database';
import { Room, RoomWithDetails } from '../types/models';
import { PaginatedResponse } from '../types/api';

export class RoomRepository {
  async findAll(params: {
    page: number;
    limit: number;
    search?: string;
    filter?: string;
    userId: string;
  }): Promise<PaginatedResponse<Room>> {
    const offset = (params.page - 1) * params.limit;
    let query = `
      SELECT DISTINCT r.*, COUNT(ru.user_id) as member_count,
        MAX(m.created_at) as last_activity
      FROM rooms r
      LEFT JOIN room_users ru ON r.id = ru.room_id
      LEFT JOIN messages m ON r.id = m.room_id
    `;

    const conditions: string[] = [];
    const values: any[] = [];

    if (params.search) {
      conditions.push(`(r.name ILIKE $${values.length + 1} OR r.description ILIKE $${values.length + 1})`);
      values.push(`%${params.search}%`);
    }

    if (params.filter === 'joined') {
      conditions.push(`r.id IN (SELECT room_id FROM room_users WHERE user_id = $${values.length + 1})`);
      values.push(params.userId);
    } else if (params.filter === 'public') {
      conditions.push('r.is_private = false');
    } else if (params.filter === 'private') {
      conditions.push('r.is_private = true');
    }

    if (conditions.length > 0) {
      query += ' WHERE ' + conditions.join(' AND ');
    }

    query += ` GROUP BY r.id ORDER BY last_activity DESC NULLS LAST LIMIT $${values.length + 1} OFFSET $${values.length + 2}`;
    values.push(params.limit, offset);

    const [rooms, countResult] = await Promise.all([
      pool.query(query, values),
      pool.query(
        `SELECT COUNT(DISTINCT r.id) FROM rooms r ${conditions.length > 0 ? 'WHERE ' + conditions.join(' AND ') : ''}`,
        values.slice(0, -2)
      )
    ]);

    const total = parseInt(countResult.rows[0].count);

    return {
      rooms: rooms.rows.map(this.mapToRoom),
      pagination: {
        page: params.page,
        limit: params.limit,
        total,
        pages: Math.ceil(total / params.limit)
      }
    };
  }

  private mapToRoom(row: any): Room {
    return {
      id: row.id,
      name: row.name,
      description: row.description,
      isPrivate: row.is_private,
      createdBy: row.created_by,
      memberCount: parseInt(row.member_count) || 0,
      lastActivity: row.last_activity,
      createdAt: row.created_at,
      updatedAt: row.updated_at
    };
  }
}
```

## Best Practices Applied

### API Design
- Consistent URL structure
- Proper HTTP methods and status codes
- Pagination for large datasets
- Meaningful error messages
- Request/response validation

### Security
- Authentication required on all endpoints
- Authorization checks for operations
- Input sanitization
- SQL injection prevention
- Rate limiting considerations

### Performance
- Database query optimization
- Efficient pagination with cursors
- Proper indexing on foreign keys
- Connection pooling

### Code Organization
- Separation of concerns
- Repository pattern for data access
- Middleware for cross-cutting concerns
- Type safety with TypeScript