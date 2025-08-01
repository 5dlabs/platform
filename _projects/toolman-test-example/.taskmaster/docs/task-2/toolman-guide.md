# Toolman Guide for Task 2: Database Setup and Schema Design

## Overview

This guide provides detailed instructions for using the selected tools to implement Task 2, which focuses on designing and implementing PostgreSQL database schemas and Redis configuration for a real-time chat application with JWT authentication support.

## Core Tools

### 1. **create_directory** (Local - filesystem)
**Purpose**: Create the database-related directory structure for schemas, migrations, and models

**When to Use**: 
- At the beginning to establish database directory structure
- When organizing migration files by version
- For separating models and repository layers

**How to Use**:
```
# Create database structure directories
create_directory /chat-application/backend/src/database
create_directory /chat-application/backend/src/database/migrations
create_directory /chat-application/backend/src/database/models
create_directory /chat-application/backend/src/database/repositories
create_directory /chat-application/backend/src/database/config
create_directory /chat-application/backend/src/database/schemas
```

**Parameters**:
- `path`: The directory path to create (absolute or relative)

### 2. **write_file** (Local - filesystem)
**Purpose**: Create database schema definitions, migration scripts, model files, and configuration

**When to Use**: 
- To create SQL schema definition files
- To write database migration scripts
- To implement TypeScript models and interfaces
- To create database configuration files

**How to Use**:
```
# Create PostgreSQL schema file
write_file /chat-application/backend/src/database/schemas/chat_schema.sql <schema-content>

# Create migration files
write_file /chat-application/backend/src/database/migrations/001_create_users_table.sql <migration-content>

# Create TypeScript models
write_file /chat-application/backend/src/database/models/User.ts <model-content>

# Create Redis configuration
write_file /chat-application/backend/src/database/config/redis.config.ts <redis-config>
```

**Parameters**:
- `path`: File path to write to
- `content`: The complete file content

### 3. **read_file** (Local - filesystem)
**Purpose**: Review existing files and verify created content

**When to Use**: 
- To check existing database configurations
- To review created schemas before finalizing
- To verify migration scripts

**How to Use**:
```
# Read existing configuration
read_file /chat-application/backend/package.json

# Verify created schema
read_file /chat-application/backend/src/database/schemas/chat_schema.sql
```

**Parameters**:
- `path`: File path to read
- `head`: Optional - read only first N lines
- `tail`: Optional - read only last N lines

### 4. **edit_file** (Local - filesystem)
**Purpose**: Make line-based edits to existing files like package.json or configuration files

**When to Use**: 
- To add database dependencies to package.json
- To update TypeScript configurations
- To modify database connection settings

**How to Use**:
```
# Add database dependencies
edit_file /chat-application/backend/package.json
# Replace dependencies section with updated version including:
# - pg (PostgreSQL client)
# - redis
# - typeorm or knex for migrations
```

**Parameters**:
- `old_string`: Exact text to replace (including whitespace)
- `new_string`: Replacement text
- `path`: File to edit

### 5. **list_directory** (Local - filesystem)
**Purpose**: Verify directory structure and file creation

**When to Use**: 
- After creating directory structure
- To confirm all migration files are present
- To verify model files organization

**How to Use**:
```
# List database directory contents
list_directory /chat-application/backend/src/database

# Verify migrations
list_directory /chat-application/backend/src/database/migrations
```

**Parameters**:
- `path`: Directory path to list

## Implementation Flow

1. **Directory Structure Phase**
   - Use `create_directory` to build complete database directory structure
   - Organize by schemas, migrations, models, repositories, and config

2. **Schema Definition Phase**
   - Use `write_file` to create PostgreSQL schema with all tables:
     - users table with authentication fields
     - rooms table for chat rooms
     - messages table with foreign keys
     - room_users junction table
   - Include proper indexes and constraints

3. **Migration Scripts Phase**
   - Use `write_file` to create numbered migration files
   - Follow naming convention: 001_create_users_table.sql
   - Include both UP and DOWN migrations

4. **Model Implementation Phase**
   - Use `write_file` to create TypeScript interfaces and models
   - Implement User, Room, Message, and RoomUser models
   - Create repository pattern implementations

5. **Redis Configuration Phase**
   - Use `write_file` to create Redis configuration
   - Set up connection pools
   - Configure JWT token storage
   - Implement Socket.io adapter configuration

6. **Testing Setup Phase**
   - Use `write_file` to create test files for models
   - Create migration test scripts
   - Set up database seeding for tests

## Best Practices

1. **Schema Design First**: Complete full schema design before creating files
2. **Consistent Naming**: Use consistent naming for tables, columns, and files
3. **Migration Ordering**: Number migrations sequentially (001, 002, etc.)
4. **Type Safety**: Define TypeScript interfaces matching database schemas
5. **Connection Pooling**: Configure appropriate pool sizes for PostgreSQL
6. **Redis Namespacing**: Use prefixes for different Redis data types

## Task-Specific Implementation Details

### PostgreSQL Schema Structure
```sql
-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    username VARCHAR(50) UNIQUE NOT NULL,
    avatar_url VARCHAR(500),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
```

### Redis Configuration Pattern
```typescript
// JWT refresh tokens
redis.setex(`refresh_token:${userId}`, 7 * 24 * 60 * 60, token);

// Socket.io presence
redis.setex(`presence:${userId}`, 30, 'online');

// Typing indicators
redis.setex(`typing:${roomId}:${userId}`, 5, 'true');
```

## Troubleshooting

- **File Already Exists**: Use `read_file` first to check existing content
- **Directory Structure**: Verify with `list_directory` before creating files
- **Migration Order**: Ensure migrations are numbered sequentially
- **Schema Syntax**: Test SQL syntax in a temporary file first
- **TypeScript Types**: Ensure model interfaces match database schemas exactly

## Testing Considerations

1. Create migration rollback tests
2. Implement model validation tests
3. Test connection pooling under load
4. Verify Redis expiration for JWT tokens
5. Test concurrent database operations