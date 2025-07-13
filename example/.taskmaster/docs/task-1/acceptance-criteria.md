# Acceptance Criteria for Task 1: Setup Project Repository and Toolchain

## Functional Requirements

### FR-1: Development Tools Installation
- **FR-1.1**: Rust toolchain must be installed via rustup with version 1.75 or higher
- **FR-1.2**: Essential Rust components (rustfmt, clippy) must be available
- **FR-1.3**: Cargo extension tools (cargo-edit, cargo-watch) must be installed
- **FR-1.4**: Protocol Buffers compiler (protoc) version 3.21+ must be installed
- **FR-1.5**: Docker Engine version 24+ and Docker Compose must be installed

### FR-2: Project Initialization
- **FR-2.1**: A new Rust binary project named "task-board-api" must be created
- **FR-2.2**: Project must have a valid Cargo.toml with all required dependencies
- **FR-2.3**: Git repository must be initialized (if not already present)

### FR-3: Project Structure
- **FR-3.1**: Source code directories must follow the specified structure
- **FR-3.2**: All required configuration files must be present
- **FR-3.3**: Docker configuration must support both development and production

## Technical Requirements

### TR-1: Dependency Configuration
- **TR-1.1**: Cargo.toml must include:
  - Tokio with full features for async runtime
  - Tonic and prost for gRPC support
  - Diesel with PostgreSQL, UUID, and chrono features
  - Authentication libraries (jsonwebtoken, argon2)
  - Logging and error handling libraries
  - All specified utility libraries

### TR-2: Development Environment
- **TR-2.1**: rustfmt.toml must enforce consistent code formatting
- **TR-2.2**: .cargo/config.toml must optimize build performance
- **TR-2.3**: .gitignore must exclude all build artifacts and sensitive files
- **TR-2.4**: .editorconfig must ensure consistent file formatting

### TR-3: Container Configuration
- **TR-3.1**: Dockerfile must use multi-stage builds for optimal image size
- **TR-3.2**: Dockerfile must run as non-root user for security
- **TR-3.3**: docker-compose.yml must include PostgreSQL with health checks
- **TR-3.4**: Container must expose port 50051 for gRPC

### TR-4: Build Configuration
- **TR-4.1**: build.rs must be configured for protobuf compilation
- **TR-4.2**: Release profile must optimize for size with LTO enabled

## Test Cases

### TC-1: Tool Installation Verification
```bash
# Test Case 1.1: Verify Rust installation
cargo --version
# Expected: cargo 1.75.0 or higher

# Test Case 1.2: Verify rustc installation
rustc --version
# Expected: rustc 1.75.0 or higher

# Test Case 1.3: Verify protoc installation
protoc --version
# Expected: libprotoc 3.21.0 or higher

# Test Case 1.4: Verify Docker installation
docker --version
# Expected: Docker version 24.0.0 or higher

# Test Case 1.5: Verify Docker Compose
docker compose version
# Expected: Docker Compose version v2.x.x
```

### TC-2: Project Build Verification
```bash
# Test Case 2.1: Build project
cd task-board-api && cargo build
# Expected: Build completes successfully

# Test Case 2.2: Run checks
cargo check
# Expected: No errors

# Test Case 2.3: Run linter
cargo clippy -- -D warnings
# Expected: No warnings or errors

# Test Case 2.4: Check formatting
cargo fmt --check
# Expected: No formatting changes needed
```

### TC-3: Docker Build Verification
```bash
# Test Case 3.1: Build Docker image
docker build -t task-board-api .
# Expected: Image builds successfully

# Test Case 3.2: Verify image size
docker images task-board-api
# Expected: Image size is reasonable (< 200MB)

# Test Case 3.3: Build with compose
docker compose build
# Expected: All services build successfully

# Test Case 3.4: Start PostgreSQL
docker compose up -d postgres
# Expected: PostgreSQL starts and passes health check
```

### TC-4: Project Structure Verification
```bash
# Test Case 4.1: Verify directory structure
find . -type d -name "models" -o -name "services" -o -name "handlers" | wc -l
# Expected: 3 directories found

# Test Case 4.2: Verify configuration files
ls -la .gitignore .editorconfig rustfmt.toml Dockerfile docker-compose.yml build.rs
# Expected: All files exist

# Test Case 4.3: Verify proto directory
ls -d proto/
# Expected: Directory exists

# Test Case 4.4: Verify test structure
ls -d tests/unit tests/integration
# Expected: Both directories exist
```

## Verification Steps

### Step 1: Environment Verification
1. Open a new terminal session
2. Run all tool version checks (TC-1)
3. Verify all tools are accessible via PATH
4. Document any OS-specific issues encountered

### Step 2: Project Verification
1. Navigate to the project directory
2. Run all build verification tests (TC-2)
3. Ensure no errors or warnings are present
4. Verify IDE integration works properly

### Step 3: Docker Verification
1. Ensure Docker daemon is running
2. Run all Docker tests (TC-3)
3. Verify PostgreSQL connectivity
4. Check resource usage is reasonable

### Step 4: Structure Verification
1. Run all structure tests (TC-4)
2. Verify all required files are present
3. Check file permissions are correct
4. Ensure no unnecessary files are tracked by git

## Success Metrics

### Quantitative Metrics
- Tool version requirements: 100% compliance
- Build tests: 0 errors, 0 warnings
- Docker build time: < 5 minutes
- Docker image size: < 200MB
- All configuration files: Present and valid

### Qualitative Metrics
- Development environment is fully functional
- Project structure supports planned architecture
- Configuration follows Rust best practices
- Docker setup enables easy local development
- Foundation is solid for subsequent tasks

## Edge Cases and Error Handling

### EC-1: Missing System Dependencies
- If protoc installation fails, provide alternative installation methods
- If Docker requires sudo, document permission setup
- Handle different shell environments (.bashrc vs .zshrc)

### EC-2: Network Issues
- Cargo dependency downloads may fail - retry with backoff
- Docker image pulls may timeout - use mirrors if needed
- Consider offline/airgapped environments

### EC-3: Platform Differences
- Document Windows-specific setup (WSL2 recommended)
- Handle macOS arm64 vs x86_64 differences
- Account for various Linux distributions

## Dependencies and Blockers

### Dependencies
- None - this is the initial setup task

### Potential Blockers
- Corporate proxy configurations
- Restricted internet access
- Insufficient disk space (requires ~2GB)
- Incompatible system libraries

### Enables
- All subsequent tasks (2-10) depend on this task's completion
- Specifically enables immediate start of Task 2 (gRPC contracts) and Task 3 (database schema)