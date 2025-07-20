# Task 2: Setup SQLite Database - Acceptance Criteria

## Overview
This document defines the specific acceptance criteria for the SQLite database setup task. Each criterion includes verification steps and expected outcomes to ensure the database layer is properly implemented.

## Dependencies and Installation Criteria

### ✓ Database Dependencies Installed
- **Requirement**: SQLite packages installed with correct versions
- **Verification**:
  ```bash
  npm list sqlite3 better-sqlite3
  ```
- **Expected Versions**:
  - sqlite3: ^5.1.7
  - better-sqlite3: ^9.4.3
- **Additional Check**:
  ```bash
  npm list bcrypt
  ```
- **Expected**: bcrypt: ^5.1.1 (for password hashing in seeds)

### ✓ Database File Created
- **Requirement**: SQLite database file exists
- **Verification**:
  ```bash
  npm run db:init
  ls -la database.sqlite
  ```
- **Expected Result**: 
  - File `database.sqlite` exists
  - File size > 0 bytes
  - File has read/write permissions

## Database Schema Criteria

### ✓ Users Table Structure
- **Requirement**: Users table exists with correct schema
- **Verification**:
  ```bash
  sqlite3 database.sqlite ".schema users"
  ```
- **Expected Schema**:
  ```sql
  CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
  );
  CREATE INDEX idx_users_email ON users(email);
  ```

### ✓ Tasks Table Structure
- **Requirement**: Tasks table exists with correct schema and constraints
- **Verification**:
  ```bash
  sqlite3 database.sqlite ".schema tasks"
  ```
- **Expected Schema**:
  ```sql
  CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    completed BOOLEAN DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
  );
  CREATE INDEX idx_tasks_user_id ON tasks(user_id);
  CREATE TRIGGER update_tasks_updated_at...
  ```

### ✓ Foreign Key Constraints
- **Requirement**: Foreign keys are enabled and enforced
- **Verification**:
  ```bash
  sqlite3 database.sqlite "PRAGMA foreign_keys;"
  ```
- **Expected Result**: Returns `1` (enabled)

### ✓ Indexes Created
- **Requirement**: Performance indexes exist
- **Verification**:
  ```bash
  sqlite3 database.sqlite ".indexes"
  ```
- **Expected Indexes**:
  - idx_users_email
  - idx_tasks_user_id

## Model Implementation Criteria

### ✓ User Model - Create Operation
- **Requirement**: Can create new users
- **Test Script**:
  ```javascript
  const User = require('./src/models/User');
  const user = User.create('test@example.com', 'password');
  console.log(user.id > 0); // Should be true
  ```
- **Expected**: Returns user object with id, email, created_at

### ✓ User Model - Find Operations
- **Requirement**: Can find users by email and ID
- **Test Script**:
  ```javascript
  const byEmail = User.findByEmail('test@example.com');
  const byId = User.findById(1);
  ```
- **Expected**: Both return user objects or undefined if not found

### ✓ User Model - Unique Email Constraint
- **Requirement**: Duplicate emails are rejected
- **Test**: Try to create two users with same email
- **Expected**: Second attempt throws "Email already exists" error

### ✓ Task Model - Create Operation
- **Requirement**: Can create tasks for users
- **Test Script**:
  ```javascript
  const Task = require('./src/models/Task');
  const task = Task.create(1, 'Test Task', 'Description');
  console.log(task.id > 0); // Should be true
  ```
- **Expected**: Returns task object with all fields

### ✓ Task Model - Find by User
- **Requirement**: Can retrieve all tasks for a user
- **Test Script**:
  ```javascript
  const tasks = Task.findByUserId(1);
  console.log(Array.isArray(tasks)); // Should be true
  ```
- **Expected**: Returns array of tasks (may be empty)

### ✓ Task Model - Update with Ownership Check
- **Requirement**: Can only update own tasks
- **Test Script**:
  ```javascript
  const updated = Task.update(1, 1, { title: 'Updated' });
  const wrongUser = Task.update(1, 999, { title: 'Hack' });
  ```
- **Expected**: 
  - First returns true
  - Second returns false

### ✓ Task Model - Delete with Ownership Check
- **Requirement**: Can only delete own tasks
- **Test Script**:
  ```javascript
  const deleted = Task.delete(1, 1);
  const wrongUser = Task.delete(2, 999);
  ```
- **Expected**:
  - First returns true if task exists
  - Second returns false

## Database Operations Criteria

### ✓ Prepared Statements Used
- **Requirement**: All queries use prepared statements
- **Verification**: Code review of models
- **Expected**: No string concatenation in SQL queries, only `?` placeholders

### ✓ Transaction Support
- **Requirement**: Database supports transactions
- **Test**: Multiple operations in transaction
- **Expected**: All succeed or all fail together

### ✓ Cascade Delete
- **Requirement**: Deleting user removes their tasks
- **Test**:
  1. Create user with tasks
  2. Delete user
  3. Check tasks table
- **Expected**: User's tasks are also deleted

## Migration and Initialization Criteria

### ✓ Database Initialization Script
- **Requirement**: npm script initializes database
- **Verification**:
  ```bash
  rm -f database.sqlite
  npm run db:init
  ```
- **Expected**: 
  - "Database initialized successfully"
  - Tables created
  - No errors

### ✓ Database Reset Script
- **Requirement**: Can reset database to clean state
- **Verification**:
  ```bash
  npm run db:reset
  ```
- **Expected**:
  - "Database reset successfully"
  - All data cleared
  - Schema recreated

### ✓ Migration System
- **Requirement**: Migrations can be run up and down
- **Verification**: Check migration file exists and exports up/down
- **Expected**: Both functions execute without errors

## Seed Data Criteria

### ✓ Seed Script Execution
- **Requirement**: Seed script populates test data
- **Verification**:
  ```bash
  npm run db:seed
  ```
- **Expected Output**:
  ```
  Seeding database...
  Created user: user1@example.com
  Created user: user2@example.com
  Database seeded successfully
  Created 2 users and 10 tasks
  
  Test credentials:
  Email: user1@example.com, Password: password123
  Email: user2@example.com, Password: password123
  ```

### ✓ Seed Data Verification
- **Requirement**: Correct amount of test data created
- **Verification**:
  ```bash
  sqlite3 database.sqlite "SELECT COUNT(*) FROM users;"
  sqlite3 database.sqlite "SELECT COUNT(*) FROM tasks;"
  ```
- **Expected**:
  - 2 users
  - 10 tasks (5 per user)

### ✓ Seed Data Quality
- **Requirement**: Seed data is realistic and varied
- **Verification**:
  ```bash
  sqlite3 database.sqlite "SELECT * FROM tasks LIMIT 5;"
  ```
- **Expected**:
  - Mix of completed and pending tasks
  - Meaningful titles and descriptions

## Integration Criteria

### ✓ Database Initialization on Server Start
- **Requirement**: Database initializes when server starts
- **Verification**:
  ```bash
  npm run dev
  ```
- **Expected Console Output**:
  - "Initializing database..."
  - "Database connected successfully"
  - "Database initialized successfully"
  - "Server running on port 3000"

### ✓ Graceful Shutdown
- **Requirement**: Database closes properly on shutdown
- **Verification**:
  1. Start server: `npm run dev`
  2. Stop with Ctrl+C
- **Expected Console Output**:
  - "Received shutdown signal..."
  - "Database connection closed"
  - "Server and database closed"

### ✓ Error Handling
- **Requirement**: Database errors are handled gracefully
- **Test**: Temporarily rename database file while running
- **Expected**: Error logged, appropriate error response

## npm Scripts Criteria

### ✓ All Database Scripts Work
- **Requirement**: All db:* scripts function correctly
- **Verification**:
  ```bash
  npm run db:init   # Should initialize
  npm run db:reset  # Should reset
  npm run db:seed   # Should seed
  ```
- **Expected**: Each completes without errors

## Performance Criteria

### ✓ Indexes Improve Query Performance
- **Requirement**: Queries use indexes efficiently
- **Verification**: For large datasets, queries should be fast
- **Expected**: 
  - Email lookups use idx_users_email
  - Task queries use idx_tasks_user_id

### ✓ Updated_at Trigger
- **Requirement**: Trigger updates timestamp on task changes
- **Test**:
  1. Create a task
  2. Wait 1 second
  3. Update the task
  4. Check updated_at changed
- **Expected**: updated_at is newer than created_at

## Security Criteria

### ✓ SQL Injection Prevention
- **Requirement**: All queries safe from injection
- **Test**: Try to inject SQL in model methods
- **Expected**: Injection attempts fail safely

### ✓ Password Storage
- **Requirement**: Passwords are hashed in seed data
- **Verification**:
  ```bash
  sqlite3 database.sqlite "SELECT password FROM users LIMIT 1;"
  ```
- **Expected**: See bcrypt hash, not plain text

## Test Summary Checklist

- [ ] SQLite dependencies installed correctly
- [ ] Database file created successfully
- [ ] Users table has correct schema
- [ ] Tasks table has correct schema with foreign key
- [ ] Indexes created for performance
- [ ] Foreign keys enforced
- [ ] User model CRUD operations work
- [ ] Task model CRUD operations work
- [ ] Ownership checks prevent unauthorized access
- [ ] Prepared statements used throughout
- [ ] Cascade delete removes related tasks
- [ ] Database initialization script works
- [ ] Database reset script works
- [ ] Seed script creates test data
- [ ] Database initializes on server start
- [ ] Database closes on graceful shutdown
- [ ] All npm scripts function correctly
- [ ] Updated_at trigger works
- [ ] No SQL injection vulnerabilities

## Definition of Done

Task 2 is complete when:
1. All acceptance criteria above are met
2. Database operations are stable and error-free
3. Models provide secure CRUD operations
4. Test data is available for development
5. Integration with Express server is seamless
6. All database queries use prepared statements
7. Foreign key relationships are properly enforced

## Notes

- The database file should not be committed to version control
- Always use prepared statements to prevent SQL injection
- Test with both empty database and seeded data
- Ensure proper error handling for all database operations
- The better-sqlite3 library provides synchronous operations which simplifies code