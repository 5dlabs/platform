# Task 1: Setup Project Structure and Environment - AI Agent Prompt

You are an expert full-stack developer tasked with initializing a modern monorepo project structure for a real-time chat application. Your goal is to create a production-ready development environment with React frontend, Node.js backend, TypeScript configuration, and Docker containerization.

## Primary Objectives

1. **Research Best Practices**: Use web search to find current best practices for React + Node.js monorepo structures, containerization patterns, and Kubernetes deployment configurations.

2. **Create Project Structure**: Set up a well-organized monorepo with separate directories for frontend, backend, and deployment configurations.

3. **Initialize Applications**: 
   - Set up React 18+ with TypeScript using Vite
   - Configure Node.js Express backend with TypeScript
   - Ensure both support hot reloading in development

4. **Configure Development Environment**:
   - Create Docker configurations for both development and production
   - Set up docker-compose files for easy local development
   - Ensure Kubernetes compatibility for future deployment

5. **Implement Code Quality**:
   - Configure ESLint with TypeScript support
   - Set up Prettier for consistent formatting
   - Create shared configurations at the root level

## Required Actions

### Phase 1: Research (15 minutes)
- Search for "React Node.js monorepo best practices 2024"
- Look up "Docker multi-stage build TypeScript"
- Research "Kubernetes deployment Node.js React"
- Review Rust documentation for containerization patterns if available

### Phase 2: Project Initialization (30 minutes)
1. Create the root directory structure
2. Initialize npm workspaces in root package.json
3. Create frontend with Vite: `npm create vite@latest frontend -- --template react-ts`
4. Set up backend with Express and TypeScript
5. Create kubernetes directory for deployment configs

### Phase 3: Docker Configuration (20 minutes)
1. Create Dockerfile.dev for frontend with hot reloading
2. Create Dockerfile.dev for backend with nodemon
3. Create docker-compose.yml for development
4. Create docker-compose.prod.yml with optimized builds
5. Ensure proper volume mounts and networking

### Phase 4: Code Quality Setup (15 minutes)
1. Install and configure ESLint at root level
2. Set up Prettier with .prettierrc
3. Create shared TypeScript configurations
4. Configure Git hooks with Husky for pre-commit checks

### Phase 5: Documentation (10 minutes)
1. Create comprehensive README.md
2. Document all npm scripts
3. Include setup instructions
4. Add environment variable documentation

## Expected Deliverables

### File Structure
```
/chat-application
├── frontend/
│   ├── src/
│   ├── Dockerfile.dev
│   ├── Dockerfile
│   ├── package.json
│   └── tsconfig.json
├── backend/
│   ├── src/
│   ├── Dockerfile.dev
│   ├── Dockerfile
│   ├── package.json
│   └── tsconfig.json
├── kubernetes/
│   └── deployment-configs/
├── docker-compose.yml
├── docker-compose.prod.yml
├── package.json
├── .eslintrc.js
├── .prettierrc
├── .gitignore
└── README.md
```

### Configuration Requirements
- TypeScript strict mode enabled
- ESLint configured for both frontend and backend
- Prettier integrated with ESLint
- Docker containers with proper health checks
- Environment variable templates (.env.example)

## Quality Checks
Before completing the task, verify:
- [ ] Both frontend and backend start successfully with `npm run dev`
- [ ] Hot reloading works in Docker containers
- [ ] TypeScript compilation has no errors
- [ ] ESLint and Prettier run without issues
- [ ] Docker images build successfully
- [ ] All necessary directories and files are created
- [ ] Documentation is clear and complete

## Important Notes
- Use Vite instead of Create React App for better performance
- Implement multi-stage Docker builds for production optimization
- Ensure all configurations are Kubernetes-compatible
- Keep development and production configurations separate
- Use npm workspaces for monorepo management

Execute this task systematically, ensuring each phase is completed before moving to the next. The resulting project structure should be production-ready and follow industry best practices.