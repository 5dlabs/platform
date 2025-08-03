# Create Argo Workflows API Client

Build MCP server client:

1. Basic structure:
   ```go
   package argo

   import (
     "k8s.io/client-go/kubernetes"
     wfclientset "github.com/argoproj/argo-workflows/pkg/client/clientset/versioned"
   )

   type Client struct {
     wfClient    *wfclientset.Clientset
     config      *Config
     breaker     *gobreaker.CircuitBreaker
   }
   ```

2. Implement:
   - Authentication
   - Template submission
   - Status monitoring
   - Log retrieval

3. Test functionality

Success: Client handles workflow operations with proper error handling.