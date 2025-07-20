# Task 2: Setup SQLite Database - Autonomous AI Agent Prompt

You are tasked with setting up the SQLite database layer for the Express application. This includes installing database dependencies, creating the database schema, implementing User and Task models with CRUD operations, and setting up seed data for development.

## Your Mission

Configure a SQLite database with proper schema design, implement model classes with secure CRUD operations, create a migration system, and provide seed data for testing. Ensure all database operations are protected against SQL injection using prepared statements.

## Prerequisites

Ensure Task 1 is complete:
- Express server is running
- Project structure is in place
- Environment variables are configured

## Step-by-Step Instructions

### 1. Install Database Dependencies

```bash
npm install sqlite3@^5.1.7 better-sqlite3@^9.4.3
```

Note: We're using better-sqlite3 for its synchronous API and better performance.

### 2. Create Database Configuration

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

### 3. Create Migration System

Create directory structure:
```bash
mkdir -p src/db/migrations
```

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

### 4. Create User Model

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
      .filter(key => key !== 'id' && ['email', 'password'].includes(key))
      .map(key => `${key} = @${key}`)
      .join(', ');
    
    if (!fields) {
      throw new Error('No valid fields to update');
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
  
  static count() {
    const db = getDatabase();
    const stmt = db.prepare('SELECT COUNT(*) as count FROM users');
    const result = stmt.get();
    return result.count;
  }
}

module.exports = User;
```

### 5. Create Task Model

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
    
    if (filters.limit) {
      query += ' LIMIT ?';
      params.push(filters.limit);
    }
    
    if (filters.offset) {
      query += ' OFFSET ?';
      params.push(filters.offset);
    }
    
    query += ' ORDER BY created_at DESC';
    
    const stmt = db.prepare(query);
    return stmt.all(...params);
  }
  
  static countByUserId(userId, filters = {}) {
    const db = getDatabase();
    let query = 'SELECT COUNT(*) as count FROM tasks WHERE user_id = ?';
    const params = [userId];
    
    if (filters.completed !== undefined) {
      query += ' AND completed = ?';
      params.push(filters.completed ? 1 : 0);
    }
    
    const stmt = db.prepare(query);
    const result = stmt.get(...params);
    return result.count;
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
      completed: updates.completed !== undefined ? (updates.completed ? 1 : 0) : undefined
    });
    
    return result.changes > 0;
  }
  
  static delete(id, userId) {
    const db = getDatabase();
    const stmt = db.prepare('DELETE FROM tasks WHERE id = ? AND user_id = ?');
    const result = stmt.run(id, userId);
    
    return result.changes > 0;
  }
  
  static deleteAllByUserId(userId) {
    const db = getDatabase();
    const stmt = db.prepare('DELETE FROM tasks WHERE user_id = ?');
    const result = stmt.run(userId);
    
    return result.changes;
  }
}

module.exports = Task;
```

### 6. Create Database Initialization Script

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

// Run initialization if called directly
if (require.main === module) {
  const command = process.argv[2];
  
  if (command === 'reset') {
    resetDatabase();
  } else {
    initializeDatabase();
  }
  
  const { closeDatabase } = require('../config/database');
  closeDatabase();
}
```

### 7. Create Seed Data Script

First install bcrypt (needed for Task 3 but used here for realistic seeds):
```bash
npm install bcrypt@^5.1.1
```

Create `src/db/seeds.js`:

```javascript
const { getDatabase } = require('../config/database');
const bcrypt = require('bcrypt');

const seedDatabase = async () => {
  console.log('Seeding database...');
  
  const db = getDatabase();
  
  try {
    // Clear existing data
    db.exec('DELETE FROM tasks');
    db.exec('DELETE FROM users');
    
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
      console.log(`Created user: ${user.email}`);
    }
    
    // Create sample tasks
    const tasks = [
      { title: 'Complete project setup', description: 'Initialize Express and database', completed: true },
      { title: 'Implement authentication', description: 'Add JWT-based auth system', completed: true },
      { title: 'Create API endpoints', description: 'Build RESTful API for tasks', completed: false },
      { title: 'Add frontend interface', description: 'Simple UI for testing', completed: false },
      { title: 'Write tests', description: 'Unit and integration tests', completed: false }
    ];
    
    const insertTask = db.prepare(
      'INSERT INTO tasks (user_id, title, description, completed) VALUES (?, ?, ?, ?)'
    );
    
    // Add tasks for each user
    for (const userId of userIds) {
      for (const task of tasks) {
        insertTask.run(
          userId,
          task.title,
          task.description,
          task.completed ? 1 : 0
        );
      }
    }
    
    console.log('Database seeded successfully');
    console.log(`Created ${userIds.length} users and ${userIds.length * tasks.length} tasks`);
    console.log('\nTest credentials:');
    console.log('Email: user1@example.com, Password: password123');
    console.log('Email: user2@example.com, Password: password123');
  } catch (error) {
    console.error('Seeding failed:', error);
    throw error;
  }
};

module.exports = { seedDatabase };

// Run seeding if called directly
if (require.main === module) {
  seedDatabase()
    .then(() => {
      const { closeDatabase } = require('../config/database');
      closeDatabase();
    })
    .catch(error => {
      console.error(error);
      process.exit(1);
    });
}
```

### 8. Update npm Scripts

Add these scripts to `package.json`:

```json
{
  "scripts": {
    "db:init": "node src/db/init.js",
    "db:reset": "node src/db/init.js reset",
    "db:seed": "node src/db/seeds.js"
  }
}
```

### 9. Update App.js to Initialize Database

Modify `src/app.js` to include database initialization:

```javascript
// Add at the top after requires
const { initializeDatabase } = require('./db/init');

// Initialize database on startup
initializeDatabase();

// Update graceful shutdown
const gracefulShutdown = () => {
  console.log('Received shutdown signal, closing server gracefully...');
  
  server.close(() => {
    const { closeDatabase } = require('./config/database');
    closeDatabase();
    console.log('Server and database closed');
    process.exit(0);
  });
  
  setTimeout(() => {
    console.error('Could not close connections in time, forcefully shutting down');
    process.exit(1);
  }, 10000);
};
```

## Verification Steps

1. **Initialize the database**:
   ```bash
   npm run db:init
   ```
   Expected: "Database initialized successfully"

2. **Seed the database**:
   ```bash
   npm run db:seed
   ```
   Expected: Creation of 2 users and 10 tasks

3. **Verify database file**:
   ```bash
   ls -la database.sqlite
   ```
   Expected: File exists with size > 0

4. **Test with SQLite CLI** (optional):
   ```bash
   sqlite3 database.sqlite ".tables"
   ```
   Expected: Shows users and tasks tables

5. **Run the server**:
   ```bash
   npm run dev
   ```
   Expected: Server starts with "Database connected successfully"

## Testing the Models

Create a simple test script `test-models.js`:

```javascript
const User = require('./src/models/User');
const Task = require('./src/models/Task');
const { initializeDatabase } = require('./src/db/init');

// Initialize for testing
process.env.DATABASE_URL = './test.sqlite';
initializeDatabase();

// Test User model
console.log('Testing User model...');
const user = User.create('test@example.com', 'hashed_password');
console.log('Created user:', user);

const foundUser = User.findByEmail('test@example.com');
console.log('Found user:', foundUser);

// Test Task model
console.log('\nTesting Task model...');
const task = Task.create(user.id, 'Test Task', 'This is a test');
console.log('Created task:', task);

const userTasks = Task.findByUserId(user.id);
console.log('User tasks:', userTasks);

// Cleanup
const { closeDatabase } = require('./src/config/database');
closeDatabase();
require('fs').unlinkSync('./test.sqlite');
```

## Success Criteria

- SQLite database file is created
- Both tables (users and tasks) exist with proper schema
- Indexes are created for performance
- Foreign key constraints are enforced
- All CRUD operations work for both models
- Prepared statements prevent SQL injection
- Seed data creates test users and tasks
- Database connection is properly managed
- Graceful shutdown closes database connection

## Important Notes

- Never use string concatenation for SQL queries - always use prepared statements
- The better-sqlite3 library is synchronous, which simplifies the code
- Foreign keys must be explicitly enabled in SQLite
- The trigger automatically updates the updated_at timestamp
- Database file location is configurable via DATABASE_URL environment variable

You have now successfully set up the database layer. The application has persistent data storage with secure models ready for the authentication system in Task 3.