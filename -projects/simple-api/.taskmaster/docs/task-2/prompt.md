# Task 2: Database Setup and Model Implementation - Autonomous Prompt

You are an AI agent tasked with implementing the database layer for the Simple Todo REST API. Your goal is to create the SQLite database infrastructure and implement the Todo model with complete CRUD operations.

## Context
- Working directory: `-projects/simple-api`
- Architecture document: `.taskmaster/docs/architecture.md`
- Product requirements: `.taskmaster/docs/prd.txt`
- Task 1 (Project Setup) has been completed
- Dependencies including better-sqlite3 are already installed

## Your Mission
Implement the complete database layer including SQLite database setup, table creation with constraints, and a Todo model with all CRUD operations. Ensure the implementation follows the architecture's model layer specifications and data model requirements.

## Required Actions

### 1. Create Database Connection Module
Create `src/models/db.js`:
- Import better-sqlite3
- Create data directory if it doesn't exist
- Initialize SQLite database at `data/todos.db`
- Enable foreign keys and WAL mode for performance
- Create todos table with proper schema
- Create trigger for automatic updatedAt timestamp

Table schema requirements:
- id: INTEGER PRIMARY KEY AUTOINCREMENT
- title: TEXT NOT NULL with max length 200
- description: TEXT with max length 1000
- completed: INTEGER (boolean) DEFAULT 0
- createdAt: TEXT DEFAULT CURRENT_TIMESTAMP
- updatedAt: TEXT DEFAULT CURRENT_TIMESTAMP

### 2. Implement Todo Model
Create `src/models/todo.js` with the following methods:

**findAll(filters)**: 
- Support optional filters: completed, limit, offset
- Return array of todos ordered by id DESC
- Handle pagination correctly

**findById(id)**:
- Return single todo or undefined if not found

**create({ title, description })**:
- Insert new todo
- Return created todo with generated id

**update(id, updates)**:
- Update only provided fields
- Return updated todo or null if not found
- Support updating: title, description, completed

**delete(id)**:
- Delete todo by id
- Return true if deleted, false if not found

**count(filters)**:
- Return count of todos
- Support optional completed filter

### 3. Create Model Index
Create `src/models/index.js`:
- Export Todo model
- Export database connection

### 4. Database Features to Implement
- Automatic timestamp management
- Field validation through CHECK constraints
- Proper error handling for constraint violations
- Synchronous operations using better-sqlite3
- Prepared statements for all queries

## Validation Criteria
- Database file is created at `data/todos.db`
- Table schema matches specifications exactly
- All CRUD operations work correctly
- Field constraints are enforced (max lengths)
- Timestamps update automatically
- Update trigger works properly
- Model methods handle edge cases gracefully
- Prepared statements are used for performance

## Important Notes
- Use synchronous better-sqlite3 API (not callbacks/promises)
- Implement proper null handling for optional fields
- Ensure completed field uses 0/1 for false/true
- Follow the exact schema from architecture document
- Handle SQLite type conversions correctly
- Return consistent data structures from all methods

## Testing the Implementation
After implementation, verify:
1. Database file exists and table is created
2. Can create todos with and without description
3. Field length constraints are enforced
4. Timestamps are auto-generated
5. Updates only modify specified fields
6. updatedAt changes on update
7. Filtering and pagination work correctly
8. Delete operations work as expected

## Expected Outcome
A complete database layer with:
- SQLite database properly configured
- Todo table with all constraints
- Full CRUD model implementation
- Automatic timestamp management
- Proper error handling
- Ready for use by controllers in Task 4

Execute all steps and ensure each component works correctly before considering the task complete.