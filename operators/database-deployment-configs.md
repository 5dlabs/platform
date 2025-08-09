# Database Deployment Configurations

This document contains the specific YAML configurations for deploying the desired database instances:
- 2x Redis instances (general + cache)
- 1x PostgreSQL with pgvector support
- 1x QuestDB for time-series data

## PostgreSQL with pgvector Support

### File: `infra/gitops/databases/postgres-general.yaml`

```yaml
apiVersion: acid.zalan.do/v1
kind: postgresql
metadata:
  name: postgres-general
  namespace: databases
  labels:
    team: platform
    app.kubernetes.io/name: postgres-general
    app.kubernetes.io/component: database
spec:
  teamId: platform
  postgresql:
    version: "16"
    parameters:
      shared_preload_libraries: "bg_mon,pg_stat_statements,pgextwlist,pg_auth_mon"
      max_connections: "200"
      shared_buffers: "256MB"
      effective_cache_size: "1GB"
      maintenance_work_mem: "64MB"
      checkpoint_completion_target: "0.9"
      wal_buffers: "16MB"
      default_statistics_target: "100"
      random_page_cost: "1.1"
      effective_io_concurrency: "200"
      work_mem: "4MB"
      min_wal_size: "1GB"
      max_wal_size: "4GB"
  numberOfInstances: 1
  volume:
    size: 50Gi
    storageClass: local-path
  resources:
    requests:
      memory: 1Gi
      cpu: 500m
    limits:
      memory: 4Gi
      cpu: 2000m
  users:
    admin:
      - SUPERUSER
      - CREATEDB
    app_user:
      - NOSUPERUSER
      - INHERIT
      - CREATEDB
      - NOCREATEROLE
      - NOREPLICATION
    vector_user:
      - NOSUPERUSER
      - INHERIT
      - CREATEDB
      - NOCREATEROLE
      - NOREPLICATION
    readonly:
      - NOSUPERUSER
      - INHERIT
      - NOCREATEDB
      - NOCREATEROLE
      - NOREPLICATION
  databases:
    app_db: app_user
    general_db: admin
    vector_db: vector_user     # Dedicated database for vector operations
    metrics_db: admin
  preparedDatabases:
    app_db:
      defaultUsers: true
      extensions:
        pg_stat_statements: "public"
        uuid-ossp: "public"
    general_db:
      defaultUsers: true
      extensions:
        hstore: "public"
        pg_stat_statements: "public"
        pg_trgm: "public"
        pgcrypto: "public"
        uuid-ossp: "public"
    vector_db:
      defaultUsers: true
      extensions:
        vector: "public"           # Enable pgvector extension
        pg_stat_statements: "public"
        uuid-ossp: "public"
        pg_trgm: "public"          # Useful for text similarity with vectors
        pgcrypto: "public"
    metrics_db:
      defaultUsers: true
      extensions:
        pg_stat_statements: "public"
  enableConnectionPooler: true
  connectionPooler:
    numberOfInstances: 1
    mode: "transaction"
    schema: "pooler"
    user: "pooler"
    resources:
      requests:
        cpu: 100m
        memory: 100Mi
      limits:
        cpu: 500m
        memory: 500Mi
  maintenanceWindows:
    - "03:00-04:00"
```

## Redis General-Purpose Instance

### File: `infra/gitops/databases/redis-general.yaml`

```yaml
apiVersion: databases.spotahome.com/v1
kind: RedisFailover
metadata:
  name: redis-general
  namespace: databases
  labels:
    team: platform
    app.kubernetes.io/name: redis-general
    app.kubernetes.io/component: cache
    purpose: general
spec:
  sentinel:
    replicas: 1
    resources:
      requests:
        cpu: 50m
        memory: 64Mi
      limits:
        cpu: 100m
        memory: 128Mi
    customConfig:
      - "down-after-milliseconds 5000"
      - "failover-timeout 10000"
      - "parallel-syncs 1"
  redis:
    replicas: 1
    resources:
      requests:
        cpu: 100m
        memory: 512Mi
      limits:
        cpu: 500m
        memory: 2Gi
    image: redis:7.2-alpine
    imagePullPolicy: IfNotPresent
    customConfig:
      - "maxmemory 1536mb"
      - "maxmemory-policy allkeys-lru"
      - "save 900 1"
      - "save 300 10"
      - "save 60 10000"
      - "appendonly yes"
      - "appendfsync everysec"
      - "no-appendfsync-on-rewrite no"
      - "auto-aof-rewrite-percentage 100"
      - "auto-aof-rewrite-min-size 64mb"
      - "tcp-keepalive 60"
      - "tcp-backlog 511"
      - "timeout 300"
      - "databases 16"
      - "slowlog-log-slower-than 10000"
      - "slowlog-max-len 128"
      - "rename-command FLUSHDB \"\""
      - "rename-command FLUSHALL \"\""
      - "rename-command KEYS \"\""
    storage:
      persistentVolumeClaim:
        metadata:
          name: redis-general-data
        spec:
          accessModes:
            - ReadWriteOnce
          storageClassName: local-path
          resources:
            requests:
              storage: 20Gi
```

## Redis Cache Instance (New)

### File: `infra/gitops/databases/redis-cache.yaml`

```yaml
apiVersion: databases.spotahome.com/v1
kind: RedisFailover
metadata:
  name: redis-cache
  namespace: databases
  labels:
    team: platform
    app.kubernetes.io/name: redis-cache
    app.kubernetes.io/component: cache
    purpose: cache
spec:
  sentinel:
    replicas: 3  # Higher availability for cache
    resources:
      requests:
        cpu: 50m
        memory: 64Mi
      limits:
        cpu: 100m
        memory: 128Mi
    customConfig:
      - "down-after-milliseconds 3000"  # Faster failover for cache
      - "failover-timeout 8000"
      - "parallel-syncs 2"
  redis:
    replicas: 3  # Higher availability for cache
    resources:
      requests:
        cpu: 200m
        memory: 1Gi
      limits:
        cpu: 1000m
        memory: 4Gi
    image: redis:7.2-alpine
    imagePullPolicy: IfNotPresent
    customConfig:
      - "maxmemory 3072mb"                    # Higher memory for cache
      - "maxmemory-policy allkeys-lru"        # LRU eviction for cache
      - "save \"\""                           # Disable persistence for pure cache
      - "appendonly no"                       # No persistence for cache
      - "tcp-keepalive 60"
      - "tcp-backlog 511"
      - "timeout 300"
      - "databases 16"
      - "slowlog-log-slower-than 5000"       # Faster slow log threshold
      - "slowlog-max-len 256"                # More slow log entries
      - "rename-command FLUSHDB \"\""
      - "rename-command FLUSHALL \"\""
      - "rename-command KEYS \"\""
    # No persistent storage for cache instance
```

## QuestDB Time-Series Instance

### File: `infra/gitops/databases/questdb-general.yaml`

```yaml
apiVersion: crd.questdb.io/v1beta1
kind: QuestDB
metadata:
  name: questdb-timeseries
  namespace: databases
  labels:
    team: platform
    app.kubernetes.io/name: questdb-timeseries
    app.kubernetes.io/component: timeseries
spec:
  image: questdb/questdb:7.3.10
  imagePullPolicy: IfNotPresent
  volume:
    size: 50Gi
    storageClassName: local-path
  resources:
    requests:
      memory: 2Gi
      cpu: 500m
    limits:
      memory: 8Gi
      cpu: 2000m
  config:
    # HTTP interface (Web Console and REST API)
    http.enabled: "true"
    http.bind.to: "0.0.0.0:9000"
    
    # PostgreSQL wire protocol (for SQL queries)
    pg.enabled: "true"
    pg.bind.to: "0.0.0.0:8812"
    
    # InfluxDB line protocol over TCP
    line.tcp.enabled: "true"
    line.tcp.bind.to: "0.0.0.0:9009"
    
    # Performance optimizations for time-series workloads
    cairo.sql.copy.root: "/tmp/questdb/copy"
    shared.worker.count: "4"
    http.worker.count: "2"
    http.connection.pool.initial.capacity: "16"
```

## Updated Database Instances Application

### File: `infra/gitops/applications/database-instances.yaml`

```yaml
---
# Argo CD Application for Database Instances
# Manages actual database clusters (PostgreSQL, Redis, QuestDB) for platform services
#
# DEPLOYS:
# - postgres-general.yaml - PostgreSQL cluster with pgvector support
# - redis-general.yaml - General-purpose Redis cluster
# - redis-cache.yaml - Cache-specific Redis cluster  
# - questdb-general.yaml - QuestDB instance for time-series data
# - agent-docs-*.yaml - Service-specific database instances
#
# EXCLUDES:
# - examples/ subdirectory (development and production templates)
# - README.md documentation file

apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: database-instances
  namespace: argocd
  labels:
    app.kubernetes.io/name: database-instances
    app.kubernetes.io/part-of: platform
    app.kubernetes.io/component: database
  finalizers:
    - resources-finalizer.argocd.argoproj.io
spec:
  # Project configuration
  project: platform

  # Source configuration - Deploy database instances
  source:
    repoURL: https://github.com/5dlabs/cto
    targetRevision: main
    path: infra/gitops/databases

    # Directory configuration - Deploy all production database instances
    directory:
      recurse: false  # Don't recurse into subdirectories (excludes examples/)
      include: "{agent-docs-*.yaml,*general.yaml,*cache.yaml}"  # All production instances
      exclude: "{README.md,*example*,*template*,*dev*,*test*}"  # Exclude docs and test files

  # Destination configuration
  destination:
    server: https://kubernetes.default.svc
    namespace: databases

  # Sync policy
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
      allowEmpty: false

    syncOptions:
      - CreateNamespace=true
      - PrunePropagationPolicy=foreground
      - RespectIgnoreDifferences=true

    retry:
      limit: 5
      backoff:
        duration: 10s
        factor: 2
        maxDuration: 3m

  # Ignore certain differences
  ignoreDifferences:
    # Ignore generated fields in PostgreSQL resources
    - group: acid.zalan.do
      kind: postgresql
      jsonPointers:
        - /status
    # Ignore generated fields in Redis resources
    - group: databases.spotahome.com
      kind: RedisFailover
      jsonPointers:
        - /status
    # Ignore generated fields in QuestDB resources
    - group: crd.questdb.io
      kind: QuestDB
      jsonPointers:
        - /status

  # Revision history
  revisionHistoryLimit: 5
```

## pgvector Usage Examples

### Connecting to PostgreSQL with pgvector

```bash
# Connect to vector database
kubectl exec -n databases postgres-general-0 -- psql -d vector_db -U vector_user

# Or port-forward for external access
kubectl port-forward -n databases svc/postgres-general 5432:5432
# Then connect: psql -h localhost -p 5432 -d vector_db -U vector_user
```

### pgvector SQL Examples

```sql
-- Verify pgvector is installed
SELECT * FROM pg_extension WHERE extname = 'vector';

-- Create a table with vector column
CREATE TABLE embeddings (
    id SERIAL PRIMARY KEY,
    content TEXT,
    embedding VECTOR(1536)  -- OpenAI embedding size
);

-- Insert vector data
INSERT INTO embeddings (content, embedding) VALUES
    ('Sample text 1', '[0.1, 0.2, 0.3, ...]'),
    ('Sample text 2', '[0.4, 0.5, 0.6, ...]');

-- Find similar vectors (cosine similarity)
SELECT content, 1 - (embedding <=> '[0.1, 0.2, 0.3, ...]') AS similarity
FROM embeddings
ORDER BY embedding <=> '[0.1, 0.2, 0.3, ...]'
LIMIT 5;

-- Create an index for faster similarity search
CREATE INDEX ON embeddings USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);
```

## Connection Details

### PostgreSQL Connections
```bash
# Main database (general use)
Host: postgres-general.databases.svc.cluster.local
Port: 5432
Database: general_db
User: admin

# Vector database (pgvector enabled)  
Host: postgres-general.databases.svc.cluster.local
Port: 5432
Database: vector_db
User: vector_user

# Connection via pooler (recommended for applications)
Host: postgres-general-pooler.databases.svc.cluster.local
Port: 5432
```

### Redis Connections
```bash
# General-purpose Redis
Host: rfs-redis-general.databases.svc.cluster.local  
Port: 26379 (Sentinel)
Service: redis-general

# Cache Redis (high availability)
Host: rfs-redis-cache.databases.svc.cluster.local
Port: 26379 (Sentinel)
Service: redis-cache
```

### QuestDB Connections
```bash
# HTTP/REST API and Web Console
Host: questdb-timeseries.databases.svc.cluster.local
Port: 9000
URL: http://questdb-timeseries.databases.svc.cluster.local:9000

# PostgreSQL protocol (for SQL)
Host: questdb-timeseries.databases.svc.cluster.local  
Port: 8812

# InfluxDB line protocol
Host: questdb-timeseries.databases.svc.cluster.local
Port: 9009
```

## Deployment Order

1. **Operators First** (if not already deployed):
   ```bash
   kubectl apply -f infra/gitops/applications/postgres-operator.yaml
   kubectl apply -f infra/gitops/applications/redis-operator.yaml  
   kubectl apply -f infra/gitops/applications/questdb-operator.yaml
   ```

2. **Database Instances**:
   ```bash
   kubectl apply -f infra/gitops/databases/postgres-general.yaml
   kubectl apply -f infra/gitops/databases/redis-general.yaml
   kubectl apply -f infra/gitops/databases/redis-cache.yaml
   kubectl apply -f infra/gitops/databases/questdb-general.yaml
   ```

3. **ArgoCD Application**:
   ```bash
   kubectl apply -f infra/gitops/applications/database-instances.yaml
   ```

## Monitoring and Health Checks

```bash
# Check all database instances
kubectl get postgresql,redisfailover,questdb -n databases

# Check specific instance health
kubectl describe postgresql postgres-general -n databases
kubectl describe redisfailover redis-cache -n databases
kubectl describe questdb questdb-timeseries -n databases

# Check logs
kubectl logs -n databases -l application=spilo,cluster-name=postgres-general
kubectl logs -n databases -l app=redis-cache
kubectl logs -n databases -l app=questdb-timeseries
```

---

*This configuration provides the complete database infrastructure with:*
- *PostgreSQL 16 with pgvector extension for vector operations*
- *Two Redis instances (general + cache) with different configurations*
- *QuestDB for time-series data with multiple protocol support*
- *Proper resource allocation and high availability where needed*