# Update MCP Server for Argo Workflows

Refactor server code:

1. Remove old client:
   ```go
   // Delete old client code
   // Update imports
   ```

2. Integrate new client:
   ```go
   // Update service methods
   func (s *Service) SubmitJob(ctx context.Context, req *Request) (*Response, error) {
     return s.argoClient.SubmitWorkflow(ctx, req.ToWorkflow())
   }
   ```

3. Handle migration:
   - Support in-progress jobs
   - Add error handling
   - Update configuration

Success: Server uses Argo Workflows API with proper error handling.