# Autonomous Agent Prompt: Database Setup and Schema Design

You are tasked with setting up the database infrastructure for a real-time chat application using PostgreSQL and Redis.

## Objective
Design and implement a robust database schema with PostgreSQL for persistent data and Redis for session management and real-time features.

## Detailed Requirements

### 1. PostgreSQL Database Setup
- Create a PostgreSQL database named 'chatdb'
- Enable UUID extension for primary key generation
- Configure connection pooling for optimal performance
- Set up proper environment variables for database connection

### 2. Database Schema Implementation
Create the following tables with appropriate constraints and relationships:

#### Users Table
- id (UUID, primary key)
- email (unique, not null)
- username (unique, not null)
- password_hash (not null)
- avatar_url (optional)
- is_online (boolean, default false)
- last_seen (timestamp)
- created_at, updated_at (timestamps)

#### Rooms Table
- id (UUID, primary key)
- name (not null)
- description (optional)
- is_private (boolean, default false)
- created_by (foreign key to users)
- created_at, updated_at (timestamps)

#### Messages Table
- id (UUID, primary key)
- room_id (foreign key to rooms, cascade delete)
- user_id (foreign key to users, cascade delete)
- content (not null)
- message_type (default 'text')
- is_edited (boolean, default false)
- created_at, updated_at (timestamps)

#### Room Users Junction Table
- room_id (foreign key to rooms)
- user_id (foreign key to users)
- joined_at (timestamp)
- role (default 'member')
- last_read_message_id (foreign key to messages)
- Composite primary key (room_id, user_id)

#### Message Read Receipts Table
- message_id (foreign key to messages)
- user_id (foreign key to users)
- read_at (timestamp)
- Composite primary key (message_id, user_id)

### 3. Performance Optimization
Create indexes for:
- User email and username (for authentication)
- Room and message foreign keys (for joins)
- Created_at timestamps (for pagination)
- Online status (for presence queries)

### 4. Redis Configuration
Set up Redis for:
- **Session Storage**: Store JWT refresh tokens with TTL
- **Typing Indicators**: Track users currently typing in rooms
- **Online Presence**: Monitor user online status
- **Socket Connections**: Map user IDs to socket IDs

Redis key patterns:
- Sessions: `session:{sessionId}`
- Typing: `typing:{roomId}` (Set with 5s TTL)
- Presence: `presence:{userId}` (5min TTL)
- Sockets: `socket:{userId}` (Set of socket IDs)

### 5. Database Migrations
- Set up a migration system (e.g., node-pg-migrate)
- Create initial migration with all tables
- Include rollback procedures
- Add migration scripts to package.json

### 6. Data Access Layer
Implement TypeScript interfaces and repositories:

```typescript
interface User {
  id: string;
  email: string;
  username: string;
  passwordHash: string;
  avatarUrl?: string;
  isOnline: boolean;
  lastSeen?: Date;
  createdAt: Date;
  updatedAt: Date;
}
```

Create repository classes with methods:
- UserRepository: create, findById, findByEmail, updateOnlineStatus
- RoomRepository: create, findById, getUserRooms, addUser
- MessageRepository: create, findByRoom, markAsRead

### 7. Connection Management
- Implement database connection pool
- Configure Redis client with retry logic
- Handle connection errors gracefully
- Set up health check endpoints

### 8. Docker Integration
Update docker-compose.yml to include:
- PostgreSQL service with persistent volume
- Redis service with AOF persistence
- Proper networking between services
- Environment variable configuration

## Expected Deliverables

1. SQL schema files with all table definitions
2. Database migration files
3. TypeScript model interfaces
4. Repository implementation files
5. Database connection configuration
6. Redis client setup
7. Updated docker-compose.yml
8. Environment variables documentation

## Quality Criteria

- All foreign key constraints properly defined
- Indexes optimize common query patterns
- No N+1 query problems in repository methods
- Connection pooling prevents exhaustion
- Redis keys follow consistent naming patterns
- Migrations are reversible
- TypeScript types match database schema exactly

## Testing Requirements

1. Unit tests for all repository methods
2. Integration tests for database operations
3. Performance tests for concurrent connections
4. Redis operation latency tests
5. Migration up/down verification

## Verification Steps

1. Run database migrations: `npm run migrate:up`
2. Verify all tables and constraints created
3. Test repository methods with sample data
4. Confirm Redis operations complete in <10ms
5. Validate connection pool handling
6. Check indexes improve query performance
7. Ensure Docker services start correctly

Begin by creating the database schema, then implement the data access layer with proper TypeScript types and repository patterns.