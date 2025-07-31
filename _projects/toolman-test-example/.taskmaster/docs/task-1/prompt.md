# Autonomous Agent Prompt: Setup Project Structure and Environment

You are tasked with initializing a modern chat application project with the following requirements:

## Objective
Create a production-ready monorepo structure for a real-time chat application using React (frontend) and Node.js (backend), with TypeScript, Docker containerization, and Kubernetes deployment configurations.

## Detailed Requirements

### 1. Research Phase
- Search for current best practices for React + Node.js monorepo structures
- Investigate containerization patterns from the Rust ecosystem documentation
- Study Kubernetes orchestration requirements for microservices deployment

### 2. Project Structure
Create the following directory structure:
```
/chat-application
  /frontend (React 18+ with TypeScript)
  /backend (Node.js Express with TypeScript)
  /kubernetes (Deployment configurations)
  docker-compose.yml
  .gitignore
  README.md
```

### 3. Frontend Setup
- Initialize React 18+ with TypeScript (prefer Vite over create-react-app)
- Configure TypeScript with strict mode enabled
- Set up proper folder structure: components/, hooks/, services/, types/
- Install essential dependencies

### 4. Backend Setup
- Initialize Node.js project with Express and TypeScript
- Configure TypeScript for Node.js environment
- Set up folder structure: controllers/, middleware/, models/, routes/, utils/
- Install core dependencies: express, cors, dotenv, helmet
- Configure nodemon for development

### 5. Code Quality
- Configure ESLint for both frontend and backend
- Set up Prettier with consistent formatting rules
- Add pre-commit hooks if applicable
- Ensure both tools work with TypeScript

### 6. Docker Configuration
- Create multi-stage Dockerfile for frontend (build + nginx)
- Create Dockerfile for backend with production optimizations
- Write docker-compose.yml for local development
- Ensure containers can communicate properly

### 7. Kubernetes Preparation
- Create deployment YAML files for frontend and backend
- Configure services for pod communication
- Set up ConfigMaps and Secrets structure
- Design for horizontal scaling

### 8. Environment Setup
- Create .env.example with all required variables
- Configure environment variables for different stages
- Set up proper defaults and documentation

### 9. Development Scripts
Configure package.json scripts for:
- Development server with hot reloading
- Production builds
- Linting and formatting
- Testing (structure only)

### 10. Git Configuration
- Initialize Git repository
- Create comprehensive .gitignore
- Ensure no sensitive data is tracked

## Expected Deliverables

1. Complete project structure with all directories
2. Initialized frontend and backend with TypeScript
3. Working Docker setup (testable with docker-compose up)
4. ESLint and Prettier configurations
5. Kubernetes deployment configurations
6. Environment variable setup
7. Development scripts in package.json
8. Git repository with proper .gitignore

## Quality Criteria

- All TypeScript files must compile without errors
- Docker containers must build and run successfully
- Development servers must support hot reloading
- ESLint must pass with no errors (warnings acceptable)
- Project structure must follow modern best practices
- Kubernetes configs must be valid YAML

## Research Integration

Use the research findings to:
- Apply container optimization techniques from Rust ecosystem
- Implement Kubernetes best practices from the start
- Follow industry-standard project organization
- Ensure security best practices in all configurations

## Verification Steps

1. Run `npm install` in both frontend and backend directories
2. Execute `npm run dev` in both directories - servers should start
3. Run `docker-compose up` - containers should build and start
4. Validate Kubernetes YAML files syntax
5. Ensure TypeScript compilation succeeds
6. Verify ESLint and Prettier work correctly

Begin by researching best practices, then systematically create each component of the project structure.