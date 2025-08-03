# Create Argo CD Applications

Generate manifests:

1. Basic structure:
   ```yaml
   apiVersion: argoproj.io/v1alpha1
   kind: Application
   metadata:
     name: service-name
     namespace: argocd
   spec:
     project: default
     source:
       repoURL: git@github.com:org/repo.git
       path: helm/service-name
     destination:
       server: https://kubernetes.default.svc
       namespace: service-ns
     syncPolicy:
       automated:
         prune: true
         selfHeal: true
   ```

2. Configure:
   - Health checks
   - Resource tracking
   - Sync options
   - Notifications

3. Test deployment

Success: Applications sync and maintain state automatically.