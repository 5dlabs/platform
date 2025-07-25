# Task 3: Deploy Talos OS Kubernetes Cluster - AI Agent Prompt

You are tasked with deploying a production-ready Kubernetes cluster using the custom Talos OS image created in Task 1. This cluster will serve as the foundation for running a high-performance Solana validator with optimal resource allocation and network configuration.

## Context

You have access to:
- Custom Talos OS image with Solana optimizations (from Task 1)
- Provisioned bare metal infrastructure with 3 control plane nodes and 1 worker node
- Network configuration with dedicated subnets and VLANs
- Storage devices pre-configured on the worker node (15TB ledger, 6TB accounts, 2TB snapshots)

## Objective

Deploy and configure a highly available Kubernetes cluster with:
- 3-node control plane for high availability
- 1 dedicated worker node optimized for Solana validator workloads
- NUMA and huge pages configuration on the worker
- Performance-optimized settings throughout the stack

## Requirements

### 1. Control Plane Configuration
- Deploy 3 control plane nodes in HA configuration
- Configure etcd with appropriate performance settings
- Enable audit logging and security features
- Set up shared VIP for API server access
- Configure proper certificates and encryption

### 2. Worker Node Optimization
- Configure NUMA pinning for single-node topology
- Allocate 100GB huge pages (50,000 x 2MB pages)
- Reserve system and Kubernetes resources (16GB + 16GB memory)
- Enable static CPU manager policy
- Configure topology manager for best performance

### 3. Storage Configuration
- Mount NVMe devices with optimal filesystem settings:
  - Ledger (15TB): ext4 with noatime,nodiratime,nobarrier
  - Accounts (6TB): ext4 with noatime,nodiratime,nobarrier
  - Snapshots (2TB): btrfs with compression
- Create appropriate storage classes for each volume type

### 4. Network Optimization
- Configure jumbo frames (MTU 9000) on worker
- Enable TCP BBR congestion control
- Optimize network buffers for high throughput
- Prepare for Cilium CNI integration

## Implementation Steps

1. **Generate Talos Configurations**
   ```bash
   # Generate secrets once for the entire cluster
   talosctl gen secrets -o secrets.yaml
   
   # Generate configs with appropriate patches
   talosctl gen config solana-cluster https://<VIP>:6443 \
     --with-secrets secrets.yaml
   ```

2. **Apply Configurations**
   - Apply control plane configs to nodes 10.0.1.11-13
   - Apply worker config to node 10.0.1.20
   - Ensure all nodes boot with custom Talos image

3. **Bootstrap Cluster**
   - Bootstrap etcd on first control plane node
   - Wait for cluster initialization
   - Verify all control plane components are healthy

4. **Configure Worker Node**
   - Join worker with specialized configuration
   - Verify huge pages allocation
   - Confirm NUMA topology
   - Check CPU and memory reservations

5. **Post-Bootstrap Setup**
   - Create priority classes for workload scheduling
   - Configure storage classes for local NVMe
   - Apply security policies and RBAC

6. **Validation**
   - Test control plane failover
   - Verify resource allocations
   - Benchmark storage performance
   - Confirm network optimization

## Configuration Requirements

### Control Plane Nodes
- CPU: Minimum 4 cores per node
- Memory: Minimum 16GB per node
- Storage: 100GB OS disk
- Network: Gigabit minimum, 10Gbps preferred

### Worker Node (Solana Validator)
- CPU: 128 threads with NUMA awareness
- Memory: 512GB with huge pages enabled
- Storage: Multiple NVMe devices as specified
- Network: 25Gbps with jumbo frames

### Kernel Parameters (Worker)
```
hugepagesz=2M hugepages=50000
numa_balancing=1
processor.max_cstate=1
amd_pstate=active
amd_iommu=on iommu=pt
```

### Sysctls (Worker)
```
vm.max_map_count=1048576
vm.dirty_ratio=50
vm.nr_hugepages=50000
net.core.rmem_max=268435456
net.core.wmem_max=268435456
net.ipv4.tcp_congestion_control=bbr
```

## Testing Requirements

1. **High Availability Tests**
   - Simulate control plane node failure
   - Verify etcd remains healthy with 2/3 nodes
   - Confirm API server remains accessible
   - Test automatic failover of workloads

2. **Performance Validation**
   - Verify huge pages allocation: 100GB available
   - Confirm NUMA binding works correctly
   - Test storage IOPS: >300K for ledger, >500K for accounts
   - Validate network throughput: >20Gbps

3. **Resource Verification**
   - Check CPU manager static policy active
   - Verify topology manager configuration
   - Confirm memory reservations enforced
   - Test pod scheduling with resource limits

## Expected Outputs

1. **Cluster State**
   - 3 control plane nodes: Ready
   - 1 worker node: Ready with huge pages
   - All system pods: Running and healthy

2. **Configuration Files**
   - secrets.yaml (keep secure!)
   - controlplane-1/2/3.yaml
   - worker-solana.yaml
   - Applied Kubernetes manifests

3. **Verification Results**
   - kubectl get nodes showing all Ready
   - Huge pages verified in /proc/meminfo
   - NUMA topology confirmed
   - Storage mounted and accessible

## Success Criteria

- [ ] All nodes joined and Ready
- [ ] Control plane survives single node failure
- [ ] 100GB huge pages allocated on worker
- [ ] NUMA configuration verified
- [ ] Storage volumes mounted with correct options
- [ ] Network optimizations applied
- [ ] All health checks passing
- [ ] Performance benchmarks meet targets

## Important Considerations

1. **Security**: Never commit secrets.yaml to version control
2. **Ordering**: Control plane must be healthy before adding worker
3. **Validation**: Test each step before proceeding
4. **Monitoring**: Set up observability early for troubleshooting
5. **Documentation**: Record all custom configurations

## Error Handling

Common issues and solutions:

1. **etcd fails to form cluster**
   - Check network connectivity between nodes
   - Verify time synchronization
   - Review firewall rules

2. **Huge pages not allocated**
   - Verify kernel command line parameters
   - Check total system memory
   - Review boot logs for allocation failures

3. **Worker fails to join**
   - Verify control plane is healthy first
   - Check node certificates
   - Review kubelet logs on worker

4. **Storage mount failures**
   - Verify device names match configuration
   - Check filesystem creation
   - Review Talos logs for mount errors

Remember: This cluster is the foundation for a production Solana validator. Ensure all configurations are tested and validated before proceeding to application deployment.