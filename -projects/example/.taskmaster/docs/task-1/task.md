# Task 1: Setup Project Repository and Toolchain

## Overview

This task establishes the foundational development environment for the Task Board API project. It involves setting up Rust, required tools, Docker, and initializing the project structure with proper configuration for a production-ready microservice.

## Task Context

### Description
Initialize the Rust project, install required tools, and configure the development environment.

### Priority
High - This is the foundation upon which all other tasks depend.

### Dependencies
None - This is the initial setup task.

### Subtasks
1. Install Rust and Required Tools
2. Initialize the Rust Project
3. Configure Docker for Rust Development
4. Prepare Project Documentation Files

## Architecture Context

Per the architecture.md, this task sets up the technology stack:
- **Language**: Rust (latest stable version 1.75+)
- **Communication**: gRPC with Tonic framework (preparation)
- **Database**: PostgreSQL with Diesel ORM (tool installation)
- **Async Runtime**: Tokio (dependency setup)
- **Containerization**: Docker + Kubernetes (Docker setup)

The setup aligns with the high-level architecture's requirements for a Rust-based gRPC server that will communicate with PostgreSQL and support multiple client types.

## Implementation Details

### 1. Install Rust and Required Tools

#### Install Rust via rustup
```bash
# Download and install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add Rust to PATH (if not automatically done)
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

#### Install Required Components
```bash
# Install rustfmt for code formatting
rustup component add rustfmt

# Install Clippy for linting
rustup component add clippy

# Install cargo-edit for dependency management
cargo install cargo-edit

# Install cargo-watch for development
cargo install cargo-watch
```

#### Install Protocol Buffers Compiler
```bash
# On macOS
brew install protobuf

# On Ubuntu/Debian
sudo apt-get update && sudo apt-get install -y protobuf-compiler

# On other systems, download from GitHub releases
# https://github.com/protocolbuffers/protobuf/releases

# Verify installation (should be 3.21+ as per requirements)
protoc --version
```

#### Install Docker
```bash
# Follow official Docker installation guide for your OS
# https://docs.docker.com/get-docker/

# Verify installation (should be 24+ as per requirements)
docker --version
docker compose version
```

### 2. Initialize the Rust Project

#### Create New Cargo Project
```bash
# Create the project
cargo new task-board-api --bin
cd task-board-api

# Initialize git repository (if not already done)
git init
```

#### Configure Cargo.toml
```toml
[package]
name = "task-board-api"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "A collaborative task management API using gRPC"
license = "MIT"

[dependencies]
# Core async runtime
tokio = { version = "1.35", features = ["full"] }

# gRPC framework
tonic = "0.10"
prost = "0.12"

# Database ORM
diesel = { version = "2.1", features = ["postgres", "uuid", "chrono", "r2d2"] }
diesel_migrations = "2.1"

# Authentication
jsonwebtoken = "9.2"
argon2 = "0.5"

# Utilities
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Environment configuration
dotenv = "0.15"
config = "0.13"

[build-dependencies]
tonic-build = "0.10"

[dev-dependencies]
# Testing utilities
tokio-test = "0.4"
mockall = "0.12"
testcontainers = "0.15"

[profile.release]
# Optimize for size
opt-level = "z"
lto = true
codegen-units = 1
strip = true
```

### 3. Configure Development Environment

#### Create .gitignore
```gitignore
# Rust
target/
Cargo.lock
**/*.rs.bk
*.pdb

# IDE
.idea/
.vscode/
*.swp
*.swo
*~

# Environment
.env
.env.local
.env.*.local

# OS
.DS_Store
Thumbs.db

# Test coverage
tarpaulin-report.html
cobertura.xml

# Logs
*.log
```

#### Create .editorconfig
```ini
root = true

[*]
charset = utf-8
end_of_line = lf
insert_final_newline = true
trim_trailing_whitespace = true

[*.rs]
indent_style = space
indent_size = 4

[*.{toml,yaml,yml}]
indent_style = space
indent_size = 2

[*.md]
trim_trailing_whitespace = false
```

#### Create rustfmt.toml
```toml
edition = "2021"
max_width = 100
use_small_heuristics = "Max"
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
```

#### Create .cargo/config.toml
```toml
[build]
target-dir = "target"

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[alias]
check-all = "check --all-targets --all-features"
test-all = "test --all-targets --all-features"
```

### 4. Setup Docker Configuration

#### Create Dockerfile
```dockerfile
# Build stage
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Build dependencies (this is cached if manifests don't change)
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy source code
COPY . .

# Build application
RUN touch src/main.rs
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1001 -s /bin/bash appuser

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/task-board-api /usr/local/bin/task-board-api

# Change ownership
RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 50051

CMD ["task-board-api"]
```

#### Create docker-compose.yml
```yaml
version: '3.8'

services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: taskboard
      POSTGRES_PASSWORD: taskboard_password
      POSTGRES_DB: taskboard_db
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U taskboard"]
      interval: 10s
      timeout: 5s
      retries: 5

  app:
    build: .
    ports:
      - "50051:50051"
    environment:
      DATABASE_URL: postgres://taskboard:taskboard_password@postgres:5432/taskboard_db
      RUST_LOG: info
      JWT_SECRET: your-secret-key-here
    depends_on:
      postgres:
        condition: service_healthy
    volumes:
      - ./config:/app/config:ro

volumes:
  postgres_data:
```

### 5. Create Initial Project Structure

```bash
# Create directory structure
mkdir -p src/{models,services,handlers,utils}
mkdir -p proto
mkdir -p migrations
mkdir -p config
mkdir -p tests/{unit,integration}

# Create build.rs for protobuf compilation
cat > build.rs << 'EOF'
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/taskboard.proto")?;
    Ok(())
}
EOF

# Create initial main.rs
cat > src/main.rs << 'EOF'
use anyhow::Result;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "task_board_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Task Board API server...");

    // TODO: Initialize services and start gRPC server

    Ok(())
}
EOF
```

### 6. Documentation Setup

#### Create README.md
```markdown
# Task Board API

A collaborative task management service implemented as a Rust microservice using gRPC.

## Features
- User authentication with JWT
- Project board management
- Task CRUD operations
- Real-time updates via gRPC streaming
- PostgreSQL persistence

## Development Setup

### Prerequisites
- Rust 1.75+
- Docker 24+
- Protocol Buffers Compiler 3.21+

### Quick Start
1. Clone the repository
2. Install dependencies: `cargo build`
3. Start PostgreSQL: `docker compose up postgres`
4. Run migrations: `diesel migration run`
5. Start server: `cargo run`

## Architecture
See [architecture.md](.taskmaster/docs/architecture.md) for detailed system design.

## License
MIT
```

## Dependencies

This task has no dependencies as it's the initial setup. However, it enables:
- Task 2: Define gRPC Service Contracts (requires protoc)
- Task 3: Implement Database Schema and ORM (requires Diesel setup)
- Task 4-10: All subsequent implementation tasks

## Testing Strategy

### Verification Steps
1. **Toolchain Verification**
   ```bash
   cargo --version  # Should show 1.75+
   rustc --version  # Should match cargo version
   protoc --version # Should show 3.21+
   docker --version # Should show 24+
   ```

2. **Project Build Test**
   ```bash
   cd task-board-api
   cargo build
   cargo check
   cargo clippy
   cargo fmt --check
   ```

3. **Docker Build Test**
   ```bash
   docker build -t task-board-api .
   docker compose build
   ```

4. **Development Environment Test**
   - Open project in VS Code/preferred IDE
   - Verify Rust Analyzer works
   - Test code formatting on save
   - Verify clippy warnings appear

### Expected Outcomes
- All tools installed and accessible via PATH
- Project builds without errors
- Docker image builds successfully
- IDE integration works properly
- Initial project structure matches specification