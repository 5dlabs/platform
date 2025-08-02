# Task 2: Database Setup and Schema Design - Toolman Usage Guide

## Overview
This guide explains how to use the selected Toolman tools to implement the database layer for the chat application. The tools focus on file creation for schemas, migrations, and model implementations, plus research capabilities for best practices.

## Core Tools

### 1. brave_web_search
**Purpose**: Research database design patterns and best practices
**When to use**: 
- Before implementing schema design
- When researching optimization techniques
- To find solutions for specific database challenges

**How to use**:
```json
{
  "tool": "brave_web_search",
  "query": "PostgreSQL chat application schema design best practices",
  "freshness": "year"
}
```

**Example searches**:
- "PostgreSQL connection pooling Node.js best practices"
- "Redis session management pattern Node.js"
- "Database migration tools TypeScript Knex"
- "PostgreSQL indexing strategies for messaging apps"
- "Redis pub/sub vs PostgreSQL LISTEN/NOTIFY"

### 2. create_directory
**Purpose**: Create directory structure for database-related files
**When to use**:
- Setting up migrations directory
- Creating models and repositories folders
- Organizing database configuration files

**How to use**:
```json
{
  "tool": "create_directory",
  "path": "/chat-application/backend/src/database/migrations"
}
```

**Required directories**:
```
/backend/src/database/
├── migrations/
├── seeds/
├── models/
├── repositories/
└── config/
```

### 3. write_file
**Purpose**: Create all database-related files
**When to use**:
- Writing SQL migration files
- Creating TypeScript model interfaces
- Implementing repository classes
- Setting up configuration files

**How to use**:
```json
{
  "tool": "write_file",
  "path": "/chat-application/backend/src/database/migrations/001_create_users_table.sql",
  "content": "CREATE TABLE users (...);"
}
```

### 4. edit_file
**Purpose**: Modify existing configuration files
**When to use**:
- Updating docker-compose.yml to add database services
- Modifying package.json to add database scripts
- Updating .env files with database credentials

**How to use**:
```json
{
  "tool": "edit_file",
  "path": "/chat-application/docker-compose.yml",
  "old_string": "services:",
  "new_string": "services:\n  postgres:\n    image: postgres:14-alpine"
}
```

### 5. read_file
**Purpose**: Review existing files before modification
**When to use**:
- Before editing configuration files
- To understand current project structure
- To verify file contents after creation

**How to use**:
```json
{
  "tool": "read_file",
  "path": "/chat-application/backend/package.json"
}
```

## Implementation Flow

### Phase 1: Research and Planning (15 minutes)
1. **Research schema patterns**:
   ```json
   {
     "tool": "brave_web_search",
     "query": "PostgreSQL schema design real-time chat application"
   }
   ```

2. **Research Redis patterns**:
   ```json
   {
     "tool": "brave_web_search",
     "query": "Redis session management Socket.io best practices"
   }
   ```

3. **Research migration tools**:
   ```json
   {
     "tool": "brave_web_search",
     "query": "Knex.js vs node-pg-migrate TypeScript"
   }
   ```

### Phase 2: Directory Setup (10 minutes)
1. Create database directory structure:
   ```json
   {
     "tool": "create_directory",
     "path": "/chat-application/backend/src/database/migrations"
   }
   ```

2. Create all required subdirectories:
   - `/database/seeds`
   - `/database/config`
   - `/models`
   - `/repositories`

### Phase 3: Schema Implementation (25 minutes)
1. **Create migration files**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/database/migrations/001_create_users_table.sql",
     "content": "-- Users table migration\nCREATE TABLE users (...);"
   }
   ```

2. **Write all migration files**:
   - 001_create_users_table.sql
   - 002_create_rooms_table.sql
   - 003_create_messages_table.sql
   - 004_create_room_users_table.sql
   - 005_create_message_receipts_table.sql

### Phase 4: Model Implementation (20 minutes)
1. **Create TypeScript interfaces**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/types/models.ts",
     "content": "export interface User { ... }"
   }
   ```

2. **Implement repository classes**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/repositories/userRepository.ts",
     "content": "export class UserRepository { ... }"
   }
   ```

### Phase 5: Configuration (15 minutes)
1. **Update docker-compose.yml**:
   ```json
   {
     "tool": "edit_file",
     "path": "/chat-application/docker-compose.yml",
     "old_string": "services:\n  frontend:",
     "new_string": "services:\n  postgres:\n    image: postgres:14-alpine\n    environment:..."
   }
   ```

2. **Create database config**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/config/database.ts",
     "content": "import { Pool } from 'pg';\n..."
   }
   ```

## Best Practices

### File Organization
```
backend/src/
├── config/
│   ├── database.ts      # PostgreSQL config
│   └── redis.ts         # Redis config
├── database/
│   ├── migrations/      # SQL migration files
│   └── seeds/          # Development seed data
├── models/             # TypeScript interfaces
├── repositories/       # Data access layer
└── services/          # Business logic
```

### Migration Naming Convention
- Use sequential numbers: 001_, 002_, etc.
- Descriptive names: create_users_table, add_index_to_messages
- Include up and down migrations

### Repository Pattern
```typescript
// Standard repository methods
interface Repository<T> {
  create(data: Omit<T, 'id'>): Promise<T>;
  findById(id: string): Promise<T | null>;
  update(id: string, data: Partial<T>): Promise<T>;
  delete(id: string): Promise<void>;
}
```

## Common Patterns

### Research Before Implementation
```javascript
// 1. Search for best practices
await brave_web_search("PostgreSQL indexing strategy messaging");

// 2. Apply findings to schema
await write_file("migration.sql", optimizedSchema);
```

### Configuration Update Pattern
```javascript
// 1. Read existing file
const content = await read_file("docker-compose.yml");

// 2. Edit with database services
await edit_file("docker-compose.yml", oldSection, newSection);

// 3. Verify changes
const updated = await read_file("docker-compose.yml");
```

## Troubleshooting

### Issue: Migration order confusion
**Solution**: Use numbered prefixes (001_, 002_) for proper sequencing

### Issue: TypeScript compilation errors
**Solution**: Ensure all interfaces are properly exported and imported

### Issue: Connection pool exhaustion
**Solution**: Research optimal pool settings for your use case

### Issue: Redis connection failures
**Solution**: Verify Redis service is running in Docker

## Task Completion Checklist
- [ ] All database tables designed and documented
- [ ] Migration files created in correct order
- [ ] TypeScript models match database schema
- [ ] Repository pattern implemented for all entities
- [ ] Redis configuration complete
- [ ] Docker services added and tested
- [ ] Environment variables documented
- [ ] Connection pooling configured
- [ ] All tests passing

This systematic approach ensures a robust, scalable database layer that supports the chat application's real-time requirements.