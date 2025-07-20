# Task 2: Database Setup and Model Implementation - Acceptance Criteria

## Overview
This document defines the acceptance criteria for Task 2: Database Setup and Model Implementation. All criteria must be met for the task to be considered complete.

## Functional Acceptance Criteria

### 1. Database Connection Setup ✓
- [ ] `src/models/db.js` file exists and exports database connection
- [ ] Data directory (`data/`) is created automatically if missing
- [ ] SQLite database file created at `data/todos.db`
- [ ] Foreign keys are enabled with `PRAGMA foreign_keys = ON`
- [ ] WAL mode enabled with `PRAGMA journal_mode = WAL`
- [ ] Database closes gracefully on process exit
- [ ] Verbose logging enabled in development mode only

### 2. Database Schema ✓
- [ ] `todos` table created with correct schema:
  - [ ] `id` - INTEGER PRIMARY KEY AUTOINCREMENT
  - [ ] `title` - TEXT NOT NULL with CHECK(length <= 200)
  - [ ] `description` - TEXT with CHECK(length <= 1000)
  - [ ] `completed` - INTEGER DEFAULT 0 with CHECK(value IN (0,1))
  - [ ] `createdAt` - TEXT DEFAULT CURRENT_TIMESTAMP
  - [ ] `updatedAt` - TEXT DEFAULT CURRENT_TIMESTAMP
- [ ] Update trigger created for `updatedAt` field
- [ ] Trigger fires on UPDATE and sets current timestamp

### 3. Todo Model Implementation ✓
- [ ] `src/models/todo.js` file exists and exports Todo object
- [ ] **findAll() method**:
  - [ ] Returns array of all todos when called without parameters
  - [ ] Filters by completed status when `completed` parameter provided
  - [ ] Limits results when `limit` parameter provided
  - [ ] Skips results when `offset` parameter provided
  - [ ] Orders results by id DESC (newest first)
  - [ ] Converts completed field from integer to boolean
- [ ] **findById() method**:
  - [ ] Returns todo object when found
  - [ ] Returns null when not found
  - [ ] Converts completed field from integer to boolean
- [ ] **create() method**:
  - [ ] Creates new todo with provided title
  - [ ] Accepts optional description
  - [ ] Returns created todo with generated id
  - [ ] Sets timestamps automatically
- [ ] **update() method**:
  - [ ] Updates specified fields only
  - [ ] Returns updated todo object
  - [ ] Returns null if todo not found
  - [ ] Handles title, description, and completed updates
  - [ ] Trigger updates updatedAt timestamp
- [ ] **delete() method**:
  - [ ] Deletes todo by id
  - [ ] Returns true if deleted
  - [ ] Returns false if not found
- [ ] **deleteAll() method**:
  - [ ] Deletes all todos
  - [ ] Returns count of deleted records
- [ ] **count() method**:
  - [ ] Returns total count without parameters
  - [ ] Filters by completed status when parameter provided

### 4. Database Initialization Script ✓
- [ ] `scripts/initDb.js` file exists
- [ ] Script is executable (shebang line present)
- [ ] Loads environment variables with dotenv
- [ ] Clears existing todos before seeding
- [ ] Seeds at least 5 sample todos
- [ ] Mix of completed and pending todos in seed data
- [ ] Displays initialization summary with counts

### 5. Model Index File ✓
- [ ] `src/models/index.js` exists
- [ ] Exports Todo model
- [ ] Exports db connection
- [ ] Allows destructured imports

## Non-Functional Acceptance Criteria

### Performance
- [ ] All queries use prepared statements
- [ ] WAL mode improves concurrent access
- [ ] Database operations complete in < 10ms for single records
- [ ] Bulk operations handle 1000+ records efficiently

### Data Integrity
- [ ] Title length constraint enforced (max 200 chars)
- [ ] Description length constraint enforced (max 1000 chars)
- [ ] Completed field only accepts 0 or 1
- [ ] Timestamps in ISO 8601 format
- [ ] Auto-increment IDs never reused

### Code Quality
- [ ] All methods have JSDoc comments
- [ ] Consistent error handling approach
- [ ] No async/await (synchronous API)
- [ ] Prepared statements prevent SQL injection
- [ ] Boolean conversion handled consistently

## Test Cases

### Test Case 1: Database Initialization
```bash
node scripts/initDb.js
```
**Expected Result**: 
- No errors
- Output shows "Database initialized successfully!"
- Reports count of seeded todos

### Test Case 2: Model CRUD Operations
```javascript
// In Node REPL
const { Todo } = require('./src/models');

// Create
const todo = Todo.create({ title: 'Test', description: 'Testing' });
console.log(typeof todo.completed); // 'boolean'
console.log(todo.id > 0); // true

// Read
const found = Todo.findById(todo.id);
console.log(found.title === 'Test'); // true

// Update
const updated = Todo.update(todo.id, { completed: true });
console.log(updated.completed === true); // true

// Delete
const deleted = Todo.delete(todo.id);
console.log(deleted === true); // true
console.log(Todo.findById(todo.id) === null); // true
```
**Expected Result**: All console.log statements output `true`

### Test Case 3: Filtering and Pagination
```javascript
const { Todo } = require('./src/models');

// Setup: Create 10 todos, 5 completed
for (let i = 1; i <= 10; i++) {
  const todo = Todo.create({ title: `Todo ${i}` });
  if (i <= 5) Todo.update(todo.id, { completed: true });
}

// Test filtering
const completed = Todo.findAll({ completed: true });
console.log(completed.length === 5); // true

// Test pagination
const page1 = Todo.findAll({ limit: 3, offset: 0 });
const page2 = Todo.findAll({ limit: 3, offset: 3 });
console.log(page1.length === 3); // true
console.log(page1[0].id !== page2[0].id); // true
```
**Expected Result**: All tests pass

### Test Case 4: Constraint Validation
```javascript
const { Todo } = require('./src/models');

// Test title length constraint
try {
  Todo.create({ title: 'x'.repeat(201) });
  console.log('FAIL: Should have thrown error');
} catch (e) {
  console.log('PASS: Title length constraint enforced');
}

// Test description length constraint
try {
  Todo.create({ title: 'Test', description: 'x'.repeat(1001) });
  console.log('FAIL: Should have thrown error');
} catch (e) {
  console.log('PASS: Description length constraint enforced');
}
```
**Expected Result**: Both constraints throw errors

### Test Case 5: Timestamp Auto-Update
```javascript
const { Todo } = require('./src/models');

// Create todo
const todo = Todo.create({ title: 'Timestamp Test' });
const originalUpdated = todo.updatedAt;

// Wait 1 second
setTimeout(() => {
  // Update todo
  const updated = Todo.update(todo.id, { title: 'Updated' });
  console.log(updated.updatedAt !== originalUpdated); // true
}, 1000);
```
**Expected Result**: updatedAt changes after update

## Definition of Done
- [ ] All functional acceptance criteria are met
- [ ] All non-functional acceptance criteria are met
- [ ] All test cases pass successfully
- [ ] Database file created and accessible
- [ ] Model methods work as specified
- [ ] No SQL injection vulnerabilities
- [ ] Proper error messages for constraint violations
- [ ] Code follows project conventions
- [ ] Ready for use in controllers

## Notes
- SQLite integers (0/1) must be converted to JavaScript booleans
- better-sqlite3 is synchronous - no promises/callbacks needed
- Prepared statements are crucial for security and performance
- Database file permissions may need adjustment on some systems