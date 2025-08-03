# Configure Infrastructure Operators

Deploy operators:

1. Basic structure:
   ```yaml
   apiVersion: argoproj.io/v1alpha1
   kind: Application
   metadata:
     name: postgres-operator
     annotations:
       argocd.argoproj.io/sync-wave: "1"
   spec:
     project: infrastructure
     source:
       path: operators/postgres
     destination:
       namespace: postgres-system
     syncPolicy:
       automated: {}
   ```

2. Configure:
   - RBAC/security
   - Resources/limits
   - Monitoring
   - High availability

3. Test deployment

Success: Production-ready operators with proper monitoring.