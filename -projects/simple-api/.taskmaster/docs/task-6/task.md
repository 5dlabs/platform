# Task 6: Implement API Documentation with Swagger

## Overview
This task adds comprehensive API documentation using OpenAPI/Swagger specifications. The documentation provides an interactive UI for exploring and testing API endpoints, making the API more accessible to developers and stakeholders.

## Task Details

### Priority
Low

### Dependencies
- Task 5: Implement API Routes (must be completed - we need routes to document)

### Status
Pending

## Implementation Guide

### 1. Create Swagger Configuration

**File: `src/middleware/swagger.js`**
```javascript
const swaggerJsdoc = require('swagger-jsdoc');
const swaggerUi = require('swagger-ui-express');

const options = {
  definition: {
    openapi: '3.0.0',
    info: {
      title: 'Simple Todo REST API',
      version: '1.0.0',
      description: 'A lightweight REST API for managing todo items',
      contact: {
        name: 'API Support',
        email: 'support@example.com'
      },
      license: {
        name: 'MIT',
        url: 'https://opensource.org/licenses/MIT'
      }
    },
    servers: [
      {
        url: 'http://localhost:3000',
        description: 'Development server'
      },
      {
        url: 'https://api.example.com',
        description: 'Production server'
      }
    ],
    components: {
      schemas: {
        Todo: {
          type: 'object',
          required: ['title'],
          properties: {
            id: {
              type: 'integer',
              description: 'Auto-generated unique identifier',
              example: 1
            },
            title: {
              type: 'string',
              description: 'Todo title',
              minLength: 1,
              maxLength: 200,
              example: 'Complete API documentation'
            },
            description: {
              type: 'string',
              description: 'Detailed description of the todo',
              maxLength: 1000,
              nullable: true,
              example: 'Add Swagger documentation to all endpoints'
            },
            completed: {
              type: 'boolean',
              description: 'Completion status',
              default: false,
              example: false
            },
            createdAt: {
              type: 'string',
              format: 'date-time',
              description: 'Creation timestamp',
              example: '2023-12-01T10:30:00Z'
            },
            updatedAt: {
              type: 'string',
              format: 'date-time',
              description: 'Last update timestamp',
              example: '2023-12-01T14:45:00Z'
            }
          }
        },
        TodoInput: {
          type: 'object',
          required: ['title'],
          properties: {
            title: {
              type: 'string',
              description: 'Todo title',
              minLength: 1,
              maxLength: 200,
              example: 'New todo item'
            },
            description: {
              type: 'string',
              description: 'Todo description',
              maxLength: 1000,
              nullable: true,
              example: 'Description of the new todo'
            }
          }
        },
        TodoUpdate: {
          type: 'object',
          properties: {
            title: {
              type: 'string',
              description: 'Updated title',
              minLength: 1,
              maxLength: 200,
              example: 'Updated todo title'
            },
            description: {
              type: 'string',
              description: 'Updated description',
              maxLength: 1000,
              nullable: true,
              example: 'Updated description'
            },
            completed: {
              type: 'boolean',
              description: 'Updated completion status',
              example: true
            }
          }
        },
        Error: {
          type: 'object',
          properties: {
            error: {
              type: 'object',
              properties: {
                message: {
                  type: 'string',
                  description: 'Error message',
                  example: 'Validation failed'
                },
                code: {
                  type: 'string',
                  description: 'Error code',
                  example: 'VALIDATION_ERROR'
                },
                details: {
                  type: 'array',
                  description: 'Additional error details',
                  items: {
                    type: 'object',
                    properties: {
                      field: {
                        type: 'string',
                        example: 'title'
                      },
                      message: {
                        type: 'string',
                        example: 'Title is required'
                      }
                    }
                  }
                }
              }
            }
          }
        },
        HealthStatus: {
          type: 'object',
          properties: {
            status: {
              type: 'string',
              enum: ['ok', 'degraded', 'error'],
              example: 'ok'
            },
            timestamp: {
              type: 'string',
              format: 'date-time',
              example: '2023-12-01T10:30:00Z'
            },
            uptime: {
              type: 'number',
              description: 'Server uptime in seconds',
              example: 3600
            },
            environment: {
              type: 'string',
              example: 'development'
            }
          }
        },
        TodoStats: {
          type: 'object',
          properties: {
            total: {
              type: 'integer',
              description: 'Total number of todos',
              example: 10
            },
            completed: {
              type: 'integer',
              description: 'Number of completed todos',
              example: 3
            },
            pending: {
              type: 'integer',
              description: 'Number of pending todos',
              example: 7
            },
            completionRate: {
              type: 'number',
              format: 'float',
              description: 'Completion rate (0-1)',
              example: 0.3
            }
          }
        }
      }
    }
  },
  apis: ['./src/routes/*.js'] // Path to files with JSDoc comments
};

const specs = swaggerJsdoc(options);

module.exports = {
  serve: swaggerUi.serve,
  setup: swaggerUi.setup(specs, {
    customCss: '.swagger-ui .topbar { display: none }',
    customSiteTitle: 'Todo API Documentation'
  })
};
```

### 2. Add JSDoc Comments to Todo Routes

**Update `src/routes/todos.js`** with Swagger documentation:
```javascript
/**
 * @swagger
 * /api/todos:
 *   get:
 *     summary: Get all todos
 *     description: Retrieve a list of all todos with optional filtering and pagination
 *     tags: [Todos]
 *     parameters:
 *       - in: query
 *         name: completed
 *         schema:
 *           type: boolean
 *         description: Filter by completion status
 *       - in: query
 *         name: limit
 *         schema:
 *           type: integer
 *           minimum: 1
 *           maximum: 100
 *           default: 100
 *         description: Maximum number of todos to return
 *       - in: query
 *         name: offset
 *         schema:
 *           type: integer
 *           minimum: 0
 *           default: 0
 *         description: Number of todos to skip
 *     responses:
 *       200:
 *         description: List of todos retrieved successfully
 *         content:
 *           application/json:
 *             schema:
 *               type: array
 *               items:
 *                 $ref: '#/components/schemas/Todo'
 *       400:
 *         description: Invalid query parameters
 *         content:
 *           application/json:
 *             schema:
 *               $ref: '#/components/schemas/Error'
 */

/**
 * @swagger
 * /api/todos/stats:
 *   get:
 *     summary: Get todo statistics
 *     description: Retrieve statistics about todos including counts and completion rate
 *     tags: [Todos]
 *     responses:
 *       200:
 *         description: Statistics retrieved successfully
 *         content:
 *           application/json:
 *             schema:
 *               $ref: '#/components/schemas/TodoStats'
 */

/**
 * @swagger
 * /api/todos/{id}:
 *   get:
 *     summary: Get a todo by ID
 *     description: Retrieve a specific todo item by its ID
 *     tags: [Todos]
 *     parameters:
 *       - in: path
 *         name: id
 *         required: true
 *         schema:
 *           type: integer
 *           minimum: 1
 *         description: Todo ID
 *     responses:
 *       200:
 *         description: Todo retrieved successfully
 *         content:
 *           application/json:
 *             schema:
 *               $ref: '#/components/schemas/Todo'
 *       404:
 *         description: Todo not found
 *         content:
 *           application/json:
 *             schema:
 *               $ref: '#/components/schemas/Error'
 */

/**
 * @swagger
 * /api/todos:
 *   post:
 *     summary: Create a new todo
 *     description: Create a new todo item
 *     tags: [Todos]
 *     requestBody:
 *       required: true
 *       content:
 *         application/json:
 *           schema:
 *             $ref: '#/components/schemas/TodoInput'
 *     responses:
 *       201:
 *         description: Todo created successfully
 *         content:
 *           application/json:
 *             schema:
 *               $ref: '#/components/schemas/Todo'
 *       400:
 *         description: Invalid input data
 *         content:
 *           application/json:
 *             schema:
 *               $ref: '#/components/schemas/Error'
 */

/**
 * @swagger
 * /api/todos/{id}:
 *   put:
 *     summary: Update a todo
 *     description: Update an existing todo item
 *     tags: [Todos]
 *     parameters:
 *       - in: path
 *         name: id
 *         required: true
 *         schema:
 *           type: integer
 *           minimum: 1
 *         description: Todo ID
 *     requestBody:
 *       required: true
 *       content:
 *         application/json:
 *           schema:
 *             $ref: '#/components/schemas/TodoUpdate'
 *     responses:
 *       200:
 *         description: Todo updated successfully
 *         content:
 *           application/json:
 *             schema:
 *               $ref: '#/components/schemas/Todo'
 *       404:
 *         description: Todo not found
 *         content:
 *           application/json:
 *             schema:
 *               $ref: '#/components/schemas/Error'
 *       400:
 *         description: Invalid input data
 *         content:
 *           application/json:
 *             schema:
 *               $ref: '#/components/schemas/Error'
 */

/**
 * @swagger
 * /api/todos/{id}:
 *   delete:
 *     summary: Delete a todo
 *     description: Delete an existing todo item
 *     tags: [Todos]
 *     parameters:
 *       - in: path
 *         name: id
 *         required: true
 *         schema:
 *           type: integer
 *           minimum: 1
 *         description: Todo ID
 *     responses:
 *       204:
 *         description: Todo deleted successfully
 *       404:
 *         description: Todo not found
 *         content:
 *           application/json:
 *             schema:
 *               $ref: '#/components/schemas/Error'
 */

/**
 * @swagger
 * tags:
 *   name: Todos
 *   description: Todo management operations
 */
```

### 3. Add JSDoc Comments to Health Routes

**Update `src/routes/health.js`**:
```javascript
/**
 * @swagger
 * /api/health:
 *   get:
 *     summary: Basic health check
 *     description: Check if the API is running
 *     tags: [System]
 *     responses:
 *       200:
 *         description: API is healthy
 *         content:
 *           application/json:
 *             schema:
 *               $ref: '#/components/schemas/HealthStatus'
 */

/**
 * @swagger
 * /api/health/detailed:
 *   get:
 *     summary: Detailed health check
 *     description: Get detailed health status including database connectivity
 *     tags: [System]
 *     responses:
 *       200:
 *         description: System is healthy
 *         content:
 *           application/json:
 *             schema:
 *               type: object
 *               properties:
 *                 status:
 *                   type: string
 *                   enum: [ok, degraded]
 *                 timestamp:
 *                   type: string
 *                   format: date-time
 *                 uptime:
 *                   type: number
 *                 environment:
 *                   type: string
 *                 checks:
 *                   type: object
 *                   properties:
 *                     database:
 *                       type: object
 *                       properties:
 *                         status:
 *                           type: string
 *                           enum: [healthy, unhealthy]
 *                         message:
 *                           type: string
 *       503:
 *         description: System is unhealthy
 *         content:
 *           application/json:
 *             schema:
 *               type: object
 *               properties:
 *                 status:
 *                   type: string
 *                   enum: [error]
 *                 timestamp:
 *                   type: string
 *                   format: date-time
 *                 checks:
 *                   type: object
 */

/**
 * @swagger
 * tags:
 *   name: System
 *   description: System health and status endpoints
 */
```

### 4. Update App.js to Include Swagger

**Update `src/app.js`**:
```javascript
const express = require('express');
const path = require('path');
const { errorHandler, notFoundHandler } = require('./middleware');
const swagger = require('./middleware/swagger');
const routes = require('./routes');

// Create Express application
const app = express();

// [Previous middleware configuration...]

// Swagger documentation
app.use('/api-docs', swagger.serve, swagger.setup);

// API Routes
app.use('/api', routes);

// [Rest of configuration...]
```

### 5. Add API Documentation to Root Endpoint

**Update `src/routes/index.js`**:
```javascript
/**
 * @swagger
 * /api:
 *   get:
 *     summary: API information
 *     description: Get basic information about the API
 *     tags: [System]
 *     responses:
 *       200:
 *         description: API information
 *         content:
 *           application/json:
 *             schema:
 *               type: object
 *               properties:
 *                 message:
 *                   type: string
 *                   example: Simple Todo REST API
 *                 version:
 *                   type: string
 *                   example: 1.0.0
 *                 endpoints:
 *                   type: object
 *                   properties:
 *                     todos:
 *                       type: string
 *                       example: /api/todos
 *                     health:
 *                       type: string
 *                       example: /api/health
 *                     documentation:
 *                       type: string
 *                       example: /api-docs
 */
```

## Key Implementation Considerations

### Documentation Standards
- Comprehensive schema definitions for all data types
- Clear descriptions for all endpoints
- Example values for better understanding
- Proper categorization with tags

### OpenAPI Compliance
- OpenAPI 3.0.0 specification
- Complete request/response documentation
- Parameter validation rules included
- Error response formats documented

### Interactive Features
- Try-it-out functionality in Swagger UI
- Server selection for different environments
- Schema visualization
- Example request/response display

## Testing the Documentation

1. **Access Swagger UI**:
   ```bash
   npm run dev
   # Open browser to http://localhost:3000/api-docs
   ```

2. **Verify All Endpoints Listed**:
   - Todos section with all CRUD operations
   - System section with health checks
   - Proper grouping by tags

3. **Test Interactive Features**:
   - Try executing requests from Swagger UI
   - Verify parameter validation
   - Check response schemas

## Common Issues and Solutions

### Issue: Swagger Not Finding Routes
**Solution**: Ensure the `apis` path in swagger config points to correct files

### Issue: Missing Schemas in Documentation
**Solution**: Check that all $ref paths match defined component schemas

### Issue: Documentation Not Updating
**Solution**: Restart the server after adding JSDoc comments

## Next Steps
After completing this task:
1. Access documentation at `/api-docs`
2. Test all endpoints through Swagger UI
3. Verify documentation accuracy
4. Proceed to Task 7: Write Comprehensive Tests
5. Share documentation URL with stakeholders

## References
- [OpenAPI Specification](https://swagger.io/specification/)
- [Swagger JSDoc](https://github.com/Surnet/swagger-jsdoc)
- [Swagger UI Express](https://github.com/scottie1984/swagger-ui-express)