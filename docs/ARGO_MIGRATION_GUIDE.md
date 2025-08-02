# Argo CD & Argo Workflows Integration Guide

This guide explains how to migrate your platform from direct Helm deployments to a full GitOps approach using Argo CD and Argo Workflows.

## 🎯 Overview

The migration introduces:
- **Argo CD**: GitOps continuous deployment
- **Argo Workflows**: Advanced task orchestration replacing direct Kubernetes jobs
- **GitOps Applications**: Declarative infrastructure management
- **Enhanced CI/CD**: GitHub Actions integration with GitOps

## 🚀 Quick Start

### One-Command Migration
```bash
./infra/scripts/migrate-to-gitops.sh
```

This script performs the complete migration automatically.

### Manual Step-by-Step

#### 1. Install Argo CD
```bash
./infra/scripts/install-argocd.sh
```

#### 2. Update Repository Secrets
Edit `infra/charts/argocd/secrets.yaml` with your GitHub credentials:
```yaml
stringData:
  username: "your-github-username"
  password: "your-github-token"
```

#### 3. Install Argo Workflows
```bash
./infra/scripts/install-argo-workflows.sh
```

#### 4. Deploy Workflow Templates
```bash
./infra/scripts/install-workflow-templates.sh
```

#### 5. Setup CI/CD Access
```bash
./infra/scripts/setup-argocd-access.sh
```

#### 6. Deploy GitOps Applications
```bash
kubectl apply -f infra/gitops/projects/platform-project.yaml
kubectl apply -f infra/gitops/app-of-apps.yaml
```

## 🏗️ Architecture

### Before (Helm-based)
```
GitHub Actions → Direct Helm Deploy → Kubernetes
```

### After (GitOps)
```
GitHub Actions → Update Git → Argo CD → Kubernetes
               → Submit Workflow → Argo Workflows → Task Execution
```

## 📁 New Directory Structure

```
infra/
├── charts/argocd/              # Argo CD installation
├── charts/argo-workflows/      # Argo Workflows installation
├── gitops/                     # GitOps applications
│   ├── applications/           # Application definitions
│   ├── projects/              # Argo CD projects
│   └── app-of-apps.yaml       # Root application
├── workflow-templates/         # Argo Workflow templates
└── scripts/                   # Installation scripts
```

## 🔄 Workflow Changes

### CodeRun Tasks
- **Before**: Direct Kubernetes Job creation
- **After**: Argo Workflow submission using `coderun-template`

### DocsRun Tasks
- **Before**: Direct Kubernetes Job creation  
- **After**: Argo Workflow submission using `docsrun-template`

### Deployments
- **Before**: `helm upgrade` in GitHub Actions
- **After**: Git commit → Argo CD sync

## 🛠️ GitHub Actions Integration

### New Workflow: `.github/workflows/deploy-gitops.yml`

The new workflow:
1. **Builds** container images
2. **Updates** GitOps configuration
3. **Triggers** Argo CD sync
4. **Verifies** deployment

### Key Features
- Automatic image tag updates
- GitOps configuration management
- Argo CD sync with verification
- Rollback capability

## 🔐 Security & Access

### Repository Access
- GitHub token for private repository access
- SSH keys for git operations
- Service account tokens for CI/CD

### RBAC Configuration
- **Platform Project**: Controls application permissions
- **Workflow RBAC**: Manages workflow execution permissions
- **CI/CD Service Account**: Limited access for GitHub Actions

## 🌐 Access Points

### Argo CD UI
- **URL**: http://localhost:30080
- **Port Forward**: `kubectl port-forward svc/argocd-server -n argocd 8080:443`
- **Login**: admin / (get password from secret)

### Argo Workflows UI
- **URL**: http://localhost:30081
- **Port Forward**: `kubectl port-forward svc/argo-workflows-server -n argo 2746:2746`

## 📊 Monitoring

### Application Health
- Argo CD automatically monitors all applications
- Health status visible in UI
- Automatic sync and self-healing

### Workflow Execution
- Workflow status in Argo Workflows UI
- Detailed step execution logs
- Artifact management

## 🔧 Configuration

### Argo CD Applications

#### Orchestrator
```yaml
# infra/gitops/applications/orchestrator.yaml
spec:
  source:
    path: infra/charts/orchestrator
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
```

#### Monitoring Stack
```yaml
# infra/gitops/applications/monitoring-stack.yaml
spec:
  source:
    path: infra/telemetry
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
```

### Workflow Templates

#### CodeRun Template
- Multi-step workflow for code tasks
- Git repository management
- Claude Code agent execution
- Result publishing

#### DocsRun Template  
- Documentation generation workflow
- Repository cloning and setup
- Claude Docs agent execution
- Documentation publishing

## 🚨 Troubleshooting

### Application Sync Issues
```bash
# Check application status
kubectl get applications -n argocd

# Manual sync
kubectl patch application orchestrator -n argocd --type merge -p '{"operation":{"sync":{"revision":"HEAD"}}}'

# View sync logs
kubectl logs -n argocd deployment/argocd-application-controller
```

### Workflow Execution Issues
```bash
# Check workflow status
kubectl get workflows -n argo

# View workflow logs
kubectl logs -n argo -l workflows.argoproj.io/workflow=WORKFLOW_NAME

# Debug workflow steps
kubectl describe workflow WORKFLOW_NAME -n argo
```

### Repository Access Issues
```bash
# Check repository secrets
kubectl get secrets -n argocd | grep repo

# Update repository credentials
kubectl apply -f infra/charts/argocd/secrets.yaml
```

## 📈 Benefits

### GitOps Advantages
- **Declarative**: All configuration in Git
- **Auditable**: Complete change history
- **Recoverable**: Easy rollback and disaster recovery
- **Secure**: Git-based access control

### Workflow Advantages
- **Structured**: Multi-step task orchestration
- **Scalable**: Parallel workflow execution
- **Observable**: Detailed execution monitoring
- **Reusable**: Template-based workflows

### CI/CD Improvements
- **Faster deployments**: Git-driven automation
- **Better reliability**: Automatic sync and healing
- **Enhanced security**: Token-based access
- **Improved observability**: Centralized monitoring

## 🔄 Migration Checklist

- [ ] Install Argo CD
- [ ] Configure repository access
- [ ] Install Argo Workflows
- [ ] Deploy workflow templates
- [ ] Setup CI/CD access
- [ ] Deploy GitOps applications
- [ ] Test new deployment pipeline
- [ ] Update team documentation
- [ ] Train team on new tools
- [ ] Monitor migration success

## 📚 Additional Resources

- [Argo CD Documentation](https://argo-cd.readthedocs.io/)
- [Argo Workflows Documentation](https://argoproj.github.io/argo-workflows/)
- [GitOps Best Practices](https://www.gitops.tech/)
- [Platform GitOps README](infra/gitops/README.md)