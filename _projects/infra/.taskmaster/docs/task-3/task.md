# Task 3: Deploy Talos OS Kubernetes Cluster

## Overview

This task focuses on deploying the custom Talos OS image to control plane and worker nodes, bootstrapping a production-ready Kubernetes cluster, and applying performance-optimized configurations specifically tailored for Solana validator workloads. The deployment creates a highly available control plane across three nodes and configures the Solana validator node as a performance-optimized worker.

## Objectives

- Deploy custom Talos OS to all nodes using the image from Task 1
- Bootstrap a highly available Kubernetes control plane with 3 nodes
- Configure NUMA and huge pages on the Solana validator worker node
- Apply performance-optimized Kubernetes settings
- Establish secure node communication with proper certificates
- Configure storage mounts with appropriate permissions
- Verify cluster stability and failover capabilities

## Architecture Context

According to the architecture document, the Kubernetes cluster serves as the orchestration layer that enables:

- **High Availability**: 3-node control plane with etcd consensus
- **Performance Isolation**: Dedicated worker node for Solana with NUMA pinning
- **Resource Management**: Static CPU allocation and memory reservations
- **Storage Orchestration**: Direct NVMe access with optimized mount options
- **Network Performance**: Integration with Cilium CNI for eBPF acceleration

## Implementation Details

### 1. Generate Talos Configuration

First, generate the Talos configuration files for all nodes:

```bash
# Generate cluster secrets
talosctl gen secrets -o secrets.yaml

# Generate control plane configurations
for i in 1 2 3; do
  talosctl gen config solana-cluster https://10.0.1.10:6443 \
    --with-secrets secrets.yaml \
    --config-patch-control-plane @controlplane-patch.yaml \
    --output controlplane-$i.yaml
done

# Generate worker node configuration
talosctl gen config solana-cluster https://10.0.1.10:6443 \
  --with-secrets secrets.yaml \
  --config-patch-worker @worker-patch.yaml \
  --output worker-solana.yaml
```

### 2. Control Plane Configuration Patch

Create the control plane patch file:

```yaml
# controlplane-patch.yaml
machine:
  type: controlplane
  install:
    disk: /dev/nvme0n1
    image: ghcr.io/our-org/installer:custom-solana-v1.10.3
  
  network:
    hostname: {{ .hostname }}
    interfaces:
      - interface: eth0
        dhcp: false
        addresses:
          - {{ .nodeIP }}/24
        routes:
          - network: 0.0.0.0/0
            gateway: 10.0.1.1
        vip:
          ip: 10.0.1.10  # Shared VIP for API server
  
  kubelet:
    extraArgs:
      rotate-server-certificates: true
      cluster-dns: 10.96.0.10,1.1.1.1
  
  kernel:
    modules:
      - name: br_netfilter
      - name: overlay
      
cluster:
  controlPlane:
    endpoint: https://10.0.1.10:6443
  
  etcd:
    advertisedSubnets:
      - 10.0.1.0/24
    extraArgs:
      election-timeout: 5000
      heartbeat-interval: 500
      snapshot-count: 10000
  
  apiServer:
    extraArgs:
      audit-log-maxage: 30
      audit-log-maxbackup: 10
      enable-admission-plugins: NodeRestriction,ResourceQuota
      
  controllerManager:
    extraArgs:
      bind-address: 0.0.0.0
      node-monitor-grace-period: 40s
      node-monitor-period: 5s
```

### 3. Worker Node Configuration Patch

Create the Solana validator worker configuration:

```yaml
# worker-patch.yaml
machine:
  type: worker
  install:
    disk: /dev/nvme0n1
    image: ghcr.io/our-org/installer:custom-solana-v1.10.3
    extensions:
      - image: ghcr.io/our-org/solana-optimizer:latest
  
  network:
    hostname: solana-validator
    interfaces:
      - interface: eth0
        dhcp: false
        addresses:
          - 10.0.1.20/24
        routes:
          - network: 0.0.0.0/0
            gateway: 10.0.1.1
        mtu: 9000  # Jumbo frames
  
  kubelet:
    extraArgs:
      system-reserved: memory=16Gi,cpu=4
      kube-reserved: memory=16Gi,cpu=4
      eviction-hard: memory.available<32Gi
      cpu-manager-policy: static
      topology-manager-policy: single-numa-node
      reserved-cpus: 0-3  # Reserve CPUs for system
      feature-gates: CPUManager=true,TopologyManager=true
    
    nodeIP:
      validSubnets:
        - 10.0.1.0/24
  
  kernel:
    modules:
      - name: tcp_bbr
      - name: sch_fq
      - name: solana_optimizer
      
  extraKernelArgs:
    # Huge pages (100GB)
    - hugepagesz=2M
    - hugepages=50000
    - default_hugepagesz=2M
    
    # CPU performance
    - processor.max_cstate=1
    - intel_idle.max_cstate=0
    - amd_pstate=active
    
    # NUMA
    - numa_balancing=1
    
    # IOMMU for SR-IOV
    - amd_iommu=on
    - iommu=pt
    
  sysctls:
    # Memory
    vm.max_map_count: "1048576"
    vm.dirty_ratio: "50"
    vm.dirty_background_ratio: "5"
    vm.swappiness: "1"
    vm.nr_hugepages: "50000"
    
    # Network
    net.core.rmem_max: "268435456"
    net.core.wmem_max: "268435456"
    net.core.netdev_max_backlog: "30000"
    net.ipv4.tcp_rmem: "4096 131072 268435456"
    net.ipv4.tcp_wmem: "4096 65536 268435456"
    net.ipv4.tcp_congestion_control: "bbr"
    net.ipv4.tcp_slow_start_after_idle: "0"
    net.ipv4.tcp_mtu_probing: "1"
    
  # Storage configuration
  disks:
    - device: /dev/nvme1n1  # 15TB Ledger
      partitions:
        - mountpoint: /var/mnt/ledger
          size: 0  # Use entire disk
    - device: /dev/nvme2n1  # 6TB Accounts
      partitions:
        - mountpoint: /var/mnt/accounts
          size: 0
    - device: /dev/nvme3n1  # 2TB Snapshots
      partitions:
        - mountpoint: /var/mnt/snapshots
          size: 0
  
  # Mount options for performance
  files:
    - content: |
        /dev/nvme1n1p1 /var/mnt/ledger ext4 noatime,nodiratime,nobarrier 0 0
        /dev/nvme2n1p1 /var/mnt/accounts ext4 noatime,nodiratime,nobarrier 0 0
        /dev/nvme3n1p1 /var/mnt/snapshots btrfs noatime,nodiratime,compress=zstd 0 0
      path: /etc/fstab
      permissions: 0644
```

### 4. Bootstrap Process

Execute the bootstrap process:

```bash
#!/bin/bash
# bootstrap-cluster.sh

set -e

# Control plane IPs
CONTROL_PLANE_1="10.0.1.11"
CONTROL_PLANE_2="10.0.1.12"
CONTROL_PLANE_3="10.0.1.13"
WORKER_NODE="10.0.1.20"

echo "Applying Talos configuration to control plane nodes..."

# Apply config to control plane nodes
for i in 1 2 3; do
  IP="CONTROL_PLANE_$i"
  talosctl apply-config --insecure \
    --nodes ${!IP} \
    --file controlplane-$i.yaml
done

echo "Waiting for control plane nodes to be ready..."
sleep 60

# Bootstrap etcd on first control plane
echo "Bootstrapping Kubernetes cluster..."
talosctl bootstrap --nodes $CONTROL_PLANE_1 --endpoints $CONTROL_PLANE_1

# Wait for cluster to be ready
echo "Waiting for cluster initialization..."
talosctl health --nodes $CONTROL_PLANE_1 --wait-timeout 10m

# Get kubeconfig
echo "Retrieving kubeconfig..."
talosctl kubeconfig --nodes $CONTROL_PLANE_1 --force

# Verify control plane
kubectl get nodes
kubectl get pods -A

# Apply worker configuration
echo "Adding worker node..."
talosctl apply-config --insecure \
  --nodes $WORKER_NODE \
  --file worker-solana.yaml

# Wait for worker to join
echo "Waiting for worker node to join..."
kubectl wait --for=condition=Ready node/solana-validator --timeout=5m

echo "Cluster deployment complete!"
```

### 5. Post-Bootstrap Configuration

Apply additional Kubernetes configurations:

```yaml
# priority-classes.yaml
apiVersion: scheduling.k8s.io/v1
kind: PriorityClass
metadata:
  name: solana-critical
value: 1000000
globalDefault: false
description: "Critical priority for Solana validator pods"
---
apiVersion: scheduling.k8s.io/v1
kind: PriorityClass
metadata:
  name: system-cluster-critical
value: 2000000
globalDefault: false
description: "System cluster critical priority"
```

```yaml
# storage-classes.yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: local-nvme-ledger
provisioner: kubernetes.io/no-provisioner
volumeBindingMode: WaitForFirstConsumer
---
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: local-nvme-accounts
provisioner: kubernetes.io/no-provisioner
volumeBindingMode: WaitForFirstConsumer
---
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: local-nvme-snapshots
provisioner: kubernetes.io/no-provisioner
volumeBindingMode: WaitForFirstConsumer
```

### 6. NUMA and CPU Pinning Verification

Verify NUMA configuration on the worker node:

```bash
# Check NUMA topology
talosctl -n $WORKER_NODE read /proc/cmdline
# Should show: hugepagesz=2M hugepages=50000 numa_balancing=1

# Verify huge pages
talosctl -n $WORKER_NODE cat /proc/meminfo | grep Huge
# Should show: HugePages_Total: 50000

# Check NUMA nodes
talosctl -n $WORKER_NODE get numa
# Should show NUMA topology with memory distribution

# Verify CPU manager
kubectl describe node solana-validator | grep -A5 "Capacity:"
# Should show hugepages-2Mi: 100Gi
```

### 7. Cluster Health Checks

Create comprehensive health check script:

```bash
#!/bin/bash
# cluster-health-check.sh

echo "=== Cluster Health Check ==="

# Check node status
echo "Node Status:"
kubectl get nodes -o wide

# Check control plane components
echo -e "\nControl Plane Health:"
kubectl get pods -n kube-system -l tier=control-plane

# Check etcd health
echo -e "\netcd Cluster Health:"
kubectl -n kube-system exec etcd-controlplane-1 -- etcdctl \
  --endpoints=https://127.0.0.1:2379 \
  --cacert=/etc/kubernetes/pki/etcd/ca.crt \
  --cert=/etc/kubernetes/pki/etcd/server.crt \
  --key=/etc/kubernetes/pki/etcd/server.key \
  endpoint health

# Check Talos services
echo -e "\nTalos Service Status:"
for node in $CONTROL_PLANE_1 $CONTROL_PLANE_2 $CONTROL_PLANE_3 $WORKER_NODE; do
  echo "Node: $node"
  talosctl -n $node service
done

# Verify huge pages on worker
echo -e "\nWorker Node Huge Pages:"
kubectl exec -it deployment/debug-tools -- cat /proc/meminfo | grep Huge

# Test pod scheduling
echo -e "\nTesting Pod Scheduling:"
kubectl run test-pod --image=busybox --rm -it --restart=Never -- echo "Scheduling works!"
```

## Testing Strategy

### 1. Control Plane High Availability

```bash
# Test control plane failover
echo "Simulating control plane node failure..."

# Stop kubelet on one control plane node
talosctl -n $CONTROL_PLANE_2 service kubelet stop

# Verify cluster remains operational
kubectl get nodes
kubectl run ha-test --image=nginx --rm -it --restart=Never -- echo "HA works!"

# Restore service
talosctl -n $CONTROL_PLANE_2 service kubelet start
```

### 2. NUMA Performance Testing

```bash
# Deploy NUMA test pod
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: Pod
metadata:
  name: numa-test
spec:
  nodeSelector:
    kubernetes.io/hostname: solana-validator
  containers:
  - name: numa-test
    image: ubuntu:22.04
    command: ["sleep", "infinity"]
    resources:
      requests:
        memory: 32Gi
        cpu: 16
        hugepages-2Mi: 10Gi
      limits:
        memory: 32Gi
        cpu: 16
        hugepages-2Mi: 10Gi
EOF

# Verify NUMA binding
kubectl exec numa-test -- numactl --show
kubectl exec numa-test -- numastat -p 1
```

### 3. Storage Performance Validation

```bash
# Test storage performance on worker
kubectl exec -it deployment/debug-tools -- bash -c "
  # Test ledger storage
  fio --name=ledger-test --filename=/mnt/ledger/test \
      --ioengine=io_uring --rw=randread --bs=4k \
      --numjobs=16 --iodepth=64 --size=10G --runtime=60
  
  # Test accounts storage  
  fio --name=accounts-test --filename=/mnt/accounts/test \
      --ioengine=io_uring --rw=randwrite --bs=4k \
      --numjobs=16 --iodepth=64 --size=10G --runtime=60
"
```

## Dependencies

- Task 1: Custom Talos OS image must be built and available
- Task 2: Infrastructure must be provisioned with correct networking

## Success Criteria

1. All 3 control plane nodes running and healthy
2. etcd cluster formed with proper quorum
3. Worker node joined with correct resource allocations
4. Huge pages (100GB) available on worker node
5. NUMA topology correctly configured
6. Storage volumes mounted with optimal settings
7. Cluster survives single control plane node failure
8. All health checks pass consistently

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| etcd split-brain | High | Odd number of control plane nodes, proper network segmentation |
| Huge pages allocation failure | High | Pre-allocate at boot, monitor memory pressure |
| NUMA misconfiguration | Medium | Automated verification scripts, performance testing |
| Storage performance issues | Medium | Benchmark before production, monitor IOPS |

## Resources and References

- [Talos OS Cluster Bootstrapping](https://www.talos.dev/v1.10/talos-guides/install/bare-metal-platforms/equinix-metal/)
- [Kubernetes CPU Manager](https://kubernetes.io/docs/tasks/administer-cluster/cpu-management-policies/)
- [NUMA-aware Scheduling](https://kubernetes.io/docs/tasks/administer-cluster/numa-aware-scheduling/)
- [etcd Performance Tuning](https://etcd.io/docs/v3.5/op-guide/performance/)
- Architecture Document: Section 4 - Talos OS Configuration

## Timeline

Estimated Duration: 3 days

- Day 1: Generate configurations and bootstrap control plane
- Day 2: Add worker node and verify NUMA/huge pages
- Day 3: Performance testing and failover validation

## Next Steps

Once this task is complete:
- Task 4: Deploy Cilium CNI with eBPF optimizations
- Task 5: Deploy Solana validator container
- Configure network policies and storage classes for workloads