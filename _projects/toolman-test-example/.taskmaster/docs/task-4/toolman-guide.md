# Toolman Usage Guide for Chat Room API Implementation

## Overview

This guide explains how to use Toolman AI agents to implement the Chat Room API efficiently. The task involves creating a comprehensive REST API with proper security, performance optimization, and production-ready deployment configurations.

## Phase 1: Research and Analysis

### 1.1 Rust Pattern Research

Use the web search tool to research Rust HTTP server patterns:

```
toolman search "Actix-web middleware patterns REST API design"
toolman search "Rocket request guards authorization patterns"
toolman search "Warp filter composition API structure"
```

Expected outputs:
- Document middleware patterns applicable to Express.js
- Extract authorization strategies
- Identify error handling patterns
- Note performance optimization techniques

### 1.2 Express.js Best Practices

Research current Express.js security and design patterns:

```
toolman search "Express.js REST API security best practices 2024"
toolman search "Express.js performance optimization techniques"
toolman search "JWT authentication Express.js production"
```

Focus areas:
- Security middleware configuration
- Input validation strategies
- Rate limiting implementation
- CORS configuration

### 1.3 Kubernetes Deployment Research

Investigate Kubernetes deployment patterns:

```
toolman search "Kubernetes API service deployment best practices"
toolman search "Kubernetes ingress configuration Node.js"
toolman search "Horizontal pod autoscaling REST API"
```

Document findings on:
- Service definitions
- Ingress rules
- Resource limits
- Scaling strategies

## Phase 2: Design and Architecture

### 2.1 Database Schema Design

Use the database tool to create the schema:

```
toolman db create-schema --file database/schema.sql
```

Include:
- Rooms table with soft delete support
- Messages table with threading
- Room-user relationships
- Appropriate indexes for performance

### 2.2 API Specification

Generate OpenAPI specification:

```
toolman docs generate-openapi --output api/v1/spec.yaml
```

Ensure specification includes:
- All endpoints with descriptions
- Request/response schemas
- Authentication requirements
- Error response formats

### 2.3 Architecture Documentation

Create architecture diagram:

```
toolman docs create-diagram --type sequence --output docs/api-flow.md
```

Document:
- Request flow through middleware
- Database interaction patterns
- Caching strategy
- Real-time event emission

## Phase 3: Implementation

### 3.1 Project Structure Setup

Create the project structure:

```
toolman fs create-structure --template express-api
```

This creates:
- Controllers directory
- Routes configuration
- Middleware setup
- Model definitions
- Repository pattern

### 3.2 Room Management Implementation

Implement room endpoints:

```
toolman implement room-controller --spec api/v1/spec.yaml
```

The tool will:
- Generate controller methods
- Create route definitions
- Add validation middleware
- Implement authorization checks

### 3.3 Message Management Implementation

Implement message endpoints:

```
toolman implement message-controller --spec api/v1/spec.yaml
```

Features to implement:
- Pagination logic
- Threading support
- Soft delete functionality
- Real-time event emission

### 3.4 Security Middleware

Implement security layers:

```
toolman security add-middleware --type jwt
toolman security add-middleware --type rate-limit
toolman security add-middleware --type validation
```

Configure:
- JWT validation
- Role-based access control
- Input sanitization
- Rate limiting rules

## Phase 4: Testing

### 4.1 Unit Tests

Generate and run unit tests:

```
toolman test generate-unit --controllers
toolman test generate-unit --validators
toolman test run --type unit
```

Test coverage should include:
- All controller methods
- Validation logic
- Authorization checks
- Error handling

### 4.2 Integration Tests

Create integration tests:

```
toolman test generate-integration --spec api/v1/spec.yaml
toolman test run --type integration
```

Test scenarios:
- Complete room lifecycle
- Message posting and retrieval
- Authorization flows
- Pagination functionality

### 4.3 Performance Testing

Run performance benchmarks:

```
toolman performance test --tool k6 --config tests/k6-config.js
```

Metrics to validate:
- Response times under load
- Concurrent user handling
- Database query performance
- Memory usage patterns

## Phase 5: Deployment Preparation

### 5.1 Containerization

Create Docker configuration:

```
toolman container create-dockerfile --type node
toolman container build --tag chat-api:latest
```

Optimize for:
- Multi-stage builds
- Security best practices
- Minimal image size
- Health check endpoints

### 5.2 Kubernetes Manifests

Generate Kubernetes configurations:

```
toolman k8s generate-manifests --type api
```

Creates:
- Deployment with resource limits
- Service definition
- Ingress configuration
- ConfigMap for environment
- HPA for auto-scaling

### 5.3 Documentation Generation

Generate comprehensive documentation:

```
toolman docs generate-api --format markdown
toolman docs generate-readme
```

Include:
- API endpoint documentation
- Setup instructions
- Environment variables
- Deployment guide

## Best Practices for Toolman Usage

### 1. Research Phase
- Always start with research to understand patterns
- Document findings for future reference
- Compare multiple sources for best practices

### 2. Implementation Phase
- Use code generation for boilerplate
- Review generated code for quality
- Customize based on specific requirements

### 3. Testing Phase
- Generate comprehensive test suites
- Run tests incrementally during development
- Use performance testing early to identify issues

### 4. Security Considerations
- Enable all security tools
- Review security findings regularly
- Implement fixes before deployment

### 5. Documentation
- Generate documentation as you develop
- Keep API specs synchronized with code
- Include examples for all endpoints

## Common Toolman Commands

### File Operations
```bash
toolman fs create <path> --content <content>
toolman fs update <path> --pattern <pattern> --replacement <replacement>
toolman fs list <directory> --filter "*.ts"
```

### Code Analysis
```bash
toolman analyze security --path src/
toolman analyze performance --path src/api/
toolman analyze quality --strict
```

### Database Operations
```bash
toolman db migrate --up
toolman db seed --file seeds/test-data.sql
toolman db optimize --analyze
```

### Testing Operations
```bash
toolman test coverage --threshold 80
toolman test watch --path src/
toolman test report --format html
```

## Troubleshooting

### Common Issues

1. **Performance Issues**
   - Use `toolman analyze performance` to identify bottlenecks
   - Check database query optimization
   - Review caching implementation

2. **Security Vulnerabilities**
   - Run `toolman security scan` regularly
   - Update dependencies with `toolman deps update --security`
   - Review authentication logic

3. **Test Failures**
   - Use `toolman test debug <test-name>` for detailed output
   - Check test database state
   - Verify environment variables

### Getting Help

```bash
toolman help <command>
toolman docs <topic>
toolman examples <pattern>
```

## Workflow Optimization Tips

1. **Parallel Execution**
   - Run research tasks in parallel
   - Execute independent tests concurrently
   - Build and test simultaneously

2. **Incremental Development**
   - Implement one endpoint at a time
   - Test immediately after implementation
   - Commit working code frequently

3. **Continuous Validation**
   - Run linting on save
   - Execute unit tests on change
   - Monitor performance metrics

4. **Documentation as Code**
   - Update API specs with code changes
   - Generate docs from code comments
   - Validate documentation accuracy

## Final Checklist

Before considering the task complete:

- [ ] All endpoints implemented and tested
- [ ] Security measures in place
- [ ] Performance benchmarks met
- [ ] Documentation complete
- [ ] Kubernetes deployment ready
- [ ] Code quality standards met
- [ ] Integration tests passing
- [ ] Research findings documented

Use this guide to systematically implement the Chat Room API, leveraging Toolman's capabilities for efficient development and high-quality output.