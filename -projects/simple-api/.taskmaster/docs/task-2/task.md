# Task 2: Database Setup and Model Implementation

## Overview
This task implements the data persistence layer for the Simple Todo REST API using SQLite with the better-sqlite3 driver. It establishes the database connection, creates the schema, and implements the Todo model with full CRUD operations following the architecture specifications.

## Task Details

### Priority
High

### Dependencies
- Task 1: Project Setup and Configuration (must be completed first)

### Status
Pending

## Implementation Guide

### 1. Create Database Directory and Connection Module

First, ensure the data directory exists and create the database connection module:

**File: `src/models/db.js`**
```javascript
const Database = require('better-sqlite3');
const path = require('path');
const fs = require('fs');

// Ensure data directory exists
const dataDir = path.join(__dirname, '../../data');
if (!fs.existsSync(dataDir)) {
  fs.mkdirSync(dataDir, { recursive: true });
}

// Initialize database connection
const dbPath = process.env.DB_PATH || path.join(dataDir, 'todos.db');
const db = new Database(dbPath, {
  verbose: process.env.NODE_ENV === 'development' ? console.log : null
});

// Enable foreign keys
db.pragma('foreign_keys = ON');

// Optimize for performance
db.pragma('journal_mode = WAL');

// Create todos table if it doesn't exist
const createTableSQL = `
  CREATE TABLE IF NOT EXISTS todos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL CHECK(length(title) <= 200),
    description TEXT CHECK(length(description) <= 1000),
    completed INTEGER NOT NULL DEFAULT 0 CHECK(completed IN (0, 1)),
    createdAt TEXT NOT NULL DEFAULT (datetime('now')),
    updatedAt TEXT NOT NULL DEFAULT (datetime('now'))
  )
`;

db.exec(createTableSQL);

// Create trigger for updating updatedAt timestamp
const createTriggerSQL = `
  CREATE TRIGGER IF NOT EXISTS update_todos_timestamp
  AFTER UPDATE ON todos
  FOR EACH ROW
  BEGIN
    UPDATE todos SET updatedAt = datetime('now') WHERE id = NEW.id;
  END
`;

db.exec(createTriggerSQL);

// Graceful shutdown
process.on('exit', () => db.close());
process.on('SIGINT', () => {
  db.close();
  process.exit(0);
});

module.exports = db;
```

### 2. Implement Todo Model

Create the Todo model with all CRUD operations:

**File: `src/models/todo.js`**
```javascript
const db = require('./db');

const Todo = {
  /**
   * Find all todos with optional filtering and pagination
   * @param {Object} options - Filter options
   * @param {boolean} options.completed - Filter by completion status
   * @param {number} options.limit - Maximum number of results
   * @param {number} options.offset - Number of results to skip
   * @returns {Array} Array of todo objects
   */
  findAll({ completed, limit = 100, offset = 0 } = {}) {
    let query = 'SELECT * FROM todos';
    const params = [];
    
    if (completed !== undefined) {
      query += ' WHERE completed = ?';
      params.push(completed ? 1 : 0);
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
    const todos = stmt.all(...params);
    
    // Convert SQLite integers to booleans
    return todos.map(todo => ({
      ...todo,
      completed: Boolean(todo.completed)
    }));
  },
  
  /**
   * Find a todo by ID
   * @param {number} id - Todo ID
   * @returns {Object|null} Todo object or null if not found
   */
  findById(id) {
    const stmt = db.prepare('SELECT * FROM todos WHERE id = ?');
    const todo = stmt.get(id);
    
    if (!todo) return null;
    
    return {
      ...todo,
      completed: Boolean(todo.completed)
    };
  },
  
  /**
   * Create a new todo
   * @param {Object} data - Todo data
   * @param {string} data.title - Todo title (required)
   * @param {string} data.description - Todo description (optional)
   * @returns {Object} Created todo object
   */
  create({ title, description = null }) {
    const stmt = db.prepare(
      'INSERT INTO todos (title, description) VALUES (?, ?)'
    );
    
    const result = stmt.run(title, description);
    return this.findById(result.lastInsertRowid);
  },
  
  /**
   * Update an existing todo
   * @param {number} id - Todo ID
   * @param {Object} updates - Fields to update
   * @param {string} updates.title - New title
   * @param {string} updates.description - New description
   * @param {boolean} updates.completed - New completion status
   * @returns {Object|null} Updated todo object or null if not found
   */
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
  
  /**
   * Delete a todo
   * @param {number} id - Todo ID
   * @returns {boolean} True if deleted, false if not found
   */
  delete(id) {
    const stmt = db.prepare('DELETE FROM todos WHERE id = ?');
    const result = stmt.run(id);
    return result.changes > 0;
  },
  
  /**
   * Delete all todos (useful for testing)
   * @returns {number} Number of deleted todos
   */
  deleteAll() {
    const stmt = db.prepare('DELETE FROM todos');
    const result = stmt.run();
    return result.changes;
  },
  
  /**
   * Count todos with optional filtering
   * @param {Object} options - Filter options
   * @param {boolean} options.completed - Filter by completion status
   * @returns {number} Count of todos
   */
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

### 3. Create Database Initialization Script

Create a script for database initialization and seeding:

**File: `scripts/initDb.js`**
```javascript
#!/usr/bin/env node

require('dotenv').config();
const Todo = require('../src/models/todo');

console.log('Initializing database...');

// Clear existing data
const deletedCount = Todo.deleteAll();
console.log(`Cleared ${deletedCount} existing todos`);

// Seed with sample data
const sampleTodos = [
  {
    title: 'Set up project structure',
    description: 'Initialize Node.js project and install dependencies',
    completed: true
  },
  {
    title: 'Implement database models',
    description: 'Create SQLite database and Todo model',
    completed: true
  },
  {
    title: 'Build REST API endpoints',
    description: 'Implement CRUD operations for todos'
  },
  {
    title: 'Add API documentation',
    description: 'Set up Swagger for API documentation'
  },
  {
    title: 'Write comprehensive tests',
    description: 'Achieve 90% code coverage with Jest'
  }
];

console.log('Seeding sample todos...');

sampleTodos.forEach((todoData, index) => {
  const todo = Todo.create(todoData);
  if (todoData.completed) {
    Todo.update(todo.id, { completed: true });
  }
  console.log(`Created todo ${index + 1}: ${todo.title}`);
});

const totalCount = Todo.count();
const completedCount = Todo.count({ completed: true });

console.log(`\nDatabase initialized successfully!`);
console.log(`Total todos: ${totalCount}`);
console.log(`Completed: ${completedCount}`);
console.log(`Pending: ${totalCount - completedCount}`);
```

Make the script executable:
```bash
chmod +x scripts/initDb.js
```

### 4. Create Model Index File

Create an index file for easier model imports:

**File: `src/models/index.js`**
```javascript
const Todo = require('./todo');
const db = require('./db');

module.exports = {
  Todo,
  db
};
```

## Key Implementation Considerations

### Architecture Alignment
- Database setup follows the architecture document's data model specifications
- SQLite with better-sqlite3 provides synchronous API for simpler code
- WAL mode enabled for better concurrency
- Proper constraints enforce data integrity at the database level

### Data Integrity
- CHECK constraints ensure title and description length limits
- Trigger automatically updates the updatedAt timestamp
- Boolean values properly converted between JavaScript and SQLite
- Foreign key support enabled for future extensibility

### Performance Optimizations
- WAL (Write-Ahead Logging) mode for better concurrent access
- Prepared statements for all queries
- Proper indexing on primary key
- Efficient pagination with LIMIT and OFFSET

### Error Handling
- Database connection errors will throw during initialization
- Model methods return null for not-found resources
- Constraint violations will throw descriptive errors
- Graceful shutdown handling for database connection

## Testing Considerations

The Todo model should be tested with:
1. Unit tests using an in-memory database
2. Testing all CRUD operations
3. Testing edge cases (empty strings, long strings, invalid data)
4. Testing pagination and filtering
5. Testing constraint violations

Example test setup:
```javascript
// tests/unit/models/todo.test.js
const Database = require('better-sqlite3');
const db = new Database(':memory:');

// Mock the db module
jest.mock('../../../src/models/db', () => db);
```

## Common Issues and Solutions

### Issue: Database File Permissions
**Solution**: Ensure the data directory has write permissions:
```bash
chmod 755 data/
```

### Issue: SQLite Version Compatibility
**Solution**: better-sqlite3 requires SQLite 3.26.0 or higher. The package includes its own SQLite build.

### Issue: Database Locked Errors
**Solution**: WAL mode should prevent most locking issues. Ensure only one process accesses the database file.

## Next Steps
After completing this task:
1. Test the model thoroughly using the Node.js REPL
2. Run the database initialization script
3. Proceed to Task 3: Implement Express Application and Middleware
4. Use the Todo model in the controller implementation

## References
- [Better-SQLite3 Documentation](https://github.com/WiseLibs/better-sqlite3/wiki)
- [SQLite Documentation](https://www.sqlite.org/docs.html)
- [Architecture Document - Data Model](../architecture.md#data-model)