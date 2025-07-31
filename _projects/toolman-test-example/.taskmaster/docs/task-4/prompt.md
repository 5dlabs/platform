# Autonomous AI Agent Prompt for Chat Room API Implementation

## Mission Statement

You are tasked with implementing a comprehensive REST API for a chat room application. Your goal is to create secure, scalable, and well-documented API endpoints that handle room management, messaging, and user participation features. You will research best practices from the Rust ecosystem and Kubernetes deployment patterns to inform your implementation.

## Primary Objectives

1. **Research Phase**
   - Study Rust HTTP server patterns (Actix-web, Rocket, Warp) for API design inspiration
   - Analyze Express.js REST API best practices and security patterns
   - Research Kubernetes API service deployment and ingress configurations
   - Document findings that can be applied to the Node.js/Express implementation

2. **API Development**
   - Implement complete REST API for chat rooms and messages
   - Create all specified endpoints with proper HTTP methods and status codes
   - Ensure comprehensive authorization and authentication
   - Implement efficient pagination for message history
   - Integrate PostgreSQL for data persistence

3. **Security Implementation**
   - Implement JWT-based authentication middleware
   - Create role-based authorization for room operations
   - Add input validation and sanitization
   - Implement rate limiting for API endpoints
   - Protect against common vulnerabilities (SQL injection, XSS, CSRF)

## Detailed Requirements

### Room Management Endpoints

Implement the following room endpoints:

1. **GET /api/v1/rooms**
   - List all public rooms or user's joined rooms
   - Support pagination (page, limit parameters)
   - Support search functionality
   - Return room metadata including member count

2. **POST /api/v1/rooms**
   - Create a new chat room
   - Required fields: name, description
   - Optional fields: isPrivate, maxMembers
   - Automatically add creator as owner
   - Return created room with ID

3. **GET /api/v1/rooms/:id**
   - Get detailed room information
   - Include member list with roles
   - Require membership for private rooms
   - Include last message preview

4. **PUT /api/v1/rooms/:id**
   - Update room details (name, description)
   - Require owner or admin role
   - Validate all input fields
   - Return updated room data

5. **DELETE /api/v1/rooms/:id**
   - Soft delete room (mark as deleted)
   - Require owner role
   - Archive all messages
   - Notify all members

6. **POST /api/v1/rooms/:id/join**
   - Join a public room or accept invitation
   - Check room capacity limits
   - Add user with 'member' role
   - Return membership details

7. **POST /api/v1/rooms/:id/leave**
   - Leave a room
   - Remove user from room members
   - Handle owner leaving (transfer ownership)
   - Return success status

### Message Management Endpoints

1. **GET /api/v1/rooms/:id/messages**
   - Retrieve paginated message history
   - Support cursor-based pagination
   - Optional filters: before, after timestamps
   - Include user details with messages
   - Sort by newest first

2. **POST /api/v1/rooms/:id/messages**
   - Post a new message to room
   - Required field: content
   - Optional field: replyTo (for threading)
   - Validate message length (max 1000 chars)
   - Emit real-time event for connected users

3. **DELETE /api/v1/rooms/:id/messages/:messageId**
   - Delete a message (soft delete)
   - Require message author or room admin
   - Update message to show [deleted]
   - Maintain message ID for thread continuity

### Database Schema Requirements

Design and implement the following tables:

```sql
-- Rooms table
CREATE TABLE rooms (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name VARCHAR(100) NOT NULL,
  description TEXT,
  is_private BOOLEAN DEFAULT false,
  max_members INTEGER DEFAULT 100,
  created_by UUID REFERENCES users(id),
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  deleted_at TIMESTAMP
);

-- Room members table
CREATE TABLE room_users (
  room_id UUID REFERENCES rooms(id),
  user_id UUID REFERENCES users(id),
  role VARCHAR(20) NOT NULL DEFAULT 'member',
  joined_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (room_id, user_id)
);

-- Messages table
CREATE TABLE messages (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  room_id UUID REFERENCES rooms(id),
  user_id UUID REFERENCES users(id),
  content TEXT NOT NULL,
  reply_to UUID REFERENCES messages(id),
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  deleted_at TIMESTAMP
);
```

### Authorization Requirements

1. **Authentication**
   - All endpoints require valid JWT token
   - Extract user ID from token payload
   - Handle token expiration gracefully

2. **Room Roles**
   - Owner: Full control over room
   - Admin: Manage members and messages
   - Member: Post messages and view room

3. **Permission Matrix**
   - View room: Member or higher
   - Post message: Member or higher
   - Delete own message: Member or higher
   - Delete any message: Admin or higher
   - Update room: Admin or higher
   - Delete room: Owner only

### Performance Requirements

1. **Response Times**
   - Room list: < 200ms for 1000 rooms
   - Message history: < 300ms for 100 messages
   - Message posting: < 100ms

2. **Scalability**
   - Support 10,000 concurrent users
   - Handle 1000 messages per second
   - Efficient pagination for millions of messages

3. **Caching Strategy**
   - Cache room metadata (5 minutes)
   - Cache user permissions
   - Use Redis for session management

### Research Tasks

1. **Rust API Patterns**
   - Study Actix-web middleware patterns
   - Analyze Rocket's request guards
   - Review Warp's filter composition
   - Extract applicable patterns for Express.js

2. **Kubernetes Deployment**
   - Research service definitions
   - Study ingress configurations
   - Analyze horizontal pod autoscaling
   - Document deployment best practices

3. **Security Best Practices**
   - OWASP API Security Top 10
   - JWT best practices
   - Rate limiting strategies
   - Input validation patterns

## Implementation Approach

1. **Phase 1: Research and Planning**
   - Conduct all research tasks
   - Document findings and patterns
   - Create API specification document
   - Design database schema

2. **Phase 2: Core Implementation**
   - Set up project structure
   - Implement authentication middleware
   - Create room management endpoints
   - Implement message endpoints

3. **Phase 3: Security and Optimization**
   - Add comprehensive validation
   - Implement rate limiting
   - Add caching layer
   - Optimize database queries

4. **Phase 4: Testing and Documentation**
   - Write unit tests for all endpoints
   - Create integration tests
   - Generate API documentation
   - Prepare deployment configurations

## Success Criteria

Your implementation will be considered complete when:

1. All specified endpoints are functional
2. Authorization is properly implemented
3. Database persistence is working correctly
4. Pagination is efficient and scalable
5. Security measures are in place
6. API is well-documented with examples
7. Tests achieve > 80% code coverage
8. Performance benchmarks are met
9. Kubernetes deployment configs are ready
10. Research findings are documented

## Additional Considerations

- Use environment variables for configuration
- Implement proper error handling and logging
- Follow RESTful conventions consistently
- Consider WebSocket integration for real-time features
- Plan for future API versioning
- Document rate limits in API responses
- Include health check endpoints
- Prepare for horizontal scaling

Begin by researching the specified topics, then proceed with the implementation following the outlined phases. Ensure all code follows industry best practices and is production-ready.