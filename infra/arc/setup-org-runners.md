# Setting Up Organization-Level ARC Runners

## Prerequisites

### 1. GitHub Token Permissions

For organization-level runners, your GitHub Personal Access Token (PAT) needs these permissions:

- `admin:org` - Full control of orgs and teams, read and write org projects
- `repo` - Full control of private repositories (if you want runners to access private repos)

### 2. Create a New PAT

1. Go to https://github.com/settings/tokens/new
2. Select scopes:
   - ✅ `admin:org`
   - ✅ `repo` (if needed)
3. Generate token and save it

### 3. Update the ARC Secret

```bash
# Delete old secret
kubectl delete secret arc-github-token -n arc-systems

# Create new secret with org-level permissions
kubectl create secret generic arc-github-token \
  --from-literal=github_token=YOUR_NEW_PAT \
  -n arc-systems
```

### 4. Deploy Organization Runners

```bash
# Apply the org-level runner deployment
kubectl apply -f infra/arc/runner-deployment-org.yaml

# Watch for the new runners
kubectl get runners -n arc-systems -w
```

### 5. Verify in GitHub

1. Go to https://github.com/organizations/5dlabs/settings/actions/runners
2. You should see your runners listed there

## Using GitHub App Instead (Recommended for Production)

Instead of a PAT, you can use a GitHub App which is more secure:

1. Create a GitHub App:
   - Go to https://github.com/organizations/5dlabs/settings/apps/new
   - Name: "5dlabs ARC Controller"
   - Homepage URL: https://github.com/5dlabs
   - Webhook: Uncheck "Active"
   - Permissions:
     - Actions: Read
     - Administration: Read & Write
     - Metadata: Read
   - Where can this GitHub App be installed: Only on this account

2. Install the App:
   - After creation, click "Install App"
   - Choose your organization
   - Select "All repositories" or specific ones

3. Get App Credentials:
   - App ID: Found in the app settings
   - Installation ID: Found in the installation URL
   - Private Key: Generate and download

4. Create Secret for GitHub App:
```bash
kubectl create secret generic arc-github-app \
  --from-literal=github_app_id=YOUR_APP_ID \
  --from-literal=github_app_installation_id=YOUR_INSTALLATION_ID \
  --from-file=github_app_private_key=path/to/private-key.pem \
  -n arc-systems
```

5. Update runner deployment to use App:
```yaml
githubAPICredentialsFrom:
  secretRef:
    name: arc-github-app
```

## Troubleshooting

### Runners Not Showing in Org

1. Check token permissions:
```bash
# Test with curl
curl -H "Authorization: token YOUR_PAT" \
  https://api.github.com/orgs/5dlabs/actions/runners
```

2. Check ARC controller logs:
```bash
kubectl logs -n arc-systems deployment/arc-actions-runner-controller -c manager --tail=50
```

3. Common issues:
- Token lacks `admin:org` permission
- Using repository-level configuration instead of organization
- GitHub App not installed on the organization

### Using Different Runners for Different Purposes

You can have both org-level and repo-level runners:

```yaml
# Org runners for general use
organization: 5dlabs
labels:
  - self-hosted
  - org-runner

# Repo runners for specific projects
repository: 5dlabs/platform
labels:
  - self-hosted
  - platform-specific
```

Then in workflows:
```yaml
# Use org runner
runs-on: [self-hosted, org-runner]

# Use repo-specific runner  
runs-on: [self-hosted, platform-specific]
```