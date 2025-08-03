# Configure Argo Monitoring

Set up monitoring:

1. Prometheus config:
   ```yaml
   scrape_configs:
     - job_name: 'argocd'
       metrics_path: /metrics
       static_configs:
         - targets: ['argocd-server:8080']
     - job_name: 'argo-workflows'
       metrics_path: /metrics
       static_configs:
         - targets: ['workflow-controller:9090']
   ```

2. Configure:
   - Grafana dashboards
   - Alert rules
   - Log collection
   - Health checks

3. Test monitoring

Success: Complete visibility into Argo components.