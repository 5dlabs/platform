# Set Up Argo Events

Implement integration:

1. Event configuration:
   ```yaml
   apiVersion: argoproj.io/v1alpha1
   kind: EventSource
   metadata:
     name: github
   spec:
     github:
       pr:
         webhook:
           endpoint: /pr
         events: ["pull_request"]
         repositories:
           - owner: org
             names: ["repo"]
   ---
   apiVersion: argoproj.io/v1alpha1
   kind: Sensor
   metadata:
     name: pr-sensor
   spec:
     dependencies:
       - name: pr-event
         eventSourceName: github
         eventName: pr
   ```

2. Configure:
   - Event routing
   - Workflow triggers
   - Agent chaining
   - RBAC rules

3. Test events

Success: Event-driven orchestration with proper coordination.