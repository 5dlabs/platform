# Task Requirements System

## Overview

The Task Requirements system provides a declarative way to specify environment variables and secrets needed for task execution. Instead of having agents manually specify these in their MCP calls (which is error-prone), tasks can now include a `requirements.yaml` file that automatically configures the execution environment.

## How It Works

1. **Task Author** creates a `requirements.yaml` file in the task directory
2. **MCP Server** detects and reads the requirements file when the task is executed
3. **Controller** processes the requirements and configures the container environment
4. **Agent** receives the properly configured environment without needing to know about secrets

## Requirements File Location

Place the requirements file at:
```
.taskmaster/docs/task-{id}/requirements.yaml
```

For example:
- Task 5: `.taskmaster/docs/task-5/requirements.yaml`
- Task 10: `.taskmaster/docs/task-10/requirements.yaml`

## File Format

The requirements file uses YAML format with two main sections:

### 1. Secrets Section

References existing Kubernetes secrets and specifies how to mount them.

#### Option A: Mount Entire Secret
```yaml
secrets:
  - name: birdeye-secret  # All keys become env vars
  - name: stripe-secret
```

With this approach, all keys in the secret are mounted as environment variables with the same names as the keys.

#### Option B: Mount Specific Keys with Custom Names
```yaml
secrets:
  - name: stripe-secret
    keys:
      - secret_key: STRIPE_SECRET_KEY        # Maps 'secret_key' to env var 'STRIPE_SECRET_KEY'
      - webhook_secret: STRIPE_WEBHOOK_SECRET
      - publishable_key: STRIPE_PUBLISHABLE_KEY
```

This gives you precise control over which keys are mounted and what environment variable names they use.

### 2. Environment Section

Static environment variables that don't need to be kept secret.

```yaml
environment:
  NODE_ENV: "production"
  API_VERSION: "v2"
  LOG_LEVEL: "info"
  ENABLE_CACHE: "true"
  CACHE_TTL: "3600"
```

## Complete Example

```yaml
# .taskmaster/docs/task-5/requirements.yaml
secrets:
  # Mount entire secret - all keys become env vars
  - name: openai-credentials
  
  # Mount specific keys with custom env var names
  - name: database-credentials
    keys:
      - host: DATABASE_HOST
      - port: DATABASE_PORT
      - username: DATABASE_USER
      - password: DATABASE_PASSWORD
      - database: DATABASE_NAME
  
  - name: stripe-secret
    keys:
      - secret_key: STRIPE_SECRET_KEY
      - webhook_secret: STRIPE_WEBHOOK_SECRET

environment:
  # API configuration
  API_VERSION: "v2"
  API_BASE_URL: "https://api.example.com"
  
  # Node environment
  NODE_ENV: "production"
  LOG_LEVEL: "info"
  
  # Feature flags
  ENABLE_ANALYTICS: "true"
  ENABLE_CACHE: "true"
```

## Creating Kubernetes Secrets

Before using secrets in requirements files, they must exist in the Kubernetes cluster:

```bash
# Create a secret with API credentials
kubectl create secret generic birdeye-secret \
  --from-literal=api_key=your-api-key \
  --from-literal=api_secret=your-api-secret \
  -n agent-platform

# Create database credentials
kubectl create secret generic postgres-credentials \
  --from-literal=host=postgres.example.com \
  --from-literal=port=5432 \
  --from-literal=username=dbuser \
  --from-literal=password=dbpass \
  --from-literal=database=myapp \
  -n agent-platform
```

## Using External Secrets Operator

If you're using External Secrets Operator to sync secrets from external stores (AWS Secrets Manager, Vault, etc.), reference the created Kubernetes secret names in your requirements file:

```yaml
secrets:
  # This secret is synced from AWS Secrets Manager by External Secrets
  - name: aws-synced-birdeye-secret
```

## Benefits

1. **Declarative**: Requirements are version-controlled with the task
2. **Secure**: No secret values in code, only references
3. **Validated**: System can check if required secrets exist before running
4. **Simple**: Agents don't need to handle secret mounting
5. **Auditable**: Clear record of what each task requires

## Migration from Legacy System

The system maintains backward compatibility. If no `requirements.yaml` file exists, it falls back to the legacy `env` and `envFromSecrets` parameters in the MCP call.

## Troubleshooting

### Secret Not Found
If you get an error about a secret not being found:
1. Verify the secret exists: `kubectl get secret <secret-name> -n agent-platform`
2. Check the secret has the expected keys: `kubectl describe secret <secret-name> -n agent-platform`

### Environment Variable Not Set
If an expected environment variable isn't available:
1. Check the requirements.yaml syntax
2. Verify the key mapping is correct
3. Check controller logs for any parsing errors

### Requirements File Not Detected
If the MCP server doesn't detect your requirements file:
1. Verify the file path matches the pattern: `.taskmaster/docs/task-{id}/requirements.yaml`
2. Ensure the file is in the correct project directory
3. Check MCP server logs for file reading errors

## Best Practices

1. **Group Related Secrets**: Keep related credentials in the same secret (e.g., all database credentials together)
2. **Use Descriptive Names**: Secret names should indicate their purpose (e.g., `stripe-production-keys` not just `stripe`)
3. **Document Requirements**: Add comments in the YAML explaining what each secret is used for
4. **Minimize Scope**: Only request the secrets and environment variables actually needed
5. **Test Locally**: Verify secrets exist before running tasks in production

## Security Considerations

1. **Least Privilege**: Only mount secrets that are actually needed for the task
2. **Secret Rotation**: Design tasks to work with rotated credentials
3. **No Hardcoding**: Never put actual secret values in requirements.yaml
4. **Audit Trail**: The requirements file provides a clear audit trail of what each task accesses

## Future Enhancements

Potential future improvements to the system:
- Pre-flight validation to check all secrets exist before starting
- Support for optional vs required secrets
- Integration with secret rotation workflows
- Automatic secret usage reporting
- Support for ConfigMap references in addition to Secrets
