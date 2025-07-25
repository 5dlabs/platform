# Task 1: Build Custom Talos OS Image - Acceptance Criteria

## Overview

This document defines the acceptance criteria for the custom Talos OS image build task. All criteria must be met for the task to be considered complete and ready for use in production Solana validator deployment.

## Functional Acceptance Criteria

### 1. Kernel Build and Configuration

- [ ] **Custom kernel compilation successful**
  - Kernel version: Linux 6.x (latest stable)
  - Build completes without errors
  - All Solana-specific patches applied

- [ ] **Kernel configuration verified**
  ```bash
  # Verify kernel config
  zcat /proc/config.gz | grep -E "HUGEPAGE|NUMA|IO_URING|SR_IOV"
  # All required options must be enabled
  ```

- [ ] **Boot parameters applied**
  ```bash
  cat /proc/cmdline
  # Must include: hugepagesz=2M hugepages=50000 numa_balancing=1
  ```

### 2. Memory Management

- [ ] **Huge pages allocation confirmed**
  ```bash
  cat /proc/meminfo | grep Huge
  # Expected:
  # HugePages_Total:   50000
  # HugePages_Free:    ≥49000 (some system usage allowed)
  # Hugepagesize:      2048 kB
  ```

- [ ] **Memory pressure handling**
  ```bash
  # Test under memory pressure
  stress-ng --vm 8 --vm-bytes 80% --timeout 60s
  # System must remain stable, huge pages must not be reclaimed
  ```

### 3. NUMA Optimizations

- [ ] **NUMA topology detected**
  ```bash
  numactl --hardware
  # Must show correct NUMA nodes for AMD EPYC
  # Memory and CPU distribution must be balanced
  ```

- [ ] **NUMA memory binding functional**
  ```bash
  numactl --cpunodebind=0 --membind=0 echo "NUMA test"
  # Command must execute without errors
  ```

### 4. Storage and I/O

- [ ] **io_uring support enabled**
  ```bash
  # Test io_uring functionality
  io_uring-bench --runtime 10
  # Must complete successfully with performance metrics
  ```

- [ ] **NVMe optimization**
  ```bash
  # Check NVMe queue depth
  cat /sys/block/nvme*/queue/nr_requests
  # Should be ≥1024
  ```

### 5. Network Optimizations

- [ ] **SR-IOV capability verified**
  ```bash
  lspci | grep -i virtual
  # Must show SR-IOV capable network devices
  
  # Check VF support
  cat /sys/class/net/*/device/sriov_totalvfs
  # Should show >0 for SR-IOV capable interfaces
  ```

- [ ] **Network stack optimizations**
  ```bash
  sysctl net.ipv4.tcp_congestion_control
  # Must return: net.ipv4.tcp_congestion_control = bbr
  
  sysctl net.core.rmem_max
  # Must return: net.core.rmem_max = 268435456
  ```

### 6. Compatibility Features

- [ ] **gcompat layer functional**
  ```bash
  # Test glibc binary execution
  ldd /test/glibc-test-binary
  # Must resolve all dependencies through gcompat
  
  # Run test binary
  /test/glibc-test-binary
  # Must execute successfully
  ```

- [ ] **Kernel module loaded**
  ```bash
  lsmod | grep solana_optimizer
  # Module must be listed and loaded
  
  # Check sysctl interface
  ls /proc/sys/solana/
  # Must show: hugepages_target, numa_balancing
  ```

## Performance Acceptance Criteria

### 1. Memory Performance

- [ ] **Huge pages performance improvement**
  ```bash
  # Baseline (without huge pages)
  sysbench memory --memory-total-size=50G run
  
  # With huge pages
  sysbench memory --memory-total-size=50G --memory-hugetlb=on run
  
  # Acceptance: >20% improvement in throughput
  ```

### 2. CPU Performance

- [ ] **NUMA-aware performance**
  ```bash
  # Cross-NUMA test
  numactl --cpunodebind=0 --membind=1 sysbench cpu run
  
  # Local NUMA test
  numactl --cpunodebind=0 --membind=0 sysbench cpu run
  
  # Acceptance: Local NUMA >30% faster than cross-NUMA
  ```

### 3. Storage Performance

- [ ] **io_uring performance targets**
  ```bash
  fio --name=test --ioengine=io_uring --rw=randread --bs=4k \
      --numjobs=16 --iodepth=64 --size=10G --runtime=60
  
  # Acceptance criteria:
  # - Read IOPS: >500K
  # - Latency P99: <1ms
  # - CPU usage: <50%
  ```

### 4. Network Performance

- [ ] **Throughput targets**
  ```bash
  iperf3 -c localhost -t 30 -P 8
  
  # Acceptance criteria:
  # - Throughput: >20Gbps
  # - Retransmissions: <0.01%
  # - CPU usage: <60%
  ```

## Build and Deployment Criteria

### 1. Image Artifacts

- [ ] **All images built and tagged**
  ```bash
  docker images | grep our-org
  # Must show:
  # ghcr.io/our-org/kernel:custom-solana-v1.10.3
  # ghcr.io/our-org/solana-optimizer:latest
  # ghcr.io/our-org/installer:custom-solana-v1.10.3
  ```

- [ ] **Images pushed to registry**
  ```bash
  docker pull ghcr.io/our-org/installer:custom-solana-v1.10.3
  # Must download successfully
  ```

### 2. Installation Testing

- [ ] **Clean installation successful**
  ```bash
  talosctl apply-config --insecure --nodes $NODE_IP --file machine.yaml
  # Installation must complete without errors
  ```

- [ ] **Upgrade from stock Talos**
  ```bash
  talosctl upgrade --nodes $NODE_IP \
    --image ghcr.io/our-org/installer:custom-solana-v1.10.3
  # Upgrade must preserve data and complete successfully
  ```

## Documentation Criteria

### 1. Build Documentation

- [ ] **Build process documented**
  - Step-by-step build instructions
  - Required dependencies listed
  - Common issues and solutions

- [ ] **Configuration reference**
  - All kernel parameters explained
  - Sysctl settings documented
  - Machine config examples provided

### 2. Operations Documentation

- [ ] **Deployment guide**
  - Installation procedures
  - Upgrade procedures
  - Rollback procedures

- [ ] **Performance tuning guide**
  - Benchmark procedures
  - Optimization recommendations
  - Troubleshooting steps

## Security Criteria

### 1. Security Compliance

- [ ] **Talos security model maintained**
  - No SSH access enabled
  - API-only management preserved
  - Immutable filesystem intact

- [ ] **Kernel security features**
  ```bash
  # Check security features
  grep -E "RANDOMIZE_BASE|FORTIFY_SOURCE|STACKPROTECTOR" /proc/config.gz
  # All must be enabled
  ```

### 2. Supply Chain Security

- [ ] **Reproducible builds**
  - Build process in Git
  - All dependencies pinned
  - Checksums documented

- [ ] **Image signing**
  - Images signed with org key
  - Signature verification documented

## Test Suite Execution

### Automated Test Suite

Create and run the comprehensive test suite:

```bash
#!/bin/bash
# acceptance-test.sh

echo "Running Talos Custom Image Acceptance Tests..."

# Function to check test results
check_result() {
    if [ $1 -eq 0 ]; then
        echo "✓ $2"
    else
        echo "✗ $2"
        exit 1
    fi
}

# 1. Kernel checks
grep -q "CONFIG_HUGETLB_PAGE=y" /proc/config.gz
check_result $? "Huge pages support in kernel"

# 2. Memory checks
huge_total=$(grep HugePages_Total /proc/meminfo | awk '{print $2}')
[ "$huge_total" -eq "50000" ]
check_result $? "Huge pages allocation (50000 pages)"

# 3. NUMA checks
numactl --hardware > /dev/null 2>&1
check_result $? "NUMA support functional"

# 4. io_uring checks
io_uring-bench --probe-only > /dev/null 2>&1
check_result $? "io_uring support available"

# 5. Network checks
[ -f /sys/class/net/eth0/device/sriov_totalvfs ]
check_result $? "SR-IOV capability present"

# 6. Module checks
lsmod | grep -q solana_optimizer
check_result $? "Solana optimizer module loaded"

# 7. Performance benchmark
echo "Running performance benchmarks..."
./run-benchmarks.sh
check_result $? "Performance benchmarks passed"

echo "All acceptance tests passed!"
```

## Sign-off Requirements

### Technical Sign-off
- [ ] Lead Engineer approval
- [ ] Performance benchmarks reviewed
- [ ] Security review completed

### Operational Sign-off
- [ ] Documentation reviewed
- [ ] Runbooks tested
- [ ] Team training completed

### Management Sign-off
- [ ] Cost analysis reviewed
- [ ] Risk assessment accepted
- [ ] Production readiness confirmed

## Definition of Done

The task is considered complete when:

1. All functional acceptance criteria are met
2. All performance targets are achieved
3. All images are built and published
4. Documentation is complete and reviewed
5. Test suite passes 100%
6. All sign-offs are obtained
7. Images are deployed successfully in staging environment

## Rollback Criteria

If any critical issues are discovered post-deployment:

1. Stock Talos images remain available
2. Rollback procedure documented and tested
3. Data preservation confirmed during rollback
4. Performance baselines for comparison available

This acceptance criteria document serves as the definitive checklist for task completion and production readiness.