# Task 2: Database Setup and Model Implementation

## Overview

This task focuses on setting up the SQLite database, creating the database connection module, and implementing the Todo model with full CRUD (Create, Read, Update, Delete) operations. This establishes the data persistence layer for the Simple Todo REST API.

## Context

According to the [architecture document](../architecture.md), we're using SQLite with the better-sqlite3 library for data persistence. SQLite was chosen for its simplicity, zero-configuration setup, and suitability for lightweight applications. The Todo model will implement all necessary database operations using synchronous methods provided by better-sqlite3.

## Dependencies

- **Task 1**: Project Setup and Configuration must be completed (dependencies installed)

## Implementation Guide

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

// Create or open database
const db = new Database(path.join(dataDir, 'todos.db'));

// Enable foreign keys
db.pragma('foreign_keys = ON');

// Create todos table if it doesn't exist
db.exec(`
  CREATE TABLE IF NOT EXISTS todos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    completed BOOLEAN DEFAULT 0,
    createdAt TEXT DEFAULT CURRENT_TIMESTAMP,
    updatedAt TEXT DEFAULT CURRENT_TIMESTAMP
  )
`);

// Create trigger to update the updatedAt timestamp
db.exec(`
  CREATE TRIGGER IF NOT EXISTS update_todo_timestamp 
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
  /**
   * Find all todos with optional filtering
   * @param {Object} options - Filter options
   * @param {boolean} options.completed - Filter by completion status
   * @param {number} options.limit - Maximum number of results
   * @param {number} options.offset - Number of results to skip
   * @returns {Array} Array of todo objects
   */
  findAll({ completed, limit, offset } = {}) {
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
    return stmt.all(...params);
  },
  
  /**
   * Find a todo by ID
   * @param {number} id - Todo ID
   * @returns {Object|undefined} Todo object or undefined if not found
   */
  findById(id) {
    const stmt = db.prepare('SELECT * FROM todos WHERE id = ?');
    return stmt.get(id);
  },
  
  /**
   * Create a new todo
   * @param {Object} data - Todo data
   * @param {string} data.title - Todo title (required)
   * @param {string} data.description - Todo description (optional)
   * @returns {Object} Created todo object
   */
  create({ title, description }) {
    const stmt = db.prepare(
      'INSERT INTO todos (title, description) VALUES (?, ?)'
    );
    const result = stmt.run(title, description || null);
    return this.findById(result.lastInsertRowid);
  },
  
  /**
   * Update an existing todo
   * @param {number} id - Todo ID
   * @param {Object} updates - Fields to update
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

### 3. Create Database Utilities (Optional)

Create `src/models/dbUtils.js` for database maintenance:

```javascript
const db = require('./db');

const dbUtils = {
  /**
   * Clear all todos from the database
   * USE WITH CAUTION - This deletes all data
   */
  clearAllTodos() {
    const stmt = db.prepare('DELETE FROM todos');
    return stmt.run();
  },
  
  /**
   * Get database statistics
   */
  getStats() {
    const totalCount = db.prepare('SELECT COUNT(*) as count FROM todos').get();
    const completedCount = db.prepare('SELECT COUNT(*) as count FROM todos WHERE completed = 1').get();
    const incompleteCount = db.prepare('SELECT COUNT(*) as count FROM todos WHERE completed = 0').get();
    
    return {
      total: totalCount.count,
      completed: completedCount.count,
      incomplete: incompleteCount.count
    };
  },
  
  /**
   * Close database connection
   */
  close() {
    db.close();
  }
};

module.exports = dbUtils;
```

## Key Implementation Details

### Database Design Decisions

1. **SQLite Choice**: Lightweight, serverless, zero-configuration database perfect for this use case
2. **better-sqlite3**: Synchronous API that's faster and simpler than the async sqlite3 package
3. **Auto-timestamps**: Using SQLite's CURRENT_TIMESTAMP and triggers for automatic timestamp management
4. **Boolean Handling**: SQLite doesn't have a boolean type, so we use 0/1 for false/true

### Model Design Patterns

1. **Object Pattern**: Using an object with methods rather than a class for simplicity
2. **Synchronous Operations**: Leveraging better-sqlite3's synchronous API
3. **Prepared Statements**: All queries use prepared statements to prevent SQL injection
4. **Consistent Return Values**: Methods return predictable types (object, array, boolean, null)

## Testing Strategy

See the [test implementation guide](../task-7/task.md) for detailed testing approach. Key test areas:

1. Database initialization and table creation
2. CRUD operations with valid data
3. Edge cases (non-existent IDs, empty values)
4. Filtering and pagination
5. Data integrity and constraints

## Success Criteria

1. ✅ Database file is created in the `data` directory
2. ✅ Todos table is created with correct schema
3. ✅ All CRUD operations work correctly
4. ✅ Filtering by completion status works
5. ✅ Pagination (limit/offset) works correctly
6. ✅ Timestamps are automatically managed
7. ✅ SQL injection is prevented through prepared statements

## Common Issues and Solutions

1. **Database locked error**: Ensure only one connection is open to the database
2. **Module not found**: Verify better-sqlite3 is installed correctly
3. **Permission denied**: Check write permissions on the data directory
4. **Build errors**: SQLite3 requires build tools - install them for your OS

## Next Steps

After completing this task:
- Task 4: Implement Todo Controller (depends on this model)
- Task 7: Write Comprehensive Tests (to test this model)

The model is now ready to be used by the controller layer to handle HTTP requests.