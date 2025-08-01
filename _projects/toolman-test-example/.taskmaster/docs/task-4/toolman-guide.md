# Toolman Guide for Task 4: Chat Room API Implementation

## Overview

This guide provides detailed instructions for using the selected tools to implement Task 4, which focuses on building a comprehensive REST API for chat room management with message history persistence, incorporating best practices from Rust ecosystem and Kubernetes deployment patterns.

## Core Tools

### 1. **brave_web_search** (Remote)
**Purpose**: Research Express.js REST API best practices and security patterns

**When to Use**: 
- At the beginning for API design patterns
- For pagination best practices
- For REST API security patterns
- For authorization middleware patterns

**How to Use**:
```
# Search for Express.js best practices
brave_web_search "Express.js REST API best practices 2024 security"
brave_web_search "Node.js API pagination patterns PostgreSQL"
brave_web_search "Express middleware authorization patterns JWT"
brave_web_search "REST API error handling Express.js"
```

**Parameters**:
- `query`: Search query string
- `count`: Number of results (max 20)
- `freshness`: Filter by recency

### 2. **query_rust_docs** (Remote)
**Purpose**: Research Rust HTTP server patterns and API design

**When to Use**: 
- For understanding efficient API patterns
- For learning about error handling patterns
- For performance optimization techniques

**How to Use**:
```
# Query Rust API patterns
query_rust_docs {
  "crate": "axum",
  "query": "REST API design patterns error handling",
  "max_results": 10
}

query_rust_docs {
  "crate": "actix-web",
  "query": "middleware authorization patterns",
  "max_results": 10
}
```

**Parameters**:
- `crate`: Rust crate to search (axum, actix-web, rocket)
- `query`: Semantic search query
- `max_results`: Number of results

### 3. **resolveProviderDocID** & **getProviderDocs** (Remote - terraform)
**Purpose**: Research Kubernetes API service deployment and ingress configurations

**When to Use**: 
- For Kubernetes service configuration patterns
- For ingress controller setup
- For API gateway patterns

**How to Use**:
```
# First resolve the provider doc ID
resolveProviderDocID {
  "serviceSlug": "kubernetes",
  "providerDataType": "service"
}

# Then get the documentation
getProviderDocs {
  "providerDocID": <resolved-doc-id>
}
```

**Parameters**:
- `serviceSlug`: Service to search for
- `providerDataType`: Type of documentation
- `providerDocID`: ID from resolve step

### 4. **create_directory** (Local - filesystem)
**Purpose**: Create API directory structure for routes, controllers, and middleware

**When to Use**: 
- To organize API components
- To separate room and message logic
- For middleware organization

**How to Use**:
```
# Create API structure
create_directory /chat-application/backend/src/api
create_directory /chat-application/backend/src/api/routes
create_directory /chat-application/backend/src/api/controllers
create_directory /chat-application/backend/src/api/middleware
create_directory /chat-application/backend/src/api/validators
create_directory /chat-application/backend/src/api/services
```

**Parameters**:
- `path`: Directory path to create

### 5. **write_file** (Local - filesystem)
**Purpose**: Create API routes, controllers, and middleware files

**When to Use**: 
- To implement room and message endpoints
- To create authorization middleware
- To implement pagination utilities
- To create validation schemas

**How to Use**:
```
# Create room controller
write_file /chat-application/backend/src/api/controllers/roomController.ts <controller-content>

# Create message controller
write_file /chat-application/backend/src/api/controllers/messageController.ts <controller-content>

# Create room routes
write_file /chat-application/backend/src/api/routes/roomRoutes.ts <routes-content>

# Create authorization middleware
write_file /chat-application/backend/src/api/middleware/roomAuth.ts <auth-content>
```

**Parameters**:
- `path`: File path to write
- `content`: Complete file content

## Supporting Tools

### **read_file** (Local - filesystem)
**Purpose**: Review existing models and authentication middleware

**When to Use**: 
- To check database models from Task 2
- To review authentication middleware from Task 3
- To understand existing patterns

### **edit_file** (Local - filesystem)
**Purpose**: Update server configuration and dependencies

**When to Use**: 
- To add new routes to main server
- To update package.json with API dependencies
- To modify middleware stack

### **list_directory** (Local - filesystem)
**Purpose**: Verify API structure creation

**When to Use**: 
- After creating directories
- To confirm file organization

## Implementation Flow

1. **Research Phase** (Start with remote tools)
   - Use `brave_web_search` for Express.js REST patterns
   - Use `query_rust_docs` for efficient API design patterns
   - Use terraform tools for Kubernetes deployment patterns

2. **Structure Creation Phase**
   - Use `create_directory` to build API structure
   - Organize by routes, controllers, middleware, services

3. **Room API Implementation**
   - Use `write_file` to create roomController.ts with 7 endpoints:
     - GET /api/rooms (list with pagination)
     - POST /api/rooms (create with validation)
     - GET /api/rooms/:id (with authorization)
     - PUT /api/rooms/:id (owner only)
     - DELETE /api/rooms/:id (owner only)
     - POST /api/rooms/:id/join
     - POST /api/rooms/:id/leave

4. **Message API Implementation**
   - Use `write_file` to create messageController.ts with 3 endpoints:
     - GET /api/rooms/:id/messages (paginated history)
     - POST /api/rooms/:id/messages (real-time persist)
     - DELETE /api/rooms/:id/messages/:messageId (author only)

5. **Middleware Implementation**
   - Create room authorization middleware
   - Implement request validation middleware
   - Add error handling middleware
   - Create pagination middleware

6. **Integration Phase**
   - Use `edit_file` to update main server
   - Add API routes with proper middleware chain
   - Configure error handlers

## Best Practices

1. **RESTful Design**: Follow REST conventions for URLs and HTTP methods
2. **Authorization**: Check user permissions for each operation
3. **Validation**: Validate all inputs using schemas
4. **Error Handling**: Consistent error response format
5. **Pagination**: Implement cursor-based pagination for scalability
6. **Security**: Sanitize inputs, prevent SQL injection

## Task-Specific Implementation Patterns

### Room Controller Pattern
```typescript
// roomController.ts
export const createRoom = async (req, res) => {
  try {
    const { name, description } = req.body;
    const userId = req.userId; // from auth middleware
    
    const room = await roomService.create({
      name,
      description,
      created_by: userId
    });
    
    res.status(201).json({ room });
  } catch (error) {
    next(error);
  }
};
```

### Pagination Pattern
```typescript
// Cursor-based pagination
export const getMessages = async (req, res) => {
  const { id } = req.params;
  const { cursor, limit = 50 } = req.query;
  
  const messages = await messageService.getByRoom(id, {
    cursor,
    limit: Math.min(limit, 100)
  });
  
  res.json({
    messages,
    nextCursor: messages[messages.length - 1]?.id
  });
};
```

### Authorization Middleware Pattern
```typescript
// roomAuth.ts
export const isRoomMember = async (req, res, next) => {
  const { id } = req.params;
  const userId = req.userId;
  
  const isMember = await roomService.checkMembership(id, userId);
  if (!isMember) {
    return res.status(403).json({ error: 'Not a room member' });
  }
  
  next();
};
```

## Troubleshooting

- **Route Conflicts**: Define specific routes before generic ones
- **Async Errors**: Use async error handling middleware
- **Database Queries**: Optimize with proper indexes
- **Pagination**: Handle edge cases (empty results, invalid cursor)
- **Authorization**: Cache membership checks for performance

## Testing Approach

1. **Unit Tests**:
   - Test individual controller methods
   - Test validation schemas
   - Test pagination logic

2. **Integration Tests**:
   - Test complete API flows
   - Test authorization scenarios
   - Test error handling

3. **Performance Tests**:
   - Test pagination with large datasets
   - Test concurrent room operations
   - Test message throughput