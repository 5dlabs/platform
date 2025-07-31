# Toolman Usage Guide: Setup Project Structure and Environment

## Overview

This guide provides detailed instructions on how to effectively use Toolman's tools for completing Task 1 - Setup Project Structure and Environment. Each tool serves a specific purpose in the workflow, from research to implementation and validation.

## Tool Categories and Usage

### 1. Research Tools

#### Web Search Tool
**Purpose**: Gather current best practices and industry standards

**When to use**:
- At the beginning of the task to inform architectural decisions
- When encountering specific technical questions
- To find solutions to common problems

**How to use**:
```
toolman search "React TypeScript monorepo best practices 2024"
toolman search "Docker multi-stage build Node.js optimization"
toolman search "Kubernetes deployment patterns microservices"
```

**Best practices**:
- Use specific, current search terms (include year)
- Search for multiple perspectives on the same topic
- Document findings for future reference

#### Documentation Lookup Tool
**Purpose**: Access official documentation for accurate configurations

**When to use**:
- After initial research to get specific implementation details
- When configuring tools and frameworks
- To verify syntax and options

**How to use**:
```
toolman docs react --topic "project-setup typescript"
toolman docs docker --topic "node best-practices"
toolman docs kubernetes --topic "deployment-configurations"
```

**Best practices**:
- Always verify version compatibility
- Cross-reference with multiple sources
- Save important snippets for reuse

### 2. Implementation Tools

#### Filesystem Tool
**Purpose**: Create and manipulate project files and directories

**When to use**:
- Creating the initial project structure
- Writing configuration files
- Generating boilerplate code

**How to use**:
```
toolman fs create-dir /chat-application/frontend/src/components
toolman fs create-file /chat-application/frontend/tsconfig.json
toolman fs write /chat-application/.gitignore --content "..."
```

**Best practices**:
- Create directories before files
- Use templates for common file types
- Verify paths before creating files

#### Shell Commands Tool
**Purpose**: Execute system commands for project initialization

**When to use**:
- Installing dependencies
- Running initialization scripts
- Testing builds and configurations

**How to use**:
```
toolman shell "cd frontend && npm create vite@latest . -- --template react-ts"
toolman shell "cd backend && npm init -y"
toolman shell "docker-compose build"
```

**Best practices**:
- Always use the correct working directory
- Check command output for errors
- Use --dry-run options when available

#### Code Generation Tool
**Purpose**: Generate boilerplate code and configurations

**When to use**:
- Creating standard configuration files
- Generating component templates
- Setting up common patterns

**How to use**:
```
toolman generate dockerfile --type node --stage multi
toolman generate config --type eslint --preset react-typescript
toolman generate kubernetes-deployment --name chat-backend
```

**Best practices**:
- Customize generated code for project needs
- Review generated code before committing
- Use consistent naming conventions

### 3. Validation Tools

#### Validation Tool
**Purpose**: Verify configurations and code quality

**When to use**:
- After creating configuration files
- Before committing changes
- As part of the completion checklist

**How to use**:
```
toolman validate typescript --project ./frontend
toolman validate dockerfile ./frontend/Dockerfile
toolman validate yaml ./kubernetes/deployment-configs/
```

**Best practices**:
- Run validation after each major change
- Fix issues immediately
- Use validation as a learning tool

## Workflow Sequence

### Phase 1: Research (30-45 minutes)

1. **Start with web searches** for current best practices:
   ```
   toolman search "React Node.js monorepo architecture 2024"
   toolman search "TypeScript project structure best practices"
   toolman search "Docker containerization patterns Node.js"
   ```

2. **Consult official documentation** for specific details:
   ```
   toolman docs react --topic "getting-started"
   toolman docs vite --topic "guide"
   toolman docs typescript --topic "tsconfig-reference"
   ```

3. **Document findings** for reference:
   ```
   toolman fs create-file ./research-notes.md
   toolman fs write ./research-notes.md --content "..."
   ```

### Phase 2: Structure Creation (15-20 minutes)

1. **Create root directory**:
   ```
   toolman fs create-dir /chat-application
   ```

2. **Build directory tree**:
   ```
   toolman shell "mkdir -p frontend/src/{components,pages,services,hooks,utils,types}"
   toolman shell "mkdir -p backend/src/{controllers,models,routes,services,middleware,utils,types}"
   toolman shell "mkdir -p kubernetes/deployment-configs shared/{types,utils}"
   ```

3. **Initialize Git repository**:
   ```
   toolman shell "cd /chat-application && git init"
   ```

### Phase 3: Frontend Setup (20-30 minutes)

1. **Initialize React application**:
   ```
   toolman shell "cd frontend && npm create vite@latest . -- --template react-ts"
   ```

2. **Configure TypeScript**:
   ```
   toolman generate config --type tsconfig --preset react
   toolman fs write ./frontend/tsconfig.json --content "..."
   ```

3. **Install additional dependencies**:
   ```
   toolman shell "cd frontend && npm install axios react-router-dom"
   toolman shell "cd frontend && npm install -D @types/react-router-dom"
   ```

### Phase 4: Backend Setup (20-30 minutes)

1. **Initialize Node.js project**:
   ```
   toolman shell "cd backend && npm init -y"
   ```

2. **Install dependencies**:
   ```
   toolman shell "cd backend && npm install express cors dotenv helmet morgan"
   toolman shell "cd backend && npm install -D typescript @types/node @types/express nodemon ts-node"
   ```

3. **Configure TypeScript**:
   ```
   toolman generate config --type tsconfig --preset node
   toolman fs write ./backend/tsconfig.json --content "..."
   ```

### Phase 5: Containerization (30-40 minutes)

1. **Create Dockerfiles**:
   ```
   toolman generate dockerfile --type react --output ./frontend/Dockerfile
   toolman generate dockerfile --type node --output ./backend/Dockerfile
   ```

2. **Create Docker Compose configuration**:
   ```
   toolman generate docker-compose --services "frontend,backend"
   toolman fs write ./docker-compose.yml --content "..."
   ```

3. **Create Kubernetes configurations**:
   ```
   toolman generate kubernetes-deployment --name chat-backend
   toolman generate kubernetes-service --name chat-backend
   ```

### Phase 6: Tooling Configuration (15-20 minutes)

1. **Set up ESLint**:
   ```
   toolman generate config --type eslint --preset monorepo
   toolman fs write ./.eslintrc.js --content "..."
   ```

2. **Configure Prettier**:
   ```
   toolman generate config --type prettier
   toolman fs write ./.prettierrc --content "..."
   ```

3. **Create .gitignore**:
   ```
   toolman generate gitignore --type node
   toolman fs write ./.gitignore --content "..."
   ```

### Phase 7: Validation (20-30 minutes)

1. **Validate TypeScript configurations**:
   ```
   toolman validate typescript --project ./frontend
   toolman validate typescript --project ./backend
   ```

2. **Test Docker builds**:
   ```
   toolman shell "docker build -t test-frontend ./frontend"
   toolman shell "docker build -t test-backend ./backend"
   ```

3. **Run development environment**:
   ```
   toolman shell "docker-compose up"
   toolman validate endpoints --url "http://localhost:5173"
   toolman validate endpoints --url "http://localhost:3000"
   ```

## Common Issues and Solutions

### Issue: TypeScript compilation errors
**Solution**:
```
toolman validate typescript --verbose
toolman docs typescript --topic "compiler-options"
```

### Issue: Docker build failures
**Solution**:
```
toolman shell "docker build --no-cache -t debug ."
toolman search "Docker build error [specific error message]"
```

### Issue: Hot reloading not working
**Solution**:
```
toolman search "Vite Docker hot reload issues"
toolman fs check-permissions ./frontend/src
```

## Tips for Efficient Tool Usage

1. **Batch similar operations**: Group filesystem operations together
2. **Use tool aliases**: Create shortcuts for frequently used commands
3. **Leverage templates**: Use code generation for repetitive tasks
4. **Document as you go**: Write notes while using tools
5. **Validate early and often**: Catch issues before they compound

## Tool Integration

Tools can be combined for powerful workflows:

```bash
# Research, generate, and validate in sequence
toolman search "Dockerfile best practices" | \
toolman generate dockerfile --input-stdin | \
toolman validate dockerfile --input-stdin

# Create and immediately edit files
toolman fs create-file ./config.json && \
toolman generate config --type app | \
toolman fs write ./config.json --input-stdin
```

## Conclusion

Effective use of Toolman's tools can significantly accelerate the project setup process while ensuring best practices are followed. The key is to use tools in the right sequence: research first, implement systematically, and validate thoroughly. This approach ensures a robust foundation for the chat application project.