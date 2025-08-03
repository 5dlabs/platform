# Create Argo Workflow Template for Code Runs

Create WorkflowTemplate:

1. Basic structure:
   ```yaml
   apiVersion: argoproj.io/v1alpha1
   kind: WorkflowTemplate
   metadata:
     name: code-run
   spec:
     entrypoint: code-agent
     templates:
     - name: code-agent
       container:
         resources:
           requests:
             cpu: "2"
             memory: 4Gi
         volumeMounts:
         - name: workspace
           mountPath: /workspace
   ```

2. Configure:
   - Environment variables
   - Volume mounts
   - Resource limits
   - Timeouts/retries

3. Test deployment and execution

Success: Template executes code agent with proper resources and persistence.