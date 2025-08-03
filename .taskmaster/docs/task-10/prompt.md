# Create Workflow Status Monitor

Build monitoring service:

1. Watch implementation:
   ```go
   // Set up workflow watcher
   watcher, err := argoClient.Watch(ctx, metav1.ListOptions{})
   
   // Handle events
   for event := range watcher.ResultChan() {
     wf := event.Object.(*wfv1.Workflow)
     handleStatusChange(wf)
   }
   ```

2. Configure:
   - Status mapping
   - Log streaming
   - Caching
   - Metrics

3. Test monitoring

Success: Service tracks workflows with logs and metrics.