# Task 2: Database Setup and Schema Design - Autonomous AI Agent Prompt

## Objective

Set up a robust PostgreSQL database with Redis for a real-time chat application. Design and implement a comprehensive database schema supporting users, chat rooms, messages, and real-time features. Configure both databases for optimal performance, security, and scalability.

## Context

You are implementing the data persistence layer for a real-time chat application. The system requires PostgreSQL for structured data storage and Redis for session management, caching, and real-time communication features. The implementation must support high concurrency, data integrity, and real-time performance requirements.

## Database Requirements

### PostgreSQL Setup
1. **Database Configuration**
   - Create database named `chatapp_db`
   - Set up dedicated database user with appropriate permissions
   - Configure connection pooling with 20 max connections
   - Enable SSL for production environments
   - Set up automatic timestamp updates

2. **Schema Design Requirements**
   
   **Users Table:**
   - `id` (UUID, primary key)
   - `email` (unique, indexed)
   - `password_hash` (bcrypt hashed)
   - `username` (unique, indexed)
   - `avatar_url` (optional)
   - `is_active` (boolean, default true)
   - `last_seen` (timestamp)
   - `created_at`, `updated_at` (timestamps)

   **Rooms Table:**
   - `id` (UUID, primary key)
   - `name` (required)
   - `description` (optional)
   - `type` (enum: public, private, direct)
   - `created_by` (foreign key to users)
   - `is_active` (boolean, default true)
   - `created_at`, `updated_at` (timestamps)

   **Messages Table:**
   - `id` (UUID, primary key)
   - `room_id` (foreign key, indexed)
   - `user_id` (foreign key, indexed)
   - `content` (text, required)
   - `type` (enum: text, image, file, system)
   - `is_edited` (boolean)
   - `edited_at` (timestamp)
   - `created_at` (timestamp, indexed)

   **Room Users Table:**
   - Composite primary key: `(room_id, user_id)`
   - `role` (enum: owner, admin, member)
   - `joined_at` (timestamp)
   - `last_read_at` (timestamp)

   **Message Read Status Table:**
   - Composite primary key: `(message_id, user_id)`
   - `read_at` (timestamp)

3. **Database Constraints and Indexes**
   - Foreign key constraints with CASCADE deletes
   - Unique constraints on email and username
   - Composite indexes for query performance
   - Timestamp triggers for automatic updates

### Redis Configuration
1. **Connection Setup**
   - Configure Redis client with retry strategy
   - Set up separate pub/sub connections
   - Implement connection error handling

2. **Data Structures**
   - Session storage with TTL
   - JWT refresh token management
   - User presence tracking
   - Typing indicators
   - Socket ID mappings
   - Room presence data

3. **Key Naming Conventions**
   - `session:{userId}` - User sessions
   - `refresh:{token}` - Refresh tokens
   - `presence:{userId}` - User presence
   - `room:{roomId}:presence` - Room presence
   - `typing:{roomId}` - Typing indicators
   - `socket:{socketId}` - Socket mappings

## Implementation Requirements

### Migration System
1. Create migration runner with:
   - Migration tracking table
   - Transaction-based execution
   - Rollback capability
   - Sequential execution
   - Error handling

2. Initial migration files:
   - `001_create_users.sql`
   - `002_create_rooms.sql`
   - `003_create_messages.sql`
   - `004_create_room_users.sql`
   - `005_create_message_read_status.sql`
   - `006_create_indexes.sql`
   - `007_create_triggers.sql`

### TypeScript Models
1. Define interfaces for all entities:
   - User, CreateUserDto, UpdateUserDto
   - Room, RoomWithUsers, CreateRoomDto
   - Message, MessageWithUser, CreateMessageDto
   - RoomUser, MessageReadStatus

2. Implement type-safe data access

### Repository Pattern
1. Create BaseRepository with common operations:
   - `findById`, `create`, `update`, `delete`
   - Query builders with proper escaping
   - Transaction support

2. Specialized repositories:
   - UserRepository (auth methods, profile updates)
   - RoomRepository (user management, listing)
   - MessageRepository (pagination, read status)

### Connection Management
1. Database pool configuration:
   - Min/max connections
   - Idle timeouts
   - Statement timeouts
   - Health checks

2. Redis connection management:
   - Automatic reconnection
   - Error recovery
   - Pub/sub handling

## Performance Considerations

1. **Query Optimization**
   - Use prepared statements
   - Implement query result caching
   - Batch operations where possible
   - Optimize N+1 query problems

2. **Indexing Strategy**
   - Index foreign keys
   - Composite indexes for common queries
   - Partial indexes for filtered data
   - Monitor query performance

3. **Caching Strategy**
   - Cache user profiles in Redis
   - Cache room membership
   - Implement cache invalidation
   - Set appropriate TTLs

## Security Requirements

1. **Access Control**
   - Use least privilege principle
   - Separate read/write permissions
   - Implement row-level security
   - Audit sensitive operations

2. **Data Protection**
   - Hash passwords with bcrypt (10+ rounds)
   - Sanitize all inputs
   - Use parameterized queries
   - Implement rate limiting

3. **Connection Security**
   - Use SSL/TLS in production
   - Secure connection strings
   - Implement connection timeouts
   - Monitor failed connections

## Testing Requirements

1. **Unit Tests**
   - Test all repository methods
   - Test model validations
   - Test database utilities
   - Mock database connections

2. **Integration Tests**
   - Test database connectivity
   - Test migration runner
   - Test Redis operations
   - Test transaction handling

3. **Performance Tests**
   - Load test connection pooling
   - Test query performance
   - Measure Redis latency
   - Test concurrent operations

## Deliverables

1. **Database Scripts**
   - Schema creation SQL
   - Migration files
   - Seed data scripts
   - Backup/restore procedures

2. **Source Code**
   - Database configuration
   - TypeScript models
   - Repository implementations
   - Migration runner

3. **Documentation**
   - Database schema diagrams
   - API documentation
   - Setup instructions
   - Troubleshooting guide

## Success Criteria

- PostgreSQL and Redis successfully installed and configured
- All tables created with proper constraints and indexes
- Migration system functional and tested
- Repository pattern implemented with full CRUD operations
- Connection pooling configured and optimized
- All tests passing with >90% coverage
- Performance benchmarks meet requirements
- Security best practices implemented

## Additional Notes

- Follow TypeScript best practices and strict typing
- Implement comprehensive error handling
- Use environment variables for configuration
- Document all database design decisions
- Consider future scalability requirements
- Implement monitoring and logging