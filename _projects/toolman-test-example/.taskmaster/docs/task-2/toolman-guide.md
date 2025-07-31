# Task 2: Database Setup and Schema Design - Toolman Usage Guide

## Overview

This guide explains how to use Toolman's tools effectively for setting up PostgreSQL and Redis databases, creating schemas, implementing migrations, and building a robust data access layer for the chat application.

## Tool Selection Strategy

### Primary Tools

1. **File System Tools** - For creating and managing database-related files
2. **Terminal/Shell Tools** - For database commands and package installation
3. **Documentation Tools** - For referencing PostgreSQL and Redis documentation
4. **Code Analysis Tools** - For TypeScript validation and SQL linting
5. **Testing Tools** - For verifying database functionality

## Phase 1: Database Installation and Setup

### Using Terminal Tools for Database Setup

```bash
# Step 1: Verify PostgreSQL installation
postgres --version

# Step 2: Start PostgreSQL service (if not running)
sudo systemctl start postgresql

# Step 3: Create database and user
sudo -u postgres psql << EOF
CREATE DATABASE chatapp_db;
CREATE USER chatapp_user WITH ENCRYPTED PASSWORD 'secure_password_here';
GRANT ALL PRIVILEGES ON DATABASE chatapp_db TO chatapp_user;
\q
EOF

# Step 4: Verify Redis installation
redis-server --version

# Step 5: Start Redis service
sudo systemctl start redis
```

### Environment Configuration

Use File System tools to create `.env` file:

```bash
# Create environment file
cat > .env << EOF
NODE_ENV=development
DB_HOST=localhost
DB_PORT=5432
DB_NAME=chatapp_db
DB_USER=chatapp_user
DB_PASSWORD=secure_password_here
REDIS_HOST=localhost
REDIS_PORT=6379
REDIS_PASSWORD=
EOF
```

## Phase 2: Schema Creation with Migration System

### Creating Migration Files

Use File System tools to create migration structure:

```bash
# Create migration directories
mkdir -p src/db/migrations

# Create initial migration files
touch src/db/migrations/001_create_users.sql
touch src/db/migrations/002_create_rooms.sql
touch src/db/migrations/003_create_messages.sql
touch src/db/migrations/004_create_room_users.sql
touch src/db/migrations/005_create_message_read_status.sql
touch src/db/migrations/006_create_indexes.sql
touch src/db/migrations/007_create_triggers.sql
```

### Migration File Examples

For `001_create_users.sql`:
```sql
-- Use File System tool to write this content
CREATE TABLE IF NOT EXISTS users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  email VARCHAR(255) UNIQUE NOT NULL,
  password_hash VARCHAR(255) NOT NULL,
  username VARCHAR(50) UNIQUE NOT NULL,
  avatar_url TEXT,
  is_active BOOLEAN DEFAULT true,
  last_seen TIMESTAMP WITH TIME ZONE,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
```

### Running Migrations

Use Terminal tools:

```bash
# Install migration dependencies
npm install pg dotenv

# Create migration runner script
npm run db:migrate

# Or run directly with Node.js
node -e "require('./src/db/migrate').MigrationRunner.run()"
```

## Phase 3: TypeScript Model Implementation

### Creating Model Files

Use File System tools with proper directory structure:

```bash
# Create model directories
mkdir -p src/models
mkdir -p src/types

# Create model files
touch src/models/User.ts
touch src/models/Room.ts
touch src/models/Message.ts
touch src/models/index.ts
```

### Model Implementation Pattern

```typescript
// Use File System tool to create src/models/User.ts
export interface User {
  id: string;
  email: string;
  passwordHash: string;
  username: string;
  avatarUrl?: string;
  isActive: boolean;
  lastSeen?: Date;
  createdAt: Date;
  updatedAt: Date;
}

// DTOs for data transfer
export interface CreateUserDto {
  email: string;
  password: string;
  username: string;
  avatarUrl?: string;
}
```

### Type Checking

Use Code Analysis tools:

```bash
# Run TypeScript compiler
npx tsc --noEmit

# Check specific model files
npx tsc --noEmit src/models/*.ts
```

## Phase 4: Repository Pattern Implementation

### Creating Repository Structure

```bash
# Create repository directories
mkdir -p src/repositories

# Create repository files
touch src/repositories/BaseRepository.ts
touch src/repositories/UserRepository.ts
touch src/repositories/RoomRepository.ts
touch src/repositories/MessageRepository.ts
touch src/repositories/index.ts
```

### Repository Implementation Workflow

1. **Create Base Repository** (File System tool)
2. **Implement Entity Repositories** (File System tool)
3. **Validate TypeScript Types** (Code Analysis tool)
4. **Test Repository Methods** (Testing tool)

Example workflow:
```bash
# Step 1: Implement repository
# Use File System tool to write UserRepository.ts

# Step 2: Type check
npx tsc src/repositories/UserRepository.ts --noEmit

# Step 3: Test
npm test -- UserRepository
```

## Phase 5: Redis Configuration

### Setting Up Redis Connection

```bash
# Install Redis dependencies
npm install ioredis @types/ioredis

# Create Redis configuration
mkdir -p src/config
touch src/config/redis.ts
```

### Redis Implementation Pattern

```typescript
// Use File System tool to create redis.ts
import Redis from 'ioredis';

export const redisClient = new Redis({
  host: process.env.REDIS_HOST,
  port: parseInt(process.env.REDIS_PORT || '6379'),
  retryStrategy: (times) => Math.min(times * 50, 2000)
});
```

### Testing Redis Connection

Use Terminal tool:
```bash
# Test Redis connection
node -e "
const Redis = require('ioredis');
const redis = new Redis();
redis.ping().then(console.log).catch(console.error);
"
```

## Phase 6: Testing Database Setup

### Running Database Tests

Use Testing tools in sequence:

```bash
# 1. Unit tests for models
npm test -- --testPathPattern=models

# 2. Integration tests for repositories
npm test -- --testPathPattern=repositories

# 3. Database connectivity tests
npm test -- --testPathPattern=database

# 4. Performance tests
npm test -- --testPathPattern=performance
```

### Creating Test Database

```bash
# Create test database
sudo -u postgres psql -c "CREATE DATABASE chatapp_test;"

# Run migrations on test database
DB_NAME=chatapp_test npm run db:migrate
```

## Best Practices for Tool Usage

### 1. File System Tools
- Always use absolute paths
- Verify directory existence before creating files
- Use consistent naming conventions
- Create backup before modifying existing files

### 2. Terminal Tools
- Check command exit status
- Use non-interactive commands
- Pipe output for logging
- Handle errors gracefully

### 3. Documentation Tools
- Reference official PostgreSQL docs for SQL syntax
- Check Redis docs for command usage
- Verify TypeScript types against documentation
- Keep local copies of critical documentation

### 4. Code Analysis Tools
- Run type checking after each model change
- Use strict TypeScript configuration
- Validate SQL syntax in migration files
- Check for unused dependencies

### 5. Testing Tools
- Test in isolation using transactions
- Clean up test data after each run
- Use test database, not production
- Mock external dependencies

## Common Workflows

### Workflow 1: Adding a New Table

1. Create migration file (File System)
2. Write SQL schema (File System + Documentation)
3. Run migration (Terminal)
4. Create TypeScript model (File System)
5. Type check model (Code Analysis)
6. Create repository (File System)
7. Write tests (File System)
8. Run tests (Testing)

### Workflow 2: Modifying Existing Schema

1. Create new migration file (File System)
2. Write ALTER statements (File System + Documentation)
3. Update TypeScript models (File System)
4. Update repositories (File System)
5. Run type checking (Code Analysis)
6. Test changes (Testing)
7. Run migration (Terminal)

### Workflow 3: Debugging Database Issues

1. Check database logs (Terminal)
2. Verify connection settings (File System - .env)
3. Test connection manually (Terminal)
4. Review error messages (Terminal)
5. Check documentation (Documentation)
6. Implement fix (File System)
7. Test fix (Testing)

## Troubleshooting Guide

### Database Connection Issues

```bash
# Check PostgreSQL status
sudo systemctl status postgresql

# Test connection
psql -h localhost -U chatapp_user -d chatapp_db -c "SELECT 1;"

# Check Redis status
redis-cli ping
```

### Migration Failures

```bash
# Check migration status
psql -U chatapp_user -d chatapp_db -c "SELECT * FROM migrations;"

# Rollback failed migration manually
psql -U chatapp_user -d chatapp_db -f rollback/005_rollback.sql
```

### TypeScript Errors

```bash
# Clear TypeScript cache
rm -rf node_modules/.cache/

# Rebuild TypeScript project
npx tsc --build --clean
npx tsc --build
```

## Security Considerations

### Using Tools Securely

1. **Never commit .env files** - Use File System tools to add to .gitignore
2. **Use parameter binding** - Verify with Code Analysis tools
3. **Validate input** - Test with Testing tools
4. **Encrypt sensitive data** - Reference Documentation tools
5. **Audit database access** - Monitor with Terminal tools

### Security Checklist

- [ ] Environment variables for all credentials
- [ ] SSL/TLS for production connections
- [ ] Password hashing implemented
- [ ] SQL injection prevention verified
- [ ] Access control configured
- [ ] Audit logging enabled

## Performance Optimization

### Using Tools for Performance

1. **Analyze slow queries** - Terminal tool with EXPLAIN
2. **Create indexes** - File System for migration files
3. **Monitor connections** - Terminal for pool stats
4. **Profile operations** - Testing tools with benchmarks
5. **Optimize schemas** - Documentation for best practices

## Summary

Effective use of Toolman's tools for database setup involves:

1. **Planning** - Use Documentation tools first
2. **Implementation** - File System and Terminal tools
3. **Validation** - Code Analysis tools
4. **Testing** - Testing tools comprehensively
5. **Monitoring** - Terminal tools for operations

Follow this guide to ensure robust, secure, and performant database implementation for the chat application.