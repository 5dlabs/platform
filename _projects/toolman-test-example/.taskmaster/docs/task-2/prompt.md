# Task 2: Database Setup and Schema Design - AI Agent Prompt

You are a database architect tasked with implementing a robust data layer for a real-time chat application. Your goal is to set up PostgreSQL for persistent storage and Redis for session management and real-time features, with proper schema design and optimization.

## Primary Objectives

1. **Design Database Schema**: Create well-structured tables for users, rooms, messages, and relationships with proper constraints and indexes.

2. **Configure PostgreSQL**: Set up connection pooling, create migration scripts, and implement the repository pattern for data access.

3. **Integrate Redis**: Configure Redis for session storage, implement pub/sub for real-time features, and set up caching strategies.

4. **Implement Data Models**: Create TypeScript interfaces and repository classes with proper error handling and validation.

5. **Optimize Performance**: Add appropriate indexes, configure connection pools, and implement caching where beneficial.

## Required Actions

### Phase 1: Database Design (20 minutes)
1. Create SQL schema for all tables:
   - users table with authentication fields
   - rooms table for chat rooms
   - messages table for chat history
   - room_users junction table
   - message_read_receipts table

2. Design indexes for optimal query performance
3. Add foreign key constraints and cascading rules
4. Plan for future scalability needs

### Phase 2: PostgreSQL Setup (25 minutes)
1. Create database configuration file with connection pooling
2. Set up migration system using Knex.js or similar
3. Write migration files for each table
4. Create seed data for development
5. Configure environment variables

### Phase 3: Data Model Implementation (30 minutes)
1. Create TypeScript interfaces for all entities
2. Implement repository pattern for each model:
   - UserRepository
   - RoomRepository
   - MessageRepository
3. Add validation and error handling
4. Implement transaction support for complex operations

### Phase 4: Redis Integration (20 minutes)
1. Configure Redis connection with ioredis
2. Implement session management service
3. Set up pub/sub for real-time features:
   - Typing indicators
   - User presence
   - Message delivery notifications
4. Configure Redis persistence settings

### Phase 5: Testing & Documentation (15 minutes)
1. Write unit tests for repositories
2. Create integration tests for database operations
3. Document all environment variables
4. Create database setup instructions

## Implementation Details

### PostgreSQL Schema Requirements
```sql
-- Users table must include:
- UUID primary key
- Unique email and username
- Password hash (never plain text)
- Timestamps for created/updated
- Online status and last seen

-- Messages must support:
- Different message types (text, image, file)
- Edit tracking
- Soft deletes
- Efficient pagination
```

### Redis Data Structures
```javascript
// Session storage
session:{sessionId} -> {userId, data, expires}

// Typing indicators
typing:{roomId} -> Set of userIds

// User presence
presence:{userId} -> {isOnline, lastSeen}

// Unread counts
unread:{userId}:{roomId} -> count
```

### Repository Pattern Example
```typescript
class MessageRepository {
  async create(message: CreateMessageDto): Promise<Message>
  async findByRoom(roomId: string, limit: number, offset: number): Promise<Message[]>
  async markAsRead(messageId: string, userId: string): Promise<void>
  async update(messageId: string, content: string): Promise<Message>
}
```

## Quality Requirements

### Performance Targets
- Query response time < 50ms for common operations
- Support 10,000+ concurrent connections
- Message delivery latency < 100ms
- Zero data loss for critical operations

### Security Requirements
- Parameterized queries to prevent SQL injection
- Password hashing with bcrypt (min 10 rounds)
- Environment variables for sensitive data
- Connection encryption for production

### Code Standards
- TypeScript strict mode enabled
- Comprehensive error handling
- Transaction support for data integrity
- Proper resource cleanup (connection closing)

## Docker Configuration

Add to docker-compose.yml:
```yaml
postgres:
  image: postgres:14-alpine
  environment:
    POSTGRES_DB: chatapp
    POSTGRES_USER: chatuser
    POSTGRES_PASSWORD: ${DB_PASSWORD}
  volumes:
    - postgres_data:/var/lib/postgresql/data

redis:
  image: redis:6-alpine
  command: redis-server --requirepass ${REDIS_PASSWORD}
  volumes:
    - redis_data:/data
```

## Testing Checklist

Before marking complete, ensure:
- [ ] All tables created with proper constraints
- [ ] Indexes added for performance
- [ ] Migration system working
- [ ] Repository methods tested
- [ ] Redis pub/sub functioning
- [ ] Connection pooling configured
- [ ] Environment variables documented
- [ ] Docker services running correctly

## Common Pitfalls to Avoid
- Don't forget to add indexes on foreign keys
- Always use prepared statements
- Handle connection failures gracefully
- Implement proper transaction rollback
- Set appropriate pool sizes
- Use Redis TTL for session expiry
- Test concurrent access scenarios

Execute this task methodically, ensuring data integrity and performance at each step. The resulting database layer should be production-ready and scalable.