# Simple Todo REST API Architecture

## Overview

This document outlines the architecture for a lightweight REST API for managing todo items, built with Node.js and Express.

## System Architecture

### Technology Stack
- **Runtime**: Node.js 18+
- **Framework**: Express.js 4.x
- **Database**: SQLite with better-sqlite3
- **Testing**: Jest & Supertest
- **Documentation**: OpenAPI/Swagger
- **Validation**: express-validator

### Directory Structure
```
simple-api/
├── src/
│   ├── app.js              # Express application setup
│   ├── controllers/        # Request handlers
│   ├── models/            # Data models and database
│   ├── routes/            # API route definitions
│   └── middleware/        # Custom middleware
├── tests/
│   ├── unit/              # Unit tests
│   └── integration/       # Integration tests
├── data/                  # SQLite database files
├── server.js              # Application entry point
├── package.json           # Dependencies and scripts
└── README.md             # Documentation
```

## Design Patterns

### MVC Pattern
- **Model**: SQLite database with Todo model
- **View**: JSON responses
- **Controller**: Express route handlers

### RESTful API Design
- Resource-based URLs
- Standard HTTP methods (GET, POST, PUT, DELETE)
- Consistent response formats
- Proper status codes

### Error Handling
- Centralized error handling middleware
- Consistent error response format
- Validation errors with detailed messages

## Data Model

### Todo Schema
```sql
CREATE TABLE todos (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  title TEXT NOT NULL,
  description TEXT,
  completed BOOLEAN DEFAULT 0,
  createdAt TEXT DEFAULT CURRENT_TIMESTAMP,
  updatedAt TEXT DEFAULT CURRENT_TIMESTAMP
)
```

## API Endpoints

### Core Endpoints
- `GET /api/todos` - List all todos (with filtering and pagination)
- `POST /api/todos` - Create a new todo
- `GET /api/todos/:id` - Get a specific todo
- `PUT /api/todos/:id` - Update a todo
- `DELETE /api/todos/:id` - Delete a todo
- `GET /api/health` - Health check

## Security Considerations
- Input validation on all endpoints
- SQL injection prevention through parameterized queries
- Environment-based configuration

## Performance Considerations
- SQLite for lightweight persistence
- Pagination support for list endpoints
- Efficient database queries