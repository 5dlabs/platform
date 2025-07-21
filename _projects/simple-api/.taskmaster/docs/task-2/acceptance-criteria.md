# Task 2: Database Setup and Model Implementation - Acceptance Criteria

## Overview

This document defines the acceptance criteria for Task 2: Database Setup and Model Implementation. All criteria must be met for the task to be considered complete.

## Prerequisites

- Task 1 must be completed (project setup with better-sqlite3 installed)

## Acceptance Criteria

### 1. Database Connection Module ✓

**Given** the database module is required
**When** `require('./src/models/db')` is called
**Then** the following must occur:

- The `data` directory must be created if it doesn't exist
- A SQLite database file must be created at `data/todos.db`
- The todos table must be created with the correct schema
- The module must export the database connection object

**Test**:
```javascript
const db = require('./src/models/db');
// Check if data/todos.db exists
// Verify table schema with: db.prepare("PRAGMA table_info(todos)").all()
```

### 2. Database Schema ✓

**Given** the database is initialized
**When** inspecting the todos table
**Then** it must have these columns:

| Column | Type | Constraints |
|--------|------|-------------|
| id | INTEGER | PRIMARY KEY AUTOINCREMENT |
| title | TEXT | NOT NULL |
| description | TEXT | (nullable) |
| completed | BOOLEAN | DEFAULT 0 |
| createdAt | TEXT | DEFAULT CURRENT_TIMESTAMP |
| updatedAt | TEXT | DEFAULT CURRENT_TIMESTAMP |

**Test**: 
```sql
PRAGMA table_info(todos);
```

### 3. Todo Model - Create Operation ✓

**Given** the Todo model is imported
**When** calling `Todo.create({ title, description })`
**Then**:
- A new todo must be inserted into the database
- The created todo object must be returned with all fields
- Auto-generated fields (id, timestamps) must be populated
- Title is required; error if missing
- Description is optional (null if not provided)

**Test Cases**:
```javascript
// Valid creation
const todo1 = Todo.create({ title: 'Test Todo', description: 'Test Desc' });
assert(todo1.id > 0);
assert(todo1.title === 'Test Todo');
assert(todo1.completed === 0);

// Without description
const todo2 = Todo.create({ title: 'No Description' });
assert(todo2.description === null);

// Missing title should throw error
assert.throws(() => Todo.create({ description: 'No Title' }));
```

### 4. Todo Model - Read Operations ✓

**Given** todos exist in the database
**When** using read operations
**Then** the following must work:

#### findAll:
- Returns array of todos (empty array if none)
- Supports filtering by completed status
- Supports pagination with limit and offset
- Orders by id DESC (newest first)

#### findById:
- Returns todo object if found
- Returns undefined if not found
- Handles invalid IDs gracefully

**Test Cases**:
```javascript
// Find all
const all = Todo.findAll();
assert(Array.isArray(all));

// Filter by completed
const completed = Todo.findAll({ completed: true });
const incomplete = Todo.findAll({ completed: false });

// Pagination
const page1 = Todo.findAll({ limit: 2, offset: 0 });
const page2 = Todo.findAll({ limit: 2, offset: 2 });

// Find by ID
const found = Todo.findById(1);
const notFound = Todo.findById(9999);
assert(notFound === undefined);
```

### 5. Todo Model - Update Operation ✓

**Given** a todo exists
**When** calling `Todo.update(id, updates)`
**Then**:
- Only specified fields are updated
- Updated todo object is returned
- Returns null if todo not found
- updatedAt timestamp is automatically updated
- Can update title, description, and completed fields

**Test Cases**:
```javascript
// Update single field
const updated1 = Todo.update(1, { completed: true });
assert(updated1.completed === 1);

// Update multiple fields
const updated2 = Todo.update(1, { 
  title: 'Updated Title',
  description: 'Updated Desc'
});

// Non-existent ID
const notFound = Todo.update(9999, { title: 'Test' });
assert(notFound === null);

// Verify updatedAt changed
const before = Todo.findById(1);
setTimeout(() => {
  const after = Todo.update(1, { title: 'New' });
  assert(after.updatedAt > before.updatedAt);
}, 1000);
```

### 6. Todo Model - Delete Operation ✓

**Given** a todo exists
**When** calling `Todo.delete(id)`
**Then**:
- The todo is removed from database
- Returns true if deleted
- Returns false if not found

**Test Cases**:
```javascript
// Successful delete
const success = Todo.delete(1);
assert(success === true);
assert(Todo.findById(1) === undefined);

// Non-existent ID
const notFound = Todo.delete(9999);
assert(notFound === false);
```

### 7. Additional Model Methods ✓

**Given** the Todo model
**When** using utility methods
**Then** `Todo.count({ completed })` must:
- Return total count without filter
- Return filtered count with completed parameter
- Return number type always

**Test Cases**:
```javascript
const total = Todo.count();
const completed = Todo.count({ completed: true });
const incomplete = Todo.count({ completed: false });
assert(typeof total === 'number');
assert(total === completed + incomplete);
```

### 8. Data Integrity ✓

**Given** the model is used
**When** performing any operation
**Then**:
- SQL injection must be prevented (prepared statements)
- Boolean completed field stored as 0/1
- Timestamps follow ISO format
- No orphaned data or integrity violations

## Test Scenarios

### Scenario 1: Complete CRUD Cycle
```javascript
// Create
const todo = Todo.create({ title: 'CRUD Test' });
const id = todo.id;

// Read
const fetched = Todo.findById(id);
assert(fetched.title === 'CRUD Test');

// Update
const updated = Todo.update(id, { completed: true });
assert(updated.completed === 1);

// Delete
const deleted = Todo.delete(id);
assert(deleted === true);
assert(Todo.findById(id) === undefined);
```

### Scenario 2: Pagination Test
```javascript
// Create 5 todos
for (let i = 1; i <= 5; i++) {
  Todo.create({ title: `Todo ${i}` });
}

// Get pages
const page1 = Todo.findAll({ limit: 2, offset: 0 });
const page2 = Todo.findAll({ limit: 2, offset: 2 });
const page3 = Todo.findAll({ limit: 2, offset: 4 });

assert(page1.length === 2);
assert(page2.length === 2);
assert(page3.length === 1);
```

## Definition of Done

- [ ] Database module creates database and table correctly
- [ ] All CRUD operations work as specified
- [ ] Filtering by completed status works
- [ ] Pagination works correctly
- [ ] Timestamps are managed automatically
- [ ] SQL injection is prevented (prepared statements used)
- [ ] All test cases pass
- [ ] No console.log statements in production code
- [ ] Code follows project structure from Task 1

## Performance Criteria

- Create operation < 10ms
- Read operations < 5ms for small datasets
- Update operation < 10ms
- Delete operation < 10ms
- Database file size reasonable for data volume

## Notes

- Use synchronous better-sqlite3 API (no async/await)
- Let errors bubble up to controller layer
- Focus on data integrity and security
- Keep the model layer pure (no HTTP concerns)