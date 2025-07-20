# Task 7: Write Comprehensive Tests

## Overview
Implement unit and integration tests for all components, ensuring minimum 90% code coverage and comprehensive test scenarios. This task establishes a robust testing framework that validates the entire application.

## Task Details
**ID**: 7  
**Title**: Write Comprehensive Tests  
**Priority**: High  
**Dependencies**: 
- [Task 2: Database Setup and Model Implementation](../task-2/task.md)
- [Task 3: Implement Express Application and Middleware](../task-3/task.md)
- [Task 4: Implement Todo Controller](../task-4/task.md)
- [Task 5: Implement API Routes](../task-5/task.md)  
**Status**: Pending

## Architecture Context
This task implements the Testing Strategy defined in the [architecture document](../../architecture.md):
- Unit tests for isolated component testing
- Integration tests for API endpoint validation
- Test coverage goals of minimum 90%
- Error path and edge case testing
- Database constraint validation

Testing layers:
- Model layer unit tests
- Controller logic tests
- Middleware validation tests
- Full API integration tests

## Product Requirements Alignment
Implements testing requirements from PRD:
- Unit tests for all endpoints
- Integration tests with test database
- Minimum 90% code coverage
- Test data fixtures and cleanup
- Jest with supertest for testing

## Implementation Steps

### 1. Configure Jest
Create `jest.config.js` in the root directory:
```javascript
module.exports = {
  testEnvironment: 'node',
  coverageDirectory: 'coverage',
  collectCoverageFrom: [
    'src/**/*.js',
    '!src/**/*.test.js',
    '!src/config/**'
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
    '**/tests/**/*.test.js'
  ],
  verbose: true,
  testTimeout: 10000,
  setupFilesAfterEnv: ['./tests/setup.js']
};
```

### 2. Create Test Setup
Create `tests/setup.js`:
```javascript
// Silence console during tests unless explicitly needed
global.console = {
  ...console,
  log: jest.fn(),
  error: jest.fn(),
  warn: jest.fn(),
  info: jest.fn(),
  debug: jest.fn()
};

// Set test environment
process.env.NODE_ENV = 'test';
process.env.PORT = 0; // Use random port for tests
```

### 3. Create Test Database Helper
Create `tests/helpers/testDb.js`:
```javascript
const Database = require('better-sqlite3');

// Create in-memory database for testing
const createTestDb = () => {
  const db = new Database(':memory:');
  
  // Create todos table
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
  
  // Create update trigger
  db.exec(`
    CREATE TRIGGER IF NOT EXISTS update_todos_timestamp
    AFTER UPDATE ON todos
    BEGIN
      UPDATE todos SET updatedAt = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END
  `);
  
  return db;
};

// Test data fixtures
const fixtures = {
  todos: [
    { title: 'Test Todo 1', description: 'First test todo', completed: false },
    { title: 'Test Todo 2', description: 'Second test todo', completed: true },
    { title: 'Test Todo 3', description: null, completed: false }
  ]
};

// Seed database with test data
const seedTestData = (db) => {
  const stmt = db.prepare(
    'INSERT INTO todos (title, description, completed) VALUES (?, ?, ?)'
  );
  
  fixtures.todos.forEach(todo => {
    stmt.run(todo.title, todo.description, todo.completed ? 1 : 0);
  });
};

module.exports = {
  createTestDb,
  fixtures,
  seedTestData
};
```

### 4. Model Unit Tests
Create `tests/unit/models/todo.test.js`:
```javascript
const { createTestDb, seedTestData, fixtures } = require('../../helpers/testDb');

// Mock the database module
jest.mock('../../../src/models/db', () => {
  const { createTestDb } = require('../../helpers/testDb');
  return createTestDb();
});

const Todo = require('../../../src/models/todo');
const db = require('../../../src/models/db');

describe('Todo Model', () => {
  beforeEach(() => {
    // Clear database before each test
    db.exec('DELETE FROM todos');
    db.exec('DELETE FROM sqlite_sequence WHERE name="todos"');
  });
  
  describe('create', () => {
    test('should create a new todo with all fields', () => {
      const todoData = {
        title: 'New Todo',
        description: 'Todo description'
      };
      
      const todo = Todo.create(todoData);
      
      expect(todo).toMatchObject({
        id: 1,
        title: 'New Todo',
        description: 'Todo description',
        completed: 0
      });
      expect(todo.createdAt).toBeDefined();
      expect(todo.updatedAt).toBeDefined();
    });
    
    test('should create todo without description', () => {
      const todo = Todo.create({ title: 'Todo without description' });
      
      expect(todo.title).toBe('Todo without description');
      expect(todo.description).toBeNull();
    });
    
    test('should throw error for title exceeding max length', () => {
      const longTitle = 'a'.repeat(201);
      
      expect(() => {
        Todo.create({ title: longTitle });
      }).toThrow();
    });
  });
  
  describe('findAll', () => {
    beforeEach(() => {
      seedTestData(db);
    });
    
    test('should return all todos', () => {
      const todos = Todo.findAll();
      
      expect(todos).toHaveLength(3);
      expect(todos[0]).toHaveProperty('id');
      expect(todos[0]).toHaveProperty('title');
    });
    
    test('should filter by completed status', () => {
      const completedTodos = Todo.findAll({ completed: true });
      const incompleteTodos = Todo.findAll({ completed: false });
      
      expect(completedTodos).toHaveLength(1);
      expect(completedTodos[0].title).toBe('Test Todo 2');
      expect(incompleteTodos).toHaveLength(2);
    });
    
    test('should apply limit and offset', () => {
      const todos = Todo.findAll({ limit: 2, offset: 1 });
      
      expect(todos).toHaveLength(2);
      expect(todos[0].title).toBe('Test Todo 2');
    });
  });
  
  describe('findById', () => {
    beforeEach(() => {
      seedTestData(db);
    });
    
    test('should find todo by id', () => {
      const todo = Todo.findById(1);
      
      expect(todo).toBeDefined();
      expect(todo.title).toBe('Test Todo 1');
    });
    
    test('should return undefined for non-existent id', () => {
      const todo = Todo.findById(999);
      
      expect(todo).toBeUndefined();
    });
  });
  
  describe('update', () => {
    beforeEach(() => {
      seedTestData(db);
    });
    
    test('should update todo fields', () => {
      const updates = {
        title: 'Updated Title',
        description: 'Updated Description',
        completed: true
      };
      
      const updated = Todo.update(1, updates);
      
      expect(updated.title).toBe('Updated Title');
      expect(updated.description).toBe('Updated Description');
      expect(updated.completed).toBe(1);
    });
    
    test('should update only specified fields', () => {
      const original = Todo.findById(1);
      const updated = Todo.update(1, { title: 'Only Title Updated' });
      
      expect(updated.title).toBe('Only Title Updated');
      expect(updated.description).toBe(original.description);
      expect(updated.completed).toBe(original.completed);
    });
    
    test('should return null for non-existent todo', () => {
      const result = Todo.update(999, { title: 'Test' });
      
      expect(result).toBeNull();
    });
  });
  
  describe('delete', () => {
    beforeEach(() => {
      seedTestData(db);
    });
    
    test('should delete existing todo', () => {
      const result = Todo.delete(1);
      
      expect(result).toBe(true);
      expect(Todo.findById(1)).toBeUndefined();
    });
    
    test('should return false for non-existent todo', () => {
      const result = Todo.delete(999);
      
      expect(result).toBe(false);
    });
  });
  
  describe('count', () => {
    beforeEach(() => {
      seedTestData(db);
    });
    
    test('should count all todos', () => {
      const count = Todo.count();
      
      expect(count).toBe(3);
    });
    
    test('should count filtered todos', () => {
      const completedCount = Todo.count({ completed: true });
      const incompleteCount = Todo.count({ completed: false });
      
      expect(completedCount).toBe(1);
      expect(incompleteCount).toBe(2);
    });
  });
});
```

### 5. Controller Unit Tests
Create `tests/unit/controllers/todoController.test.js`:
```javascript
const todoController = require('../../../src/controllers/todoController');
const Todo = require('../../../src/models/todo');

// Mock the Todo model
jest.mock('../../../src/models/todo');

describe('Todo Controller', () => {
  let req, res, next;
  
  beforeEach(() => {
    req = {
      params: {},
      query: {},
      body: {}
    };
    res = {
      json: jest.fn().mockReturnThis(),
      status: jest.fn().mockReturnThis(),
      end: jest.fn()
    };
    next = jest.fn();
    
    // Clear all mocks
    jest.clearAllMocks();
  });
  
  describe('getAllTodos', () => {
    test('should return all todos', async () => {
      const mockTodos = [
        { id: 1, title: 'Todo 1' },
        { id: 2, title: 'Todo 2' }
      ];
      Todo.findAll.mockReturnValue(mockTodos);
      
      await todoController.getAllTodos(req, res, next);
      
      expect(Todo.findAll).toHaveBeenCalledWith({});
      expect(res.json).toHaveBeenCalledWith({
        data: mockTodos,
        count: 2
      });
    });
    
    test('should handle query parameters', async () => {
      req.query = { completed: 'true', limit: '10', offset: '5' };
      Todo.findAll.mockReturnValue([]);
      
      await todoController.getAllTodos(req, res, next);
      
      expect(Todo.findAll).toHaveBeenCalledWith({
        completed: true,
        limit: 10,
        offset: 5
      });
    });
    
    test('should handle invalid query parameters', async () => {
      req.query = { limit: 'invalid' };
      
      await todoController.getAllTodos(req, res, next);
      
      expect(next).toHaveBeenCalledWith(
        expect.objectContaining({
          message: 'Invalid limit parameter',
          status: 400
        })
      );
    });
  });
  
  describe('createTodo', () => {
    test('should create a new todo', async () => {
      req.body = { title: 'New Todo', description: 'Description' };
      const mockTodo = { id: 1, ...req.body, completed: false };
      Todo.create.mockReturnValue(mockTodo);
      
      await todoController.createTodo(req, res, next);
      
      expect(Todo.create).toHaveBeenCalledWith({
        title: 'New Todo',
        description: 'Description'
      });
      expect(res.status).toHaveBeenCalledWith(201);
      expect(res.json).toHaveBeenCalledWith({
        data: mockTodo,
        message: 'Todo created successfully'
      });
    });
    
    test('should trim whitespace from input', async () => {
      req.body = { title: '  Trimmed Title  ', description: '  Trimmed  ' };
      Todo.create.mockReturnValue({ id: 1 });
      
      await todoController.createTodo(req, res, next);
      
      expect(Todo.create).toHaveBeenCalledWith({
        title: 'Trimmed Title',
        description: 'Trimmed'
      });
    });
  });
  
  // Add more controller tests...
});
```

### 6. Integration Tests
Create `tests/integration/todos.test.js`:
```javascript
const request = require('supertest');
const app = require('../../src/app');
const { createTestDb, seedTestData } = require('../helpers/testDb');

// Mock the database
jest.mock('../../src/models/db', () => {
  const { createTestDb } = require('../helpers/testDb');
  return createTestDb();
});

const db = require('../../src/models/db');

describe('Todo API Integration Tests', () => {
  beforeEach(() => {
    // Reset database
    db.exec('DELETE FROM todos');
    db.exec('DELETE FROM sqlite_sequence WHERE name="todos"');
  });
  
  describe('GET /api/todos', () => {
    test('should return empty array when no todos', async () => {
      const response = await request(app)
        .get('/api/todos')
        .expect(200);
      
      expect(response.body).toEqual({
        data: [],
        count: 0
      });
    });
    
    test('should return all todos', async () => {
      seedTestData(db);
      
      const response = await request(app)
        .get('/api/todos')
        .expect(200);
      
      expect(response.body.data).toHaveLength(3);
      expect(response.body.count).toBe(3);
    });
    
    test('should filter by completed status', async () => {
      seedTestData(db);
      
      const response = await request(app)
        .get('/api/todos?completed=true')
        .expect(200);
      
      expect(response.body.data).toHaveLength(1);
      expect(response.body.data[0].completed).toBe(1);
    });
  });
  
  describe('POST /api/todos', () => {
    test('should create a new todo', async () => {
      const newTodo = {
        title: 'Integration Test Todo',
        description: 'Created via integration test'
      };
      
      const response = await request(app)
        .post('/api/todos')
        .send(newTodo)
        .expect(201);
      
      expect(response.body.data).toMatchObject({
        title: newTodo.title,
        description: newTodo.description,
        completed: 0
      });
      expect(response.body.data.id).toBeDefined();
    });
    
    test('should validate required fields', async () => {
      const response = await request(app)
        .post('/api/todos')
        .send({})
        .expect(400);
      
      expect(response.body.error.code).toBe('VALIDATION_ERROR');
    });
    
    test('should validate field lengths', async () => {
      const response = await request(app)
        .post('/api/todos')
        .send({
          title: 'a'.repeat(201)
        })
        .expect(400);
      
      expect(response.body.error.code).toBe('VALIDATION_ERROR');
    });
  });
  
  describe('GET /api/todos/:id', () => {
    test('should return todo by id', async () => {
      seedTestData(db);
      
      const response = await request(app)
        .get('/api/todos/1')
        .expect(200);
      
      expect(response.body.data.id).toBe(1);
      expect(response.body.data.title).toBe('Test Todo 1');
    });
    
    test('should return 404 for non-existent todo', async () => {
      const response = await request(app)
        .get('/api/todos/999')
        .expect(404);
      
      expect(response.body.error.code).toBe('NOT_FOUND');
    });
  });
  
  describe('PUT /api/todos/:id', () => {
    test('should update todo', async () => {
      seedTestData(db);
      
      const updates = {
        title: 'Updated Title',
        completed: true
      };
      
      const response = await request(app)
        .put('/api/todos/1')
        .send(updates)
        .expect(200);
      
      expect(response.body.data.title).toBe('Updated Title');
      expect(response.body.data.completed).toBe(1);
    });
    
    test('should return 404 for non-existent todo', async () => {
      const response = await request(app)
        .put('/api/todos/999')
        .send({ title: 'Test' })
        .expect(404);
      
      expect(response.body.error.code).toBe('NOT_FOUND');
    });
  });
  
  describe('DELETE /api/todos/:id', () => {
    test('should delete todo', async () => {
      seedTestData(db);
      
      await request(app)
        .delete('/api/todos/1')
        .expect(204);
      
      // Verify deletion
      const response = await request(app)
        .get('/api/todos/1')
        .expect(404);
    });
    
    test('should return 404 for non-existent todo', async () => {
      const response = await request(app)
        .delete('/api/todos/999')
        .expect(404);
      
      expect(response.body.error.code).toBe('NOT_FOUND');
    });
  });
});
```

### 7. Add Test Scripts
Update `package.json`:
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

## Success Criteria
- All tests pass successfully
- Code coverage exceeds 90% for all metrics
- Unit tests cover all functions and methods
- Integration tests validate all API endpoints
- Error scenarios are thoroughly tested
- Edge cases are identified and tested
- Tests are maintainable and well-organized

## Testing Checklist
- [ ] Model layer unit tests
- [ ] Controller unit tests
- [ ] Middleware unit tests
- [ ] API integration tests
- [ ] Error handling tests
- [ ] Validation tests
- [ ] Edge case tests
- [ ] Database constraint tests
- [ ] Performance considerations

## Related Tasks
- **Dependencies**: Tasks 2-5 provide components to test
- **Next**: [Task 8: Finalize and Document Project](../task-8/task.md)
- **Related**: All implementation tasks require corresponding tests

## References
- [Architecture Document](../../architecture.md) - Section: Testing Strategy
- [Product Requirements](../../prd.txt) - Section: Testing Requirements