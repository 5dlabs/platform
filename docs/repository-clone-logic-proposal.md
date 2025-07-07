# Repository Clone Logic Proposal

## Current Behavior
The init container always tries to update the repository, even for subsequent tasks on the same service. This causes:
- Unnecessary network traffic
- Potential merge conflicts
- "Different repository detected" errors
- Loss of local changes between tasks

## Proposed Solution

### Smart Clone Decision Logic

```bash
# Check if we're continuing work on the same service
if [ -f "/workspace/.current-service" ]; then
    CURRENT_SERVICE=$(cat /workspace/.current-service)
    if [ "$CURRENT_SERVICE" = "{{service_name}}" ]; then
        echo "Continuing work on service: $CURRENT_SERVICE"
        # Don't clone or update - use existing code
        SKIP_CLONE=true
    else
        echo "Switching to different service: {{service_name}} (was: $CURRENT_SERVICE)"
        # Need to clone new service
        SKIP_CLONE=false
    fi
else
    echo "First task - no previous service found"
    SKIP_CLONE=false
fi

# Save current service for next task
echo "{{service_name}}" > /workspace/.current-service
```

### Implementation Options

#### Option 1: Service Tracking File (Recommended)
- Create `.current-service` file in workspace
- Compare with incoming service name
- Only clone if:
  - File doesn't exist (first task)
  - Service name differs (switching services)

**Pros:**
- Simple to implement
- Survives PVC restarts
- Clear intent

**Cons:**
- Extra file in workspace
- Could be deleted by accident

#### Option 2: Git Remote Comparison
- Check if git remote URL matches expected repository
- Only clone if URLs don't match

**Pros:**
- No extra files needed
- Uses existing git state

**Cons:**
- Doesn't handle case where repo URL is same but we want fresh clone
- More complex logic

#### Option 3: Explicit Flag
- Add `--fresh-clone` flag to CLI
- User decides when to clone

**Pros:**
- User has full control
- Very simple

**Cons:**
- User needs to remember when to use it
- Not automatic

## Recommended Approach

Combine Option 1 and 2:
1. Check `.current-service` file
2. If same service, skip clone
3. If different service or no file, check git remote
4. Only clone if necessary

This provides automatic behavior that "just works" while preserving code between tasks on the same service.