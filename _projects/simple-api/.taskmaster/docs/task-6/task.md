# Task 6: Implement API Documentation with Swagger

## Overview

This task adds comprehensive API documentation using OpenAPI/Swagger specification. The documentation will be auto-generated from code comments and served through an interactive UI, making the API self-documenting and easy to explore.

## Context

Following the API implementation from Tasks 3-5, this task creates interactive documentation that helps developers understand and test the API. As specified in the [PRD](../prd.txt), comprehensive documentation is a key requirement for maintainability and usability.

## Implementation Guide

### 1. Create Swagger Configuration (src/config/swagger.js)

Set up the OpenAPI specification:

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
              example: 'Add Swagger documentation to all endpoints',
              nullable: true
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
              example: '2023-01-01T00:00:00.000Z'
            },
            updatedAt: {
              type: 'string',
              format: 'date-time',
              description: 'Last update timestamp',
              example: '2023-01-01T00:00:00.000Z'
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
              maxLength: 200,
              example: 'Updated todo title'
            },
            description: {
              type: 'string',
              description: 'Updated description',
              maxLength: 1000,
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
              type: 'string',
              description: 'Error type',
              example: 'Validation Error'
            },
            message: {
              type: 'string',
              description: 'Error message',
              example: 'Title is required'
            },
            details: {
              type: 'array',
              description: 'Detailed error information',
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
            },
            requestId: {
              type: 'string',
              description: 'Request tracking ID',
              example: 'req_123abc'
            }
          }
        },
        HealthStatus: {
          type: 'object',
          properties: {
            status: {
              type: 'string',
              enum: ['healthy', 'unhealthy'],
              example: 'healthy'
            },
            timestamp: {
              type: 'string',
              format: 'date-time',
              example: '2023-01-01T00:00:00.000Z'
            },
            service: {
              type: 'string',
              example: 'todo-api'
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
  apis: ['./src/routes/*.js', './src/swagger/*.js']
};

const specs = swaggerJsdoc(options);

module.exports = specs;
```

### 2. Create Swagger Documentation Files

Create src/swagger/todos.js for todo-specific documentation:

```javascript
/**
 * @swagger
 * /api/todos:
 *   get:
 *     summary: List all todos
 *     description: Retrieve a list of all todos with optional filtering and pagination
 *     tags: [Todos]
 *     parameters:
 *       - in: query
 *         name: completed
 *         schema:
 *           type: boolean
 *         description: Filter by completion status
 *         example: true
 *       - in: query
 *         name: limit
 *         schema:
 *           type: integer
 *           minimum: 1
 *           maximum: 100
 *         description: Maximum number of todos to return
 *         example: 10
 *       - in: query
 *         name: offset
 *         schema:
 *           type: integer
 *           minimum: 0
 *         description: Number of todos to skip
 *         example: 0
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
 *                   example: 10
 *                 filters:
 *                   type: object
 *                   properties:
 *                     completed:
 *                       type: boolean
 *                     limit:
 *                       type: integer
 *                     offset:
 *                       type: integer
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
 *     description: Create a new todo item with a title and optional description
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
 *                 message:
 *                   type: string
 *                   example: 'Todo created successfully'
 *                 data:
 *                   $ref: '#/components/schemas/Todo'
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
 *         description: Todo ID
 *         example: 1
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
 *       400:
 *         $ref: '#/components/responses/ValidationError'
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
 *         description: Todo ID
 *         example: 1
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
 *                 message:
 *                   type: string
 *                   example: 'Todo updated successfully'
 *                 data:
 *                   $ref: '#/components/schemas/Todo'
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
 *     description: Permanently delete a todo item
 *     tags: [Todos]
 *     parameters:
 *       - in: path
 *         name: id
 *         required: true
 *         schema:
 *           type: integer
 *         description: Todo ID
 *         example: 1
 *     responses:
 *       204:
 *         description: Todo deleted successfully
 *       404:
 *         $ref: '#/components/responses/NotFound'
 *       400:
 *         $ref: '#/components/responses/ValidationError'
 *       500:
 *         $ref: '#/components/responses/InternalError'
 */

/**
 * @swagger
 * /api/todos/stats/summary:
 *   get:
 *     summary: Get todo statistics
 *     description: Retrieve statistics about todos including total count and completion rate
 *     tags: [Todos]
 *     responses:
 *       200:
 *         description: Statistics retrieved successfully
 *         content:
 *           application/json:
 *             schema:
 *               type: object
 *               properties:
 *                 data:
 *                   type: object
 *                   properties:
 *                     total:
 *                       type: integer
 *                       example: 10
 *                     completed:
 *                       type: integer
 *                       example: 3
 *                     pending:
 *                       type: integer
 *                       example: 7
 *                     completionRate:
 *                       type: integer
 *                       example: 30
 *       500:
 *         $ref: '#/components/responses/InternalError'
 */

/**
 * @swagger
 * tags:
 *   - name: Todos
 *     description: Todo management operations
 */
```

Create src/swagger/health.js for health check documentation:

```javascript
/**
 * @swagger
 * /api/health:
 *   get:
 *     summary: Health check
 *     description: Check the health status of the API and its dependencies
 *     tags: [Health]
 *     responses:
 *       200:
 *         description: Service is healthy
 *         content:
 *           application/json:
 *             schema:
 *               $ref: '#/components/schemas/HealthStatus'
 *       503:
 *         description: Service is unhealthy
 *         content:
 *           application/json:
 *             schema:
 *               $ref: '#/components/schemas/HealthStatus'
 */

/**
 * @swagger
 * /api/health/ready:
 *   get:
 *     summary: Readiness check
 *     description: Check if the service is ready to handle requests
 *     tags: [Health]
 *     responses:
 *       200:
 *         description: Service is ready
 *         content:
 *           application/json:
 *             schema:
 *               type: object
 *               properties:
 *                 ready:
 *                   type: boolean
 *                   example: true
 *       503:
 *         description: Service is not ready
 *         content:
 *           application/json:
 *             schema:
 *               type: object
 *               properties:
 *                 ready:
 *                   type: boolean
 *                   example: false
 */

/**
 * @swagger
 * /api/health/live:
 *   get:
 *     summary: Liveness check
 *     description: Check if the service is alive
 *     tags: [Health]
 *     responses:
 *       200:
 *         description: Service is alive
 *         content:
 *           application/json:
 *             schema:
 *               type: object
 *               properties:
 *                 alive:
 *                   type: boolean
 *                   example: true
 */

/**
 * @swagger
 * tags:
 *   - name: Health
 *     description: Service health monitoring
 */
```

### 3. Create Swagger Middleware (src/middleware/swagger.js)

Set up Swagger UI middleware:

```javascript
const swaggerUi = require('swagger-ui-express');
const specs = require('../config/swagger');

// Custom CSS for better UI
const customCss = `
  .swagger-ui .topbar { display: none }
  .swagger-ui .info .title { font-size: 2em }
  .swagger-ui .scheme-container { display: none }
`;

const swaggerOptions = {
  customCss,
  customSiteTitle: 'Simple Todo API Documentation',
  customfavIcon: '/favicon.ico',
  swaggerOptions: {
    persistAuthorization: true,
    displayRequestDuration: true,
    docExpansion: 'none',
    filter: true,
    showExtensions: true,
    showCommonExtensions: true,
    displayOperationId: false
  }
};

module.exports = {
  serve: swaggerUi.serve,
  setup: swaggerUi.setup(specs, swaggerOptions)
};
```

### 4. Update Express App (src/app.js)

Add Swagger documentation to the app:

```javascript
const express = require('express');
const routes = require('./routes');
const swagger = require('./middleware/swagger');
const { requestId, responseTime } = require('./middleware/common');

const app = express();

// Body parsing middleware
app.use(express.json());
app.use(express.urlencoded({ extended: true }));

// Common middleware
app.use(requestId);
app.use(responseTime);

// Swagger documentation - BEFORE routes
app.use('/api-docs', swagger.serve, swagger.setup);

// API Routes
app.use('/api', routes);

// Root endpoint with documentation link
app.get('/', (req, res) => {
  res.json({
    message: 'Welcome to Simple Todo API',
    version: '1.0.0',
    documentation: `${req.protocol}://${req.get('host')}/api-docs`,
    api: `${req.protocol}://${req.get('host')}/api`,
    endpoints: {
      todos: '/api/todos',
      health: '/api/health',
      documentation: '/api-docs'
    }
  });
});

// ... rest of the app setup
```

### 5. Create API Examples File (src/swagger/examples.js)

Add usage examples:

```javascript
/**
 * @swagger
 * /api:
 *   get:
 *     summary: API information
 *     description: Get information about available endpoints
 *     tags: [General]
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
 *                   example: 'Simple Todo REST API'
 *                 version:
 *                   type: string
 *                   example: '1.0.0'
 *                 endpoints:
 *                   type: object
 *                   properties:
 *                     todos:
 *                       type: string
 *                       example: '/api/todos'
 *                     health:
 *                       type: string
 *                       example: '/api/health'
 *                     documentation:
 *                       type: string
 *                       example: '/api-docs'
 */

/**
 * @example
 * // Create a new todo
 * POST /api/todos
 * {
 *   "title": "Complete project documentation",
 *   "description": "Add comprehensive API documentation"
 * }
 * 
 * // Update todo status
 * PUT /api/todos/1
 * {
 *   "completed": true
 * }
 * 
 * // Filter completed todos with pagination
 * GET /api/todos?completed=true&limit=10&offset=0
 */
```

## Dependencies and Relationships

- **Depends on**: 
  - Task 5 (API Routes) - Documents existing routes
  - Task 3 (Validation) - Documents validation rules
  - Task 4 (Controllers) - Documents response formats
- **Required by**: 
  - Task 8 (Finalize Project) - Documentation is part of completion

## Success Criteria

1. ✅ Swagger UI accessible at /api-docs
2. ✅ All endpoints documented with descriptions
3. ✅ Request/response schemas defined
4. ✅ Example values provided for all fields
5. ✅ Error responses documented
6. ✅ Interactive testing available in UI
7. ✅ Proper grouping with tags
8. ✅ Server URLs configured
9. ✅ OpenAPI 3.0 compliant

## Testing

1. **Access Swagger UI**:
   ```bash
   npm run dev
   # Open browser to http://localhost:3000/api-docs
   ```

2. **Verify documentation completeness**:
   - All endpoints listed
   - All parameters documented
   - All responses documented
   - Examples work correctly

3. **Test interactive features**:
   - Try executing requests from Swagger UI
   - Verify responses match documentation
   - Test with different parameters

4. **Validate OpenAPI spec**:
   ```bash
   # Install swagger-cli globally
   npm install -g @apidevtools/swagger-cli
   
   # Validate spec
   swagger-cli validate src/config/swagger.js
   ```

## Common Issues and Solutions

1. **Swagger UI not loading**: Check middleware order - Swagger must be before routes
2. **Missing endpoints**: Ensure JSDoc comments are in files specified in swagger config
3. **Schema errors**: Validate component references match exactly
4. **Examples not showing**: Use proper @example syntax

## Documentation Best Practices

- Use clear, concise descriptions
- Provide realistic examples
- Document all possible responses
- Include error scenarios
- Group related endpoints with tags
- Keep schemas DRY with references

## Next Steps

After completing this task:
- Task 7: Write tests for the documented API
- Task 8: Include API documentation in final project

The API is now fully documented with an interactive interface for exploration and testing.