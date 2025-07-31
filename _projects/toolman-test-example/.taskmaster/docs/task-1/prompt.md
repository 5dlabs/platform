# Autonomous Agent Prompt: Setup Project Structure and Environment

## Objective

Initialize a modern chat application project with a React frontend and Node.js backend in a monorepo structure, configured with TypeScript, development tooling, Docker containerization, and Kubernetes-ready deployment configurations.

## Context

You are tasked with creating the foundational structure for a real-time chat application. The project requires a scalable architecture that supports modern development practices, containerization for consistent deployments, and preparation for Kubernetes orchestration.

## Requirements

### Research Phase

Before implementation, conduct comprehensive research on:

1. **React + Node.js Best Practices**
   - Modern monorepo structures for full-stack JavaScript applications
   - Optimal directory organization for scalability
   - Dependency management strategies
   - Code sharing patterns between frontend and backend

2. **Containerization Patterns**
   - Docker best practices for Node.js applications
   - Multi-stage build optimization
   - Development vs production container strategies
   - Container security considerations

3. **Kubernetes Readiness**
   - Container configuration patterns for K8s compatibility
   - Resource limits and requests best practices
   - Health check implementation patterns
   - Service discovery preparation

### Implementation Requirements

1. **Project Structure**
   - Create a monorepo with separate frontend and backend directories
   - Include a shared directory for common code
   - Set up Kubernetes configuration directory
   - Implement proper separation of concerns

2. **Frontend Setup (React + TypeScript)**
   - Use Vite or Create React App with TypeScript template
   - Configure modern React 18+ features
   - Set up proper folder structure (components, pages, services, hooks, utils, types)
   - Configure path aliases for clean imports

3. **Backend Setup (Node.js + Express + TypeScript)**
   - Initialize Express server with TypeScript
   - Implement proper folder structure (controllers, models, routes, services, middleware)
   - Configure environment-based settings
   - Set up development and production scripts

4. **Development Tooling**
   - Configure ESLint with TypeScript support
   - Set up Prettier for code formatting
   - Create consistent coding standards
   - Configure Git hooks for code quality

5. **Containerization**
   - Create optimized Dockerfiles for both frontend and backend
   - Implement multi-stage builds for production
   - Set up docker-compose for local development
   - Ensure hot reloading works in containers

6. **Kubernetes Preparation**
   - Create basic deployment configurations
   - Design service definitions
   - Prepare for ConfigMaps and Secrets
   - Consider horizontal pod autoscaling

7. **Environment Configuration**
   - Set up environment variables for different stages
   - Implement secure secret management patterns
   - Create .env templates
   - Configure build-time vs runtime variables

## Expected Deliverables

1. Complete monorepo project structure with all required directories
2. Configured React frontend with TypeScript and modern tooling
3. Configured Node.js backend with Express and TypeScript
4. Docker configuration files for development and production
5. Basic Kubernetes deployment configurations
6. ESLint and Prettier configurations
7. Environment configuration templates
8. Comprehensive .gitignore file
9. Package.json with all necessary scripts
10. Documentation of research findings and architectural decisions

## Technical Constraints

- Use Node.js 18+ for both frontend and backend
- Implement TypeScript in strict mode
- Ensure all configurations support hot reloading in development
- Container images should be optimized for size and security
- Follow 12-factor app principles for configuration

## Success Criteria

- Project structure follows modern best practices discovered through research
- All development scripts work correctly (dev, build, lint, format)
- Docker containers build and run successfully
- Hot reloading functions properly in development environment
- TypeScript compilation has no errors
- Linting passes with no errors
- Project is ready for immediate development work
- Kubernetes configurations are valid and follow best practices

## Additional Considerations

- Research and document rationale for architectural decisions
- Consider future scalability requirements
- Implement security best practices from the start
- Ensure developer experience is smooth and efficient
- Plan for CI/CD pipeline integration
- Consider monitoring and logging requirements

## Execution Notes

Start with thorough research to inform your implementation decisions. Document your findings and use them to create an optimal project structure. Focus on creating a solid foundation that will support the application as it grows in complexity.