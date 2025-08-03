# Update MCP Server for Templates

Implement integration:

1. Job submission:
   ```go
   func (s *Service) SubmitJob(ctx context.Context, req *Request) (*Response, error) {
     params := map[string]string{
       "input": req.Input,
       "config": req.Config,
     }
     
     workflow, err := s.argoClient.SubmitWorkflowFromTemplate(ctx, 
       req.Type.String(), 
       generateName(req.ID),
       params,
     )
     
     return &Response{
       ID: req.ID,
       WorkflowName: workflow.Name,
     }, nil
   }
   ```

2. Configure:
   - Validation rules
   - Error handling
   - Metrics/logging

3. Test submission flow

Success: Jobs submitted via templates with proper tracking.