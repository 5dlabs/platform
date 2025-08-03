# Fix Argo CD Application Sync Issues

Fix sync and health issues across 14 Argo CD applications by:

1. Query application status:
   ```
   argocd app list -o wide
   ```

2. For each problematic app:
   - Check Git repository connectivity
   - Validate manifests against cluster state
   - Review application health definitions
   - Fix configuration drift
   - Test manual sync

3. Focus on these critical applications:
   - arc (OutOfSync)
   - k8s-mcp (Unknown)
   - rustdocs-mcp (Unknown)
   - twingate-pastoral (Unknown)
   - twingate-therapeutic (Unknown)

4. Document recurring issues and resolutions

Success: All applications show Synced and Healthy status.