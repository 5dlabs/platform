# Task 7: Write Comprehensive Tests

## Overview
This task implements a comprehensive test suite for the Simple Todo REST API, including unit tests for models and controllers, integration tests for API endpoints, and configuration for test coverage reporting. The goal is to achieve at least 90% code coverage while ensuring all critical paths are tested.

## Task Details

### Priority
High

### Dependencies
- Task 2: Database Setup and Model Implementation (need models to test)
- Task 3: Express Application and Middleware (need middleware to test)
- Task 4: Todo Controller (need controllers to test)
- Task 5: API Routes (need routes for integration tests)

### Status
Pending

## Implementation Guide

### 1. Create Test Directory Structure

```bash
mkdir -p tests/unit/{models,controllers,middleware}
mkdir -p tests/integration
mkdir -p tests/fixtures
```

### 2. Create Test Database Setup

**File: `tests/setup/testDb.js`**
```javascript
const Database = require('better-sqlite3');

// Create in-memory database for testing
const createTestDb = () => {
  const db = new Database(':memory:', {
    verbose: process.env.DEBUG_SQL ? console.log : null
  });

  // Create tables
  db.exec(`
    CREATE TABLE IF NOT EXISTS todos (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      title TEXT NOT NULL CHECK(length(title) <= 200),
      description TEXT CHECK(length(description) <= 1000),
      completed INTEGER NOT NULL DEFAULT 0 CHECK(completed IN (0, 1)),
      createdAt TEXT NOT NULL DEFAULT (datetime('now')),
      updatedAt TEXT NOT NULL DEFAULT (datetime('now'))
    )
  `);

  // Create update trigger
  db.exec(`
    CREATE TRIGGER IF NOT EXISTS update_todos_timestamp
    AFTER UPDATE ON todos
    FOR EACH ROW
    BEGIN
      UPDATE todos SET updatedAt = datetime('now') WHERE id = NEW.id;
    END
  `);

  return db;
};

module.exports = createTestDb;
```

**File: `tests/setup/jest.setup.js`**
```javascript
// Set test environment
process.env.NODE_ENV = 'test';
process.env.PORT = '0'; // Use random port for tests

// Increase timeout for integration tests
jest.setTimeout(10000);

// Global test utilities
global.testUtils = {
  // Create a test todo object
  createTestTodo: (overrides = {}) => ({
    title: 'Test Todo',
    description: 'Test Description',
    completed: false,
    ...overrides
  }),
  
  // Wait for async operations
  wait: (ms) => new Promise(resolve => setTimeout(resolve, ms))
};
```

### 3. Create Model Unit Tests

**File: `tests/unit/models/todo.test.js`**
```javascript
const createTestDb = require('../../setup/testDb');

// Mock the database module
let testDb;
jest.mock('../../../src/models/db', () => {
  testDb = createTestDb();
  return testDb;
});

const Todo = require('../../../src/models/todo');

describe('Todo Model', () => {
  beforeEach(() => {
    // Clear all todos before each test
    testDb.exec('DELETE FROM todos');
  });

  afterAll(() => {
    testDb.close();
  });

  describe('create', () => {
    test('should create a new todo with required fields', () => {
      const todoData = {
        title: 'Test Todo',
        description: 'Test Description'
      };

      const todo = Todo.create(todoData);

      expect(todo).toMatchObject({
        id: expect.any(Number),
        title: todoData.title,
        description: todoData.description,
        completed: false,
        createdAt: expect.any(String),
        updatedAt: expect.any(String)
      });
    });

    test('should create todo without description', () => {
      const todo = Todo.create({ title: 'No Description' });
      
      expect(todo.title).toBe('No Description');
      expect(todo.description).toBeNull();
    });

    test('should throw error for missing title', () => {
      expect(() => {
        Todo.create({ description: 'No Title' });
      }).toThrow();
    });

    test('should throw error for title exceeding max length', () => {
      expect(() => {
        Todo.create({ title: 'x'.repeat(201) });
      }).toThrow(/CHECK constraint failed/);
    });
  });

  describe('findAll', () => {
    beforeEach(() => {
      // Create test todos
      Todo.create({ title: 'Todo 1', completed: false });
      Todo.create({ title: 'Todo 2', completed: true });
      Todo.create({ title: 'Todo 3', completed: false });
      Todo.create({ title: 'Todo 4', completed: true });
      Todo.create({ title: 'Todo 5', completed: false });
    });

    test('should return all todos when no filters', () => {
      const todos = Todo.findAll();
      expect(todos).toHaveLength(5);
    });

    test('should filter by completed status', () => {
      const completed = Todo.findAll({ completed: true });
      const pending = Todo.findAll({ completed: false });
      
      expect(completed).toHaveLength(2);
      expect(pending).toHaveLength(3);
      expect(completed.every(t => t.completed === true)).toBe(true);
    });

    test('should respect limit parameter', () => {
      const todos = Todo.findAll({ limit: 3 });
      expect(todos).toHaveLength(3);
    });

    test('should respect offset parameter', () => {
      const todos = Todo.findAll({ offset: 3 });
      expect(todos).toHaveLength(2);
    });

    test('should combine filters, limit, and offset', () => {
      const todos = Todo.findAll({ 
        completed: false, 
        limit: 2, 
        offset: 1 
      });
      expect(todos).toHaveLength(2);
    });

    test('should order by id DESC', () => {
      const todos = Todo.findAll();
      const ids = todos.map(t => t.id);
      const sortedIds = [...ids].sort((a, b) => b - a);
      expect(ids).toEqual(sortedIds);
    });
  });

  describe('findById', () => {
    test('should find existing todo', () => {
      const created = Todo.create({ title: 'Find Me' });
      const found = Todo.findById(created.id);
      
      expect(found).toEqual(created);
    });

    test('should return null for non-existent id', () => {
      const found = Todo.findById(9999);
      expect(found).toBeNull();
    });

    test('should convert completed to boolean', () => {
      const todo = Todo.create({ title: 'Test' });
      const found = Todo.findById(todo.id);
      
      expect(typeof found.completed).toBe('boolean');
    });
  });

  describe('update', () => {
    let todo;
    
    beforeEach(() => {
      todo = Todo.create({ 
        title: 'Original', 
        description: 'Original Desc' 
      });
    });

    test('should update title', () => {
      const updated = Todo.update(todo.id, { title: 'Updated' });
      
      expect(updated.title).toBe('Updated');
      expect(updated.description).toBe('Original Desc');
    });

    test('should update description', () => {
      const updated = Todo.update(todo.id, { 
        description: 'Updated Desc' 
      });
      
      expect(updated.description).toBe('Updated Desc');
    });

    test('should update completed status', () => {
      const updated = Todo.update(todo.id, { completed: true });
      
      expect(updated.completed).toBe(true);
    });

    test('should update multiple fields', () => {
      const updated = Todo.update(todo.id, {
        title: 'New Title',
        description: 'New Desc',
        completed: true
      });
      
      expect(updated).toMatchObject({
        title: 'New Title',
        description: 'New Desc',
        completed: true
      });
    });

    test('should update updatedAt timestamp', async () => {
      const original = Todo.findById(todo.id);
      
      // Wait to ensure timestamp difference
      await global.testUtils.wait(10);
      
      const updated = Todo.update(todo.id, { title: 'Updated' });
      
      expect(updated.updatedAt).not.toBe(original.updatedAt);
    });

    test('should return null for non-existent id', () => {
      const result = Todo.update(9999, { title: 'Updated' });
      expect(result).toBeNull();
    });

    test('should handle empty updates', () => {
      const result = Todo.update(todo.id, {});
      expect(result).toEqual(todo);
    });
  });

  describe('delete', () => {
    test('should delete existing todo', () => {
      const todo = Todo.create({ title: 'Delete Me' });
      const result = Todo.delete(todo.id);
      
      expect(result).toBe(true);
      expect(Todo.findById(todo.id)).toBeNull();
    });

    test('should return false for non-existent id', () => {
      const result = Todo.delete(9999);
      expect(result).toBe(false);
    });
  });

  describe('deleteAll', () => {
    test('should delete all todos', () => {
      // Create multiple todos
      Todo.create({ title: 'Todo 1' });
      Todo.create({ title: 'Todo 2' });
      Todo.create({ title: 'Todo 3' });
      
      const deletedCount = Todo.deleteAll();
      
      expect(deletedCount).toBe(3);
      expect(Todo.findAll()).toHaveLength(0);
    });

    test('should return 0 when no todos exist', () => {
      const deletedCount = Todo.deleteAll();
      expect(deletedCount).toBe(0);
    });
  });

  describe('count', () => {
    beforeEach(() => {
      Todo.create({ title: 'Todo 1', completed: true });
      Todo.create({ title: 'Todo 2', completed: false });
      Todo.create({ title: 'Todo 3', completed: true });
    });

    test('should count all todos', () => {
      const count = Todo.count();
      expect(count).toBe(3);
    });

    test('should count completed todos', () => {
      const count = Todo.count({ completed: true });
      expect(count).toBe(2);
    });

    test('should count pending todos', () => {
      const count = Todo.count({ completed: false });
      expect(count).toBe(1);
    });
  });
});
```

### 4. Create Controller Unit Tests

**File: `tests/unit/controllers/todoController.test.js`**
```javascript
// Mock the models
jest.mock('../../../src/models', () => ({
  Todo: {
    findAll: jest.fn(),
    findById: jest.fn(),
    create: jest.fn(),
    update: jest.fn(),
    delete: jest.fn(),
    count: jest.fn()
  }
}));

const { todoController } = require('../../../src/controllers');
const { Todo } = require('../../../src/models');

describe('Todo Controller', () => {
  let req, res, next;

  beforeEach(() => {
    // Reset mocks
    jest.clearAllMocks();
    
    // Setup request, response, and next
    req = {
      params: {},
      query: {},
      body: {}
    };
    res = {
      status: jest.fn().mockReturnThis(),
      json: jest.fn().mockReturnThis(),
      end: jest.fn()
    };
    next = jest.fn();
  });

  describe('getAllTodos', () => {
    test('should return all todos', () => {
      const mockTodos = [
        { id: 1, title: 'Todo 1' },
        { id: 2, title: 'Todo 2' }
      ];
      Todo.findAll.mockReturnValue(mockTodos);

      todoController.getAllTodos(req, res, next);

      expect(Todo.findAll).toHaveBeenCalledWith({
        limit: 100,
        offset: 0
      });
      expect(res.status).toHaveBeenCalledWith(200);
      expect(res.json).toHaveBeenCalledWith(mockTodos);
    });

    test('should handle query parameters', () => {
      req.query = {
        completed: 'true',
        limit: '10',
        offset: '20'
      };

      todoController.getAllTodos(req, res, next);

      expect(Todo.findAll).toHaveBeenCalledWith({
        completed: true,
        limit: 10,
        offset: 20
      });
    });

    test('should handle errors', () => {
      const error = new Error('Database error');
      Todo.findAll.mockImplementation(() => {
        throw error;
      });

      todoController.getAllTodos(req, res, next);

      expect(next).toHaveBeenCalledWith(error);
    });
  });

  describe('getTodoById', () => {
    test('should return todo when found', () => {
      const mockTodo = { id: 1, title: 'Test Todo' };
      Todo.findById.mockReturnValue(mockTodo);
      req.params.id = '1';

      todoController.getTodoById(req, res, next);

      expect(Todo.findById).toHaveBeenCalledWith(1);
      expect(res.status).toHaveBeenCalledWith(200);
      expect(res.json).toHaveBeenCalledWith(mockTodo);
    });

    test('should return 404 when not found', () => {
      Todo.findById.mockReturnValue(null);
      req.params.id = '999';

      todoController.getTodoById(req, res, next);

      expect(next).toHaveBeenCalledWith(
        expect.objectContaining({
          status: 404,
          code: 'TODO_NOT_FOUND'
        })
      );
    });
  });

  describe('createTodo', () => {
    test('should create todo successfully', () => {
      const todoData = { title: 'New Todo', description: 'Desc' };
      const mockTodo = { id: 1, ...todoData, completed: false };
      Todo.create.mockReturnValue(mockTodo);
      req.body = todoData;

      todoController.createTodo(req, res, next);

      expect(Todo.create).toHaveBeenCalledWith({
        title: 'New Todo',
        description: 'Desc'
      });
      expect(res.status).toHaveBeenCalledWith(201);
      expect(res.json).toHaveBeenCalledWith(mockTodo);
    });

    test('should handle constraint errors', () => {
      const error = new Error('Constraint failed');
      error.code = 'SQLITE_CONSTRAINT';
      Todo.create.mockImplementation(() => {
        throw error;
      });
      req.body = { title: 'x'.repeat(201) };

      todoController.createTodo(req, res, next);

      expect(next).toHaveBeenCalledWith(
        expect.objectContaining({
          status: 400,
          message: 'Invalid todo data'
        })
      );
    });
  });

  describe('updateTodo', () => {
    test('should update todo successfully', () => {
      const updates = { title: 'Updated', completed: true };
      const mockTodo = { id: 1, ...updates };
      Todo.update.mockReturnValue(mockTodo);
      req.params.id = '1';
      req.body = updates;

      todoController.updateTodo(req, res, next);

      expect(Todo.update).toHaveBeenCalledWith(1, updates);
      expect(res.status).toHaveBeenCalledWith(200);
      expect(res.json).toHaveBeenCalledWith(mockTodo);
    });

    test('should return 404 when todo not found', () => {
      Todo.update.mockReturnValue(null);
      req.params.id = '999';
      req.body = { title: 'Updated' };

      todoController.updateTodo(req, res, next);

      expect(next).toHaveBeenCalledWith(
        expect.objectContaining({
          status: 404,
          code: 'TODO_NOT_FOUND'
        })
      );
    });

    test('should only update provided fields', () => {
      req.params.id = '1';
      req.body = { completed: true };

      todoController.updateTodo(req, res, next);

      expect(Todo.update).toHaveBeenCalledWith(1, { completed: true });
    });
  });

  describe('deleteTodo', () => {
    test('should delete todo successfully', () => {
      Todo.delete.mockReturnValue(true);
      req.params.id = '1';

      todoController.deleteTodo(req, res, next);

      expect(Todo.delete).toHaveBeenCalledWith(1);
      expect(res.status).toHaveBeenCalledWith(204);
      expect(res.end).toHaveBeenCalled();
    });

    test('should return 404 when todo not found', () => {
      Todo.delete.mockReturnValue(false);
      req.params.id = '999';

      todoController.deleteTodo(req, res, next);

      expect(next).toHaveBeenCalledWith(
        expect.objectContaining({
          status: 404,
          code: 'TODO_NOT_FOUND'
        })
      );
    });
  });

  describe('getTodoStats', () => {
    test('should return statistics', () => {
      Todo.count.mockReturnValueOnce(10); // total
      Todo.count.mockReturnValueOnce(3);  // completed

      todoController.getTodoStats(req, res, next);

      expect(Todo.count).toHaveBeenCalledTimes(2);
      expect(Todo.count).toHaveBeenNthCalledWith(1);
      expect(Todo.count).toHaveBeenNthCalledWith(2, { completed: true });
      expect(res.json).toHaveBeenCalledWith({
        total: 10,
        completed: 3,
        pending: 7,
        completionRate: 0.3
      });
    });

    test('should handle zero todos', () => {
      Todo.count.mockReturnValue(0);

      todoController.getTodoStats(req, res, next);

      expect(res.json).toHaveBeenCalledWith({
        total: 0,
        completed: 0,
        pending: 0,
        completionRate: 0
      });
    });
  });
});
```

### 5. Create Integration Tests

**File: `tests/integration/todos.test.js`**
```javascript
const request = require('supertest');
const createTestDb = require('../setup/testDb');

// Mock the database before importing app
let testDb;
jest.mock('../../src/models/db', () => {
  testDb = createTestDb();
  return testDb;
});

const app = require('../../src/app');
const { Todo } = require('../../src/models');

describe('Todo API Integration Tests', () => {
  beforeEach(() => {
    // Clear database before each test
    testDb.exec('DELETE FROM todos');
  });

  afterAll(() => {
    testDb.close();
  });

  describe('GET /api/todos', () => {
    test('should return empty array when no todos', async () => {
      const response = await request(app)
        .get('/api/todos')
        .expect(200);

      expect(response.body).toEqual([]);
    });

    test('should return all todos', async () => {
      // Create test todos
      Todo.create({ title: 'Todo 1' });
      Todo.create({ title: 'Todo 2' });

      const response = await request(app)
        .get('/api/todos')
        .expect(200);

      expect(response.body).toHaveLength(2);
      expect(response.body[0]).toHaveProperty('title');
      expect(response.body[0]).toHaveProperty('completed');
    });

    test('should filter by completed status', async () => {
      Todo.create({ title: 'Completed', completed: true });
      Todo.create({ title: 'Pending', completed: false });

      const response = await request(app)
        .get('/api/todos?completed=true')
        .expect(200);

      expect(response.body).toHaveLength(1);
      expect(response.body[0].title).toBe('Completed');
    });

    test('should validate query parameters', async () => {
      const response = await request(app)
        .get('/api/todos?limit=invalid')
        .expect(400);

      expect(response.body.error.code).toBe('VALIDATION_ERROR');
    });
  });

  describe('GET /api/todos/:id', () => {
    test('should return todo by id', async () => {
      const todo = Todo.create({ title: 'Find Me' });

      const response = await request(app)
        .get(`/api/todos/${todo.id}`)
        .expect(200);

      expect(response.body).toMatchObject({
        id: todo.id,
        title: 'Find Me'
      });
    });

    test('should return 404 for non-existent todo', async () => {
      const response = await request(app)
        .get('/api/todos/999')
        .expect(404);

      expect(response.body.error.code).toBe('TODO_NOT_FOUND');
    });

    test('should validate id parameter', async () => {
      const response = await request(app)
        .get('/api/todos/invalid')
        .expect(400);

      expect(response.body.error.code).toBe('VALIDATION_ERROR');
    });
  });

  describe('POST /api/todos', () => {
    test('should create new todo', async () => {
      const todoData = {
        title: 'New Todo',
        description: 'Test Description'
      };

      const response = await request(app)
        .post('/api/todos')
        .send(todoData)
        .expect(201);

      expect(response.body).toMatchObject({
        id: expect.any(Number),
        title: todoData.title,
        description: todoData.description,
        completed: false
      });

      // Verify it was saved
      const saved = Todo.findById(response.body.id);
      expect(saved).toBeTruthy();
    });

    test('should create todo without description', async () => {
      const response = await request(app)
        .post('/api/todos')
        .send({ title: 'No Description' })
        .expect(201);

      expect(response.body.description).toBeNull();
    });

    test('should require title', async () => {
      const response = await request(app)
        .post('/api/todos')
        .send({ description: 'No Title' })
        .expect(400);

      expect(response.body.error.code).toBe('VALIDATION_ERROR');
      expect(response.body.error.details).toContainEqual(
        expect.objectContaining({
          field: 'title',
          message: 'Title is required'
        })
      );
    });

    test('should validate title length', async () => {
      const response = await request(app)
        .post('/api/todos')
        .send({ title: 'x'.repeat(201) })
        .expect(400);

      expect(response.body.error.code).toBe('VALIDATION_ERROR');
    });
  });

  describe('PUT /api/todos/:id', () => {
    let todo;

    beforeEach(() => {
      todo = Todo.create({
        title: 'Original',
        description: 'Original Desc',
        completed: false
      });
    });

    test('should update todo', async () => {
      const updates = {
        title: 'Updated',
        completed: true
      };

      const response = await request(app)
        .put(`/api/todos/${todo.id}`)
        .send(updates)
        .expect(200);

      expect(response.body).toMatchObject({
        id: todo.id,
        title: 'Updated',
        description: 'Original Desc',
        completed: true
      });
    });

    test('should update only provided fields', async () => {
      const response = await request(app)
        .put(`/api/todos/${todo.id}`)
        .send({ completed: true })
        .expect(200);

      expect(response.body.title).toBe('Original');
      expect(response.body.completed).toBe(true);
    });

    test('should return 404 for non-existent todo', async () => {
      const response = await request(app)
        .put('/api/todos/999')
        .send({ title: 'Updated' })
        .expect(404);

      expect(response.body.error.code).toBe('TODO_NOT_FOUND');
    });

    test('should validate updates', async () => {
      const response = await request(app)
        .put(`/api/todos/${todo.id}`)
        .send({ title: '' })
        .expect(400);

      expect(response.body.error.code).toBe('VALIDATION_ERROR');
    });
  });

  describe('DELETE /api/todos/:id', () => {
    test('should delete todo', async () => {
      const todo = Todo.create({ title: 'Delete Me' });

      await request(app)
        .delete(`/api/todos/${todo.id}`)
        .expect(204);

      // Verify it was deleted
      expect(Todo.findById(todo.id)).toBeNull();
    });

    test('should return 404 for non-existent todo', async () => {
      const response = await request(app)
        .delete('/api/todos/999')
        .expect(404);

      expect(response.body.error.code).toBe('TODO_NOT_FOUND');
    });
  });

  describe('GET /api/todos/stats', () => {
    test('should return statistics', async () => {
      // Create test data
      Todo.create({ title: 'Todo 1', completed: true });
      Todo.create({ title: 'Todo 2', completed: false });
      Todo.create({ title: 'Todo 3', completed: true });

      const response = await request(app)
        .get('/api/todos/stats')
        .expect(200);

      expect(response.body).toEqual({
        total: 3,
        completed: 2,
        pending: 1,
        completionRate: 2/3
      });
    });
  });
});

describe('Health Check Integration Tests', () => {
  test('GET /api/health should return ok', async () => {
    const response = await request(app)
      .get('/api/health')
      .expect(200);

    expect(response.body).toMatchObject({
      status: 'ok',
      timestamp: expect.any(String),
      uptime: expect.any(Number),
      environment: 'test'
    });
  });

  test('GET /api/health/detailed should check database', async () => {
    const response = await request(app)
      .get('/api/health/detailed')
      .expect(200);

    expect(response.body).toMatchObject({
      status: 'ok',
      checks: {
        database: {
          status: 'healthy',
          message: 'Connected'
        }
      }
    });
  });
});

describe('API Information', () => {
  test('GET /api should return API info', async () => {
    const response = await request(app)
      .get('/api')
      .expect(200);

    expect(response.body).toMatchObject({
      message: 'Simple Todo REST API',
      version: '1.0.0',
      endpoints: expect.objectContaining({
        todos: '/api/todos',
        health: '/api/health',
        documentation: '/api-docs'
      })
    });
  });
});

describe('Error Handling', () => {
  test('should return 404 for unknown routes', async () => {
    const response = await request(app)
      .get('/api/unknown')
      .expect(404);

    expect(response.body.error.code).toBe('NOT_FOUND');
  });

  test('should handle JSON parsing errors', async () => {
    const response = await request(app)
      .post('/api/todos')
      .set('Content-Type', 'application/json')
      .send('invalid json')
      .expect(400);

    expect(response.body).toHaveProperty('error');
  });
});
```

### 6. Create Middleware Tests

**File: `tests/unit/middleware/validation.test.js`**
```javascript
const { validationResult } = require('express-validator');
const { todoValidation, handleValidationErrors } = require('../../../src/middleware/validation');

// Mock express-validator
jest.mock('express-validator', () => {
  const actual = jest.requireActual('express-validator');
  return {
    ...actual,
    validationResult: jest.fn()
  };
});

describe('Validation Middleware', () => {
  describe('handleValidationErrors', () => {
    let req, res, next;

    beforeEach(() => {
      req = {};
      res = {
        status: jest.fn().mockReturnThis(),
        json: jest.fn()
      };
      next = jest.fn();
    });

    test('should call next when no errors', () => {
      validationResult.mockReturnValue({
        isEmpty: () => true
      });

      handleValidationErrors(req, res, next);

      expect(next).toHaveBeenCalled();
      expect(res.status).not.toHaveBeenCalled();
    });

    test('should return 400 with errors', () => {
      const errors = [
        {
          path: 'title',
          msg: 'Title is required',
          value: undefined
        }
      ];

      validationResult.mockReturnValue({
        isEmpty: () => false,
        array: () => errors
      });

      handleValidationErrors(req, res, next);

      expect(res.status).toHaveBeenCalledWith(400);
      expect(res.json).toHaveBeenCalledWith({
        error: {
          message: 'Validation failed',
          code: 'VALIDATION_ERROR',
          details: [
            {
              field: 'title',
              message: 'Title is required',
              value: undefined
            }
          ]
        }
      });
      expect(next).not.toHaveBeenCalled();
    });
  });

  describe('todoValidation rules', () => {
    test('should have create validation rules', () => {
      expect(todoValidation.create).toBeDefined();
      expect(Array.isArray(todoValidation.create)).toBe(true);
      expect(todoValidation.create.length).toBeGreaterThan(0);
    });

    test('should have update validation rules', () => {
      expect(todoValidation.update).toBeDefined();
      expect(Array.isArray(todoValidation.update)).toBe(true);
    });

    test('should have list validation rules', () => {
      expect(todoValidation.list).toBeDefined();
      expect(Array.isArray(todoValidation.list)).toBe(true);
    });
  });
});
```

### 7. Create Jest Configuration

**File: `jest.config.js`**
```javascript
module.exports = {
  // Test environment
  testEnvironment: 'node',
  
  // Setup files
  setupFilesAfterEnv: ['<rootDir>/tests/setup/jest.setup.js'],
  
  // Test match patterns
  testMatch: [
    '**/tests/**/*.test.js'
  ],
  
  // Coverage configuration
  collectCoverage: true,
  collectCoverageFrom: [
    'src/**/*.js',
    '!src/app.js', // Exclude app setup file
    '!**/node_modules/**'
  ],
  coverageDirectory: 'coverage',
  coverageReporters: ['text', 'lcov', 'html'],
  
  // Coverage thresholds
  coverageThreshold: {
    global: {
      branches: 90,
      functions: 90,
      lines: 90,
      statements: 90
    }
  },
  
  // Module paths
  modulePaths: ['<rootDir>'],
  
  // Clear mocks between tests
  clearMocks: true,
  
  // Verbose output
  verbose: true
};
```

### 8. Update Package.json Scripts

**Update `package.json`**:
```json
{
  "scripts": {
    "test": "jest --coverage",
    "test:watch": "jest --watch",
    "test:unit": "jest tests/unit",
    "test:integration": "jest tests/integration",
    "test:coverage": "jest --coverage --coverageReporters=text-lcov | coveralls"
  }
}
```

## Key Testing Strategies

### Unit Testing Approach
- Mock all external dependencies
- Test each function in isolation
- Cover all code paths and edge cases
- Test error scenarios thoroughly

### Integration Testing Approach
- Use in-memory database for speed
- Test complete request/response cycles
- Verify middleware integration
- Test error handling end-to-end

### Coverage Requirements
- Minimum 90% overall coverage
- All critical paths must be tested
- Error handling must be covered
- Edge cases documented and tested

## Common Testing Patterns

### Mocking Database
```javascript
jest.mock('../src/models/db', () => createTestDb());
```

### Testing Async Operations
```javascript
test('async operation', async () => {
  const response = await request(app).get('/api/todos');
  expect(response.status).toBe(200);
});
```

### Testing Errors
```javascript
test('handles errors', () => {
  Todo.findAll.mockImplementation(() => {
    throw new Error('Database error');
  });
  // Test error handling
});
```

## Next Steps
After completing this task:
1. Run tests with `npm test`
2. Review coverage report in `coverage/index.html`
3. Add any missing tests to reach 90% coverage
4. Ensure all tests pass consistently
5. Proceed to Task 8: Finalize and Document Project

## References
- [Jest Documentation](https://jestjs.io/docs/getting-started)
- [Supertest Documentation](https://github.com/visionmedia/supertest)
- [Testing Best Practices](https://github.com/goldbergyoni/javascript-testing-best-practices)