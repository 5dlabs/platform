# Create Feature Branch ApplicationSet

Generate ApplicationSet:

1. Basic structure:
   ```yaml
   apiVersion: argoproj.io/v1alpha1
   kind: ApplicationSet
   metadata:
     name: feature-branches
     namespace: argocd
   spec:
     generators:
     - git:
         repoURL: git@github.com:org/repo.git
         revision: HEAD
         directories:
         - path: helm/*
     template:
       metadata:
         name: '{{path.basename}}-{{branch}}'
       spec:
         source:
           path: '{{path}}'
           repoURL: git@github.com:org/repo.git
           targetRevision: '{{branch}}'
   ```

2. Configure:
   - Namespace isolation
   - Resource limits
   - Cleanup policies
   - Network rules

3. Test branch deployment

Success: Dynamic applications created per feature branch.