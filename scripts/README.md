# Platform Scripts

This directory contains utility scripts for managing the 5D Labs Agent Platform.

## Scripts

### `wait-for-github-action.sh`

Monitors GitHub Actions workflow runs and provides real-time status updates.

**Usage:**
```bash
./scripts/wait-for-github-action.sh [workflow-run-id]
```

**What it does:**
- Monitors the specified GitHub Actions workflow run
- Provides real-time status updates
- Exits when the workflow completes (success or failure)

## Gemini CLI Integration

The Gemini CLI is integrated into the platform build process by checking out the official [google-gemini/gemini-cli](https://github.com/google-gemini/gemini-cli) repository during the CI/CD workflow. This approach keeps the platform repository clean while still building and publishing the Gemini CLI image.

**How it works:**
1. The CI/CD workflow checks out both the platform repository and the Gemini CLI repository
2. The Gemini CLI is built and packaged in the workflow
3. A Docker image is created and pushed to the container registry

**No manual management required** - the official Gemini CLI is automatically built from the latest main branch on every push to the platform repository.

## CI/CD Integration

All scripts are designed to work with the unified CI/CD pipeline (`.github/workflows/unified-ci.yml`). The Gemini CLI is automatically built and pushed to the container registry on every push.

**Built Images:**
- `ghcr.io/5dlabs/platform/gemini-cli:latest`
- `ghcr.io/5dlabs/platform/gemini-cli:main-<commit-sha>`
- `ghcr.io/5dlabs/platform/gemini-cli:<version>` (for releases)