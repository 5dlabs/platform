# Update GitHub Actions

Modify workflows:

1. Update deploy.yml:
   ```yaml
   name: Build and Push
   on:
     push:
       branches: ['**']
   jobs:
     build:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v3
         - name: Build Image
           run: docker build -t image:${{ github.sha }} .
         - name: Push Image
           run: |
             docker push image:${{ github.sha }}
             docker push image:${{ github.ref_name }}
   ```

2. Configure:
   - Build caching
   - Error handling
   - Argo CD sync

3. Remove:
   - Helm steps
   - intake.yml

Success: Fast builds with automatic Argo CD updates.