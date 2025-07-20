# Task 2: Setup SQLite Database with Models

## Overview

This task establishes the data persistence layer for the application by configuring SQLite as the database, creating User and Task models with proper schema definitions, and implementing CRUD operations with security best practices. The task builds upon the Express server foundation from Task 1.

## Objectives

- Install and configure SQLite database connection
- Design and implement database schema for Users and Tasks
- Create model classes with CRUD operations
- Implement database initialization and migration system
- Add seed data functionality for development
- Ensure proper error handling and SQL injection prevention

## Technical Requirements

### Dependencies
- **sqlite3** (^5.1.7): SQLite driver for Node.js
- **better-sqlite3** (^9.4.3): Synchronous SQLite3 bindings for better performance

### Database Schema

#### Users Table
```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

#### Tasks Table
```sql
CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    completed BOOLEAN DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);
```

## Implementation Steps

### 1. SQLite Installation and Connection Setup (Subtask 2.1)

Install dependencies:
```bash
npm install sqlite3@^5.1.7 better-sqlite3@^9.4.3
```

Create `src/config/database.js`:
```javascript
const Database = require('better-sqlite3');
const path = require('path');

const dbPath = process.env.DATABASE_URL || './database.sqlite';
const absolutePath = path.resolve(dbPath);

let db;

const initializeDatabase = () => {
  try {
    db = new Database(absolutePath, {
      verbose: process.env.NODE_ENV === 'development' ? console.log : null
    });
    
    // Enable foreign keys
    db.pragma('foreign_keys = ON');
    
    console.log('Database connected successfully');
    return db;
  } catch (error) {
    console.error('Database connection failed:', error);
    process.exit(1);
  }
};

const getDatabase = () => {
  if (!db) {
    return initializeDatabase();
  }
  return db;
};

const closeDatabase = () => {
  if (db) {
    db.close();
    console.log('Database connection closed');
  }
};

module.exports = {
  getDatabase,
  closeDatabase,
  initializeDatabase
};
```

### 2. Database Schema Design and Migration Scripts (Subtask 2.2)

Create `src/db/migrations/001_initial_schema.js`:
```javascript
const up = (db) => {
  // Create users table
  db.exec(`
    CREATE TABLE IF NOT EXISTS users (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      email TEXT UNIQUE NOT NULL,
      password TEXT NOT NULL,
      created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );
    
    CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
  `);
  
  // Create tasks table
  db.exec(`
    CREATE TABLE IF NOT EXISTS tasks (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      user_id INTEGER NOT NULL,
      title TEXT NOT NULL,
      description TEXT,
      completed BOOLEAN DEFAULT 0,
      created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
      updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
      FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
    );
    
    CREATE INDEX IF NOT EXISTS idx_tasks_user_id ON tasks(user_id);
    
    CREATE TRIGGER IF NOT EXISTS update_tasks_updated_at
    AFTER UPDATE ON tasks
    FOR EACH ROW
    BEGIN
      UPDATE tasks SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;
  `);
};

const down = (db) => {
  db.exec(`
    DROP TABLE IF EXISTS tasks;
    DROP TABLE IF EXISTS users;
  `);
};

module.exports = { up, down };
```

### 3. User Model Implementation (Subtask 2.3)

Create `src/models/User.js`:
```javascript
const { getDatabase } = require('../config/database');

class User {
  static create(email, password) {
    const db = getDatabase();
    
    try {
      const stmt = db.prepare(
        'INSERT INTO users (email, password) VALUES (?, ?)'
      );
      const result = stmt.run(email, password);
      
      return {
        id: result.lastInsertRowid,
        email,
        created_at: new Date().toISOString()
      };
    } catch (error) {
      if (error.code === 'SQLITE_CONSTRAINT_UNIQUE') {
        throw new Error('Email already exists');
      }
      throw error;
    }
  }
  
  static findByEmail(email) {
    const db = getDatabase();
    const stmt = db.prepare('SELECT * FROM users WHERE email = ?');
    return stmt.get(email);
  }
  
  static findById(id) {
    const db = getDatabase();
    const stmt = db.prepare('SELECT * FROM users WHERE id = ?');
    return stmt.get(id);
  }
  
  static update(id, updates) {
    const db = getDatabase();
    const fields = Object.keys(updates)
      .filter(key => key !== 'id')
      .map(key => `${key} = @${key}`)
      .join(', ');
    
    if (!fields) {
      throw new Error('No fields to update');
    }
    
    const stmt = db.prepare(`UPDATE users SET ${fields} WHERE id = @id`);
    const result = stmt.run({ id, ...updates });
    
    return result.changes > 0;
  }
  
  static delete(id) {
    const db = getDatabase();
    const stmt = db.prepare('DELETE FROM users WHERE id = ?');
    const result = stmt.run(id);
    
    return result.changes > 0;
  }
}

module.exports = User;
```

### 4. Task Model Implementation (Subtask 2.4)

Create `src/models/Task.js`:
```javascript
const { getDatabase } = require('../config/database');

class Task {
  static create(userId, title, description = null) {
    const db = getDatabase();
    
    const stmt = db.prepare(`
      INSERT INTO tasks (user_id, title, description) 
      VALUES (?, ?, ?)
    `);
    
    const result = stmt.run(userId, title, description);
    
    return {
      id: result.lastInsertRowid,
      user_id: userId,
      title,
      description,
      completed: false,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    };
  }
  
  static findById(id) {
    const db = getDatabase();
    const stmt = db.prepare('SELECT * FROM tasks WHERE id = ?');
    return stmt.get(id);
  }
  
  static findByUserId(userId, filters = {}) {
    const db = getDatabase();
    let query = 'SELECT * FROM tasks WHERE user_id = ?';
    const params = [userId];
    
    if (filters.completed !== undefined) {
      query += ' AND completed = ?';
      params.push(filters.completed ? 1 : 0);
    }
    
    query += ' ORDER BY created_at DESC';
    
    const stmt = db.prepare(query);
    return stmt.all(...params);
  }
  
  static update(id, userId, updates) {
    const db = getDatabase();
    
    // Verify ownership
    const task = this.findById(id);
    if (!task || task.user_id !== userId) {
      return false;
    }
    
    const fields = Object.keys(updates)
      .filter(key => ['title', 'description', 'completed'].includes(key))
      .map(key => `${key} = @${key}`)
      .join(', ');
    
    if (!fields) {
      return false;
    }
    
    const stmt = db.prepare(`
      UPDATE tasks SET ${fields} 
      WHERE id = @id AND user_id = @userId
    `);
    
    const result = stmt.run({ 
      id, 
      userId,
      ...updates,
      completed: updates.completed ? 1 : 0
    });
    
    return result.changes > 0;
  }
  
  static delete(id, userId) {
    const db = getDatabase();
    const stmt = db.prepare('DELETE FROM tasks WHERE id = ? AND user_id = ?');
    const result = stmt.run(id, userId);
    
    return result.changes > 0;
  }
}

module.exports = Task;
```

### 5. Database Initialization Script (Subtask 2.5)

Create `src/db/init.js`:
```javascript
const { getDatabase } = require('../config/database');
const migrations = require('./migrations/001_initial_schema');

const initializeDatabase = () => {
  console.log('Initializing database...');
  
  const db = getDatabase();
  
  try {
    // Run migrations
    migrations.up(db);
    console.log('Database initialized successfully');
  } catch (error) {
    console.error('Database initialization failed:', error);
    throw error;
  }
};

const resetDatabase = () => {
  console.log('Resetting database...');
  
  const db = getDatabase();
  
  try {
    migrations.down(db);
    migrations.up(db);
    console.log('Database reset successfully');
  } catch (error) {
    console.error('Database reset failed:', error);
    throw error;
  }
};

module.exports = {
  initializeDatabase,
  resetDatabase
};
```

### 6. Seed Data Implementation (Subtask 2.6)

Create `src/db/seeds.js`:
```javascript
const { getDatabase } = require('../config/database');
const bcrypt = require('bcrypt');

const seedDatabase = async () => {
  console.log('Seeding database...');
  
  const db = getDatabase();
  
  try {
    // Create sample users
    const hashedPassword = await bcrypt.hash('password123', 10);
    
    const users = [
      { email: 'user1@example.com', password: hashedPassword },
      { email: 'user2@example.com', password: hashedPassword }
    ];
    
    const insertUser = db.prepare('INSERT INTO users (email, password) VALUES (?, ?)');
    const userIds = [];
    
    for (const user of users) {
      const result = insertUser.run(user.email, user.password);
      userIds.push(result.lastInsertRowid);
    }
    
    // Create sample tasks
    const tasks = [
      { title: 'Complete project setup', description: 'Initialize Express and database' },
      { title: 'Implement authentication', description: 'Add JWT-based auth system' },
      { title: 'Create API endpoints', description: 'Build RESTful API for tasks' },
      { title: 'Add frontend interface', description: 'Simple UI for testing' },
      { title: 'Write tests', description: 'Unit and integration tests' }
    ];
    
    const insertTask = db.prepare(
      'INSERT INTO tasks (user_id, title, description, completed) VALUES (?, ?, ?, ?)'
    );
    
    // Add tasks for each user
    for (const userId of userIds) {
      for (let i = 0; i < tasks.length; i++) {
        const task = tasks[i];
        insertTask.run(
          userId,
          task.title,
          task.description,
          i < 2 ? 1 : 0  // Mark first 2 tasks as completed
        );
      }
    }
    
    console.log('Database seeded successfully');
    console.log(`Created ${userIds.length} users and ${userIds.length * tasks.length} tasks`);
  } catch (error) {
    console.error('Seeding failed:', error);
    throw error;
  }
};

module.exports = { seedDatabase };
```

### 7. Update npm Scripts

Add to `package.json`:
```json
{
  "scripts": {
    "db:init": "node -e \"require('./src/db/init').initializeDatabase()\"",
    "db:reset": "node -e \"require('./src/db/init').resetDatabase()\"",
    "db:seed": "node -e \"require('./src/db/seeds').seedDatabase()\""
  }
}
```

## Integration with Express App

Update `src/app.js` to initialize database on startup:
```javascript
const { initializeDatabase } = require('./db/init');

// Initialize database on startup
initializeDatabase();

// Add graceful shutdown for database
const gracefulShutdown = () => {
  console.log('Received shutdown signal, closing server gracefully...');
  
  server.close(() => {
    const { closeDatabase } = require('./config/database');
    closeDatabase();
    console.log('Server and database closed');
    process.exit(0);
  });
};
```

## Testing

Create basic tests in `tests/models.test.js`:
```javascript
const User = require('../src/models/User');
const Task = require('../src/models/Task');
const { initializeDatabase, resetDatabase } = require('../src/db/init');

beforeAll(() => {
  process.env.DATABASE_URL = ':memory:';
  initializeDatabase();
});

beforeEach(() => {
  resetDatabase();
});

describe('User Model', () => {
  test('creates a user', () => {
    const user = User.create('test@example.com', 'hashedpassword');
    expect(user).toHaveProperty('id');
    expect(user.email).toBe('test@example.com');
  });
  
  test('finds user by email', () => {
    User.create('test@example.com', 'hashedpassword');
    const found = User.findByEmail('test@example.com');
    expect(found).toBeTruthy();
    expect(found.email).toBe('test@example.com');
  });
});

describe('Task Model', () => {
  let userId;
  
  beforeEach(() => {
    const user = User.create('test@example.com', 'hashedpassword');
    userId = user.id;
  });
  
  test('creates a task', () => {
    const task = Task.create(userId, 'Test Task', 'Description');
    expect(task).toHaveProperty('id');
    expect(task.title).toBe('Test Task');
    expect(task.user_id).toBe(userId);
  });
  
  test('finds tasks by user', () => {
    Task.create(userId, 'Task 1');
    Task.create(userId, 'Task 2');
    
    const tasks = Task.findByUserId(userId);
    expect(tasks).toHaveLength(2);
  });
});
```

## Common Issues and Solutions

### Issue: Database file permission errors
**Solution**: Ensure the application has write permissions to the database directory

### Issue: Foreign key constraint failures
**Solution**: Ensure foreign keys are enabled with `PRAGMA foreign_keys = ON`

### Issue: Concurrent access issues
**Solution**: better-sqlite3 handles this automatically with proper locking

## Next Steps

After completing this task:
- Database is configured and operational
- User and Task models provide CRUD operations
- Sample data is available for development
- Foundation is ready for authentication implementation

Proceed to Task 3: Implement JWT Authentication System.