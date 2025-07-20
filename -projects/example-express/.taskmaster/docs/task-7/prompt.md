# Task 7: Implement Comprehensive Testing Suite - Autonomous AI Agent Prompt

You are tasked with setting up a comprehensive testing framework using Jest. This includes creating unit tests for models and utilities, integration tests for API endpoints, configuring test environments, and setting up continuous integration with GitHub Actions.

## Your Mission

Establish a complete testing suite with Jest, achieving 80% code coverage across the application. Create unit tests, integration tests, test utilities, and configure automated testing in CI/CD pipeline.

## Prerequisites

Ensure Tasks 1-6 are complete:
- Express application is functional
- All models and utilities are implemented
- API endpoints are working
- Authentication and validation in place

## Step-by-Step Instructions

### 1. Install Testing Dependencies

```bash
npm install --save-dev jest@^29.7.0 supertest@^6.3.4
```

For TypeScript projects (optional):
```bash
npm install --save-dev @types/jest @types/supertest
```

### 2. Configure Jest

Create `jest.config.js`:

```javascript
module.exports = {
  // Use Node environment for testing
  testEnvironment: 'node',
  
  // Coverage output directory
  coverageDirectory: 'coverage',
  
  // Files to collect coverage from
  collectCoverageFrom: [
    'src/**/*.js',
    '!src/**/*.test.js',
    '!src/**/*.spec.js'
  ],
  
  // Coverage thresholds - tests fail if not met
  coverageThreshold: {
    global: {
      branches: 80,
      functions: 80,
      lines: 80,
      statements: 80
    }
  },
  
  // Test file patterns
  testMatch: [
    '**/tests/**/*.test.js',
    '**/tests/**/*.spec.js'
  ],
  
  // Verbose output for better debugging
  verbose: true,
  
  // Test timeout
  testTimeout: 10000,
  
  // Setup file to run before tests
  setupFilesAfterEnv: ['<rootDir>/tests/setup.js']
};
```

### 3. Create Test Setup File

Create `tests/setup.js`:

```javascript
// Set test environment variables
process.env.NODE_ENV = 'test';
process.env.DATABASE_URL = ':memory:';  // Use in-memory SQLite for tests
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
  // Add any global utilities here
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

### 4. Create Test Factories

Create `tests/factories/userFactory.js`:

```javascript
const { hashPassword } = require('../../src/utils/password');
const User = require('../../src/models/User');

let userCounter = 0;

const userFactory = {
  // Build user data without saving
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
  
  // Create and save user
  create: async (overrides = {}) => {
    const userData = await userFactory.build(overrides);
    const hashedPassword = await hashPassword(userData.password);
    
    const user = User.create(userData.email, hashedPassword);
    
    // Return user with original password for testing
    return {
      ...user,
      password: userData.password
    };
  },
  
  // Create user with auth token
  createWithToken: async (overrides = {}) => {
    const user = await userFactory.create(overrides);
    const { generateToken } = require('../../src/utils/jwt');
    
    const token = generateToken({
      userId: user.id,
      email: user.email
    });
    
    return { user, token };
  },
  
  // Reset counter (useful between test suites)
  reset: () => {
    userCounter = 0;
  }
};

module.exports = userFactory;
```

Create `tests/factories/taskFactory.js`:

```javascript
const Task = require('../../src/models/Task');

let taskCounter = 0;

const taskFactory = {
  // Build task data without saving
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
  
  // Create and save task
  create: (userId, overrides = {}) => {
    const taskData = taskFactory.build(overrides);
    
    return Task.create(
      userId,
      taskData.title,
      taskData.description
    );
  },
  
  // Create multiple tasks
  createMany: (userId, count, overrides = {}) => {
    const tasks = [];
    
    for (let i = 0; i < count; i++) {
      tasks.push(taskFactory.create(userId, {
        ...overrides,
        title: overrides.title ? `${overrides.title} ${i + 1}` : undefined
      }));
    }
    
    return tasks;
  },
  
  // Reset counter
  reset: () => {
    taskCounter = 0;
  }
};

module.exports = taskFactory;
```

### 5. Create Unit Tests for Models

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
    it('should create a new user successfully', () => {
      const user = User.create('test@example.com', 'hashedpassword123');
      
      expect(user).toHaveProperty('id');
      expect(user.email).toBe('test@example.com');
      expect(user).toHaveProperty('created_at');
      expect(user.id).toBeGreaterThan(0);
    });
    
    it('should throw error for duplicate email', () => {
      User.create('duplicate@example.com', 'password1');
      
      expect(() => {
        User.create('duplicate@example.com', 'password2');
      }).toThrow('Email already exists');
    });
    
    it('should store password as provided (hashing done externally)', () => {
      const hashedPwd = '$2b$10$hashedpassword';
      const user = User.create('pwd@example.com', hashedPwd);
      const found = User.findById(user.id);
      
      expect(found.password).toBe(hashedPwd);
    });
  });
  
  describe('findByEmail', () => {
    it('should find existing user by email', () => {
      const created = User.create('find@example.com', 'password');
      const found = User.findByEmail('find@example.com');
      
      expect(found).toBeTruthy();
      expect(found.id).toBe(created.id);
      expect(found.email).toBe('find@example.com');
      expect(found.password).toBe('password');
    });
    
    it('should return undefined for non-existent email', () => {
      const found = User.findByEmail('notfound@example.com');
      
      expect(found).toBeUndefined();
    });
    
    it('should be case-sensitive', () => {
      User.create('case@example.com', 'password');
      
      const found = User.findByEmail('CASE@example.com');
      expect(found).toBeUndefined();
    });
  });
  
  describe('findById', () => {
    it('should find existing user by id', () => {
      const created = User.create('findid@example.com', 'password');
      const found = User.findById(created.id);
      
      expect(found).toBeTruthy();
      expect(found.email).toBe('findid@example.com');
    });
    
    it('should return undefined for non-existent id', () => {
      const found = User.findById(99999);
      
      expect(found).toBeUndefined();
    });
    
    it('should handle invalid id types', () => {
      const found = User.findById('invalid');
      
      expect(found).toBeUndefined();
    });
  });
  
  describe('update', () => {
    it('should update user email', () => {
      const user = User.create('old@example.com', 'password');
      const success = User.update(user.id, { email: 'new@example.com' });
      
      expect(success).toBe(true);
      
      const updated = User.findById(user.id);
      expect(updated.email).toBe('new@example.com');
    });
    
    it('should update user password', () => {
      const user = User.create('user@example.com', 'oldpassword');
      const success = User.update(user.id, { password: 'newpassword' });
      
      expect(success).toBe(true);
      
      const updated = User.findById(user.id);
      expect(updated.password).toBe('newpassword');
    });
    
    it('should return false for non-existent user', () => {
      const success = User.update(99999, { email: 'new@example.com' });
      
      expect(success).toBe(false);
    });
    
    it('should ignore invalid fields', () => {
      const user = User.create('test@example.com', 'password');
      const success = User.update(user.id, { 
        invalidField: 'value',
        email: 'updated@example.com'
      });
      
      expect(success).toBe(true);
      const updated = User.findById(user.id);
      expect(updated.email).toBe('updated@example.com');
      expect(updated.invalidField).toBeUndefined();
    });
  });
  
  describe('delete', () => {
    it('should delete existing user', () => {
      const user = User.create('delete@example.com', 'password');
      const success = User.delete(user.id);
      
      expect(success).toBe(true);
      
      const found = User.findById(user.id);
      expect(found).toBeUndefined();
    });
    
    it('should return false for non-existent user', () => {
      const success = User.delete(99999);
      
      expect(success).toBe(false);
    });
    
    it('should cascade delete user tasks', () => {
      const user = User.create('cascade@example.com', 'password');
      const Task = require('../../../src/models/Task');
      
      // Create tasks for user
      Task.create(user.id, 'Task 1');
      Task.create(user.id, 'Task 2');
      
      // Delete user
      User.delete(user.id);
      
      // Check tasks are gone
      const tasks = Task.findByUserId(user.id);
      expect(tasks).toHaveLength(0);
    });
  });
  
  describe('count', () => {
    it('should return correct user count', () => {
      expect(User.count()).toBe(0);
      
      User.create('user1@example.com', 'password');
      expect(User.count()).toBe(1);
      
      User.create('user2@example.com', 'password');
      expect(User.count()).toBe(2);
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
  let otherUserId;
  
  beforeAll(() => {
    initializeDatabase();
  });
  
  beforeEach(() => {
    resetDatabase();
    const user = User.create('tasktest@example.com', 'password');
    userId = user.id;
    
    const otherUser = User.create('other@example.com', 'password');
    otherUserId = otherUser.id;
  });
  
  describe('create', () => {
    it('should create a new task successfully', () => {
      const task = Task.create(userId, 'Test Task', 'Description');
      
      expect(task).toHaveProperty('id');
      expect(task.user_id).toBe(userId);
      expect(task.title).toBe('Test Task');
      expect(task.description).toBe('Description');
      expect(task.completed).toBe(false);
      expect(task).toHaveProperty('created_at');
      expect(task).toHaveProperty('updated_at');
    });
    
    it('should create task without description', () => {
      const task = Task.create(userId, 'No Description');
      
      expect(task.title).toBe('No Description');
      expect(task.description).toBeNull();
    });
    
    it('should handle empty description as null', () => {
      const task = Task.create(userId, 'Task', '');
      
      expect(task.description).toBeNull();
    });
    
    it('should set timestamps', () => {
      const before = new Date().toISOString();
      const task = Task.create(userId, 'Timed Task');
      const after = new Date().toISOString();
      
      expect(task.created_at >= before).toBe(true);
      expect(task.created_at <= after).toBe(true);
      expect(task.created_at).toBe(task.updated_at);
    });
  });
  
  describe('findById', () => {
    it('should find existing task', () => {
      const created = Task.create(userId, 'Find Me');
      const found = Task.findById(created.id);
      
      expect(found).toBeTruthy();
      expect(found.title).toBe('Find Me');
    });
    
    it('should return undefined for non-existent task', () => {
      const found = Task.findById(99999);
      
      expect(found).toBeUndefined();
    });
  });
  
  describe('findByUserId', () => {
    beforeEach(() => {
      // Create test tasks
      Task.create(userId, 'Task 1', 'First task');
      Task.create(userId, 'Task 2', 'Second task');
      Task.create(userId, 'Completed Task', 'Done');
      
      // Mark last one as completed
      const tasks = Task.findByUserId(userId);
      Task.update(tasks[0].id, userId, { completed: true });
      
      // Create task for other user
      Task.create(otherUserId, 'Other User Task');
    });
    
    it('should find all tasks for specific user', () => {
      const tasks = Task.findByUserId(userId);
      
      expect(tasks).toHaveLength(3);
      expect(tasks.every(t => t.user_id === userId)).toBe(true);
    });
    
    it('should order by created_at DESC', () => {
      const tasks = Task.findByUserId(userId);
      
      expect(tasks[0].title).toBe('Completed Task');
      expect(tasks[2].title).toBe('Task 1');
    });
    
    it('should filter by completed status', () => {
      const completed = Task.findByUserId(userId, { completed: true });
      const active = Task.findByUserId(userId, { completed: false });
      
      expect(completed).toHaveLength(1);
      expect(completed[0].title).toBe('Completed Task');
      expect(active).toHaveLength(2);
    });
    
    it('should support pagination with limit', () => {
      const limited = Task.findByUserId(userId, { limit: 2 });
      
      expect(limited).toHaveLength(2);
    });
    
    it('should support pagination with offset', () => {
      const page2 = Task.findByUserId(userId, { limit: 2, offset: 2 });
      
      expect(page2).toHaveLength(1);
      expect(page2[0].title).toBe('Task 1');
    });
    
    it('should return empty array for user with no tasks', () => {
      const newUser = User.create('empty@example.com', 'password');
      const tasks = Task.findByUserId(newUser.id);
      
      expect(tasks).toEqual([]);
    });
  });
  
  describe('countByUserId', () => {
    beforeEach(() => {
      Task.create(userId, 'Task 1');
      Task.create(userId, 'Task 2');
      const task3 = Task.create(userId, 'Task 3');
      Task.update(task3.id, userId, { completed: true });
    });
    
    it('should count all tasks for user', () => {
      const count = Task.countByUserId(userId);
      
      expect(count).toBe(3);
    });
    
    it('should count filtered tasks', () => {
      const completedCount = Task.countByUserId(userId, { completed: true });
      const activeCount = Task.countByUserId(userId, { completed: false });
      
      expect(completedCount).toBe(1);
      expect(activeCount).toBe(2);
    });
  });
  
  describe('update', () => {
    let taskId;
    
    beforeEach(() => {
      const task = Task.create(userId, 'Original Title', 'Original Description');
      taskId = task.id;
    });
    
    it('should update task fields', () => {
      const success = Task.update(taskId, userId, {
        title: 'Updated Title',
        description: 'Updated Description',
        completed: true
      });
      
      expect(success).toBe(true);
      
      const updated = Task.findById(taskId);
      expect(updated.title).toBe('Updated Title');
      expect(updated.description).toBe('Updated Description');
      expect(updated.completed).toBe(1); // SQLite stores as 0/1
    });
    
    it('should update single field', () => {
      const success = Task.update(taskId, userId, { title: 'New Title' });
      
      expect(success).toBe(true);
      
      const updated = Task.findById(taskId);
      expect(updated.title).toBe('New Title');
      expect(updated.description).toBe('Original Description');
    });
    
    it('should prevent updating other users tasks', () => {
      const success = Task.update(taskId, otherUserId, {
        title: 'Hacked Title'
      });
      
      expect(success).toBe(false);
      
      const task = Task.findById(taskId);
      expect(task.title).toBe('Original Title');
    });
    
    it('should return false for non-existent task', () => {
      const success = Task.update(99999, userId, { title: 'Ghost' });
      
      expect(success).toBe(false);
    });
    
    it('should update updated_at timestamp', (done) => {
      const original = Task.findById(taskId);
      
      // Wait to ensure timestamp difference
      setTimeout(() => {
        Task.update(taskId, userId, { title: 'New Title' });
        const updated = Task.findById(taskId);
        
        expect(updated.updated_at).not.toBe(original.updated_at);
        expect(new Date(updated.updated_at) > new Date(original.updated_at)).toBe(true);
        done();
      }, 10);
    });
  });
  
  describe('delete', () => {
    it('should delete own task', () => {
      const task = Task.create(userId, 'Delete Me');
      const success = Task.delete(task.id, userId);
      
      expect(success).toBe(true);
      
      const found = Task.findById(task.id);
      expect(found).toBeUndefined();
    });
    
    it('should prevent deleting other users tasks', () => {
      const task = Task.create(userId, 'Protected Task');
      const success = Task.delete(task.id, otherUserId);
      
      expect(success).toBe(false);
      
      const found = Task.findById(task.id);
      expect(found).toBeTruthy();
    });
    
    it('should return false for non-existent task', () => {
      const success = Task.delete(99999, userId);
      
      expect(success).toBe(false);
    });
  });
  
  describe('deleteAllByUserId', () => {
    it('should delete all user tasks', () => {
      Task.create(userId, 'Task 1');
      Task.create(userId, 'Task 2');
      Task.create(userId, 'Task 3');
      
      const deleted = Task.deleteAllByUserId(userId);
      
      expect(deleted).toBe(3);
      
      const remaining = Task.findByUserId(userId);
      expect(remaining).toHaveLength(0);
    });
    
    it('should not affect other users tasks', () => {
      Task.create(userId, 'My Task');
      Task.create(otherUserId, 'Other Task');
      
      Task.deleteAllByUserId(userId);
      
      const otherTasks = Task.findByUserId(otherUserId);
      expect(otherTasks).toHaveLength(1);
    });
  });
});
```

### 6. Create Unit Tests for Utilities

Create `tests/unit/utils/jwt.test.js`:

```javascript
const { generateToken, verifyToken, generateRefreshToken } = require('../../../src/utils/jwt');

describe('JWT Utilities', () => {
  const payload = { userId: 1, email: 'test@example.com' };
  
  describe('generateToken', () => {
    it('should generate valid JWT token', () => {
      const token = generateToken(payload);
      
      expect(token).toBeTruthy();
      expect(typeof token).toBe('string');
      
      // JWT has 3 parts separated by dots
      const parts = token.split('.');
      expect(parts).toHaveLength(3);
    });
    
    it('should include payload in token', () => {
      const token = generateToken(payload);
      const decoded = verifyToken(token);
      
      expect(decoded.userId).toBe(payload.userId);
      expect(decoded.email).toBe(payload.email);
    });
    
    it('should set expiration time', () => {
      const token = generateToken(payload);
      const decoded = verifyToken(token);
      
      expect(decoded.exp).toBeDefined();
      expect(decoded.iat).toBeDefined();
      expect(decoded.exp > decoded.iat).toBe(true);
      
      // Check it expires in ~24 hours
      const expiryTime = decoded.exp - decoded.iat;
      const twentyFourHours = 24 * 60 * 60;
      expect(expiryTime).toBeCloseTo(twentyFourHours, -2);
    });
  });
  
  describe('verifyToken', () => {
    it('should verify valid token', () => {
      const token = generateToken(payload);
      const decoded = verifyToken(token);
      
      expect(decoded.userId).toBe(payload.userId);
      expect(decoded.email).toBe(payload.email);
    });
    
    it('should throw error for invalid token', () => {
      expect(() => {
        verifyToken('invalid.token.here');
      }).toThrow('Invalid token');
    });
    
    it('should throw error for malformed token', () => {
      expect(() => {
        verifyToken('not-even-close');
      }).toThrow('Invalid token');
    });
    
    it('should throw error for expired token', () => {
      // Create token that expires immediately
      const jwt = require('jsonwebtoken');
      const expiredToken = jwt.sign(payload, process.env.JWT_SECRET, { expiresIn: '-1s' });
      
      expect(() => {
        verifyToken(expiredToken);
      }).toThrow('Token expired');
    });
    
    it('should throw error for token with wrong secret', () => {
      const jwt = require('jsonwebtoken');
      const wrongSecretToken = jwt.sign(payload, 'wrong-secret', { expiresIn: '1h' });
      
      expect(() => {
        verifyToken(wrongSecretToken);
      }).toThrow('Invalid token');
    });
  });
  
  describe('generateRefreshToken', () => {
    it('should generate refresh token with longer expiry', () => {
      const refreshToken = generateRefreshToken(payload);
      const decoded = verifyToken(refreshToken);
      
      const expiryTime = decoded.exp - decoded.iat;
      const sevenDays = 7 * 24 * 60 * 60;
      
      expect(expiryTime).toBeCloseTo(sevenDays, -2);
    });
    
    it('should contain same payload as access token', () => {
      const refreshToken = generateRefreshToken(payload);
      const decoded = verifyToken(refreshToken);
      
      expect(decoded.userId).toBe(payload.userId);
      expect(decoded.email).toBe(payload.email);
    });
  });
});
```

Create `tests/unit/utils/password.test.js`:

```javascript
const { hashPassword, comparePassword, validatePasswordStrength } = require('../../../src/utils/password');

describe('Password Utilities', () => {
  describe('hashPassword', () => {
    it('should hash password successfully', async () => {
      const password = 'testpassword123';
      const hash = await hashPassword(password);
      
      expect(hash).toBeTruthy();
      expect(hash).not.toBe(password);
      expect(hash.length).toBeGreaterThan(50);
    });
    
    it('should generate bcrypt format hash', async () => {
      const hash = await hashPassword('password');
      
      // Bcrypt hashes start with $2b$
      expect(hash.startsWith('$2b$')).toBe(true);
    });
    
    it('should generate different hashes for same password', async () => {
      const password = 'samepassword';
      const hash1 = await hashPassword(password);
      const hash2 = await hashPassword(password);
      
      expect(hash1).not.toBe(hash2);
    });
    
    it('should use correct salt rounds', async () => {
      const hash = await hashPassword('password');
      
      // Extract salt rounds from hash (e.g., $2b$10$...)
      const rounds = parseInt(hash.split('$')[2]);
      expect(rounds).toBe(10);
    });
  });
  
  describe('comparePassword', () => {
    it('should return true for matching password', async () => {
      const password = 'correctpassword';
      const hash = await hashPassword(password);
      
      const isMatch = await comparePassword(password, hash);
      
      expect(isMatch).toBe(true);
    });
    
    it('should return false for non-matching password', async () => {
      const password = 'correctpassword';
      const hash = await hashPassword(password);
      
      const isMatch = await comparePassword('wrongpassword', hash);
      
      expect(isMatch).toBe(false);
    });
    
    it('should handle empty password', async () => {
      const hash = await hashPassword('password');
      const isMatch = await comparePassword('', hash);
      
      expect(isMatch).toBe(false);
    });
    
    it('should handle invalid hash format', async () => {
      const isMatch = await comparePassword('password', 'not-a-hash');
      
      expect(isMatch).toBe(false);
    });
  });
  
  describe('validatePasswordStrength', () => {
    it('should accept valid passwords', () => {
      const validPasswords = [
        'validpass',
        'password123',
        'VeryLongPasswordThatIsDefinitelyValid',
        '12345678'
      ];
      
      validPasswords.forEach(password => {
        const result = validatePasswordStrength(password);
        expect(result.valid).toBe(true);
      });
    });
    
    it('should reject short passwords', () => {
      const shortPasswords = ['short', '1234567', 'seven77'];
      
      shortPasswords.forEach(password => {
        const result = validatePasswordStrength(password);
        expect(result.valid).toBe(false);
        expect(result.message).toContain('at least 8 characters');
      });
    });
    
    it('should reject empty password', () => {
      const result = validatePasswordStrength('');
      
      expect(result.valid).toBe(false);
      expect(result.message).toContain('required');
    });
    
    it('should reject null/undefined password', () => {
      const resultNull = validatePasswordStrength(null);
      const resultUndefined = validatePasswordStrength(undefined);
      
      expect(resultNull.valid).toBe(false);
      expect(resultUndefined.valid).toBe(false);
    });
    
    it('should handle whitespace', () => {
      const result = validatePasswordStrength('       '); // 7 spaces
      
      expect(result.valid).toBe(false);
      expect(result.message).toContain('at least 8 characters');
    });
  });
});
```

### 7. Create Integration Tests for Authentication

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
    userFactory.reset();
  });
  
  describe('POST /auth/register', () => {
    it('should register new user successfully', async () => {
      const response = await request(app)
        .post('/auth/register')
        .send({
          email: 'newuser@example.com',
          password: 'password123'
        });
      
      expect(response.status).toBe(201);
      expect(response.body).toMatchObject({
        message: 'User registered successfully',
        user: {
          id: expect.any(Number),
          email: 'newuser@example.com'
        },
        tokens: {
          accessToken: expect.any(String),
          refreshToken: expect.any(String),
          expiresIn: '24h'
        }
      });
    });
    
    it('should normalize email to lowercase', async () => {
      const response = await request(app)
        .post('/auth/register')
        .send({
          email: 'TestUser@Example.COM',
          password: 'password123'
        });
      
      expect(response.status).toBe(201);
      expect(response.body.user.email).toBe('testuser@example.com');
    });
    
    it('should validate email format', async () => {
      const response = await request(app)
        .post('/auth/register')
        .send({
          email: 'invalid-email',
          password: 'password123'
        });
      
      expect(response.status).toBe(400);
      expect(response.body.error).toMatchObject({
        message: 'Invalid email format',
        field: 'email',
        code: 'VALIDATION_ERROR'
      });
    });
    
    it('should validate password length', async () => {
      const response = await request(app)
        .post('/auth/register')
        .send({
          email: 'test@example.com',
          password: 'short'
        });
      
      expect(response.status).toBe(400);
      expect(response.body.error.message).toContain('at least 8 characters');
      expect(response.body.error.field).toBe('password');
    });
    
    it('should prevent duplicate registration', async () => {
      // First registration
      await userFactory.create({ email: 'existing@example.com' });
      
      // Try to register again
      const response = await request(app)
        .post('/auth/register')
        .send({
          email: 'existing@example.com',
          password: 'password123'
        });
      
      expect(response.status).toBe(409);
      expect(response.body.error).toMatchObject({
        message: 'Email already registered',
        code: 'CONFLICT_ERROR'
      });
    });
    
    it('should handle missing fields', async () => {
      const response = await request(app)
        .post('/auth/register')
        .send({});
      
      expect(response.status).toBe(400);
      expect(response.body.error.message).toContain('required');
    });
    
    it('should be rate limited', async () => {
      // Skip in test environment if rate limiting is disabled
      if (process.env.NODE_ENV === 'test') {
        return;
      }
      
      // Make 6 requests quickly (limit is 5 per hour)
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
      expect(lastResponse.body.error.code).toBe('RATE_LIMIT_ERROR');
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
    
    it('should login with valid credentials', async () => {
      const response = await request(app)
        .post('/auth/login')
        .send({
          email: 'login@example.com',
          password: 'password123'
        });
      
      expect(response.status).toBe(200);
      expect(response.body).toMatchObject({
        message: 'Login successful',
        user: {
          id: testUser.id,
          email: 'login@example.com'
        },
        tokens: {
          accessToken: expect.any(String),
          refreshToken: expect.any(String),
          expiresIn: '24h'
        }
      });
    });
    
    it('should reject invalid password', async () => {
      const response = await request(app)
        .post('/auth/login')
        .send({
          email: 'login@example.com',
          password: 'wrongpassword'
        });
      
      expect(response.status).toBe(401);
      expect(response.body.error).toMatchObject({
        message: 'Invalid credentials',
        code: 'AUTHENTICATION_ERROR'
      });
    });
    
    it('should reject non-existent user', async () => {
      const response = await request(app)
        .post('/auth/login')
        .send({
          email: 'nonexistent@example.com',
          password: 'password123'
        });
      
      expect(response.status).toBe(401);
      expect(response.body.error.message).toBe('Invalid credentials');
    });
    
    it('should not reveal user existence', async () => {
      const nonExistentResponse = await request(app)
        .post('/auth/login')
        .send({
          email: 'doesnotexist@example.com',
          password: 'password123'
        });
      
      const wrongPasswordResponse = await request(app)
        .post('/auth/login')
        .send({
          email: 'login@example.com',
          password: 'wrongpassword'
        });
      
      expect(nonExistentResponse.body.error.message).toBe(wrongPasswordResponse.body.error.message);
    });
  });
  
  describe('POST /auth/refresh', () => {
    it('should refresh valid token', async () => {
      const { user } = await userFactory.createWithToken();
      const { generateRefreshToken } = require('../../src/utils/jwt');
      const refreshToken = generateRefreshToken({ 
        userId: user.id, 
        email: user.email 
      });
      
      const response = await request(app)
        .post('/auth/refresh')
        .send({ refreshToken });
      
      expect(response.status).toBe(200);
      expect(response.body).toMatchObject({
        accessToken: expect.any(String),
        expiresIn: '24h'
      });
    });
    
    it('should reject invalid refresh token', async () => {
      const response = await request(app)
        .post('/auth/refresh')
        .send({ refreshToken: 'invalid.refresh.token' });
      
      expect(response.status).toBe(401);
      expect(response.body.error.code).toBe('AUTHENTICATION_ERROR');
    });
    
    it('should require refresh token', async () => {
      const response = await request(app)
        .post('/auth/refresh')
        .send({});
      
      expect(response.status).toBe(400);
      expect(response.body.error.message).toContain('required');
    });
  });
  
  describe('GET /auth/me', () => {
    it('should return current user info', async () => {
      const { user, token } = await userFactory.createWithToken();
      
      const response = await request(app)
        .get('/auth/me')
        .set('Authorization', `Bearer ${token}`);
      
      expect(response.status).toBe(200);
      expect(response.body).toMatchObject({
        user: {
          id: user.id,
          email: user.email,
          createdAt: expect.any(String)
        }
      });
    });
    
    it('should require authentication', async () => {
      const response = await request(app)
        .get('/auth/me');
      
      expect(response.status).toBe(401);
      expect(response.body.error).toMatchObject({
        message: 'Access token required',
        code: 'TOKEN_REQUIRED'
      });
    });
    
    it('should reject invalid token', async () => {
      const response = await request(app)
        .get('/auth/me')
        .set('Authorization', 'Bearer invalid.token.here');
      
      expect(response.status).toBe(401);
      expect(response.body.error.code).toBe('INVALID_TOKEN');
    });
    
    it('should reject expired token', async () => {
      const jwt = require('jsonwebtoken');
      const expiredToken = jwt.sign(
        { userId: 1, email: 'test@example.com' },
        process.env.JWT_SECRET,
        { expiresIn: '-1s' }
      );
      
      const response = await request(app)
        .get('/auth/me')
        .set('Authorization', `Bearer ${expiredToken}`);
      
      expect(response.status).toBe(401);
      expect(response.body.error.code).toBe('TOKEN_EXPIRED');
    });
  });
});
```

### 8. Create Integration Tests for Tasks

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
    userFactory.reset();
    taskFactory.reset();
    
    const result = await userFactory.createWithToken();
    user = result.user;
    token = result.token;
  });
  
  describe('GET /api/tasks', () => {
    beforeEach(() => {
      // Create 5 test tasks
      taskFactory.createMany(user.id, 5);
    });
    
    it('should return all user tasks', async () => {
      const response = await request(app)
        .get('/api/tasks')
        .set('Authorization', `Bearer ${token}`);
      
      expect(response.status).toBe(200);
      expect(response.body).toMatchObject({
        tasks: expect.arrayContaining([
          expect.objectContaining({
            id: expect.any(Number),
            title: expect.any(String),
            description: expect.any(String),
            completed: expect.any(Boolean),
            createdAt: expect.any(String),
            updatedAt: expect.any(String)
          })
        ]),
        pagination: {
          total: 5,
          limit: 20,
          offset: 0,
          hasNext: false,
          hasPrev: false
        }
      });
      expect(response.body.tasks).toHaveLength(5);
    });
    
    it('should require authentication', async () => {
      const response = await request(app)
        .get('/api/tasks');
      
      expect(response.status).toBe(401);
      expect(response.body.error.code).toBe('TOKEN_REQUIRED');
    });
    
    it('should filter by completed status', async () => {
      // Create specific tasks
      taskFactory.create(user.id, { title: 'Active 1', completed: false });
      taskFactory.create(user.id, { title: 'Active 2', completed: false });
      taskFactory.create(user.id, { title: 'Done 1', completed: true });
      
      // Get completed tasks
      const completedResponse = await request(app)
        .get('/api/tasks?completed=true')
        .set('Authorization', `Bearer ${token}`);
      
      expect(completedResponse.status).toBe(200);
      expect(completedResponse.body.tasks).toHaveLength(1);
      expect(completedResponse.body.tasks[0].completed).toBe(true);
      
      // Get active tasks
      const activeResponse = await request(app)
        .get('/api/tasks?completed=false')
        .set('Authorization', `Bearer ${token}`);
      
      expect(activeResponse.status).toBe(200);
      expect(activeResponse.body.tasks).toHaveLength(7); // 5 from beforeEach + 2
    });
    
    it('should support pagination', async () => {
      // Add more tasks for pagination test
      taskFactory.createMany(user.id, 20);
      
      // Page 1
      const page1 = await request(app)
        .get('/api/tasks?limit=10&offset=0')
        .set('Authorization', `Bearer ${token}`);
      
      expect(page1.status).toBe(200);
      expect(page1.body.tasks).toHaveLength(10);
      expect(page1.body.pagination).toMatchObject({
        total: 25,
        limit: 10,
        offset: 0,
        hasNext: true,
        hasPrev: false
      });
      
      // Page 2
      const page2 = await request(app)
        .get('/api/tasks?limit=10&offset=10')
        .set('Authorization', `Bearer ${token}`);
      
      expect(page2.body.tasks).toHaveLength(10);
      expect(page2.body.pagination.hasNext).toBe(true);
      expect(page2.body.pagination.hasPrev).toBe(true);
      
      // Page 3
      const page3 = await request(app)
        .get('/api/tasks?limit=10&offset=20')
        .set('Authorization', `Bearer ${token}`);
      
      expect(page3.body.tasks).toHaveLength(5);
      expect(page3.body.pagination.hasNext).toBe(false);
      expect(page3.body.pagination.hasPrev).toBe(true);
    });
    
    it('should validate query parameters', async () => {
      const response = await request(app)
        .get('/api/tasks?limit=200&offset=-5&completed=maybe')
        .set('Authorization', `Bearer ${token}`);
      
      expect(response.status).toBe(400);
      expect(response.body.error.field).toBeDefined();
    });
    
    it('should isolate tasks by user', async () => {
      // Create another user with tasks
      const { user: otherUser } = await userFactory.createWithToken();
      taskFactory.createMany(otherUser.id, 3);
      
      // Original user should only see their tasks
      const response = await request(app)
        .get('/api/tasks')
        .set('Authorization', `Bearer ${token}`);
      
      expect(response.status).toBe(200);
      expect(response.body.tasks).toHaveLength(5);
      expect(response.body.tasks.every(t => t.user_id !== otherUser.id)).toBe(true);
    });
  });
  
  describe('POST /api/tasks', () => {
    it('should create new task', async () => {
      const response = await request(app)
        .post('/api/tasks')
        .set('Authorization', `Bearer ${token}`)
        .send({
          title: 'New Task',
          description: 'Task Description'
        });
      
      expect(response.status).toBe(201);
      expect(response.body).toMatchObject({
        id: expect.any(Number),
        title: 'New Task',
        description: 'Task Description',
        completed: false,
        createdAt: expect.any(String),
        updatedAt: expect.any(String)
      });
    });
    
    it('should create task without description', async () => {
      const response = await request(app)
        .post('/api/tasks')
        .set('Authorization', `Bearer ${token}`)
        .send({
          title: 'No Description Task'
        });
      
      expect(response.status).toBe(201);
      expect(response.body.title).toBe('No Description Task');
      expect(response.body.description).toBeNull();
    });
    
    it('should require title', async () => {
      const response = await request(app)
        .post('/api/tasks')
        .set('Authorization', `Bearer ${token}`)
        .send({
          description: 'No title'
        });
      
      expect(response.status).toBe(400);
      expect(response.body.error).toMatchObject({
        message: 'Title is required',
        field: 'title',
        code: 'VALIDATION_ERROR'
      });
    });
    
    it('should validate title length', async () => {
      const response = await request(app)
        .post('/api/tasks')
        .set('Authorization', `Bearer ${token}`)
        .send({
          title: 'a'.repeat(256)
        });
      
      expect(response.status).toBe(400);
      expect(response.body.error.message).toContain('255 characters');
    });
    
    it('should validate description length', async () => {
      const response = await request(app)
        .post('/api/tasks')
        .set('Authorization', `Bearer ${token}`)
        .send({
          title: 'Valid Title',
          description: 'a'.repeat(1001)
        });
      
      expect(response.status).toBe(400);
      expect(response.body.error.message).toContain('1000 characters');
    });
    
    it('should trim whitespace', async () => {
      const response = await request(app)
        .post('/api/tasks')
        .set('Authorization', `Bearer ${token}`)
        .send({
          title: '  Trimmed Title  ',
          description: '  Trimmed Description  '
        });
      
      expect(response.status).toBe(201);
      expect(response.body.title).toBe('Trimmed Title');
      expect(response.body.description).toBe('Trimmed Description');
    });
  });
  
  describe('GET /api/tasks/:id', () => {
    let task;
    
    beforeEach(() => {
      task = taskFactory.create(user.id, { title: 'Get Me' });
    });
    
    it('should return specific task', async () => {
      const response = await request(app)
        .get(`/api/tasks/${task.id}`)
        .set('Authorization', `Bearer ${token}`);
      
      expect(response.status).toBe(200);
      expect(response.body).toMatchObject({
        id: task.id,
        title: 'Get Me',
        completed: false
      });
    });
    
    it('should return 404 for non-existent task', async () => {
      const response = await request(app)
        .get('/api/tasks/99999')
        .set('Authorization', `Bearer ${token}`);
      
      expect(response.status).toBe(404);
      expect(response.body.error).toMatchObject({
        message: 'Task not found',
        code: 'NOT_FOUND'
      });
    });
    
    it('should validate task ID', async () => {
      const response = await request(app)
        .get('/api/tasks/invalid-id')
        .set('Authorization', `Bearer ${token}`);
      
      expect(response.status).toBe(400);
      expect(response.body.error.field).toBe('id');
    });
    
    it('should prevent access to other users tasks', async () => {
      const { token: otherToken } = await userFactory.createWithToken();
      
      const response = await request(app)
        .get(`/api/tasks/${task.id}`)
        .set('Authorization', `Bearer ${otherToken}`);
      
      expect(response.status).toBe(403);
      expect(response.body.error).toMatchObject({
        message: 'Access denied',
        code: 'AUTHORIZATION_ERROR'
      });
    });
  });
  
  describe('PUT /api/tasks/:id', () => {
    let task;
    
    beforeEach(() => {
      task = taskFactory.create(user.id, {
        title: 'Original Title',
        description: 'Original Description'
      });
    });
    
    it('should update task fields', async () => {
      const response = await request(app)
        .put(`/api/tasks/${task.id}`)
        .set('Authorization', `Bearer ${token}`)
        .send({
          title: 'Updated Title',
          description: 'Updated Description',
          completed: true
        });
      
      expect(response.status).toBe(200);
      expect(response.body).toMatchObject({
        id: task.id,
        title: 'Updated Title',
        description: 'Updated Description',
        completed: true
      });
    });
    
    it('should allow partial updates', async () => {
      const response = await request(app)
        .put(`/api/tasks/${task.id}`)
        .set('Authorization', `Bearer ${token}`)
        .send({
          completed: true
        });
      
      expect(response.status).toBe(200);
      expect(response.body.title).toBe('Original Title');
      expect(response.body.completed).toBe(true);
    });
    
    it('should validate updates', async () => {
      const response = await request(app)
        .put(`/api/tasks/${task.id}`)
        .set('Authorization', `Bearer ${token}`)
        .send({
          title: ''
        });
      
      expect(response.status).toBe(400);
      expect(response.body.error.message).toContain('cannot be empty');
    });
    
    it('should prevent updating other users tasks', async () => {
      const { token: otherToken } = await userFactory.createWithToken();
      
      const response = await request(app)
        .put(`/api/tasks/${task.id}`)
        .set('Authorization', `Bearer ${otherToken}`)
        .send({
          title: 'Hacked Title'
        });
      
      expect(response.status).toBe(404);
      expect(response.body.error.code).toBe('NOT_FOUND');
      
      // Verify task unchanged
      const checkTask = taskFactory.build();
      const foundTask = require('../../src/models/Task').findById(task.id);
      expect(foundTask.title).toBe('Original Title');
    });
    
    it('should reject empty updates', async () => {
      const response = await request(app)
        .put(`/api/tasks/${task.id}`)
        .set('Authorization', `Bearer ${token}`)
        .send({});
      
      expect(response.status).toBe(400);
      expect(response.body.error.message).toContain('No valid fields');
    });
  });
  
  describe('DELETE /api/tasks/:id', () => {
    let task;
    
    beforeEach(() => {
      task = taskFactory.create(user.id, { title: 'Delete Me' });
    });
    
    it('should delete task successfully', async () => {
      const response = await request(app)
        .delete(`/api/tasks/${task.id}`)
        .set('Authorization', `Bearer ${token}`);
      
      expect(response.status).toBe(204);
      expect(response.body).toEqual({});
      
      // Verify deletion
      const getResponse = await request(app)
        .get(`/api/tasks/${task.id}`)
        .set('Authorization', `Bearer ${token}`);
      
      expect(getResponse.status).toBe(404);
    });
    
    it('should return 404 for non-existent task', async () => {
      const response = await request(app)
        .delete('/api/tasks/99999')
        .set('Authorization', `Bearer ${token}`);
      
      expect(response.status).toBe(404);
    });
    
    it('should prevent deleting other users tasks', async () => {
      const { token: otherToken } = await userFactory.createWithToken();
      
      const response = await request(app)
        .delete(`/api/tasks/${task.id}`)
        .set('Authorization', `Bearer ${otherToken}`);
      
      expect(response.status).toBe(404);
      
      // Verify task still exists
      const foundTask = require('../../src/models/Task').findById(task.id);
      expect(foundTask).toBeTruthy();
    });
  });
});
```

### 9. Create End-to-End Flow Tests

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
  
  describe('Complete User Journey', () => {
    it('should complete full user flow from registration to task management', async () => {
      const userData = {
        email: 'journey@example.com',
        password: 'password123'
      };
      
      // Step 1: Register new user
      const registerResponse = await request(app)
        .post('/auth/register')
        .send(userData);
      
      expect(registerResponse.status).toBe(201);
      const { accessToken } = registerResponse.body.tokens;
      
      // Step 2: Verify login works
      const loginResponse = await request(app)
        .post('/auth/login')
        .send(userData);
      
      expect(loginResponse.status).toBe(200);
      expect(loginResponse.body.user.email).toBe(userData.email);
      
      // Step 3: Check user profile
      const meResponse = await request(app)
        .get('/auth/me')
        .set('Authorization', `Bearer ${accessToken}`);
      
      expect(meResponse.status).toBe(200);
      expect(meResponse.body.user.email).toBe(userData.email);
      
      // Step 4: Create first task
      const createTask1 = await request(app)
        .post('/api/tasks')
        .set('Authorization', `Bearer ${accessToken}`)
        .send({
          title: 'My First Task',
          description: 'Getting started with task management'
        });
      
      expect(createTask1.status).toBe(201);
      const taskId1 = createTask1.body.id;
      
      // Step 5: Create second task
      const createTask2 = await request(app)
        .post('/api/tasks')
        .set('Authorization', `Bearer ${accessToken}`)
        .send({
          title: 'My Second Task'
        });
      
      expect(createTask2.status).toBe(201);
      const taskId2 = createTask2.body.id;
      
      // Step 6: List all tasks
      const listTasks = await request(app)
        .get('/api/tasks')
        .set('Authorization', `Bearer ${accessToken}`);
      
      expect(listTasks.status).toBe(200);
      expect(listTasks.body.tasks).toHaveLength(2);
      expect(listTasks.body.pagination.total).toBe(2);
      
      // Step 7: Update first task as completed
      const updateTask = await request(app)
        .put(`/api/tasks/${taskId1}`)
        .set('Authorization', `Bearer ${accessToken}`)
        .send({
          completed: true
        });
      
      expect(updateTask.status).toBe(200);
      expect(updateTask.body.completed).toBe(true);
      
      // Step 8: Filter completed tasks
      const completedTasks = await request(app)
        .get('/api/tasks?completed=true')
        .set('Authorization', `Bearer ${accessToken}`);
      
      expect(completedTasks.status).toBe(200);
      expect(completedTasks.body.tasks).toHaveLength(1);
      expect(completedTasks.body.tasks[0].id).toBe(taskId1);
      
      // Step 9: Delete second task
      const deleteTask = await request(app)
        .delete(`/api/tasks/${taskId2}`)
        .set('Authorization', `Bearer ${accessToken}`);
      
      expect(deleteTask.status).toBe(204);
      
      // Step 10: Verify final state
      const finalTasks = await request(app)
        .get('/api/tasks')
        .set('Authorization', `Bearer ${accessToken}`);
      
      expect(finalTasks.status).toBe(200);
      expect(finalTasks.body.tasks).toHaveLength(1);
      expect(finalTasks.body.tasks[0].title).toBe('My First Task');
    });
  });
  
  describe('Error Handling Flow', () => {
    it('should handle various error scenarios gracefully', async () => {
      // Unauthenticated access
      const noAuth = await request(app).get('/api/tasks');
      expect(noAuth.status).toBe(401);
      
      // Invalid registration
      const badReg = await request(app)
        .post('/auth/register')
        .send({ email: 'bad', password: '123' });
      expect(badReg.status).toBe(400);
      
      // Wrong login
      const badLogin = await request(app)
        .post('/auth/login')
        .send({ email: 'none@example.com', password: 'wrong' });
      expect(badLogin.status).toBe(401);
      
      // Register user for further tests
      const reg = await request(app)
        .post('/auth/register')
        .send({ email: 'error@example.com', password: 'password123' });
      const token = reg.body.tokens.accessToken;
      
      // Invalid task creation
      const badTask = await request(app)
        .post('/api/tasks')
        .set('Authorization', `Bearer ${token}`)
        .send({ description: 'No title' });
      expect(badTask.status).toBe(400);
      
      // Non-existent task
      const noTask = await request(app)
        .get('/api/tasks/99999')
        .set('Authorization', `Bearer ${token}`);
      expect(noTask.status).toBe(404);
      
      // Invalid token
      const badToken = await request(app)
        .get('/api/tasks')
        .set('Authorization', 'Bearer invalid.token.here');
      expect(badToken.status).toBe(401);
    });
  });
  
  describe('Multi-User Isolation', () => {
    it('should properly isolate data between users', async () => {
      // Create User A
      const userA = await request(app)
        .post('/auth/register')
        .send({ email: 'usera@example.com', password: 'password123' });
      const tokenA = userA.body.tokens.accessToken;
      
      // Create User B
      const userB = await request(app)
        .post('/auth/register')
        .send({ email: 'userb@example.com', password: 'password123' });
      const tokenB = userB.body.tokens.accessToken;
      
      // User A creates tasks
      const taskA1 = await request(app)
        .post('/api/tasks')
        .set('Authorization', `Bearer ${tokenA}`)
        .send({ title: 'User A Task 1' });
      
      const taskA2 = await request(app)
        .post('/api/tasks')
        .set('Authorization', `Bearer ${tokenA}`)
        .send({ title: 'User A Task 2' });
      
      // User B creates tasks
      const taskB1 = await request(app)
        .post('/api/tasks')
        .set('Authorization', `Bearer ${tokenB}`)
        .send({ title: 'User B Task 1' });
      
      // User A should see only their tasks
      const tasksA = await request(app)
        .get('/api/tasks')
        .set('Authorization', `Bearer ${tokenA}`);
      
      expect(tasksA.body.tasks).toHaveLength(2);
      expect(tasksA.body.tasks.every(t => t.title.includes('User A'))).toBe(true);
      
      // User B should see only their task
      const tasksB = await request(app)
        .get('/api/tasks')
        .set('Authorization', `Bearer ${tokenB}`);
      
      expect(tasksB.body.tasks).toHaveLength(1);
      expect(tasksB.body.tasks[0].title).toBe('User B Task 1');
      
      // User B cannot access User A's task
      const forbidden = await request(app)
        .get(`/api/tasks/${taskA1.body.id}`)
        .set('Authorization', `Bearer ${tokenB}`);
      
      expect(forbidden.status).toBe(403);
      
      // User B cannot update User A's task
      const hackAttempt = await request(app)
        .put(`/api/tasks/${taskA1.body.id}`)
        .set('Authorization', `Bearer ${tokenB}`)
        .send({ title: 'Hacked!' });
      
      expect(hackAttempt.status).toBe(404);
      
      // User B cannot delete User A's task
      const deleteAttempt = await request(app)
        .delete(`/api/tasks/${taskA1.body.id}`)
        .set('Authorization', `Bearer ${tokenB}`);
      
      expect(deleteAttempt.status).toBe(404);
    });
  });
});
```

### 10. Update npm Scripts

Update `package.json`:

```json
{
  "scripts": {
    "start": "node src/app.js",
    "dev": "nodemon src/app.js",
    "test": "jest",
    "test:watch": "jest --watch",
    "test:coverage": "jest --coverage",
    "test:verbose": "jest --verbose",
    "test:unit": "jest tests/unit",
    "test:integration": "jest tests/integration",
    "test:ci": "jest --coverage --ci --reporters=default --reporters=jest-junit",
    "test:debug": "node --inspect-brk ./node_modules/.bin/jest --runInBand"
  }
}
```

### 11. Create GitHub Actions Workflow

Create `.github/workflows/test.yml`:

```yaml
name: Tests

on:
  push:
    branches: [ main, develop, feature/* ]
  pull_request:
    branches: [ main, develop ]

jobs:
  test:
    runs-on: ubuntu-latest
    
    strategy:
      matrix:
        node-version: [18.x, 20.x]
    
    steps:
    - name: Checkout code
      uses: actions/checkout@v3
    
    - name: Setup Node.js ${{ matrix.node-version }}
      uses: actions/setup-node@v3
      with:
        node-version: ${{ matrix.node-version }}
        cache: 'npm'
    
    - name: Install dependencies
      run: npm ci
    
    - name: Create test environment file
      run: |
        echo "NODE_ENV=test" >> .env.test
        echo "JWT_SECRET=test-secret-key-for-ci" >> .env.test
        echo "DATABASE_URL=:memory:" >> .env.test
    
    - name: Run linter (if available)
      run: npm run lint --if-present
    
    - name: Run unit tests
      run: npm run test:unit
      env:
        NODE_ENV: test
        JWT_SECRET: test-secret-key
        SILENT_TESTS: true
    
    - name: Run integration tests
      run: npm run test:integration
      env:
        NODE_ENV: test
        JWT_SECRET: test-secret-key
        SILENT_TESTS: true
    
    - name: Run all tests with coverage
      run: npm run test:coverage
      env:
        NODE_ENV: test
        JWT_SECRET: test-secret-key
        SILENT_TESTS: true
    
    - name: Upload coverage to Codecov
      uses: codecov/codecov-action@v3
      if: matrix.node-version == '20.x'
      with:
        file: ./coverage/lcov.info
        flags: unittests
        name: codecov-umbrella
        fail_ci_if_error: false
    
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
        echo "Checking coverage thresholds..."
        npm run test:coverage -- --silent
        if [ $? -ne 0 ]; then
          echo "❌ Coverage thresholds not met!"
          exit 1
        fi
        echo "✅ Coverage thresholds met!"

  code-quality:
    runs-on: ubuntu-latest
    
    steps:
    - name: Checkout code
      uses: actions/checkout@v3
    
    - name: Setup Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '20.x'
        cache: 'npm'
    
    - name: Install dependencies
      run: npm ci
    
    - name: Check for security vulnerabilities
      run: npm audit --production
      continue-on-error: true
    
    - name: Check dependencies are up to date
      run: npm outdated
      continue-on-error: true
```

## Verification Steps

### 1. Run All Tests
```bash
npm test
```

### 2. Run with Coverage
```bash
npm run test:coverage
```

### 3. Run Specific Test Suites
```bash
# Unit tests only
npm run test:unit

# Integration tests only
npm run test:integration

# Watch mode for development
npm run test:watch
```

### 4. Check Coverage Report
```bash
# After running coverage
open coverage/lcov-report/index.html
```

### 5. Debug Failing Tests
```bash
# Run specific test file
npm test tests/unit/models/User.test.js

# Run with verbose output
npm run test:verbose

# Debug mode
npm run test:debug
```

## Success Criteria

- All tests pass successfully
- Code coverage meets 80% threshold
- Unit tests cover all models and utilities
- Integration tests cover all endpoints
- End-to-end tests verify complete flows
- CI/CD pipeline runs tests automatically
- Tests are isolated and don't interfere
- Test data factories provide consistency
- Error scenarios are tested
- Multi-user isolation is verified

## Important Notes

- Use `:memory:` database for faster tests
- Reset database between tests
- Mock external services if any
- Keep tests focused and independent
- Use descriptive test names
- Follow Arrange-Act-Assert pattern
- Don't test implementation details
- Focus on behavior and outcomes

You have now successfully implemented a comprehensive testing suite. The application has thorough test coverage ensuring reliability and maintainability.