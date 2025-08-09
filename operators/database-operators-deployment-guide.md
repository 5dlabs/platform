# Database Operators Deployment Guide

This document tracks the current state, fixes, and deployment guide for PostgreSQL, Redis, and QuestDB operators on the platform.

## Current Status Summary

### ✅ Working Components
- **PostgreSQL Operator**: Deployed and operational (v1.14.0)
- **Redis Operator**: Deployed and operational (v1.3.0)  
- **QuestDB Operator**: Deployed and operational (v0.5.1)
- **Database Instances**: Currently running but configuration drift exists

### ⚠️ Issues Identified
- **Configuration Drift**: Main branch missing latest database instance configurations
- **ArgoCD Sync**: `database-instances` application shows OutOfSync status
- **Branch Management**: Fixes scattered across multiple feature branches

## Deployment Architecture

### Operators (Working)
Located in `infra/gitops/applications/`:
- `postgres-operator.yaml` - Zalando Postgres Operator via Helm
- `redis-operator.yaml` - Spotahome Redis Operator via Git manifests
- `questdb-operator.yaml` - QuestDB Operator via Git manifests

### Database Instances (Needs Sync)
Located in `infra/gitops/databases/`:
- `postgres-general.yaml` - General-purpose PostgreSQL cluster
- `redis-general.yaml` - General-purpose Redis cluster
- `questdb-general.yaml` - General-purpose QuestDB instance

## Current Branch Analysis

### Main Branch (`main`)
**Contains:**
- Database operator definitions (postgres, redis, questdb)
- Old database instance configurations (agent-docs-* only)

**Missing:**
- General-purpose database instances (*-general.yaml files)
- Updated database-instances application configuration

### Feature Branch (`feature/db-general-instances`)
**Contains:**
- All main branch content
- New general-purpose database instances
- Updated database-instances application with proper include filters
- Fixed configuration formats

**Key Commit:** `258df6be` - "db(instances): add general-purpose Postgres/Redis/QuestDB CRs"

### Related Fix Branches
Several branches contain database-related fixes:
- `fix/argocd-project-permissions` - QuestDB and PostgreSQL config fixes
- `feature/operators` - Original operator implementations
- Various postgres-mcp related branches

## ArgoCD Configuration Issues

### Current State
```bash
database-instances     OutOfSync     Healthy
postgres-operator      Synced        Healthy
questdb-operator       Synced        Healthy
redis-operator         Synced        Healthy
```

### Root Cause
The `database-instances` application points to `main` branch but:
1. Uses old include filter: `"agent-docs-*.yaml"`  
2. Missing the updated filter: `"{agent-docs-*.yaml,*general.yaml}"`
3. General database files don't exist on `main`

### App-of-Apps Status
- **Auto-sync**: Disabled (manual control for fixes)
- **Target**: `main` branch
- **Status**: Functional but prevents automatic updates

## PostgreSQL pgvector Support

### Research Findings

Based on GitHub issues [#2518](https://github.com/zalando/postgres-operator/issues/2518) and [#2898](https://github.com/zalando/postgres-operator/issues/2898):

**✅ pgvector is SUPPORTED and available by default**

### How to Enable pgvector

1. **No operator configuration required** - pgvector is included in Spilo image
2. **Manual activation per database:**
   ```sql
   CREATE EXTENSION vector;
   ```

### Spilo Image Support
- Available since: `ghcr.io/zalando/spilo-15:3.1-p1`
- Current platform uses PostgreSQL 16 (newer version)
- Extension is built-in, just needs activation

## Deployment Solutions

### Option 1: Merge Feature Branch (Recommended)
**Steps:**
1. Test current feature branch thoroughly
2. Merge `feature/db-general-instances` to `main`
3. Verify ArgoCD syncs automatically
4. Re-enable app-of-apps auto-sync

**Pros:**
- Simplest solution
- Maintains single source of truth
- Restores normal GitOps workflow

**Cons:**
- Risk of untested changes going to production

### Option 2: Staging App-of-Apps
**Implementation:**
1. Create `infra/gitops/staging/app-of-apps-staging.yaml`
2. Point to feature branches for testing
3. Separate staging ArgoCD project
4. Promote changes after testing

**Example Structure:**
```yaml
# staging/app-of-apps-staging.yaml
spec:
  source:
    targetRevision: feature/db-general-instances  # Test branch
    # ... rest of config
```

**Pros:**
- Safe testing environment
- Isolated from production
- Parallel development possible

**Cons:**
- Added complexity
- Maintenance overhead
- Requires staging cluster or namespace isolation

### Option 3: Feature Flag Applications
**Implementation:**
1. Create branch-specific applications
2. Use ArgoCD ApplicationSets
3. Conditional deployment based on labels

**Pros:**
- Maximum flexibility
- A/B testing capabilities
- Fine-grained control

**Cons:**
- Most complex
- Requires ApplicationSet CRDs
- Steeper learning curve

### Option 4: Manual Sync Strategy  
**Current State (What you're doing):**
1. Keep auto-sync disabled
2. Test changes on feature branches
3. Manually sync applications
4. Update target branches when ready

**Pros:**
- Full control over deployments
- No surprise changes
- Easy rollback

**Cons:**
- Manual overhead
- Easy to forget to sync
- Loses GitOps benefits

## Recommended Deployment Strategy

### Phase 1: Immediate (Today)
1. **Merge `feature/db-general-instances` to `main`**
   - Contains working database instances
   - Fixes configuration issues
   - Restores ArgoCD sync

2. **Test pgvector support:**
   ```bash
   kubectl exec -n databases postgres-general-0 -- psql -c "CREATE EXTENSION vector;" -d general_db
   ```

### Phase 2: Process Improvement (This Week)
1. **Implement staging app-of-apps:**
   - Create staging applications for database testing
   - Use separate namespace or cluster
   - Test changes before production

2. **Re-enable auto-sync:**
   - Only after confirming stability
   - Monitor for 24-48 hours

### Phase 3: Long-term (Next Sprint)
1. **Implement proper CI/CD:**
   - Helm chart testing
   - Database schema validation
   - Integration tests

2. **Consider ApplicationSets:**
   - For multi-environment deployments
   - Standardized patterns

## Current Database Instances

### Deployed and Running
```bash
# PostgreSQL
NAME               TEAM       VERSION   PODS   VOLUME   STATUS
postgres-general   platform   16        1      50Gi     Running

# Redis  
NAME               REDIS   SENTINELS   AGE
redis-general      1       1           45h

# QuestDB
NAME                 READY REPLICAS   AGE
questdb-timeseries   1                46h
```

### Desired End State
- 2x Redis instances (redis-general + redis-cache)
- 1x PostgreSQL with pgvector (postgres-general)  
- 1x QuestDB for time-series (questdb-timeseries)

## PostgreSQL pgvector Configuration

### Current Instance Configuration
The `postgres-general` instance is already configured with:
- PostgreSQL 16 (pgvector supported)
- Multiple databases: `app_db`, `general_db`, `metrics_db`
- Connection pooling enabled
- Proper resource limits

### Adding pgvector Support
1. **Create pgvector-enabled database:**
   ```yaml
   # Add to postgres-general.yaml
   databases:
     vector_db: admin
   
   preparedDatabases:
     vector_db:
       defaultUsers: true
       extensions:
         vector: "public"  # Enable pgvector
         pg_stat_statements: "public"
         uuid-ossp: "public"
   ```

2. **Or enable manually:**
   ```bash
   # Connect to database
   kubectl exec -n databases postgres-general-0 -- psql -d general_db
   
   # Enable extension
   CREATE EXTENSION vector;
   
   # Test
   SELECT * FROM pg_extension WHERE extname = 'vector';
   ```

## Redis Cluster Configuration

### Adding Second Redis Instance
Create `redis-cache.yaml`:
```yaml
apiVersion: databases.spotahome.com/v1
kind: RedisFailover
metadata:
  name: redis-cache
  namespace: databases
spec:
  sentinel:
    replicas: 3  # Higher availability
  redis:
    replicas: 3  # Higher availability
    storage:
      persistentVolumeClaim:
        spec:
          resources:
            requests:
              storage: 10Gi  # Smaller cache storage
```

## TwinGate Operator (Lower Priority)

### Current Status
- **Helm Deployment**: Working manually
- **Operator Benefits**: Automated upgrades only
- **Priority**: Lower priority given working Helm deployment

### Implementation Approach
If implemented:
1. Use existing operator from `operators/kubernetes-operator/`
2. Create ArgoCD application similar to database operators
3. Migrate existing Helm deployment to operator CRDs

### Current Status Analysis
**Deployed via Helm:**
- 2 connector instances running (`twingate-pastoral-jaguar-connector`, `twingate-therapeutic-ara-connector`)
- Stable for 37+ days
- Working configuration

**Available CRDs:**
- `TwingateConnector` - Connector management
- `TwingateGroup` - Group management  
- `TwingateResource` - Resource definitions
- `TwingateResourceAccess` - Access control

### Cost/Benefit Analysis
**Benefits:**
- Automated upgrades
- Consistent operator pattern
- Kubernetes-native resource management
- Better lifecycle management

**Costs:**
- Migration effort from working Helm deployment
- Additional complexity
- Testing overhead
- Risk of disrupting working setup

**Recommendation:** 
Keep current Helm deployment. The primary benefit (automated upgrades) doesn't justify migration risk since TwinGate upgrades are infrequent and the current deployment is stable.

## Monitoring and Troubleshooting

### ArgoCD Health Checks
```bash
# Check application status
kubectl get applications -n argocd

# Check sync status
argocd app get database-instances

# Check application logs
kubectl logs -n argocd -l app.kubernetes.io/name=argocd-application-controller
```

### Database Health Checks
```bash
# PostgreSQL
kubectl get postgresql -n databases
kubectl describe postgresql postgres-general -n databases

# Redis
kubectl get redisfailover -n databases
kubectl describe redisfailover redis-general -n databases

# QuestDB  
kubectl get questdb -n databases
kubectl describe questdb questdb-timeseries -n databases
```

### Common Issues and Solutions

#### pgvector Extension Issues
```bash
# Error: extension "vector" is not available
# Solution: Ensure using Spilo image with pgvector

# Check Spilo version
kubectl get postgresql postgres-general -o jsonpath='{.spec.postgresql.version}'

# Enable extension manually
kubectl exec postgres-general-0 -- psql -c "CREATE EXTENSION vector;" -d target_db
```

#### ArgoCD Sync Issues
```bash
# OutOfSync due to missing files
# Solution: Merge feature branch or update target revision

# Temporarily point to feature branch
kubectl patch application database-instances -n argocd --type merge -p '{"spec":{"source":{"targetRevision":"feature/db-general-instances"}}}'
```

## Best Practices for Future Changes

### Development Workflow
1. **Feature Branch Development**
   - Create feature branch for database changes
   - Test using manual ArgoCD sync or staging apps
   - Merge to main after validation

2. **Testing Strategy**
   - Use staging namespace/cluster
   - Test operator upgrades in isolation
   - Validate database connectivity and performance

3. **Deployment Process**
   - Disable auto-sync during major changes
   - Use manual sync for controlled rollouts
   - Monitor applications for 24h before re-enabling auto-sync

### GitOps Principles
1. **Single Source of Truth**: Keep all configs in Git
2. **Declarative**: Use YAML manifests, not imperative commands
3. **Versioned**: Tag releases and track changes
4. **Auditable**: All changes via pull requests

## Next Steps

### Immediate Actions Required
1. [ ] Merge `feature/db-general-instances` to `main`
2. [ ] Test pgvector support in postgres-general
3. [ ] Verify all ArgoCD applications sync correctly
4. [ ] Document any additional fixes needed

### Follow-up Tasks
1. [ ] Add second Redis instance (redis-cache)
2. [ ] Implement staging app-of-apps pattern
3. [ ] Re-enable auto-sync after stability confirmed
4. [ ] Consider TwinGate operator migration

---

*Last Updated: 2025-08-09*  
*Author: Platform Team*  
*Status: Active Investigation*