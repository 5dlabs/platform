# Task 7: Write Comprehensive Tests

## Overview

This task implements comprehensive unit and integration tests for all components of the Todo REST API. Following Test-Driven Development (TDD) principles, tests ensure code quality, prevent regressions, and maintain the required 90% code coverage specified in the PRD.

## Context

Building upon all previous tasks, this creates a robust test suite that validates every aspect of the API. Tests are organized into unit tests (testing individual components in isolation) and integration tests (testing the complete request/response cycle).

## Implementation Guide

### 1. Create Test Setup and Configuration

Create test/setup.js for shared test configuration:

```javascript
// Set test environment
process.env.NODE_ENV = 'test';
process.env.PORT = 0; // Use random port for tests

// Global test timeout
jest.setTimeout(10000);

// Suppress console logs during tests unless debugging
if (!process.env.DEBUG) {
  global.console = {
    ...console,
    log: jest.fn(),
    error: jest.fn(),
    warn: jest.fn(),
    info: jest.fn(),
    debug: jest.fn(),
  };
}
```

Create test/testDb.js for test database:

```javascript
const Database = require('better-sqlite3');

// Create in-memory database for tests
const testDb = new Database(':memory:');

// Initialize schema
testDb.exec(`
  CREATE TABLE IF NOT EXISTS todos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    completed BOOLEAN DEFAULT 0,
    createdAt TEXT DEFAULT CURRENT_TIMESTAMP,
    updatedAt TEXT DEFAULT CURRENT_TIMESTAMP
  )
`);

// Helper to reset database
const resetDatabase = () => {
  testDb.exec('DELETE FROM todos');
  testDb.exec("DELETE FROM sqlite_sequence WHERE name='todos'");
};

// Helper to seed database
const seedDatabase = (todos = []) => {
  const insert = testDb.prepare(
    'INSERT INTO todos (title, description, completed) VALUES (?, ?, ?)'
  );
  
  todos.forEach(todo => {
    insert.run(
      todo.title,
      todo.description || null,
      todo.completed ? 1 : 0
    );
  });
};

module.exports = {
  testDb,
  resetDatabase,
  seedDatabase
};
```

Update jest.config.js:

```javascript
module.exports = {
  testEnvironment: 'node',
  coverageDirectory: 'coverage',
  collectCoverageFrom: [
    'src/**/*.js',
    '!src/swagger/**/*.js', // Exclude documentation files
  ],
  coverageThreshold: {
    global: {
      branches: 90,
      functions: 90,
      lines: 90,
      statements: 90
    }
  },
  testMatch: [
    '**/tests/unit/**/*.test.js',
    '**/tests/integration/**/*.test.js'
  ],
  setupFilesAfterEnv: ['<rootDir>/tests/setup.js'],
  verbose: true,
  testPathIgnorePatterns: ['/node_modules/'],
  moduleNameMapper: {
    '^@/(.*)$': '<rootDir>/src/$1'
  }
};
```

### 2. Unit Tests for Models

Create tests/unit/models/todo.test.js:

```javascript
// Mock the database module before requiring Todo model
jest.mock('../../../src/models/db', () => require('../../testDb').testDb);

const Todo = require('../../../src/models/todo');
const { resetDatabase, seedDatabase } = require('../../testDb');

describe('Todo Model', () => {
  beforeEach(() => {
    resetDatabase();
  });

  describe('findAll', () => {
    test('should return all todos when no filters', () => {
      seedDatabase([
        { title: 'Todo 1', completed: false },
        { title: 'Todo 2', completed: true },
        { title: 'Todo 3', completed: false }
      ]);

      const todos = Todo.findAll();
      
      expect(todos).toHaveLength(3);
      expect(todos[0].title).toBe('Todo 3'); // Latest first
    });

    test('should filter by completed status', () => {
      seedDatabase([
        { title: 'Todo 1', completed: false },
        { title: 'Todo 2', completed: true },
        { title: 'Todo 3', completed: true }
      ]);

      const completed = Todo.findAll({ completed: true });
      const pending = Todo.findAll({ completed: false });
      
      expect(completed).toHaveLength(2);
      expect(pending).toHaveLength(1);
      expect(completed.every(t => t.completed === 1)).toBe(true);
    });

    test('should handle pagination', () => {
      // Create 10 todos
      const todos = Array.from({ length: 10 }, (_, i) => ({
        title: `Todo ${i + 1}`,
        completed: false
      }));
      seedDatabase(todos);

      const page1 = Todo.findAll({ limit: 5, offset: 0 });
      const page2 = Todo.findAll({ limit: 5, offset: 5 });
      
      expect(page1).toHaveLength(5);
      expect(page2).toHaveLength(5);
      expect(page1[0].title).toBe('Todo 10'); // Latest first
      expect(page2[0].title).toBe('Todo 5');
    });

    test('should handle empty database', () => {
      const todos = Todo.findAll();
      expect(todos).toEqual([]);
    });
  });

  describe('findById', () => {
    test('should return todo when exists', () => {
      seedDatabase([{ title: 'Test Todo', description: 'Test' }]);
      
      const todo = Todo.findById(1);
      
      expect(todo).toBeDefined();
      expect(todo.id).toBe(1);
      expect(todo.title).toBe('Test Todo');
      expect(todo.description).toBe('Test');
    });

    test('should return undefined when not exists', () => {
      const todo = Todo.findById(999);
      expect(todo).toBeUndefined();
    });

    test('should handle invalid ID types', () => {
      const todo = Todo.findById('abc');
      expect(todo).toBeUndefined();
    });
  });

  describe('create', () => {
    test('should create todo with title only', () => {
      const todo = Todo.create({ title: 'New Todo' });
      
      expect(todo.id).toBe(1);
      expect(todo.title).toBe('New Todo');
      expect(todo.description).toBeNull();
      expect(todo.completed).toBe(0);
      expect(todo.createdAt).toBeDefined();
    });

    test('should create todo with all fields', () => {
      const todo = Todo.create({
        title: 'Complete Todo',
        description: 'With description'
      });
      
      expect(todo.title).toBe('Complete Todo');
      expect(todo.description).toBe('With description');
    });

    test('should auto-increment IDs', () => {
      const todo1 = Todo.create({ title: 'First' });
      const todo2 = Todo.create({ title: 'Second' });
      
      expect(todo2.id).toBe(todo1.id + 1);
    });

    test('should handle database errors gracefully', () => {
      // Test with invalid data that would cause DB error
      expect(() => {
        Todo.create({ title: null });
      }).toThrow();
    });
  });

  describe('update', () => {
    test('should update existing todo', () => {
      seedDatabase([{ title: 'Original', completed: false }]);
      
      const updated = Todo.update(1, {
        title: 'Updated',
        completed: true
      });
      
      expect(updated.title).toBe('Updated');
      expect(updated.completed).toBe(1);
      expect(updated.updatedAt).not.toBe(updated.createdAt);
    });

    test('should update only provided fields', () => {
      seedDatabase([{
        title: 'Original',
        description: 'Original desc',
        completed: false
      }]);
      
      const updated = Todo.update(1, { title: 'Updated' });
      
      expect(updated.title).toBe('Updated');
      expect(updated.description).toBe('Original desc');
      expect(updated.completed).toBe(0);
    });

    test('should return null for non-existent todo', () => {
      const updated = Todo.update(999, { title: 'Updated' });
      expect(updated).toBeNull();
    });

    test('should handle empty updates', () => {
      seedDatabase([{ title: 'Original' }]);
      
      const updated = Todo.update(1, {});
      
      expect(updated.title).toBe('Original');
    });
  });

  describe('delete', () => {
    test('should delete existing todo', () => {
      seedDatabase([{ title: 'To Delete' }]);
      
      const result = Todo.delete(1);
      
      expect(result).toBe(true);
      expect(Todo.findById(1)).toBeUndefined();
    });

    test('should return false for non-existent todo', () => {
      const result = Todo.delete(999);
      expect(result).toBe(false);
    });

    test('should not affect other todos', () => {
      seedDatabase([
        { title: 'Todo 1' },
        { title: 'Todo 2' },
        { title: 'Todo 3' }
      ]);
      
      Todo.delete(2);
      
      const remaining = Todo.findAll();
      expect(remaining).toHaveLength(2);
      expect(remaining.find(t => t.id === 2)).toBeUndefined();
    });
  });
});
```

### 3. Unit Tests for Controllers

Create tests/unit/controllers/todoController.test.js:

```javascript
jest.mock('../../../src/models/todo');

const todoController = require('../../../src/controllers/todoController');
const Todo = require('../../../src/models/todo');

describe('Todo Controller', () => {
  let req, res, next;

  beforeEach(() => {
    req = {
      params: {},
      query: {},
      body: {}
    };
    res = {
      json: jest.fn(),
      status: jest.fn().mockReturnThis(),
      end: jest.fn()
    };
    next = jest.fn();
    jest.clearAllMocks();
  });

  describe('getAllTodos', () => {
    test('should return all todos with count', () => {
      const mockTodos = [
        { id: 1, title: 'Todo 1' },
        { id: 2, title: 'Todo 2' }
      ];
      Todo.findAll.mockReturnValue(mockTodos);

      todoController.getAllTodos(req, res, next);

      expect(Todo.findAll).toHaveBeenCalledWith({});
      expect(res.json).toHaveBeenCalledWith({
        data: mockTodos,
        count: 2,
        filters: {
          completed: undefined,
          limit: null,
          offset: null
        }
      });
    });

    test('should handle query parameters', () => {
      req.query = {
        completed: 'true',
        limit: '10',
        offset: '5'
      };
      Todo.findAll.mockReturnValue([]);

      todoController.getAllTodos(req, res, next);

      expect(Todo.findAll).toHaveBeenCalledWith({
        completed: true,
        limit: 10,
        offset: 5
      });
    });

    test('should handle errors', () => {
      const error = new Error('Database error');
      Todo.findAll.mockImplementation(() => {
        throw error;
      });

      todoController.getAllTodos(req, res, next);

      expect(next).toHaveBeenCalledWith(error);
      expect(res.json).not.toHaveBeenCalled();
    });
  });

  describe('getTodoById', () => {
    test('should return todo when found', () => {
      const mockTodo = { id: 1, title: 'Test Todo' };
      req.params.id = '1';
      Todo.findById.mockReturnValue(mockTodo);

      todoController.getTodoById(req, res, next);

      expect(Todo.findById).toHaveBeenCalledWith(1);
      expect(res.json).toHaveBeenCalledWith({
        data: mockTodo
      });
    });

    test('should return 404 when not found', () => {
      req.params.id = '999';
      Todo.findById.mockReturnValue(null);

      todoController.getTodoById(req, res, next);

      expect(next).toHaveBeenCalledWith(
        expect.objectContaining({
          message: 'Todo not found',
          status: 404
        })
      );
    });
  });

  describe('createTodo', () => {
    test('should create todo and return 201', () => {
      const mockTodo = { id: 1, title: 'New Todo' };
      req.body = { title: '  New Todo  ', description: '  Test  ' };
      Todo.create.mockReturnValue(mockTodo);

      todoController.createTodo(req, res, next);

      expect(Todo.create).toHaveBeenCalledWith({
        title: 'New Todo',
        description: 'Test'
      });
      expect(res.status).toHaveBeenCalledWith(201);
      expect(res.json).toHaveBeenCalledWith({
        message: 'Todo created successfully',
        data: mockTodo
      });
    });

    test('should handle missing description', () => {
      req.body = { title: 'Only Title' };
      Todo.create.mockReturnValue({ id: 1, title: 'Only Title' });

      todoController.createTodo(req, res, next);

      expect(Todo.create).toHaveBeenCalledWith({
        title: 'Only Title',
        description: null
      });
    });
  });

  describe('updateTodo', () => {
    test('should update todo when exists', () => {
      const mockTodo = { id: 1, title: 'Updated' };
      req.params.id = '1';
      req.body = { title: 'Updated', completed: true };
      Todo.update.mockReturnValue(mockTodo);

      todoController.updateTodo(req, res, next);

      expect(Todo.update).toHaveBeenCalledWith(1, {
        title: 'Updated',
        completed: true
      });
      expect(res.json).toHaveBeenCalledWith({
        message: 'Todo updated successfully',
        data: mockTodo
      });
    });

    test('should return 404 when not found', () => {
      req.params.id = '999';
      req.body = { title: 'Updated' };
      Todo.update.mockReturnValue(null);

      todoController.updateTodo(req, res, next);

      expect(next).toHaveBeenCalledWith(
        expect.objectContaining({
          message: 'Todo not found',
          status: 404
        })
      );
    });
  });

  describe('deleteTodo', () => {
    test('should delete todo and return 204', () => {
      req.params.id = '1';
      Todo.delete.mockReturnValue(true);

      todoController.deleteTodo(req, res, next);

      expect(Todo.delete).toHaveBeenCalledWith(1);
      expect(res.status).toHaveBeenCalledWith(204);
      expect(res.end).toHaveBeenCalled();
    });

    test('should return 404 when not found', () => {
      req.params.id = '999';
      Todo.delete.mockReturnValue(false);

      todoController.deleteTodo(req, res, next);

      expect(next).toHaveBeenCalledWith(
        expect.objectContaining({
          message: 'Todo not found',
          status: 404
        })
      );
    });
  });
});
```

### 4. Unit Tests for Middleware

Create tests/unit/middleware/validation.test.js:

```javascript
const { validationResult } = require('express-validator');
const { todoValidation } = require('../../../src/middleware/validation');

// Mock express-validator
jest.mock('express-validator');

describe('Validation Middleware', () => {
  describe('todoValidation', () => {
    test('should have validation rules for all operations', () => {
      expect(todoValidation.create).toBeDefined();
      expect(todoValidation.update).toBeDefined();
      expect(todoValidation.getOne).toBeDefined();
      expect(todoValidation.delete).toBeDefined();
      expect(todoValidation.list).toBeDefined();
    });

    test('should include validation handler in chain', () => {
      // Each validation array should have middleware functions
      expect(Array.isArray(todoValidation.create)).toBe(true);
      expect(todoValidation.create.length).toBeGreaterThan(0);
    });
  });
});
```

### 5. Integration Tests

Create tests/integration/todos.test.js:

```javascript
const request = require('supertest');
const app = require('../../src/app');
const { resetDatabase, seedDatabase } = require('../testDb');

// Mock the database module
jest.mock('../../src/models/db', () => require('../testDb').testDb);

describe('Todo API Integration Tests', () => {
  beforeEach(() => {
    resetDatabase();
  });

  describe('GET /api/todos', () => {
    test('should return empty array when no todos', async () => {
      const res = await request(app)
        .get('/api/todos')
        .expect(200);

      expect(res.body).toEqual({
        data: [],
        count: 0,
        filters: {
          completed: undefined,
          limit: null,
          offset: null
        }
      });
    });

    test('should return all todos', async () => {
      seedDatabase([
        { title: 'Todo 1' },
        { title: 'Todo 2' }
      ]);

      const res = await request(app)
        .get('/api/todos')
        .expect(200);

      expect(res.body.count).toBe(2);
      expect(res.body.data).toHaveLength(2);
    });

    test('should filter by completed status', async () => {
      seedDatabase([
        { title: 'Todo 1', completed: true },
        { title: 'Todo 2', completed: false }
      ]);

      const res = await request(app)
        .get('/api/todos?completed=true')
        .expect(200);

      expect(res.body.count).toBe(1);
      expect(res.body.data[0].completed).toBe(1);
    });

    test('should handle pagination', async () => {
      seedDatabase(
        Array.from({ length: 10 }, (_, i) => ({
          title: `Todo ${i + 1}`
        }))
      );

      const res = await request(app)
        .get('/api/todos?limit=5&offset=5')
        .expect(200);

      expect(res.body.data).toHaveLength(5);
    });

    test('should validate query parameters', async () => {
      const res = await request(app)
        .get('/api/todos?limit=invalid')
        .expect(400);

      expect(res.body.error).toBe('Validation Error');
    });
  });

  describe('POST /api/todos', () => {
    test('should create new todo', async () => {
      const res = await request(app)
        .post('/api/todos')
        .send({ title: 'New Todo' })
        .expect(201);

      expect(res.body.message).toBe('Todo created successfully');
      expect(res.body.data).toMatchObject({
        title: 'New Todo',
        completed: 0
      });
    });

    test('should create todo with description', async () => {
      const res = await request(app)
        .post('/api/todos')
        .send({
          title: 'New Todo',
          description: 'Test description'
        })
        .expect(201);

      expect(res.body.data.description).toBe('Test description');
    });

    test('should require title', async () => {
      const res = await request(app)
        .post('/api/todos')
        .send({ description: 'No title' })
        .expect(400);

      expect(res.body.error).toBe('Validation Error');
      expect(res.body.details[0].field).toBe('title');
    });

    test('should validate title length', async () => {
      const res = await request(app)
        .post('/api/todos')
        .send({ title: 'A'.repeat(201) })
        .expect(400);

      expect(res.body.error).toBe('Validation Error');
    });
  });

  describe('GET /api/todos/:id', () => {
    test('should return todo when exists', async () => {
      seedDatabase([{ title: 'Test Todo' }]);

      const res = await request(app)
        .get('/api/todos/1')
        .expect(200);

      expect(res.body.data).toMatchObject({
        id: 1,
        title: 'Test Todo'
      });
    });

    test('should return 404 when not found', async () => {
      const res = await request(app)
        .get('/api/todos/999')
        .expect(404);

      expect(res.body.error).toBe('Todo not found');
    });

    test('should validate ID parameter', async () => {
      const res = await request(app)
        .get('/api/todos/invalid')
        .expect(400);

      expect(res.body.error).toBe('Validation Error');
    });
  });

  describe('PUT /api/todos/:id', () => {
    test('should update existing todo', async () => {
      seedDatabase([{ title: 'Original' }]);

      const res = await request(app)
        .put('/api/todos/1')
        .send({ title: 'Updated' })
        .expect(200);

      expect(res.body.data.title).toBe('Updated');
    });

    test('should update completion status', async () => {
      seedDatabase([{ title: 'Todo' }]);

      const res = await request(app)
        .put('/api/todos/1')
        .send({ completed: true })
        .expect(200);

      expect(res.body.data.completed).toBe(1);
    });

    test('should return 404 when not found', async () => {
      const res = await request(app)
        .put('/api/todos/999')
        .send({ title: 'Updated' })
        .expect(404);

      expect(res.body.error).toBe('Todo not found');
    });
  });

  describe('DELETE /api/todos/:id', () => {
    test('should delete existing todo', async () => {
      seedDatabase([{ title: 'To Delete' }]);

      await request(app)
        .delete('/api/todos/1')
        .expect(204);

      const res = await request(app)
        .get('/api/todos/1')
        .expect(404);
    });

    test('should return 404 when not found', async () => {
      const res = await request(app)
        .delete('/api/todos/999')
        .expect(404);

      expect(res.body.error).toBe('Todo not found');
    });
  });

  describe('Error Handling', () => {
    test('should handle 404 for unknown routes', async () => {
      const res = await request(app)
        .get('/api/unknown')
        .expect(404);

      expect(res.body.error).toBe('Not Found');
    });

    test('should handle malformed JSON', async () => {
      const res = await request(app)
        .post('/api/todos')
        .set('Content-Type', 'application/json')
        .send('invalid json')
        .expect(400);

      expect(res.body.error).toBeDefined();
    });
  });

  describe('Health Checks', () => {
    test('should return healthy status', async () => {
      const res = await request(app)
        .get('/api/health')
        .expect(200);

      expect(res.body.status).toBe('healthy');
      expect(res.body.database).toBe('connected');
    });
  });
});
```

### 6. Create Test Scripts

Add to package.json:

```json
{
  "scripts": {
    "test": "jest --coverage",
    "test:unit": "jest --testPathPattern=unit",
    "test:integration": "jest --testPathPattern=integration",
    "test:watch": "jest --watch",
    "test:coverage": "jest --coverage --coverageReporters=html"
  }
}
```

## Dependencies and Relationships

- **Depends on**: All previous tasks (1-6)
- **Required by**: Task 8 (Final project completion)

## Success Criteria

1. ✅ Unit tests for all models
2. ✅ Unit tests for all controllers
3. ✅ Unit tests for middleware
4. ✅ Integration tests for all endpoints
5. ✅ 90%+ code coverage achieved
6. ✅ All tests pass successfully
7. ✅ Test database isolation implemented
8. ✅ Error scenarios tested
9. ✅ Edge cases covered

## Testing

Run the complete test suite:

```bash
# Run all tests with coverage
npm test

# Run only unit tests
npm run test:unit

# Run only integration tests
npm run test:integration

# Watch mode for development
npm run test:watch

# Generate HTML coverage report
npm run test:coverage
```

## Common Issues and Solutions

1. **Database conflicts**: Use in-memory database for tests
2. **Test pollution**: Reset database before each test
3. **Mock issues**: Clear mocks between tests
4. **Coverage gaps**: Check coverage report for untested lines

## Next Steps

After completing this task:
- Task 8: Finalize project with complete documentation
- Ensure all tests pass before deployment
- Set up CI/CD to run tests automatically

The project now has comprehensive test coverage ensuring reliability and maintainability.