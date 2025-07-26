# API Surface Analysis

## routes (orchestrator/core/src/main.rs)

### Endpoints

- **POST** `/pm/tasks` - Line 71
  ```rust
  .route("/pm/tasks", post(submit_code_task))
  ```

- **POST** `/pm/docs/generate` - Line 72
  ```rust
  .route("/pm/docs/generate", post(generate_docs))
  ```

- **GET** `/health` - Line 110
  ```rust
  .route("/health", get(health_check)) // Root health check for load balancers
  ```

