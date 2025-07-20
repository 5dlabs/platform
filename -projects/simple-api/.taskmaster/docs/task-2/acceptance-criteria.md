# Task 2: Database Setup and Model Implementation - Acceptance Criteria

## Overview
This document defines the acceptance criteria for Task 2: Database Setup and Model Implementation. All criteria must be met for the task to be considered complete.

## Functional Criteria

### 1. Database Setup
- [ ] SQLite database file is created at `data/todos.db`
- [ ] Database file is created automatically on first run
- [ ] Data directory is created if it doesn't exist
- [ ] Database uses WAL mode for better performance
- [ ] Foreign keys are enabled (for future use)

### 2. Table Schema
The `todos` table exists with correct schema:
- [ ] `id` - INTEGER PRIMARY KEY AUTOINCREMENT
- [ ] `title` - TEXT NOT NULL with CHECK constraint (max 200 chars)
- [ ] `description` - TEXT with CHECK constraint (max 1000 chars)
- [ ] `completed` - INTEGER NOT NULL DEFAULT 0
- [ ] `createdAt` - TEXT DEFAULT CURRENT_TIMESTAMP
- [ ] `updatedAt` - TEXT DEFAULT CURRENT_TIMESTAMP

### 3. Database Triggers
- [ ] Update trigger exists: `update_todos_timestamp`
- [ ] Trigger updates `updatedAt` on row modification
- [ ] Trigger fires AFTER UPDATE operations

### 4. Model Methods Implementation
Todo model provides these methods:

**findAll(filters)**:
- [ ] Returns all todos when called without filters
- [ ] Filters by `completed` status when provided
- [ ] Supports `limit` parameter
- [ ] Supports `offset` parameter
- [ ] Returns todos ordered by id DESC
- [ ] Returns empty array when no todos exist

**findById(id)**:
- [ ] Returns todo object when found
- [ ] Returns undefined when not found
- [ ] Handles non-numeric IDs gracefully

**create({ title, description })**:
- [ ] Creates new todo with provided data
- [ ] Returns created todo with generated id
- [ ] Sets `completed` to false by default
- [ ] Auto-generates timestamps
- [ ] Handles null/undefined description

**update(id, updates)**:
- [ ] Updates only provided fields
- [ ] Returns updated todo
- [ ] Returns null if todo not found
- [ ] Preserves unchanged fields
- [ ] Updates `updatedAt` timestamp

**delete(id)**:
- [ ] Deletes todo and returns true
- [ ] Returns false if todo not found
- [ ] Actually removes from database

**count(filters)**:
- [ ] Returns total count without filters
- [ ] Filters by completed status when provided
- [ ] Returns numeric value

## Technical Criteria

### 1. Database Connection
- [ ] Uses better-sqlite3 synchronous API
- [ ] Connection is established correctly
- [ ] Database file has proper permissions
- [ ] Prepared statements are used

### 2. Data Validation
- [ ] Title length constraint enforced (≤200 chars)
- [ ] Description length constraint enforced (≤1000 chars)
- [ ] Title cannot be null or empty
- [ ] SQLite constraint violations throw errors

### 3. Type Handling
- [ ] Boolean `completed` stored as 0/1
- [ ] Timestamps stored as ISO strings
- [ ] NULL values handled correctly
- [ ] Type conversions are consistent

### 4. Code Quality
- [ ] No SQL injection vulnerabilities
- [ ] All queries use parameterized statements
- [ ] Error handling for database operations
- [ ] Models are properly exported

## Validation Tests

### 1. Database Creation Test
```javascript
// Should create database and table
const db = require('./src/models/db');
const result = db.prepare("SELECT name FROM sqlite_master WHERE type='table'").all();
// Should include 'todos' table
```

### 2. CRUD Operations Test
```javascript
const Todo = require('./src/models/todo');

// Create
const todo = Todo.create({ title: 'Test Todo', description: 'Test' });
console.assert(todo.id > 0);
console.assert(todo.completed === 0);

// Read
const found = Todo.findById(todo.id);
console.assert(found.title === 'Test Todo');

// Update
const updated = Todo.update(todo.id, { completed: true });
console.assert(updated.completed === 1);

// Delete
const deleted = Todo.delete(todo.id);
console.assert(deleted === true);
```

### 3. Constraint Tests
```javascript
// Should enforce max length
try {
  Todo.create({ title: 'a'.repeat(201) });
  console.assert(false, 'Should have thrown error');
} catch (e) {
  console.assert(true, 'Constraint enforced');
}
```

### 4. Timestamp Tests
```javascript
// Create todo
const todo = Todo.create({ title: 'Time Test' });
const created = todo.createdAt;

// Wait and update
setTimeout(() => {
  const updated = Todo.update(todo.id, { title: 'Updated' });
  console.assert(updated.updatedAt > created);
}, 1000);
```

## Edge Cases to Verify

1. **Empty Database**: All operations handle empty state
2. **Invalid IDs**: Non-existent IDs return appropriate values
3. **Null Values**: Description can be null
4. **Concurrent Access**: Database handles multiple operations
5. **Large Data**: Pagination works with many records

## Success Indicators

- [ ] Database layer is fully functional
- [ ] All CRUD operations work correctly
- [ ] Data constraints are enforced
- [ ] Timestamps are managed automatically
- [ ] Model provides clean API for controllers
- [ ] No SQL is needed outside model layer

## Performance Criteria

- [ ] Database uses WAL mode
- [ ] Prepared statements are cached
- [ ] Queries execute efficiently
- [ ] Indexes exist on primary key

## Notes for Reviewers

When reviewing this task:
1. Verify database file is created correctly
2. Check table schema matches specification
3. Test all CRUD operations manually
4. Verify constraints are enforced
5. Ensure timestamps update correctly
6. Confirm error handling is appropriate

Task is complete when all checkboxes above can be marked as done.