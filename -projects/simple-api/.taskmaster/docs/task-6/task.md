# Task 6: Implement API Documentation with Swagger

## Overview
Add OpenAPI/Swagger documentation for all API endpoints, providing interactive documentation and API specification. This task ensures the API is well-documented and easily testable through the Swagger UI.

## Task Details
**ID**: 6  
**Title**: Implement API Documentation with Swagger  
**Priority**: Low  
**Dependencies**: [Task 5: Implement API Routes](../task-5/task.md)  
**Status**: Pending

## Architecture Context
This task implements API documentation as specified in the [architecture document](../../architecture.md):
- OpenAPI/Swagger integration for interactive documentation
- Automatic documentation generation from code annotations
- API specification in OpenAPI 3.0 format
- Swagger UI for testing endpoints

Benefits:
- Self-documenting API
- Interactive testing interface
- Standard API specification format
- Client SDK generation support

## Product Requirements Alignment
Implements documentation requirements from PRD:
- Technology stack includes OpenAPI/Swagger documentation
- API documentation must be available
- All endpoints must be documented
- Clear request/response examples

## Implementation Steps

### 1. Create Swagger Configuration
Create `src/config/swagger.js`:
```javascript
const swaggerJsdoc = require('swagger-jsdoc');

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
        url: 'http://localhost:3000',
        description: 'Production server',
        variables: {
          port: {
            default: '3000'
          }
        }
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
              example: 'Complete project documentation'
            },
            description: {
              type: 'string',
              description: 'Detailed description of the todo',
              maxLength: 1000,
              example: 'Write comprehensive documentation for the REST API'
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
              example: '2024-01-01T00:00:00Z'
            },
            updatedAt: {
              type: 'string',
              format: 'date-time',
              description: 'Last update timestamp',
              example: '2024-01-01T00:00:00Z'
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
              maxLength: 200
            },
            description: {
              type: 'string',
              description: 'Updated description',
              maxLength: 1000
            },
            completed: {
              type: 'boolean',
              description: 'Updated completion status'
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
                  example: 'Resource not found'
                },
                code: {
                  type: 'string',
                  description: 'Error code',
                  example: 'NOT_FOUND'
                },
                details: {
                  type: 'array',
                  description: 'Additional error details',
                  items: {
                    type: 'object'
                  }
                }
              }
            }
          }
        },
        HealthCheck: {
          type: 'object',
          properties: {
            status: {
              type: 'string',
              enum: ['ok', 'error'],
              example: 'ok'
            },
            timestamp: {
              type: 'string',
              format: 'date-time',
              example: '2024-01-01T00:00:00Z'
            },
            environment: {
              type: 'string',
              example: 'development'
            },
            version: {
              type: 'string',
              example: '1.0.0'
            },
            database: {
              type: 'string',
              enum: ['connected', 'disconnected'],
              example: 'connected'
            }
          }
        }
      },
      responses: {
        NotFound: {
          description: 'Resource not found',
          content: {
            'application/json': {
              schema: {
                $ref: '#/components/schemas/Error'
              }
            }
          }
        },
        ValidationError: {
          description: 'Validation error',
          content: {
            'application/json': {
              schema: {
                $ref: '#/components/schemas/Error'
              }
            }
          }
        },
        InternalError: {
          description: 'Internal server error',
          content: {
            'application/json': {
              schema: {
                $ref: '#/components/schemas/Error'
              }
            }
          }
        }
      }
    }
  },
  apis: ['./src/routes/*.js', './src/routes/todoRoutes.js', './src/routes/healthRoutes.js']
};

const specs = swaggerJsdoc(options);

module.exports = specs;
```

### 2. Create Swagger Middleware
Create `src/middleware/swagger.js`:
```javascript
const swaggerUi = require('swagger-ui-express');
const specs = require('../config/swagger');

// Custom CSS for Swagger UI
const customCss = `
  .swagger-ui .topbar { display: none }
  .swagger-ui .info { margin-bottom: 40px }
`;

const swaggerOptions = {
  customCss,
  customSiteTitle: 'Simple Todo API Documentation',
  customfavIcon: '/favicon.ico'
};

module.exports = {
  serve: swaggerUi.serve,
  setup: swaggerUi.setup(specs, swaggerOptions)
};
```

### 3. Add Swagger Annotations to Routes
Update `src/routes/todoRoutes.js` with JSDoc comments:
```javascript
/**
 * @swagger
 * /api/todos:
 *   get:
 *     summary: Get all todos
 *     description: Retrieve a list of all todos with optional filtering
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
 *           default: 50
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
 *               type: object
 *               properties:
 *                 data:
 *                   type: array
 *                   items:
 *                     $ref: '#/components/schemas/Todo'
 *                 count:
 *                   type: integer
 *                   description: Number of todos returned
 *                 limit:
 *                   type: integer
 *                   description: Applied limit
 *                 offset:
 *                   type: integer
 *                   description: Applied offset
 *       400:
 *         $ref: '#/components/responses/ValidationError'
 *       500:
 *         $ref: '#/components/responses/InternalError'
 */

/**
 * @swagger
 * /api/todos:
 *   post:
 *     summary: Create a new todo
 *     description: Create a new todo item with title and optional description
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
 *               type: object
 *               properties:
 *                 data:
 *                   $ref: '#/components/schemas/Todo'
 *                 message:
 *                   type: string
 *                   example: 'Todo created successfully'
 *       400:
 *         $ref: '#/components/responses/ValidationError'
 *       500:
 *         $ref: '#/components/responses/InternalError'
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
 *               type: object
 *               properties:
 *                 data:
 *                   $ref: '#/components/schemas/Todo'
 *       404:
 *         $ref: '#/components/responses/NotFound'
 *       500:
 *         $ref: '#/components/responses/InternalError'
 */

/**
 * @swagger
 * /api/todos/{id}:
 *   put:
 *     summary: Update a todo
 *     description: Update an existing todo's title, description, or completion status
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
 *               type: object
 *               properties:
 *                 data:
 *                   $ref: '#/components/schemas/Todo'
 *                 message:
 *                   type: string
 *                   example: 'Todo updated successfully'
 *       404:
 *         $ref: '#/components/responses/NotFound'
 *       400:
 *         $ref: '#/components/responses/ValidationError'
 *       500:
 *         $ref: '#/components/responses/InternalError'
 */

/**
 * @swagger
 * /api/todos/{id}:
 *   delete:
 *     summary: Delete a todo
 *     description: Delete a todo item by its ID
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
 *         $ref: '#/components/responses/NotFound'
 *       500:
 *         $ref: '#/components/responses/InternalError'
 */

/**
 * @swagger
 * tags:
 *   - name: Todos
 *     description: Todo management operations
 *   - name: Health
 *     description: Health check endpoints
 */
```

### 4. Add Health Route Documentation
Update `src/routes/healthRoutes.js`:
```javascript
/**
 * @swagger
 * /api/health:
 *   get:
 *     summary: Basic health check
 *     description: Check if the API is running and database is connected
 *     tags: [Health]
 *     responses:
 *       200:
 *         description: Service is healthy
 *         content:
 *           application/json:
 *             schema:
 *               $ref: '#/components/schemas/HealthCheck'
 *       503:
 *         description: Service unavailable
 *         content:
 *           application/json:
 *             schema:
 *               type: object
 *               properties:
 *                 status:
 *                   type: string
 *                   example: 'error'
 *                 error:
 *                   type: string
 *                   example: 'Service temporarily unavailable'
 */
```

### 5. Update App.js to Include Swagger
Update `src/app.js`:
```javascript
const swagger = require('./middleware/swagger');

// ... other middleware ...

// API Documentation - place before routes
app.use('/api-docs', swagger.serve, swagger.setup);

// Mount API routes
app.use('/api', routes);
```

## Success Criteria
- Swagger UI accessible at `/api-docs`
- All endpoints are documented with descriptions
- Request/response schemas are defined
- Interactive testing works through Swagger UI
- Examples are provided for all operations
- Error responses are documented
- API specification validates against OpenAPI 3.0

## Testing Considerations
- Verify Swagger UI loads correctly
- Test interactive API calls through Swagger
- Validate OpenAPI specification
- Ensure all routes are discovered
- Check that examples match actual responses

## Related Tasks
- **Dependency**: [Task 5: Implement API Routes](../task-5/task.md) - Documents these routes
- **Next**: [Task 7: Write Comprehensive Tests](../task-7/task.md)
- **Related**: [Task 8: Finalize and Document Project](../task-8/task.md) - References API docs

## References
- [Architecture Document](../../architecture.md) - Section: Technology Stack (Documentation)
- [Product Requirements](../../prd.txt) - Section: Technical Stack (OpenAPI/Swagger)