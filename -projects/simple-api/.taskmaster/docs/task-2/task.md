# Task 2: Database Setup and Model Implementation

## Overview
Create the SQLite database infrastructure and implement the Todo model with full CRUD operations. This task establishes the data persistence layer following the architecture's model layer design patterns.

## Task Details
**ID**: 2  
**Title**: Database Setup and Model Implementation  
**Priority**: High  
**Dependencies**: [Task 1: Project Setup and Configuration](../task-1/task.md)  
**Status**: Pending

## Architecture Context
This task implements the Model Layer as defined in the [architecture document](../../architecture.md):
- Database interaction and SQL query execution
- Data access objects (DAO) pattern
- Data validation at the model level
- SQLite with better-sqlite3 driver for synchronous operations

Follows the defined data model:
- Todo entity with id, title, description, completed, createdAt, updatedAt
- Database schema with proper constraints and triggers
- Auto-timestamp management for createdAt and updatedAt fields

## Product Requirements Alignment
Implements PRD data model requirements:
- Todo structure with all specified fields
- SQLite database in `./data/todos.db`
- Auto-create database and tables on startup
- Field validations (title max 200 chars, description max 1000 chars)

## Implementation Steps

### 1. Create Database Connection Module
Create `src/models/db.js`:
```javascript
const Database = require('better-sqlite3');
const path = require('path');
const fs = require('fs');

// Ensure data directory exists
const dataDir = path.join(__dirname, '../../data');
if (!fs.existsSync(dataDir)) {
  fs.mkdirSync(dataDir, { recursive: true });
}

const db = new Database(path.join(dataDir, 'todos.db'));

// Enable foreign keys and WAL mode for better performance
db.pragma('foreign_keys = ON');
db.pragma('journal_mode = WAL');

// Create todos table if it doesn't exist
db.exec(`
  CREATE TABLE IF NOT EXISTS todos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL CHECK(length(title) <= 200),
    description TEXT CHECK(length(description) <= 1000),
    completed INTEGER NOT NULL DEFAULT 0,
    createdAt TEXT DEFAULT CURRENT_TIMESTAMP,
    updatedAt TEXT DEFAULT CURRENT_TIMESTAMP
  )
`);

// Create trigger for updatedAt
db.exec(`
  CREATE TRIGGER IF NOT EXISTS update_todos_timestamp
  AFTER UPDATE ON todos
  BEGIN
    UPDATE todos SET updatedAt = CURRENT_TIMESTAMP WHERE id = NEW.id;
  END
`);

module.exports = db;
```

### 2. Implement Todo Model
Create `src/models/todo.js`:
```javascript
const db = require('./db');

const Todo = {
  // Find all todos with optional filtering
  findAll({ completed, limit, offset } = {}) {
    let query = 'SELECT * FROM todos';
    const params = [];
    const conditions = [];
    
    if (completed !== undefined) {
      conditions.push('completed = ?');
      params.push(completed ? 1 : 0);
    }
    
    if (conditions.length > 0) {
      query += ' WHERE ' + conditions.join(' AND ');
    }
    
    query += ' ORDER BY id DESC';
    
    if (limit) {
      query += ' LIMIT ?';
      params.push(limit);
    }
    
    if (offset) {
      query += ' OFFSET ?';
      params.push(offset);
    }
    
    const stmt = db.prepare(query);
    return stmt.all(...params);
  },
  
  // Find todo by ID
  findById(id) {
    const stmt = db.prepare('SELECT * FROM todos WHERE id = ?');
    return stmt.get(id);
  },
  
  // Create new todo
  create({ title, description }) {
    const stmt = db.prepare(
      'INSERT INTO todos (title, description) VALUES (?, ?)'
    );
    const result = stmt.run(title, description || null);
    return this.findById(result.lastInsertRowid);
  },
  
  // Update existing todo
  update(id, updates) {
    const todo = this.findById(id);
    if (!todo) return null;
    
    const fields = [];
    const values = [];
    
    if (updates.title !== undefined) {
      fields.push('title = ?');
      values.push(updates.title);
    }
    
    if (updates.description !== undefined) {
      fields.push('description = ?');
      values.push(updates.description);
    }
    
    if (updates.completed !== undefined) {
      fields.push('completed = ?');
      values.push(updates.completed ? 1 : 0);
    }
    
    if (fields.length === 0) return todo;
    
    const query = `UPDATE todos SET ${fields.join(', ')} WHERE id = ?`;
    values.push(id);
    
    const stmt = db.prepare(query);
    stmt.run(...values);
    
    return this.findById(id);
  },
  
  // Delete todo by ID
  delete(id) {
    const stmt = db.prepare('DELETE FROM todos WHERE id = ?');
    const result = stmt.run(id);
    return result.changes > 0;
  },
  
  // Count todos (utility method)
  count({ completed } = {}) {
    let query = 'SELECT COUNT(*) as count FROM todos';
    const params = [];
    
    if (completed !== undefined) {
      query += ' WHERE completed = ?';
      params.push(completed ? 1 : 0);
    }
    
    const stmt = db.prepare(query);
    const result = stmt.get(...params);
    return result.count;
  }
};

module.exports = Todo;
```

### 3. Create Model Index File
Create `src/models/index.js`:
```javascript
const Todo = require('./todo');
const db = require('./db');

module.exports = {
  Todo,
  db
};
```

## Database Schema Details

### Table Structure
```sql
CREATE TABLE todos (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  title TEXT NOT NULL CHECK(length(title) <= 200),
  description TEXT CHECK(length(description) <= 1000),
  completed INTEGER NOT NULL DEFAULT 0,
  createdAt TEXT DEFAULT CURRENT_TIMESTAMP,
  updatedAt TEXT DEFAULT CURRENT_TIMESTAMP
);
```

### Update Trigger
```sql
CREATE TRIGGER update_todos_timestamp
AFTER UPDATE ON todos
BEGIN
  UPDATE todos SET updatedAt = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;
```

## Testing Considerations
- Use in-memory SQLite database for testing (`:memory:`)
- Test all CRUD operations
- Verify field constraints and validations
- Test pagination with limit and offset
- Verify timestamp auto-generation and updates

## Success Criteria
- Database file is created automatically in `data/todos.db`
- All CRUD operations work correctly
- Field constraints are enforced (max lengths)
- Timestamps are automatically managed
- Todo model methods handle edge cases gracefully
- Database connections are properly managed

## Related Tasks
- **Previous**: [Task 1: Project Setup and Configuration](../task-1/task.md)
- **Next**: [Task 3: Implement Express Application and Middleware](../task-3/task.md)
- **Dependent Tasks**: Tasks 4, 5, and 7 depend on this model implementation

## References
- [Architecture Document](../../architecture.md) - Sections: Model Layer, Data Model, Database Schema
- [Product Requirements](../../prd.txt) - Section: Data Model