# SSH Authentication Setup for Orchestrator

This guide explains how to set up SSH authentication for git repositories as an alternative to HTTPS token authentication.

## Overview

The orchestrator now supports both HTTPS and SSH authentication for git operations:

- **HTTPS**: Uses GitHub Personal Access Tokens (PAT) - existing method
- **SSH**: Uses SSH private keys - new alternative method

The system automatically detects the repository URL type and configures the appropriate authentication method.

## SSH vs HTTPS URL Detection

The system detects authentication method based on the repository URL:

- **SSH URLs**: `git@github.com:owner/repo.git` or `ssh://git@github.com/owner/repo.git`
- **HTTPS URLs**: `https://github.com/owner/repo.git`

## Setting Up SSH Authentication

### 1. Generate SSH Key Pair

If you don't already have an SSH key pair:

```bash
# Generate a new SSH key pair
ssh-keygen -t rsa -b 4096 -C "your-email@example.com"

# This creates:
# - ~/.ssh/id_rsa (private key)
# - ~/.ssh/id_rsa.pub (public key)
```

### 2. Add Public Key to GitHub

1. Copy your public key:
   ```bash
   cat ~/.ssh/id_rsa.pub
   ```

2. Go to GitHub → Settings → SSH and GPG keys → New SSH key
3. Paste the public key content
4. Give it a descriptive title

### 3. Create Kubernetes Secret

Create a secret containing your SSH private key:

```bash
# Create secret with SSH private key
kubectl create secret generic github-ssh-yourusername \
  --from-file=ssh-privatekey=/path/to/your/private/key \
  --namespace=your-namespace

# Example:
kubectl create secret generic github-ssh-jonathonfritz \
  --from-file=ssh-privatekey=~/.ssh/id_rsa \
  --namespace=default
```

### 4. Update Repository Configuration

When submitting tasks, use SSH URLs in your repository configuration:

```json
{
  "repository": {
    "url": "git@github.com:owner/repo.git",
    "branch": "main",
    "githubUser": "yourusername"
  }
}
```

## Secret Naming Convention

The orchestrator looks for secrets based on the `githubUser` field:

- **SSH secrets**: `github-ssh-{githubUser}`
- **HTTPS secrets**: `github-pat-{githubUser}`

Examples:
- For user `jonathonfritz` with SSH: `github-ssh-jonathonfritz`
- For user `jonathonfritz` with HTTPS: `github-pat-jonathonfritz`

## Verification

### Test SSH Connection

You can test SSH connectivity manually:

```bash
# Test SSH connection to GitHub
ssh -T git@github.com

# Expected output:
# Hi username! You've successfully authenticated, but GitHub does not provide shell access.
```

### Check Hook Test Files

Use the provided script to check if hooks are working:

```bash
# Check for hook test files (indicates hooks are firing)
./scripts/check-hook-test.sh [namespace] [service-name]

# Example:
./scripts/check-hook-test.sh default platform
```

The script will look for:
- `.hook-test-success` - Created when hooks fire successfully
- `.hook-test.log` - Log of hook activity

## Authentication Method Priority

1. **SSH**: If repository URL starts with `git@` or `ssh://`, SSH authentication is used
2. **HTTPS Fallback**: If SSH key not found, falls back to HTTPS (if token available)
3. **Error**: If neither SSH key nor HTTPS token is available, the job fails

## Troubleshooting

### SSH Key Not Found

```
⚠️ SSH private key not found in secret: github-ssh-username
   Expected secret with key 'ssh-privatekey'
   Falling back to HTTPS if available...
```

**Solution**: Create the SSH secret with correct naming convention.

### SSH Connection Failed

```
❌ Error: SSH private key not found at ~/.ssh/id_rsa
```

**Solution**: Verify the secret was created correctly and contains the private key.

### Permission Denied

```
Permission denied (publickey).
```

**Solution**:
1. Verify the public key is added to your GitHub account
2. Ensure the private key in the secret matches the public key
3. Check that the private key has correct permissions (600)

### Hooks Not Firing

If the hook test files (`.hook-test-success`, `.hook-test.log`) are not created:

1. Check Claude settings configuration
2. Verify hook scripts are properly configured
3. Ensure tools like "Edit" and "Write" are triggering hooks

## Benefits of SSH Authentication

1. **Security**: No tokens in environment variables or config files
2. **Simplicity**: SSH keys are standard for git operations
3. **Compatibility**: Works with existing SSH-based workflows
4. **Reliability**: Less prone to token expiration issues

## Migration from HTTPS to SSH

To migrate existing services from HTTPS to SSH:

1. Create SSH secret (step 3 above)
2. Update repository URL to use SSH format
3. Resubmit tasks - the system will automatically detect and use SSH

The HTTPS secret can remain - it will be ignored for SSH URLs but available as fallback.