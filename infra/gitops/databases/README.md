# Database Operators and Resources

This directory contains database operator configurations and custom resources for managing PostgreSQL and Redis clusters in Kubernetes.

## PostgreSQL Operator (Zalando)

The Zalando PostgreSQL operator manages PostgreSQL clusters with high availability, automatic failover, and backup capabilities.

### Features
- **High Availability**: Multi-replica PostgreSQL clusters with streaming replication via Patroni
- **Automatic Failover**: Built-in leader election and automatic promotion
- **Connection Pooling**: Integrated PgBouncer support
- **Backup & Recovery**: Point-in-time recovery with pg_basebackup/WAL-E
- **Rolling Updates**: Zero-downtime updates and version upgrades
- **Resource Management**: CPU/memory limits and requests per cluster

### Creating a PostgreSQL Cluster

Example minimal cluster:
```yaml
apiVersion: acid.zalan.do/v1
kind: postgresql
metadata:
  name: my-postgres-cluster
spec:
  teamId: "platform"
  volume:
    size: 10Gi
  numberOfInstances: 2
  users:
    myapp:
    - superuser
    - createdb
  databases:
    mydb: myapp
  postgresql:
    version: "17"
```

### Useful Commands
```bash
# List PostgreSQL clusters
kubectl get postgresql -A

# Get cluster details
kubectl describe postgresql my-postgres-cluster

# Access master pod
kubectl exec -it my-postgres-cluster-0 -- psql -U postgres

# Check cluster status
kubectl get pods -l cluster-name=my-postgres-cluster
```

## Redis Operator (Spotahome)

The Spotahome Redis operator manages Redis failover clusters with Sentinel for high availability.

### Features
- **High Availability**: Redis with Sentinel-based automatic failover
- **Automatic Failover**: Master election via Redis Sentinel
- **Data Persistence**: Optional RDB/AOF persistence
- **Security**: Password authentication and TLS support
- **Monitoring**: Prometheus metrics export
- **Resource Management**: Configurable resources per component

### Creating a Redis Failover Cluster

Example minimal cluster:
```yaml
apiVersion: databases.spotahome.com/v1
kind: RedisFailover
metadata:
  name: my-redis-cluster
spec:
  sentinel:
    replicas: 3
    resources:
      requests:
        cpu: 100m
        memory: 100Mi
      limits:
        cpu: 500m
        memory: 200Mi
  redis:
    replicas: 3
    resources:
      requests:
        cpu: 100m
        memory: 200Mi
      limits:
        cpu: 1000m
        memory: 1Gi
    storage:
      persistentVolumeClaim:
        metadata:
          name: redis-data
        spec:
          accessModes:
            - ReadWriteOnce
          resources:
            requests:
              storage: 10Gi
```

### Useful Commands
```bash
# List Redis failover clusters
kubectl get redisfailover -A

# Get cluster details
kubectl describe redisfailover my-redis-cluster

# Access Redis master
kubectl exec -it rfr-my-redis-cluster-0 -- redis-cli

# Check Sentinel status
kubectl exec -it rfs-my-redis-cluster-0 -- redis-cli -p 26379 sentinel masters
```

## Best Practices

### PostgreSQL
1. **Always use at least 2 instances** for high availability
2. **Configure appropriate resource limits** based on workload
3. **Enable monitoring** via the built-in metrics exporter
4. **Use connection pooling** for applications with many connections
5. **Regular backups** - configure WAL archiving for PITR
6. **Version upgrades** - test in non-production first

### Redis
1. **Use odd number of Sentinels** (3 or 5) for proper quorum
2. **Configure persistence** based on data criticality
3. **Set memory policies** (`maxmemory-policy`) appropriately
4. **Monitor memory usage** to prevent OOM situations
5. **Use AUTH** for production deployments
6. **Regular backups** if data persistence is critical

## Monitoring

Both operators expose Prometheus metrics:

- **PostgreSQL**: Port 9187 on each PostgreSQL pod
- **Redis**: Port 9121 on Redis pods, port 26379 on Sentinel pods

Configure ServiceMonitors or scrape configs to collect these metrics.

## Troubleshooting

### PostgreSQL Issues
```bash
# Check operator logs
kubectl logs -n postgres-operator deployment/postgres-operator

# Check Patroni status
kubectl exec my-postgres-cluster-0 -- patronictl list

# Check PostgreSQL logs
kubectl logs my-postgres-cluster-0 -c postgres
```

### Redis Issues
```bash
# Check operator logs
kubectl logs -n redis-operator deployment/redisoperator

# Check Sentinel logs
kubectl logs rfs-my-redis-cluster-0

# Check Redis logs
kubectl logs rfr-my-redis-cluster-0
```

## Security Considerations

1. **Network Policies**: Implement network policies to restrict database access
2. **Secrets Management**: Use external-secrets operator for credential management
3. **Encryption**: Enable TLS for client connections and replication
4. **RBAC**: Limit operator permissions to necessary namespaces
5. **Audit Logging**: Enable audit logging for compliance requirements

## References

- [Zalando PostgreSQL Operator Documentation](https://postgres-operator.readthedocs.io/)
- [Spotahome Redis Operator Documentation](https://github.com/spotahome/redis-operator/tree/master/docs)
- [PostgreSQL CRD Reference](https://postgres-operator.readthedocs.io/en/latest/reference/cluster_manifest/)
- [RedisFailover CRD Reference](https://github.com/spotahome/redis-operator/blob/master/api/redisfailover/v1/validate.go)
