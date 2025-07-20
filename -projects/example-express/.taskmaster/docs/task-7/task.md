# Task 7: Implement Comprehensive Testing Suite

## Overview

This task establishes a comprehensive testing framework using Jest, covering unit tests for models and utilities, integration tests for API endpoints, and setting up continuous integration with GitHub Actions. The testing suite ensures code reliability, catches regressions early, and maintains high code quality standards with coverage requirements.

## Objectives

- Set up Jest testing framework with proper configuration
- Create unit tests for models, utilities, and middleware
- Implement integration tests for complete user flows
- Configure test database using in-memory SQLite
- Establish code coverage thresholds (80% minimum)
- Set up GitHub Actions for automated testing
- Create test data factories for consistent test data
- Add npm scripts for different testing scenarios

## Technical Requirements

### Dependencies
- **jest** (^29.7.0): Testing framework
- **supertest** (^6.3.4): HTTP assertion library for API testing
- **@types/jest** (optional): TypeScript support for Jest

### Testing Standards
- Unit tests for all models and utilities
- Integration tests for all API endpoints
- 80% minimum code coverage
- Tests run in isolated environments
- Automated CI/CD pipeline

## Implementation Steps

### 1. Install Testing Dependencies (Subtask 7.1)

```bash
npm install --save-dev jest@^29.7.0 supertest@^6.3.4
```

If using TypeScript:
```bash
npm install --save-dev @types/jest @types/supertest
```

### 2. Configure Jest (Subtask 7.1)

Create `jest.config.js`:
```javascript
module.exports = {
  testEnvironment: 'node',
  coverageDirectory: 'coverage',
  collectCoverageFrom: [
    'src/**/*.js',
    '!src/**/*.test.js',
    '!src/**/*.spec.js'
  ],
  coverageThreshold: {
    global: {
      branches: 80,
      functions: 80,
      lines: 80,
      statements: 80
    }
  },
  testMatch: [
    '**/tests/**/*.test.js',
    '**/tests/**/*.spec.js'
  ],
  verbose: true,
  testTimeout: 10000,
  setupFilesAfterEnv: ['<rootDir>/tests/setup.js']
};
```

### 3. Create Test Setup (Subtask 7.1)

Create `tests/setup.js`:
```javascript
// Set test environment
process.env.NODE_ENV = 'test';
process.env.DATABASE_URL = ':memory:';
process.env.JWT_SECRET = 'test-secret-key';

// Silence console during tests (optional)
if (process.env.SILENT_TESTS === 'true') {
  global.console = {
    ...console,
    log: jest.fn(),
    error: jest.fn(),
    warn: jest.fn(),
    info: jest.fn(),
    debug: jest.fn()
  };
}

// Global test utilities
global.testUtils = {
  // Add any global test utilities here
};

// Cleanup after all tests
afterAll(async () => {
  // Close database connections
  const { closeDatabase } = require('../src/config/database');
  closeDatabase();
  
  // Close server if running
  if (global.server) {
    await new Promise(resolve => global.server.close(resolve));
  }
});
```

### 4. Create Test Utilities and Factories (Subtask 7.6)

Create `tests/factories/userFactory.js`:
```javascript
const { hashPassword } = require('../../src/utils/password');
const User = require('../../src/models/User');

let userCounter = 0;

const userFactory = {
  build: async (overrides = {}) => {
    userCounter++;
    const defaultUser = {
      email: `test${userCounter}@example.com`,
      password: 'password123'
    };
    
    return {
      ...defaultUser,
      ...overrides
    };
  },
  
  create: async (overrides = {}) => {
    const userData = await userFactory.build(overrides);
    const hashedPassword = await hashPassword(userData.password);
    
    const user = User.create(userData.email, hashedPassword);
    
    return {
      ...user,
      password: userData.password // Keep original for testing
    };
  },
  
  createWithToken: async (overrides = {}) => {
    const user = await userFactory.create(overrides);
    const { generateToken } = require('../../src/utils/jwt');
    
    const token = generateToken({
      userId: user.id,
      email: user.email
    });
    
    return { user, token };
  }
};

module.exports = userFactory;
```

Create `tests/factories/taskFactory.js`:
```javascript
const Task = require('../../src/models/Task');

let taskCounter = 0;

const taskFactory = {
  build: (overrides = {}) => {
    taskCounter++;
    const defaultTask = {
      title: `Test Task ${taskCounter}`,
      description: `Description for task ${taskCounter}`,
      completed: false
    };
    
    return {
      ...defaultTask,
      ...overrides
    };
  },
  
  create: (userId, overrides = {}) => {
    const taskData = taskFactory.build(overrides);
    
    return Task.create(
      userId,
      taskData.title,
      taskData.description
    );
  },
  
  createMany: (userId, count, overrides = {}) => {
    const tasks = [];
    
    for (let i = 0; i < count; i++) {
      tasks.push(taskFactory.create(userId, overrides));
    }
    
    return tasks;
  }
};

module.exports = taskFactory;
```

### 5. Unit Tests for Models (Subtask 7.2)

Create `tests/unit/models/User.test.js`:
```javascript
const User = require('../../../src/models/User');
const { initializeDatabase, resetDatabase } = require('../../../src/db/init');

describe('User Model', () => {
  beforeAll(() => {
    initializeDatabase();
  });
  
  beforeEach(() => {
    resetDatabase();
  });
  
  describe('create', () => {
    test('creates a new user successfully', () => {
      const user = User.create('test@example.com', 'hashedpassword');
      
      expect(user).toHaveProperty('id');
      expect(user.email).toBe('test@example.com');
      expect(user).toHaveProperty('created_at');
    });
    
    test('throws error for duplicate email', () => {
      User.create('duplicate@example.com', 'password1');
      
      expect(() => {
        User.create('duplicate@example.com', 'password2');
      }).toThrow('Email already exists');
    });
  });
  
  describe('findByEmail', () => {
    test('finds existing user by email', () => {
      const created = User.create('find@example.com', 'password');
      const found = User.findByEmail('find@example.com');
      
      expect(found).toBeTruthy();
      expect(found.id).toBe(created.id);
      expect(found.email).toBe('find@example.com');
    });
    
    test('returns undefined for non-existent email', () => {
      const found = User.findByEmail('notfound@example.com');
      
      expect(found).toBeUndefined();
    });
  });
  
  describe('findById', () => {
    test('finds existing user by id', () => {
      const created = User.create('findid@example.com', 'password');
      const found = User.findById(created.id);
      
      expect(found).toBeTruthy();
      expect(found.email).toBe('findid@example.com');
    });
    
    test('returns undefined for non-existent id', () => {
      const found = User.findById(99999);
      
      expect(found).toBeUndefined();
    });
  });
  
  describe('update', () => {
    test('updates user email', () => {
      const user = User.create('old@example.com', 'password');
      const success = User.update(user.id, { email: 'new@example.com' });
      
      expect(success).toBe(true);
      
      const updated = User.findById(user.id);
      expect(updated.email).toBe('new@example.com');
    });
    
    test('returns false for non-existent user', () => {
      const success = User.update(99999, { email: 'new@example.com' });
      
      expect(success).toBe(false);
    });
  });
  
  describe('delete', () => {
    test('deletes existing user', () => {
      const user = User.create('delete@example.com', 'password');
      const success = User.delete(user.id);
      
      expect(success).toBe(true);
      
      const found = User.findById(user.id);
      expect(found).toBeUndefined();
    });
    
    test('returns false for non-existent user', () => {
      const success = User.delete(99999);
      
      expect(success).toBe(false);
    });
  });
});
```

Create `tests/unit/models/Task.test.js`:
```javascript
const Task = require('../../../src/models/Task');
const User = require('../../../src/models/User');
const { initializeDatabase, resetDatabase } = require('../../../src/db/init');

describe('Task Model', () => {
  let userId;
  
  beforeAll(() => {
    initializeDatabase();
  });
  
  beforeEach(() => {
    resetDatabase();
    const user = User.create('tasktest@example.com', 'password');
    userId = user.id;
  });
  
  describe('create', () => {
    test('creates a new task successfully', () => {
      const task = Task.create(userId, 'Test Task', 'Description');
      
      expect(task).toHaveProperty('id');
      expect(task.user_id).toBe(userId);
      expect(task.title).toBe('Test Task');
      expect(task.description).toBe('Description');
      expect(task.completed).toBe(false);
      expect(task).toHaveProperty('created_at');
      expect(task).toHaveProperty('updated_at');
    });
    
    test('creates task without description', () => {
      const task = Task.create(userId, 'No Description');
      
      expect(task.description).toBeNull();
    });
  });
  
  describe('findByUserId', () => {
    beforeEach(() => {
      Task.create(userId, 'Task 1');
      Task.create(userId, 'Task 2');
      Task.create(userId, 'Completed Task', null);
      
      // Mark one as completed
      const tasks = Task.findByUserId(userId);
      Task.update(tasks[2].id, userId, { completed: true });
    });
    
    test('finds all tasks for user', () => {
      const tasks = Task.findByUserId(userId);
      
      expect(tasks).toHaveLength(3);
      expect(tasks[0].title).toBe('Completed Task');
    });
    
    test('filters by completed status', () => {
      const completed = Task.findByUserId(userId, { completed: true });
      const active = Task.findByUserId(userId, { completed: false });
      
      expect(completed).toHaveLength(1);
      expect(active).toHaveLength(2);
    });
    
    test('supports pagination', () => {
      const page1 = Task.findByUserId(userId, { limit: 2, offset: 0 });
      const page2 = Task.findByUserId(userId, { limit: 2, offset: 2 });
      
      expect(page1).toHaveLength(2);
      expect(page2).toHaveLength(1);
    });
    
    test('returns empty array for user with no tasks', () => {
      const otherUser = User.create('other@example.com', 'password');
      const tasks = Task.findByUserId(otherUser.id);
      
      expect(tasks).toEqual([]);
    });
  });
  
  describe('update', () => {
    let taskId;
    
    beforeEach(() => {
      const task = Task.create(userId, 'Original Title', 'Original Description');
      taskId = task.id;
    });
    
    test('updates task fields', () => {
      const success = Task.update(taskId, userId, {
        title: 'Updated Title',
        description: 'Updated Description',
        completed: true
      });
      
      expect(success).toBe(true);
      
      const updated = Task.findById(taskId);
      expect(updated.title).toBe('Updated Title');
      expect(updated.description).toBe('Updated Description');
      expect(updated.completed).toBe(1);
    });
    
    test('prevents updating other users tasks', () => {
      const otherUser = User.create('other@example.com', 'password');
      
      const success = Task.update(taskId, otherUser.id, {
        title: 'Hacked Title'
      });
      
      expect(success).toBe(false);
      
      const task = Task.findById(taskId);
      expect(task.title).toBe('Original Title');
    });
    
    test('updated_at changes on update', (done) => {
      const original = Task.findById(taskId);
      
      setTimeout(() => {
        Task.update(taskId, userId, { title: 'New Title' });
        const updated = Task.findById(taskId);
        
        expect(updated.updated_at).not.toBe(original.updated_at);
        done();
      }, 10);
    });
  });
  
  describe('delete', () => {
    test('deletes own task', () => {
      const task = Task.create(userId, 'Delete Me');
      const success = Task.delete(task.id, userId);
      
      expect(success).toBe(true);
      
      const found = Task.findById(task.id);
      expect(found).toBeUndefined();
    });
    
    test('prevents deleting other users tasks', () => {
      const task = Task.create(userId, 'Protected Task');
      const otherUser = User.create('other@example.com', 'password');
      
      const success = Task.delete(task.id, otherUser.id);
      
      expect(success).toBe(false);
      
      const found = Task.findById(task.id);
      expect(found).toBeTruthy();
    });
  });
});
```

### 6. Unit Tests for Utilities (Subtask 7.3)

Create `tests/unit/utils/jwt.test.js`:
```javascript
const { generateToken, verifyToken, generateRefreshToken } = require('../../../src/utils/jwt');

describe('JWT Utilities', () => {
  const payload = { userId: 1, email: 'test@example.com' };
  
  describe('generateToken', () => {
    test('generates valid JWT token', () => {
      const token = generateToken(payload);
      
      expect(token).toBeTruthy();
      expect(typeof token).toBe('string');
      expect(token.split('.')).toHaveLength(3);
    });
  });
  
  describe('verifyToken', () => {
    test('verifies valid token', () => {
      const token = generateToken(payload);
      const decoded = verifyToken(token);
      
      expect(decoded.userId).toBe(payload.userId);
      expect(decoded.email).toBe(payload.email);
      expect(decoded).toHaveProperty('iat');
      expect(decoded).toHaveProperty('exp');
    });
    
    test('throws error for invalid token', () => {
      expect(() => {
        verifyToken('invalid.token.here');
      }).toThrow('Invalid token');
    });
    
    test('throws error for expired token', () => {
      // Create token that expires immediately
      const jwt = require('jsonwebtoken');
      const expiredToken = jwt.sign(payload, process.env.JWT_SECRET, { expiresIn: '0s' });
      
      expect(() => {
        verifyToken(expiredToken);
      }).toThrow('Token expired');
    });
  });
  
  describe('generateRefreshToken', () => {
    test('generates refresh token with longer expiry', () => {
      const refreshToken = generateRefreshToken(payload);
      const decoded = verifyToken(refreshToken);
      
      const tokenExpiry = decoded.exp - decoded.iat;
      const sevenDaysInSeconds = 7 * 24 * 60 * 60;
      
      expect(tokenExpiry).toBeGreaterThanOrEqual(sevenDaysInSeconds - 10);
    });
  });
});
```

Create `tests/unit/utils/password.test.js`:
```javascript
const { hashPassword, comparePassword, validatePasswordStrength } = require('../../../src/utils/password');

describe('Password Utilities', () => {
  describe('hashPassword', () => {
    test('hashes password successfully', async () => {
      const password = 'testpassword123';
      const hash = await hashPassword(password);
      
      expect(hash).toBeTruthy();
      expect(hash).not.toBe(password);
      expect(hash.startsWith('$2b$')).toBe(true);
    });
    
    test('generates different hashes for same password', async () => {
      const password = 'samepassword';
      const hash1 = await hashPassword(password);
      const hash2 = await hashPassword(password);
      
      expect(hash1).not.toBe(hash2);
    });
  });
  
  describe('comparePassword', () => {
    test('returns true for matching password', async () => {
      const password = 'correctpassword';
      const hash = await hashPassword(password);
      
      const isMatch = await comparePassword(password, hash);
      
      expect(isMatch).toBe(true);
    });
    
    test('returns false for non-matching password', async () => {
      const password = 'correctpassword';
      const hash = await hashPassword(password);
      
      const isMatch = await comparePassword('wrongpassword', hash);
      
      expect(isMatch).toBe(false);
    });
  });
  
  describe('validatePasswordStrength', () => {
    test('accepts valid passwords', () => {
      const result = validatePasswordStrength('validpass123');
      
      expect(result.valid).toBe(true);
    });
    
    test('rejects short passwords', () => {
      const result = validatePasswordStrength('short');
      
      expect(result.valid).toBe(false);
      expect(result.message).toContain('8 characters');
    });
    
    test('rejects empty passwords', () => {
      const result = validatePasswordStrength('');
      
      expect(result.valid).toBe(false);
      expect(result.message).toContain('required');
    });
  });
});
```

### 7. Integration Tests for Auth Endpoints (Subtask 7.4)

Create `tests/integration/auth.test.js`:
```javascript
const request = require('supertest');
const { app, server } = require('../../src/app');
const { initializeDatabase, resetDatabase } = require('../../src/db/init');
const userFactory = require('../factories/userFactory');

describe('Authentication Endpoints', () => {
  beforeAll(() => {
    initializeDatabase();
    global.server = server;
  });
  
  beforeEach(() => {
    resetDatabase();
  });
  
  describe('POST /auth/register', () => {
    test('registers new user successfully', async () => {
      const response = await request(app)
        .post('/auth/register')
        .send({
          email: 'newuser@example.com',
          password: 'password123'
        });
      
      expect(response.status).toBe(201);
      expect(response.body).toHaveProperty('message', 'User registered successfully');
      expect(response.body.user).toHaveProperty('id');
      expect(response.body.user.email).toBe('newuser@example.com');
      expect(response.body.tokens).toHaveProperty('accessToken');
      expect(response.body.tokens).toHaveProperty('refreshToken');
    });
    
    test('validates email format', async () => {
      const response = await request(app)
        .post('/auth/register')
        .send({
          email: 'invalid-email',
          password: 'password123'
        });
      
      expect(response.status).toBe(400);
      expect(response.body.error.message).toContain('Invalid email format');
    });
    
    test('validates password length', async () => {
      const response = await request(app)
        .post('/auth/register')
        .send({
          email: 'test@example.com',
          password: 'short'
        });
      
      expect(response.status).toBe(400);
      expect(response.body.error.message).toContain('at least 8 characters');
    });
    
    test('prevents duplicate registration', async () => {
      await userFactory.create({ email: 'existing@example.com' });
      
      const response = await request(app)
        .post('/auth/register')
        .send({
          email: 'existing@example.com',
          password: 'password123'
        });
      
      expect(response.status).toBe(409);
      expect(response.body.error.message).toContain('already registered');
    });
    
    test('rate limits registration attempts', async () => {
      // Make 6 requests (limit is 5 per hour)
      const requests = [];
      for (let i = 0; i < 6; i++) {
        requests.push(
          request(app)
            .post('/auth/register')
            .send({
              email: `test${i}@example.com`,
              password: 'password123'
            })
        );
      }
      
      const responses = await Promise.all(requests);
      const lastResponse = responses[5];
      
      expect(lastResponse.status).toBe(429);
      expect(lastResponse.body.error.message).toContain('Too many');
    });
  });
  
  describe('POST /auth/login', () => {
    let testUser;
    
    beforeEach(async () => {
      testUser = await userFactory.create({
        email: 'login@example.com',
        password: 'password123'
      });
    });
    
    test('logs in with valid credentials', async () => {
      const response = await request(app)
        .post('/auth/login')
        .send({
          email: 'login@example.com',
          password: 'password123'
        });
      
      expect(response.status).toBe(200);
      expect(response.body).toHaveProperty('message', 'Login successful');
      expect(response.body.user.email).toBe('login@example.com');
      expect(response.body.tokens).toHaveProperty('accessToken');
    });
    
    test('rejects invalid password', async () => {
      const response = await request(app)
        .post('/auth/login')
        .send({
          email: 'login@example.com',
          password: 'wrongpassword'
        });
      
      expect(response.status).toBe(401);
      expect(response.body.error.message).toBe('Invalid credentials');
    });
    
    test('rejects non-existent user', async () => {
      const response = await request(app)
        .post('/auth/login')
        .send({
          email: 'nonexistent@example.com',
          password: 'password123'
        });
      
      expect(response.status).toBe(401);
      expect(response.body.error.message).toBe('Invalid credentials');
    });
  });
  
  describe('POST /auth/refresh', () => {
    test('refreshes valid token', async () => {
      const { user, token } = await userFactory.createWithToken();
      const { generateRefreshToken } = require('../../src/utils/jwt');
      const refreshToken = generateRefreshToken({ userId: user.id, email: user.email });
      
      const response = await request(app)
        .post('/auth/refresh')
        .send({ refreshToken });
      
      expect(response.status).toBe(200);
      expect(response.body).toHaveProperty('accessToken');
      expect(response.body.accessToken).not.toBe(token);
    });
  });
  
  describe('GET /auth/me', () => {
    test('returns current user info', async () => {
      const { user, token } = await userFactory.createWithToken();
      
      const response = await request(app)
        .get('/auth/me')
        .set('Authorization', `Bearer ${token}`);
      
      expect(response.status).toBe(200);
      expect(response.body.user.id).toBe(user.id);
      expect(response.body.user.email).toBe(user.email);
    });
    
    test('requires authentication', async () => {
      const response = await request(app)
        .get('/auth/me');
      
      expect(response.status).toBe(401);
      expect(response.body.error.message).toContain('Access token required');
    });
  });
});
```

### 8. Integration Tests for Task Endpoints (Subtask 7.4)

Create `tests/integration/tasks.test.js`:
```javascript
const request = require('supertest');
const { app, server } = require('../../src/app');
const { initializeDatabase, resetDatabase } = require('../../src/db/init');
const userFactory = require('../factories/userFactory');
const taskFactory = require('../factories/taskFactory');

describe('Task Management Endpoints', () => {
  let user;
  let token;
  
  beforeAll(() => {
    initializeDatabase();
    global.server = server;
  });
  
  beforeEach(async () => {
    resetDatabase();
    const result = await userFactory.createWithToken();
    user = result.user;
    token = result.token;
  });
  
  describe('GET /api/tasks', () => {
    beforeEach(() => {
      taskFactory.createMany(user.id, 5);
    });
    
    test('returns user tasks', async () => {
      const response = await request(app)
        .get('/api/tasks')
        .set('Authorization', `Bearer ${token}`);
      
      expect(response.status).toBe(200);
      expect(response.body.tasks).toHaveLength(5);
      expect(response.body.pagination).toHaveProperty('total', 5);
    });
    
    test('requires authentication', async () => {
      const response = await request(app)
        .get('/api/tasks');
      
      expect(response.status).toBe(401);
    });
    
    test('filters by completed status', async () => {
      // Mark some tasks as completed
      const tasks = taskFactory.createMany(user.id, 3, { completed: true });
      
      const response = await request(app)
        .get('/api/tasks?completed=true')
        .set('Authorization', `Bearer ${token}`);
      
      expect(response.status).toBe(200);
      expect(response.body.tasks).toHaveLength(3);
      expect(response.body.tasks.every(t => t.completed)).toBe(true);
    });
    
    test('supports pagination', async () => {
      const response = await request(app)
        .get('/api/tasks?limit=2&offset=2')
        .set('Authorization', `Bearer ${token}`);
      
      expect(response.status).toBe(200);
      expect(response.body.tasks).toHaveLength(2);
      expect(response.body.pagination.limit).toBe(2);
      expect(response.body.pagination.offset).toBe(2);
      expect(response.body.pagination.hasNext).toBe(true);
      expect(response.body.pagination.hasPrev).toBe(true);
    });
    
    test('isolates users tasks', async () => {
      // Create another user with tasks
      const { user: otherUser } = await userFactory.createWithToken();
      taskFactory.createMany(otherUser.id, 3);
      
      const response = await request(app)
        .get('/api/tasks')
        .set('Authorization', `Bearer ${token}`);
      
      expect(response.status).toBe(200);
      expect(response.body.tasks).toHaveLength(5); // Only original user's tasks
    });
  });
  
  describe('POST /api/tasks', () => {
    test('creates new task', async () => {
      const response = await request(app)
        .post('/api/tasks')
        .set('Authorization', `Bearer ${token}`)
        .send({
          title: 'New Task',
          description: 'Task Description'
        });
      
      expect(response.status).toBe(201);
      expect(response.body.title).toBe('New Task');
      expect(response.body.description).toBe('Task Description');
      expect(response.body.completed).toBe(false);
    });
    
    test('validates required title', async () => {
      const response = await request(app)
        .post('/api/tasks')
        .set('Authorization', `Bearer ${token}`)
        .send({
          description: 'No title'
        });
      
      expect(response.status).toBe(400);
      expect(response.body.error.message).toContain('Title is required');
    });
    
    test('validates title length', async () => {
      const response = await request(app)
        .post('/api/tasks')
        .set('Authorization', `Bearer ${token}`)
        .send({
          title: 'a'.repeat(256)
        });
      
      expect(response.status).toBe(400);
      expect(response.body.error.message).toContain('255 characters');
    });
  });
  
  describe('GET /api/tasks/:id', () => {
    let task;
    
    beforeEach(() => {
      task = taskFactory.create(user.id);
    });
    
    test('returns specific task', async () => {
      const response = await request(app)
        .get(`/api/tasks/${task.id}`)
        .set('Authorization', `Bearer ${token}`);
      
      expect(response.status).toBe(200);
      expect(response.body.id).toBe(task.id);
      expect(response.body.title).toBe(task.title);
    });
    
    test('returns 404 for non-existent task', async () => {
      const response = await request(app)
        .get('/api/tasks/99999')
        .set('Authorization', `Bearer ${token}`);
      
      expect(response.status).toBe(404);
      expect(response.body.error.message).toContain('not found');
    });
    
    test('prevents access to other users tasks', async () => {
      const { user: otherUser, token: otherToken } = await userFactory.createWithToken();
      
      const response = await request(app)
        .get(`/api/tasks/${task.id}`)
        .set('Authorization', `Bearer ${otherToken}`);
      
      expect(response.status).toBe(403);
      expect(response.body.error.message).toContain('Access denied');
    });
  });
  
  describe('PUT /api/tasks/:id', () => {
    let task;
    
    beforeEach(() => {
      task = taskFactory.create(user.id);
    });
    
    test('updates task successfully', async () => {
      const response = await request(app)
        .put(`/api/tasks/${task.id}`)
        .set('Authorization', `Bearer ${token}`)
        .send({
          title: 'Updated Title',
          completed: true
        });
      
      expect(response.status).toBe(200);
      expect(response.body.title).toBe('Updated Title');
      expect(response.body.completed).toBe(true);
    });
    
    test('validates update data', async () => {
      const response = await request(app)
        .put(`/api/tasks/${task.id}`)
        .set('Authorization', `Bearer ${token}`)
        .send({
          title: ''
        });
      
      expect(response.status).toBe(400);
      expect(response.body.error.message).toContain('cannot be empty');
    });
    
    test('prevents updating other users tasks', async () => {
      const { token: otherToken } = await userFactory.createWithToken();
      
      const response = await request(app)
        .put(`/api/tasks/${task.id}`)
        .set('Authorization', `Bearer ${otherToken}`)
        .send({
          title: 'Hacked'
        });
      
      expect(response.status).toBe(404);
    });
  });
  
  describe('DELETE /api/tasks/:id', () => {
    let task;
    
    beforeEach(() => {
      task = taskFactory.create(user.id);
    });
    
    test('deletes task successfully', async () => {
      const response = await request(app)
        .delete(`/api/tasks/${task.id}`)
        .set('Authorization', `Bearer ${token}`);
      
      expect(response.status).toBe(204);
      
      // Verify deletion
      const getResponse = await request(app)
        .get(`/api/tasks/${task.id}`)
        .set('Authorization', `Bearer ${token}`);
      
      expect(getResponse.status).toBe(404);
    });
    
    test('prevents deleting other users tasks', async () => {
      const { token: otherToken } = await userFactory.createWithToken();
      
      const response = await request(app)
        .delete(`/api/tasks/${task.id}`)
        .set('Authorization', `Bearer ${otherToken}`);
      
      expect(response.status).toBe(404);
    });
  });
});
```

### 9. End-to-End Flow Tests (Subtask 7.4)

Create `tests/integration/flows.test.js`:
```javascript
const request = require('supertest');
const { app, server } = require('../../src/app');
const { initializeDatabase, resetDatabase } = require('../../src/db/init');

describe('End-to-End User Flows', () => {
  beforeAll(() => {
    initializeDatabase();
    global.server = server;
  });
  
  beforeEach(() => {
    resetDatabase();
  });
  
  test('complete user flow: register → login → create task → update → delete', async () => {
    // 1. Register
    const registerResponse = await request(app)
      .post('/auth/register')
      .send({
        email: 'flow@example.com',
        password: 'password123'
      });
    
    expect(registerResponse.status).toBe(201);
    const { accessToken } = registerResponse.body.tokens;
    
    // 2. Login
    const loginResponse = await request(app)
      .post('/auth/login')
      .send({
        email: 'flow@example.com',
        password: 'password123'
      });
    
    expect(loginResponse.status).toBe(200);
    
    // 3. Create Task
    const createResponse = await request(app)
      .post('/api/tasks')
      .set('Authorization', `Bearer ${accessToken}`)
      .send({
        title: 'Flow Test Task',
        description: 'Testing complete flow'
      });
    
    expect(createResponse.status).toBe(201);
    const taskId = createResponse.body.id;
    
    // 4. Update Task
    const updateResponse = await request(app)
      .put(`/api/tasks/${taskId}`)
      .set('Authorization', `Bearer ${accessToken}`)
      .send({
        completed: true
      });
    
    expect(updateResponse.status).toBe(200);
    expect(updateResponse.body.completed).toBe(true);
    
    // 5. Delete Task
    const deleteResponse = await request(app)
      .delete(`/api/tasks/${taskId}`)
      .set('Authorization', `Bearer ${accessToken}`);
    
    expect(deleteResponse.status).toBe(204);
  });
  
  test('error handling flow', async () => {
    // Try to access protected route without auth
    const noAuthResponse = await request(app)
      .get('/api/tasks');
    
    expect(noAuthResponse.status).toBe(401);
    
    // Try to register with invalid data
    const invalidRegResponse = await request(app)
      .post('/auth/register')
      .send({
        email: 'invalid',
        password: '123'
      });
    
    expect(invalidRegResponse.status).toBe(400);
    
    // Try to login with wrong credentials
    const wrongLoginResponse = await request(app)
      .post('/auth/login')
      .send({
        email: 'wrong@example.com',
        password: 'wrongpassword'
      });
    
    expect(wrongLoginResponse.status).toBe(401);
  });
});
```

### 10. Update npm Scripts (Subtask 7.1)

Update `package.json`:
```json
{
  "scripts": {
    "test": "jest",
    "test:watch": "jest --watch",
    "test:coverage": "jest --coverage",
    "test:verbose": "jest --verbose",
    "test:unit": "jest tests/unit",
    "test:integration": "jest tests/integration",
    "test:ci": "jest --coverage --ci --reporters=default --reporters=jest-junit"
  }
}
```

### 11. GitHub Actions Configuration (Subtask 7.7)

Create `.github/workflows/test.yml`:
```yaml
name: Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    
    strategy:
      matrix:
        node-version: [18.x, 20.x]
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Use Node.js ${{ matrix.node-version }}
      uses: actions/setup-node@v3
      with:
        node-version: ${{ matrix.node-version }}
        cache: 'npm'
    
    - name: Install dependencies
      run: npm ci
    
    - name: Run linter
      run: npm run lint --if-present
    
    - name: Run tests with coverage
      run: npm run test:coverage
      env:
        NODE_ENV: test
        JWT_SECRET: test-secret-key
        SILENT_TESTS: true
    
    - name: Upload coverage reports
      uses: codecov/codecov-action@v3
      if: matrix.node-version == '20.x'
      with:
        file: ./coverage/lcov.info
        flags: unittests
        name: codecov-umbrella
    
    - name: Archive test results
      if: always()
      uses: actions/upload-artifact@v3
      with:
        name: test-results-${{ matrix.node-version }}
        path: |
          coverage/
          test-results/
    
    - name: Check coverage thresholds
      run: |
        npm run test:coverage -- --silent
        if [ $? -ne 0 ]; then
          echo "Coverage thresholds not met"
          exit 1
        fi
```

## Testing Best Practices

1. **Test Isolation**: Each test should be independent
2. **Use Factories**: Consistent test data generation
3. **Test Naming**: Descriptive test names
4. **Arrange-Act-Assert**: Clear test structure
5. **Mock External Services**: Don't rely on external APIs
6. **Test Edge Cases**: Not just happy paths
7. **Coverage Goals**: Aim for quality, not just quantity

## Common Issues and Solutions

### Issue: Tests interfere with each other
**Solution**: Use beforeEach to reset database state

### Issue: Async tests timeout
**Solution**: Increase timeout or use done callback properly

### Issue: Coverage not accurate
**Solution**: Exclude test files and config from coverage

## Next Steps

After completing this task:
- Comprehensive test coverage ensures reliability
- CI/CD pipeline runs tests automatically
- Code quality maintained through automated checks
- Ready for Task 8: Prepare Deployment Configuration

The testing suite provides confidence in code changes and helps maintain high quality standards throughout the development lifecycle.