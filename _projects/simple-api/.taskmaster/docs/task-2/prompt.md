# Task 2: Database Setup and Model Implementation - Autonomous Prompt

You are tasked with implementing the database layer for a Simple Todo REST API using SQLite and better-sqlite3. This includes creating the database connection, defining the schema, and implementing a complete Todo model with CRUD operations.

## Your Mission

Create a robust data persistence layer that:
1. Establishes a SQLite database connection
2. Creates the todos table with proper schema
3. Implements a Todo model with all CRUD operations
4. Handles data validation and error cases appropriately

## Required Implementations

### 1. Database Connection Module (`src/models/db.js`)

Create a module that:
- Ensures the `data` directory exists
- Creates/opens the SQLite database file at `data/todos.db`
- Creates the todos table if it doesn't exist
- Sets up automatic timestamp updates

Schema requirements:
```sql
CREATE TABLE IF NOT EXISTS todos (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  title TEXT NOT NULL,
  description TEXT,
  completed BOOLEAN DEFAULT 0,
  createdAt TEXT DEFAULT CURRENT_TIMESTAMP,
  updatedAt TEXT DEFAULT CURRENT_TIMESTAMP
)
```

### 2. Todo Model (`src/models/todo.js`)

Implement these methods:

```javascript
Todo.findAll({ completed, limit, offset })
// Returns array of todos with optional filtering and pagination

Todo.findById(id)
// Returns single todo or undefined

Todo.create({ title, description })
// Creates new todo and returns it

Todo.update(id, { title, description, completed })
// Updates todo and returns updated version or null

Todo.delete(id)
// Deletes todo and returns boolean success

Todo.count({ completed })
// Returns count of todos with optional filter
```

### 3. Key Requirements

- Use prepared statements for all queries (security)
- Handle the boolean completed field as 0/1 in SQLite
- Maintain createdAt and updatedAt timestamps
- Return consistent data types from each method
- Support pagination with limit and offset
- Filter by completion status when requested

## Implementation Guidelines

1. **Error Handling**
   - Let database errors bubble up (don't catch them in the model)
   - Return null/undefined for not found cases
   - Validate required fields before database operations

2. **Performance**
   - Use synchronous better-sqlite3 API
   - Prepare statements for reuse
   - Add appropriate indexes if needed

3. **Data Integrity**
   - Ensure title is always required
   - Handle null/undefined description gracefully
   - Maintain timestamp accuracy

## Example Usage

```javascript
const Todo = require('./models/todo');

// Create
const newTodo = Todo.create({
  title: 'Buy groceries',
  description: 'Milk, eggs, bread'
});

// Read
const allTodos = Todo.findAll({ completed: false, limit: 10 });
const singleTodo = Todo.findById(1);

// Update
const updated = Todo.update(1, { completed: true });

// Delete
const success = Todo.delete(1);
```

## Success Verification

Test your implementation:
1. Database file is created at `data/todos.db`
2. Can create todos with just title
3. Can retrieve all todos
4. Can filter by completed status
5. Can paginate results
6. Can update any field
7. Can delete todos
8. Timestamps work correctly

## Important Notes

- Do NOT implement authentication or authorization
- Do NOT add fields beyond the specified schema
- Do NOT use async/await - better-sqlite3 is synchronous
- Ensure the data directory is created if it doesn't exist
- Let the controller layer handle HTTP responses

## Context from Previous Tasks

- Task 1 has set up the project and installed better-sqlite3
- The project structure has `src/models/` directory ready
- No existing database or model code exists yet

Once complete, the Todo model will be used by controllers (Task 4) to handle HTTP requests. Ensure your implementation is robust and follows the interface exactly as specified.