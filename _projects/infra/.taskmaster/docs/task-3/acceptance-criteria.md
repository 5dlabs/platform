# Task 3: Deploy Talos OS Kubernetes Cluster - Acceptance Criteria

## Overview

This document defines the acceptance criteria for deploying a production-ready Kubernetes cluster on Talos OS optimized for Solana validator workloads. All criteria must be met before proceeding to subsequent tasks.

## Functional Acceptance Criteria

### 1. Control Plane Deployment

- [ ] **All control plane nodes operational**
  ```bash
  kubectl get nodes | grep control-plane
  # Expected: 3 nodes with Ready status
  # controlplane-1   Ready    control-plane   5m   v1.31.x
  # controlplane-2   Ready    control-plane   5m   v1.31.x
  # controlplane-3   Ready    control-plane   5m   v1.31.x
  ```

- [ ] **etcd cluster health verified**
  ```bash
  kubectl -n kube-system exec -it etcd-controlplane-1 -- etcdctl \
    --endpoints=https://127.0.0.1:2379 \
    --cacert=/etc/kubernetes/pki/etcd/ca.crt \
    --cert=/etc/kubernetes/pki/etcd/server.crt \
    --key=/etc/kubernetes/pki/etcd/server.key \
    endpoint health
  # All endpoints must report as healthy
  ```

- [ ] **API server high availability**
  ```bash
  # Test VIP accessibility
  curl -k https://10.0.1.10:6443/healthz
  # Expected: ok
  
  # Test failover by stopping one control plane
  talosctl -n controlplane-1 service kubelet stop
  curl -k https://10.0.1.10:6443/healthz
  # Expected: still returns ok
  ```

### 2. Worker Node Configuration

- [ ] **Worker node joined successfully**
  ```bash
  kubectl get node solana-validator -o wide
  # Expected: Ready status with correct labels
  ```

- [ ] **Huge pages allocation verified**
  ```bash
  kubectl describe node solana-validator | grep hugepages
  # Expected:
  # hugepages-2Mi:  100Gi
  # hugepages-2Mi:  100Gi (no requests)
  
  # Direct verification
  talosctl -n 10.0.1.20 cat /proc/meminfo | grep HugePages
  # Expected:
  # HugePages_Total:   50000
  # HugePages_Free:    50000 (or close to it)
  # Hugepagesize:       2048 kB
  ```

- [ ] **NUMA configuration active**
  ```bash
  # Check NUMA policy
  kubectl get node solana-validator -o jsonpath='{.status.capacity}'
  # Should show NUMA resources
  
  # Verify on node
  talosctl -n 10.0.1.20 get numa
  # Should show NUMA topology
  ```

### 3. Resource Management

- [ ] **CPU manager policy configured**
  ```bash
  # Check kubelet configuration
  kubectl get configmap -n kube-system kubelet-config -o yaml | grep cpuManagerPolicy
  # Expected: cpuManagerPolicy: static
  
  # Verify reserved CPUs
  kubectl describe node solana-validator | grep -A5 "Allocated resources"
  # Should show reserved CPUs 0-3
  ```

- [ ] **Memory reservations enforced**
  ```bash
  kubectl describe node solana-validator | grep -A10 "Allocatable:"
  # Expected reduced allocatable memory:
  # memory: ~480Gi (512Gi - 32Gi reserved)
  ```

- [ ] **Topology manager enabled**
  ```bash
  kubectl get configmap -n kube-system kubelet-config -o yaml | grep topologyManagerPolicy
  # Expected: topologyManagerPolicy: single-numa-node
  ```

### 4. Storage Configuration

- [ ] **NVMe volumes mounted correctly**
  ```bash
  # Verify mounts on worker
  talosctl -n 10.0.1.20 ls /var/mnt/
  # Expected: ledger, accounts, snapshots directories
  
  talosctl -n 10.0.1.20 df -h | grep nvme
  # Expected:
  # /dev/nvme1n1p1  15T  ...  /var/mnt/ledger
  # /dev/nvme2n1p1   6T  ...  /var/mnt/accounts
  # /dev/nvme3n1p1   2T  ...  /var/mnt/snapshots
  ```

- [ ] **Storage classes created**
  ```bash
  kubectl get storageclass
  # Expected:
  # local-nvme-ledger
  # local-nvme-accounts
  # local-nvme-snapshots
  ```

- [ ] **Mount options optimized**
  ```bash
  talosctl -n 10.0.1.20 cat /proc/mounts | grep nvme
  # Should show: noatime,nodiratime,nobarrier for ext4
  # Should show: compress=zstd for btrfs snapshots
  ```

### 5. Network Configuration

- [ ] **Jumbo frames enabled**
  ```bash
  talosctl -n 10.0.1.20 ip link show eth0 | grep mtu
  # Expected: mtu 9000
  ```

- [ ] **TCP optimizations applied**
  ```bash
  talosctl -n 10.0.1.20 sysctl net.ipv4.tcp_congestion_control
  # Expected: net.ipv4.tcp_congestion_control = bbr
  
  talosctl -n 10.0.1.20 sysctl net.core.rmem_max
  # Expected: net.core.rmem_max = 268435456
  ```

## Performance Acceptance Criteria

### 1. Cluster Response Times

- [ ] **API server latency**
  ```bash
  # Measure API response time
  time kubectl get nodes
  # Expected: <100ms for simple queries
  
  # Load test
  for i in {1..100}; do time kubectl get nodes > /dev/null; done
  # Average should be <50ms
  ```

### 2. Storage Performance

- [ ] **NVMe IOPS targets**
  ```bash
  # Test on worker node
  kubectl exec -it test-pod -- fio \
    --name=randread --ioengine=io_uring \
    --rw=randread --bs=4k --numjobs=16 \
    --iodepth=64 --size=1G --runtime=30 \
    --filename=/mnt/ledger/test
  
  # Acceptance criteria:
  # Ledger read IOPS: >300,000
  # Accounts read IOPS: >500,000
  # Write IOPS: >200,000
  ```

### 3. Network Performance

- [ ] **Bandwidth validation**
  ```bash
  # Test between nodes
  kubectl run iperf-server --image=networkstatic/iperf3 -- -s
  kubectl run iperf-client --rm -it --image=networkstatic/iperf3 -- \
    -c iperf-server -t 30 -P 8
  
  # Expected: >20Gbps throughput
  ```

## High Availability Criteria

### 1. Control Plane Resilience

- [ ] **Single node failure tolerance**
  ```bash
  # Stop one control plane node
  talosctl -n controlplane-2 shutdown
  
  # Verify cluster operations
  kubectl get nodes  # Should work
  kubectl create deployment test --image=nginx  # Should work
  
  # Verify etcd health
  kubectl -n kube-system get pods | grep etcd
  # 2/3 etcd pods should be running
  ```

- [ ] **Automatic recovery**
  ```bash
  # Restart failed node
  # After node returns, verify:
  kubectl get nodes
  # All 3 control plane nodes should be Ready
  
  # Check etcd member list
  # Should show all 3 members healthy
  ```

### 2. Data Persistence

- [ ] **Configuration persistence**
  ```bash
  # Create test configmap
  kubectl create configmap test-persistence --from-literal=test=value
  
  # Reboot worker node
  talosctl -n 10.0.1.20 reboot
  
  # After reboot, verify:
  kubectl get configmap test-persistence
  # ConfigMap should still exist
  ```

## Security Criteria

### 1. Cluster Security

- [ ] **RBAC enabled**
  ```bash
  kubectl auth can-i --list --as=system:anonymous
  # Should show minimal permissions
  ```

- [ ] **Audit logging configured**
  ```bash
  kubectl logs -n kube-system kube-apiserver-controlplane-1 | grep audit
  # Should show audit log entries
  ```

- [ ] **Network policies ready**
  ```bash
  kubectl api-resources | grep networkpolicies
  # Should be available (after Cilium installation)
  ```

### 2. Node Security

- [ ] **Talos API secured**
  ```bash
  # Verify no SSH access
  ssh root@10.0.1.20
  # Should fail - no SSH daemon
  
  # API requires authentication
  talosctl -n 10.0.1.20 --insecure version
  # Should fail without proper config
  ```

## Operational Criteria

### 1. Monitoring Readiness

- [ ] **Metrics available**
  ```bash
  kubectl top nodes
  # Should show resource usage for all nodes
  
  kubectl top pods -A
  # Should show pod metrics
  ```

- [ ] **Logs accessible**
  ```bash
  kubectl logs -n kube-system kube-apiserver-controlplane-1 --tail=10
  # Should return recent logs
  
  talosctl -n 10.0.1.20 logs kubelet --tail=10
  # Should show kubelet logs
  ```

### 2. Maintenance Operations

- [ ] **Node drain functionality**
  ```bash
  kubectl drain controlplane-3 --ignore-daemonsets
  # Should complete successfully
  
  kubectl uncordon controlplane-3
  # Node should return to Ready state
  ```

- [ ] **Upgrade readiness**
  ```bash
  talosctl -n controlplane-1 version
  # Should show current Talos version
  
  kubectl version
  # Should show current Kubernetes version
  ```

## Test Suite Execution

### Automated Acceptance Test

Create and run the comprehensive test suite:

```bash
#!/bin/bash
# acceptance-test.sh

set -e

echo "Running Kubernetes Cluster Acceptance Tests..."

# Function to check test results
check_result() {
    if [ $1 -eq 0 ]; then
        echo "✓ $2"
    else
        echo "✗ $2"
        exit 1
    fi
}

# Node checks
node_count=$(kubectl get nodes --no-headers | wc -l)
[ "$node_count" -eq 4 ]
check_result $? "All 4 nodes present"

ready_nodes=$(kubectl get nodes --no-headers | grep Ready | wc -l)
[ "$ready_nodes" -eq 4 ]
check_result $? "All nodes in Ready state"

# Huge pages check
huge_pages=$(kubectl describe node solana-validator | grep "hugepages-2Mi:" | head -1 | awk '{print $2}')
[ "$huge_pages" = "100Gi" ]
check_result $? "Huge pages allocated (100Gi)"

# Storage check
storage_count=$(kubectl get storageclass --no-headers | wc -l)
[ "$storage_count" -ge 3 ]
check_result $? "Storage classes created"

# Network check
mtu=$(talosctl -n 10.0.1.20 cat /sys/class/net/eth0/mtu)
[ "$mtu" -eq 9000 ]
check_result $? "Jumbo frames enabled"

# Control plane HA test
kubectl -n kube-system get pods | grep -q etcd
check_result $? "etcd pods running"

# Performance benchmark
echo "Running performance validation..."
kubectl apply -f performance-test-pod.yaml
sleep 30
kubectl logs performance-test-pod
check_result $? "Performance tests completed"

echo "All acceptance tests passed!"
```

## Definition of Done

The task is considered complete when:

1. All functional acceptance criteria are met
2. Performance targets are achieved
3. High availability is verified
4. Security configurations are in place
5. Operational procedures are tested
6. Automated test suite passes 100%
7. Documentation is complete and accurate

## Rollback Plan

If critical issues arise:

1. Preserve etcd backup before changes
2. Document all configuration modifications
3. Test rollback procedure in staging
4. Maintain previous Talos configurations
5. Have recovery snapshots available

This cluster forms the foundation for the Solana validator deployment. Ensure all criteria are thoroughly validated before proceeding.