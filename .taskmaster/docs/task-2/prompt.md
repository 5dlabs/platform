# Configure Argo Workflows for MCP Integration

Configure Argo Workflows infrastructure:

1. Create service account and RBAC:
   ```yaml
   apiVersion: v1
   kind: ServiceAccount
   metadata:
     name: workflow-executor
     namespace: workflows
   ---
   # Add corresponding RBAC
   ```

2. Set up storage:
   ```yaml
   apiVersion: v1
   kind: PersistentVolumeClaim
   metadata:
     name: workflow-workspace
   spec:
     accessModes: [ReadWriteMany]
   ```

3. Configure resource quotas:
   ```yaml
   apiVersion: v1
   kind: ResourceQuota
   metadata:
     name: workflow-quota
   ```

4. Enable metrics collection
5. Create workflow templates directory
6. Test API access from MCP server

Success: Workflows run successfully with proper permissions, storage, and resource limits.