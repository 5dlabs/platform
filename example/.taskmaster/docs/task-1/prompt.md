# Autonomous Prompt for Task 1: Setup Project Repository and Toolchain

## Context

You are tasked with setting up the initial development environment for the Task Board API project. This is a Rust-based microservice that uses gRPC for communication and PostgreSQL for persistence. The project follows the architecture defined in the Task Master platform and serves as an example application for demonstrating autonomous AI development workflows.

## Task Requirements

### Primary Objective
Initialize a complete Rust development environment with all necessary tools, dependencies, and project structure for building a gRPC-based task management API.

### Required Tool Installations
1. **Rust Toolchain**
   - Install Rust via rustup (latest stable, 1.75+)
   - Add rustfmt and clippy components
   - Install cargo-edit and cargo-watch

2. **Protocol Buffers**
   - Install protoc compiler (version 3.21+)
   - Ensure it's accessible via PATH

3. **Docker**
   - Install Docker Engine (version 24+)
   - Install Docker Compose
   - Verify both are working correctly

### Project Initialization
1. Create new Rust project named `task-board-api`
2. Configure Cargo.toml with all required dependencies:
   - Tokio (async runtime)
   - Tonic (gRPC framework)
   - Diesel (ORM with PostgreSQL support)
   - JWT and password hashing libraries
   - Tracing for structured logging
   - Error handling libraries

3. Set up project directory structure:
   ```
   task-board-api/
   ├── src/
   │   ├── models/
   │   ├── services/
   │   ├── handlers/
   │   └── utils/
   ├── proto/
   ├── migrations/
   ├── config/
   └── tests/
       ├── unit/
       └── integration/
   ```

### Configuration Files
Create all necessary configuration files:
- `.gitignore` - Rust-specific ignores
- `.editorconfig` - Editor configuration
- `rustfmt.toml` - Rust formatting rules
- `.cargo/config.toml` - Cargo configuration
- `Dockerfile` - Multi-stage build for production
- `docker-compose.yml` - Local development setup with PostgreSQL
- `build.rs` - Proto compilation setup

## Implementation Instructions

### Step 1: Install Development Tools
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Add components
rustup component add rustfmt clippy
cargo install cargo-edit cargo-watch

# Install protoc (adapt for your OS)
# macOS: brew install protobuf
# Ubuntu: sudo apt-get install -y protobuf-compiler

# Install Docker (follow official guide for your OS)
```

### Step 2: Create Project Structure
```bash
cargo new task-board-api --bin
cd task-board-api
mkdir -p src/{models,services,handlers,utils}
mkdir -p proto migrations config tests/{unit,integration}
```

### Step 3: Configure Dependencies
Update Cargo.toml with the full dependency list provided in the task documentation. Ensure all versions match the architecture requirements.

### Step 4: Docker Setup
Create both Dockerfile and docker-compose.yml files. The Dockerfile should use multi-stage builds for optimal image size. The compose file should include PostgreSQL with health checks.

### Step 5: Development Environment
Set up all configuration files for a professional development workflow including formatting, linting, and IDE integration.

### Step 6: Initial Code
Create a basic main.rs that initializes tracing and prepares for the gRPC server implementation.

## Success Criteria

### Verification Checklist
1. **Tool Versions**
   - [ ] `cargo --version` shows 1.75 or higher
   - [ ] `protoc --version` shows 3.21 or higher
   - [ ] `docker --version` shows 24 or higher
   - [ ] All cargo tools installed successfully

2. **Project Structure**
   - [ ] All directories created as specified
   - [ ] Configuration files present and properly formatted
   - [ ] Cargo.toml contains all required dependencies

3. **Build Verification**
   - [ ] `cargo build` completes without errors
   - [ ] `cargo check` passes
   - [ ] `cargo clippy` runs without errors
   - [ ] `cargo fmt --check` passes

4. **Docker Verification**
   - [ ] `docker build -t task-board-api .` succeeds
   - [ ] `docker compose build` completes
   - [ ] PostgreSQL container starts with health check

5. **IDE Integration**
   - [ ] Project opens in IDE with Rust support
   - [ ] Code completion and error checking work
   - [ ] Format on save functions correctly

### Expected Outcomes
- Fully configured Rust development environment
- Project structure ready for gRPC service implementation
- Docker setup for local development and production builds
- All tools and dependencies installed and verified
- Foundation laid for all subsequent development tasks

### Important Notes
- This task has no dependencies but enables all other tasks
- Pay special attention to version requirements from architecture.md
- Ensure all paths are properly configured in environment
- Document any OS-specific installation steps encountered