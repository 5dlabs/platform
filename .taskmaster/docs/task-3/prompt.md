# Remove API Server from Orchestrator Core

Prerequisites:
- Argo Workflows system functioning
- Dependencies 6, 7, 14 completed
- Backup branch created

Remove HTTP API server:

1. Remove from main.rs:
   ```rust
   // Remove server init and routes
   // Keep controller initialization
   ```

2. Delete API components:
   - handlers/ directory
   - route definitions
   - middleware code

3. Update Cargo.toml:
   ```toml
   # Remove:
   axum = { version = "..." }
   tower = { version = "..." }
   ```

4. Update configuration
5. Test controller functionality

Success: Controller runs independently with proper CRD reconciliation.