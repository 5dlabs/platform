# Task 4: Chat Room API Implementation - Toolman Usage Guide

## Overview
This guide explains how to use the selected Toolman tools to implement a comprehensive REST API for chat room management. The tools cover file operations, research capabilities for best practices, and Kubernetes deployment preparation.

## Core Tools

### 1. brave_web_search
**Purpose**: Research REST API best practices and patterns
**When to use**: 
- Before implementing API structure
- When designing pagination strategies
- To find error handling patterns
- For performance optimization techniques

**How to use**:
```json
{
  "tool": "brave_web_search",
  "query": "Express.js REST API pagination best practices 2024",
  "freshness": "year"
}
```

**Key research topics**:
- "Express.js REST API folder structure best practices"
- "Cursor-based vs offset pagination performance"
- "REST API error response standards"
- "Node.js API rate limiting strategies"
- "REST API authorization patterns"

### 2. query_rust_docs
**Purpose**: Research Rust HTTP server patterns for design inspiration
**When to use**:
- To understand error handling patterns from Rust
- For API response structure ideas
- To learn about type-safe API design
- For performance optimization patterns

**How to use**:
```json
{
  "tool": "query_rust_docs",
  "crate": "actix-web",
  "query": "REST API error handling patterns"
}
```

**Research areas**:
- Actix-web middleware patterns
- Error propagation in Rust APIs
- Response serialization strategies
- Type-safe route definitions

### 3. getAPIResources & describeResource
**Purpose**: Research Kubernetes API deployment patterns
**When to use**:
- Understanding Kubernetes service definitions
- Learning about ingress configurations
- Researching health check patterns
- Planning for horizontal scaling

**How to use**:
```json
{
  "tool": "getAPIResources",
  "apiVersion": "apps/v1",
  "namespaceScoped": true
}
```

```json
{
  "tool": "describeResource",
  "kind": "Service",
  "name": "api-service"
}
```

### 4. create_directory
**Purpose**: Organize API code structure
**When to use**:
- Setting up controller directories
- Creating route organization
- Structuring validators and middleware

**How to use**:
```json
{
  "tool": "create_directory",
  "path": "/chat-application/backend/src/controllers"
}
```

**Directory structure**:
```
/backend/src/
├── controllers/
│   ├── roomController.ts
│   └── messageController.ts
├── routes/
│   ├── roomRoutes.ts
│   └── messageRoutes.ts
├── validators/
│   ├── roomValidators.ts
│   └── messageValidators.ts
└── middleware/
    └── roomAuth.ts
```

### 5. write_file
**Purpose**: Create API implementation files
**When to use**:
- Writing controller logic
- Creating route definitions
- Implementing validators
- Setting up middleware

**How to use**:
```json
{
  "tool": "write_file",
  "path": "/chat-application/backend/src/controllers/roomController.ts",
  "content": "// Room controller implementation"
}
```

## Implementation Flow

### Phase 1: Research Best Practices (20 minutes)
1. **Express.js patterns**:
   ```json
   {
     "tool": "brave_web_search",
     "query": "Express.js REST API controller pattern TypeScript 2024"
   }
   ```

2. **Rust API patterns**:
   ```json
   {
     "tool": "query_rust_docs",
     "crate": "actix-web",
     "query": "REST API response handling"
   }
   ```

3. **Kubernetes deployment**:
   ```json
   {
     "tool": "brave_web_search",
     "query": "Kubernetes REST API deployment health checks"
   }
   ```

### Phase 2: Create API Structure (15 minutes)
1. **Create directories**:
   ```json
   {
     "tool": "create_directory",
     "path": "/chat-application/backend/src/routes"
   }
   ```

2. **Write room controller**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/controllers/roomController.ts",
     "content": "// Complete room controller with all endpoints"
   }
   ```

3. **Create room routes**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/routes/roomRoutes.ts",
     "content": "// Room route definitions"
   }
   ```

### Phase 3: Implement Room Endpoints (25 minutes)
1. **Room management methods**:
   - listRooms with pagination
   - createRoom with validation
   - getRoomDetails with auth check
   - updateRoom with admin check
   - deleteRoom with creator check
   - joinRoom/leaveRoom

2. **Add authorization middleware**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/middleware/roomAuth.ts",
     "content": "// Room authorization checks"
   }
   ```

### Phase 4: Implement Message Endpoints (20 minutes)
1. **Message controller**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/controllers/messageController.ts",
     "content": "// Message handling logic"
   }
   ```

2. **Cursor pagination**:
   - Implement before parameter
   - Add hasMore flag
   - Set reasonable limits

### Phase 5: Repository Updates (15 minutes)
1. **Update existing repositories**:
   ```json
   {
     "tool": "read_file",
     "path": "/chat-application/backend/src/repositories/roomRepository.ts"
   }
   ```

2. **Add new methods**:
   ```json
   {
     "tool": "edit_file",
     "path": "/chat-application/backend/src/repositories/roomRepository.ts",
     "old_string": "export class RoomRepository {",
     "new_string": "export class RoomRepository {\n  // New pagination methods"
   }
   ```

## Best Practices

### API Design Patterns
```typescript
// Consistent response format
interface ApiResponse<T> {
  data?: T;
  error?: ApiError;
  meta?: ResponseMeta;
}

// Pagination interface
interface PaginatedResponse<T> {
  items: T[];
  pagination: {
    page?: number;
    limit?: number;
    total?: number;
    hasMore?: boolean;
    cursor?: string;
  };
}
```

### Error Handling Pattern
```typescript
// Centralized error handling
class ApiError extends Error {
  constructor(
    public statusCode: number,
    public code: string,
    message: string
  ) {
    super(message);
  }
}

// Usage
throw new ApiError(403, 'FORBIDDEN', 'Only room admins can update');
```

### Authorization Pattern
```typescript
// Middleware composition
router.put('/rooms/:id',
  authenticate,           // From Task 3
  validateRequest('updateRoom'),
  requireRoomAdmin,      // Room-specific
  roomController.updateRoom
);
```

## Common Patterns

### Research → Design → Implement
```javascript
// 1. Research best practice
const patterns = await brave_web_search("REST API versioning strategies");

// 2. Design based on findings
const apiStructure = designFromPatterns(patterns);

// 3. Implement
await write_file("routes/v1/index.ts", apiStructure);
```

### Incremental Development
```javascript
// 1. Create basic endpoint
await write_file("controllers/roomController.ts", basicImplementation);

// 2. Add validation
await edit_file("controllers/roomController.ts", 
  "createRoom = async",
  "createRoom = validateInput(roomSchema)(async"
);

// 3. Add authorization
await edit_file("routes/roomRoutes.ts",
  "router.post('/rooms'",
  "router.post('/rooms', authenticate, requirePermission('create_room')"
);
```

## Rust-Inspired Patterns

### Result Type Pattern
```typescript
// Inspired by Rust's Result<T, E>
type ApiResult<T> = {
  ok: true;
  data: T;
} | {
  ok: false;
  error: ApiError;
};
```

### Strong Typing
```typescript
// Type-safe route parameters
interface RoomParams {
  id: string;
}

interface MessageQuery {
  before?: string;
  limit?: number;
}

// Usage
app.get<RoomParams, {}, {}, MessageQuery>('/rooms/:id/messages', ...);
```

## Kubernetes Preparation

### Health Check Endpoints
```typescript
// Add to main app
app.get('/health', (req, res) => {
  res.json({ status: 'healthy' });
});

app.get('/ready', async (req, res) => {
  const dbHealthy = await checkDatabase();
  const redisHealthy = await checkRedis();
  
  if (dbHealthy && redisHealthy) {
    res.json({ status: 'ready' });
  } else {
    res.status(503).json({ status: 'not ready' });
  }
});
```

## Troubleshooting

### Issue: Pagination performance slow
**Solution**: Add database indexes, use cursor-based pagination for large datasets

### Issue: Authorization checks repetitive
**Solution**: Create reusable middleware functions

### Issue: N+1 queries in room list
**Solution**: Use joins to fetch member counts in single query

### Issue: Message order inconsistent
**Solution**: Always order by created_at DESC with consistent timezone

## Task Completion Checklist
- [ ] All room endpoints implemented
- [ ] All message endpoints implemented
- [ ] Authorization working correctly
- [ ] Pagination implemented efficiently
- [ ] Input validation on all endpoints
- [ ] Error handling consistent
- [ ] Repository methods optimized
- [ ] API documentation complete
- [ ] Tests cover all endpoints
- [ ] Performance targets met

This systematic approach ensures a well-structured, performant, and secure REST API ready for production use.