# Task 2: Database Setup and Model Implementation - Autonomous Prompt

You are an AI agent tasked with implementing the database layer for a Simple Todo REST API. You will create the SQLite database connection, schema, and a complete Todo model with CRUD operations.

## Context
- **Project**: Simple Todo REST API
- **Prerequisites**: Task 1 (Project Setup) must be completed
- **Database**: SQLite with better-sqlite3 driver
- **Working Directory**: Project root (simple-api/)
- **References**: 
  - Architecture: .taskmaster/docs/architecture.md (see Data Model section)
  - Requirements: .taskmaster/docs/prd.txt

## Your Mission

Implement a robust database layer with SQLite, including database initialization, schema creation, and a Todo model that provides all necessary CRUD operations with proper data validation and type conversion.

## Detailed Implementation Steps

1. **Create Database Connection Module** (`src/models/db.js`)
   - Import required modules: better-sqlite3, path, fs
   - Create data directory if it doesn't exist
   - Initialize SQLite database connection
   - Enable foreign keys and WAL mode for performance
   - Create todos table with proper constraints:
     - id: INTEGER PRIMARY KEY AUTOINCREMENT
     - title: TEXT NOT NULL with length check <= 200
     - description: TEXT with length check <= 1000
     - completed: INTEGER (0 or 1) DEFAULT 0
     - createdAt: TEXT DEFAULT current timestamp
     - updatedAt: TEXT DEFAULT current timestamp
   - Create trigger to auto-update updatedAt on modifications
   - Handle graceful shutdown

2. **Implement Todo Model** (`src/models/todo.js`)
   - Import the database connection
   - Implement these methods:
     - `findAll({ completed, limit, offset })` - List todos with filtering/pagination
     - `findById(id)` - Get single todo by ID
     - `create({ title, description })` - Create new todo
     - `update(id, updates)` - Update existing todo
     - `delete(id)` - Delete a todo
     - `deleteAll()` - Delete all todos (for testing)
     - `count({ completed })` - Count todos with optional filter
   - Convert SQLite integers (0/1) to JavaScript booleans for completed field
   - Use prepared statements for all queries
   - Return null for not-found resources

3. **Create Database Initialization Script** (`scripts/initDb.js`)
   - Make it executable with shebang `#!/usr/bin/env node`
   - Load environment variables with dotenv
   - Clear existing data
   - Seed 5 sample todos (mix of completed and pending)
   - Display summary of initialization

4. **Create Model Index** (`src/models/index.js`)
   - Export Todo model and db connection for convenient importing

## Code Quality Requirements

- Use synchronous better-sqlite3 API (not callbacks/promises)
- All SQL queries must use prepared statements
- Proper error handling - let SQLite errors bubble up
- Convert boolean values correctly between JS and SQLite
- Add JSDoc comments for all public methods
- Follow consistent code style

## Implementation Example Structure

```javascript
// Example of a model method
findAll({ completed, limit = 100, offset = 0 } = {}) {
  let query = 'SELECT * FROM todos';
  const params = [];
  
  // Build query dynamically based on filters
  // Use prepared statements
  // Convert SQLite integers to booleans
  // Return array of todo objects
}
```

## Success Criteria
- ✅ Database file created in data/todos.db
- ✅ Todos table created with all required columns
- ✅ Auto-update trigger works for updatedAt field
- ✅ All CRUD operations functional
- ✅ Boolean conversion works correctly
- ✅ Pagination and filtering work as expected
- ✅ Database constraints enforced (length limits)
- ✅ Initialization script seeds sample data
- ✅ Graceful shutdown handles database closing

## Testing Your Implementation

After implementation, test in Node.js REPL:
```javascript
const { Todo } = require('./src/models');

// Test create
const todo = Todo.create({ title: 'Test Todo', description: 'Test' });
console.log(todo); // Should have id, timestamps, completed: false

// Test find
const found = Todo.findById(todo.id);
console.log(found); // Should match created todo

// Test update
const updated = Todo.update(todo.id, { completed: true });
console.log(updated.completed); // Should be true

// Test list with filters
const completed = Todo.findAll({ completed: true });
console.log(completed.length); // Should show completed todos

// Test delete
const deleted = Todo.delete(todo.id);
console.log(deleted); // Should be true
```

## Important Notes
- Do NOT use async/await - better-sqlite3 is synchronous
- Do NOT create API endpoints yet - just the model layer
- Ensure data directory has proper write permissions
- Let constraint violations throw - don't catch them
- Use transactions if implementing batch operations

## Common Pitfalls to Avoid
1. Forgetting to convert completed field between integer and boolean
2. Not using prepared statements (security risk)
3. Catching and hiding SQLite errors
4. Forgetting to close database on process exit
5. Not enabling WAL mode for better concurrency

Remember: This is the foundation of data persistence. Ensure it's rock-solid as all API operations will depend on this model.