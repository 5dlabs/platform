# Task 1: Build Custom Talos OS Image with Solana Optimizations

## Overview

This task focuses on creating a custom Talos OS build specifically optimized for running high-performance Solana validators. The custom build includes kernel-level optimizations, huge pages allocation, NUMA optimizations for AMD EPYC processors, and compatibility layers required for future Solana validator implementations like Firedancer.

## Objectives

- Create a custom Talos OS kernel with Solana-specific optimizations
- Enable static huge pages allocation (100GB at boot time)
- Implement NUMA optimizations for AMD EPYC processors
- Add io_uring support for future Firedancer compatibility
- Enable SR-IOV for network acceleration
- Integrate gcompat layer for glibc compatibility
- Build and publish a custom Talos installer image

## Architecture Context

According to the architecture document, the custom Talos build is critical for achieving near-bare-metal performance in a containerized environment. The optimizations directly support:

- **Memory Management**: Static huge pages allocation reduces TLB misses
- **CPU Performance**: NUMA optimizations ensure memory locality
- **Network Performance**: SR-IOV support enables hardware-accelerated networking
- **Future Compatibility**: io_uring and gcompat prepare for Firedancer migration

## Implementation Details

### 1. Build Environment Setup

First, fork the Talos OS repository and set up the build environment:

```bash
# Clone the Talos repository
git clone https://github.com/siderolabs/talos.git
cd talos

# Install build dependencies
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    docker.io \
    docker-buildx-plugin \
    qemu-user-static \
    binfmt-support

# Set up Docker buildx
docker buildx create --name talos-builder --use
docker buildx inspect --bootstrap
```

### 2. Kernel Configuration

Create a custom kernel configuration file with Solana optimizations:

```bash
# Create custom kernel config
cat > kernel/build/config-amd64-solana << EOF
# Base configuration
CONFIG_NR_CPUS=128
CONFIG_NUMA_BALANCING=y
CONFIG_TRANSPARENT_HUGEPAGE=y
CONFIG_TRANSPARENT_HUGEPAGE_MADVISE=y
CONFIG_HUGETLBFS=y
CONFIG_HUGETLB_PAGE=y

# Huge pages configuration
CONFIG_HUGETLB_PAGE_SIZE_VARIABLE=y
CONFIG_ARCH_WANT_HUGE_PMD_SHARE=y
CONFIG_ARCH_HAS_GIGANTIC_PAGE=y

# NUMA optimizations
CONFIG_NUMA=y
CONFIG_AMD_NUMA=y
CONFIG_ACPI_NUMA=y
CONFIG_NODES_SHIFT=4

# Networking optimizations
CONFIG_XDP_SOCKETS=y
CONFIG_BPF_STREAM_PARSER=y
CONFIG_NET_SCH_FQ=y
CONFIG_TCP_CONG_BBR=y
CONFIG_TCP_CONG_BBR2=m

# SR-IOV support
CONFIG_PCI_IOV=y
CONFIG_VFIO=y
CONFIG_VFIO_PCI=y
CONFIG_IXGBE=m
CONFIG_IXGBEVF=m

# io_uring for Firedancer
CONFIG_IO_URING=y
CONFIG_IO_WQ=y

# Crypto acceleration
CONFIG_CRYPTO_SHA256_SSSE3=y
CONFIG_CRYPTO_SHA512_SSSE3=y
CONFIG_CRYPTO_AES_NI_INTEL=y

# Performance monitoring
CONFIG_PERF_EVENTS=y
CONFIG_PERF_EVENTS_AMD_POWER=y
CONFIG_PERF_EVENTS_AMD_UNCORE=y
EOF
```

### 3. Custom Kernel Module

Create a kernel module for runtime optimizations:

```c
// kernel/modules/solana-optimizer/solana_optimizer.c
#include <linux/init.h>
#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/sysctl.h>
#include <linux/mm.h>
#include <linux/sched.h>

static int hugepages_target = 50000; // 100GB with 2MB pages
static int numa_balancing = 1;

static struct ctl_table_header *solana_sysctl_header;

static struct ctl_table solana_sysctl_table[] = {
    {
        .procname = "hugepages_target",
        .data = &hugepages_target,
        .maxlen = sizeof(int),
        .mode = 0644,
        .proc_handler = proc_dointvec,
    },
    {
        .procname = "numa_balancing",
        .data = &numa_balancing,
        .maxlen = sizeof(int),
        .mode = 0644,
        .proc_handler = proc_dointvec,
    },
    {}
};

static struct ctl_table solana_sysctl_root[] = {
    {
        .procname = "solana",
        .mode = 0555,
        .child = solana_sysctl_table,
    },
    {}
};

static int __init solana_optimizer_init(void)
{
    printk(KERN_INFO "Solana Optimizer: Initializing\n");
    
    // Register sysctl interface
    solana_sysctl_header = register_sysctl_table(solana_sysctl_root);
    if (!solana_sysctl_header) {
        return -ENOMEM;
    }
    
    // Apply initial optimizations
    // Note: Actual huge pages allocation is done via kernel cmdline
    
    printk(KERN_INFO "Solana Optimizer: Initialized successfully\n");
    return 0;
}

static void __exit solana_optimizer_exit(void)
{
    unregister_sysctl_table(solana_sysctl_header);
    printk(KERN_INFO "Solana Optimizer: Unloaded\n");
}

module_init(solana_optimizer_init);
module_exit(solana_optimizer_exit);

MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("Solana Validator Performance Optimizer");
MODULE_AUTHOR("Platform Team");
```

### 4. Talos Extension Package

Create a Talos extension for the custom module and gcompat:

```dockerfile
# Dockerfile.extension
FROM scratch AS extension

# Add gcompat for glibc compatibility
COPY --from=alpine:3.19 /lib/ld-musl-x86_64.so.1 /lib/
COPY --from=alpine:3.19 /usr/lib/libgcompat.so.0 /usr/lib/
COPY --from=alpine:3.19 /lib/libc.musl-x86_64.so.1 /lib/

# Add custom kernel module
COPY solana-optimizer.ko /lib/modules/*/kernel/misc/

# Add module load configuration
COPY extension.yaml /etc/talos/
```

```yaml
# extension.yaml
name: solana-optimizer
version: 1.0.0
author: Platform Team
description: Solana validator optimizations for Talos OS
compatibility:
  talos:
    version: ">= 1.10.0"

modules:
  - name: solana_optimizer
    load: true
```

### 5. Machine Configuration

Configure the custom kernel parameters in the Talos machine config:

```yaml
version: v1alpha1
machine:
  type: worker
  install:
    disk: /dev/nvme0n1
    image: ghcr.io/our-org/installer:custom-solana-v1.10.3
    extensions:
      - image: ghcr.io/our-org/solana-optimizer:latest
  
  kernel:
    modules:
      - name: tcp_bbr
      - name: sch_fq
      - name: solana_optimizer
      
  extraKernelArgs:
    # Huge pages configuration
    - hugepagesz=2M
    - hugepages=50000
    - default_hugepagesz=2M
    
    # CPU performance
    - processor.max_cstate=1
    - intel_idle.max_cstate=0
    - amd_pstate=active
    
    # NUMA optimization
    - numa_balancing=1
    
    # Memory management
    - transparent_hugepage=madvise
    
    # IOMMU for SR-IOV
    - intel_iommu=on
    - amd_iommu=on
    - iommu=pt
    
  sysctls:
    # File limits
    fs.file-max: "2000000"
    fs.nr_open: "2000000"
    
    # Memory management
    vm.max_map_count: "1048576"
    vm.dirty_ratio: "50"
    vm.dirty_background_ratio: "5"
    vm.swappiness: "1"
    
    # Network performance
    net.core.rmem_max: "268435456"
    net.core.wmem_max: "268435456"
    net.ipv4.tcp_rmem: "4096 131072 268435456"
    net.ipv4.tcp_wmem: "4096 65536 268435456"
    net.ipv4.tcp_congestion_control: "bbr"
    net.core.netdev_max_backlog: "30000"
```

### 6. Build Process

Create the build script:

```bash
#!/bin/bash
# build-custom-talos.sh

set -e

# Build custom kernel
echo "Building custom kernel..."
make kernel \
  PLATFORM=linux/amd64 \
  PUSH=true \
  USERNAME=our-org \
  REGISTRY=ghcr.io \
  KERNEL_CONFIG=config-amd64-solana

# Build kernel module
echo "Building Solana optimizer module..."
cd kernel/modules/solana-optimizer
make -C /lib/modules/$(uname -r)/build M=$PWD modules
cd ../../../

# Build extension
echo "Building Talos extension..."
docker build -t ghcr.io/our-org/solana-optimizer:latest \
  -f Dockerfile.extension .
docker push ghcr.io/our-org/solana-optimizer:latest

# Build custom installer
echo "Building custom Talos installer..."
docker run --rm -t \
  -v $PWD/_out:/out \
  -v /dev:/dev \
  --privileged \
  ghcr.io/siderolabs/imager:v1.10.3 \
  installer \
  --arch amd64 \
  --platform metal \
  --system-extension-image ghcr.io/our-org/solana-optimizer:latest \
  --customization-image ghcr.io/our-org/kernel:custom-solana-v1.10.3

# Tag and push
docker tag _out/installer-amd64.tar ghcr.io/our-org/installer:custom-solana-v1.10.3
docker push ghcr.io/our-org/installer:custom-solana-v1.10.3

echo "Custom Talos build complete!"
```

## Testing Strategy

### 1. Performance Benchmarking

```bash
# Test huge pages allocation
cat /proc/meminfo | grep -i huge

# Expected output:
# HugePages_Total:   50000
# HugePages_Free:    50000
# Hugepagesize:       2048 kB

# Test NUMA configuration
numactl --hardware

# Test io_uring support
io_uring-bench

# Test network performance with SR-IOV
ethtool -i eth0 | grep driver
# Should show VF driver if SR-IOV is enabled
```

### 2. Compatibility Testing

```bash
# Test gcompat layer
docker run --rm -it alpine:3.19 sh -c "
  apk add --no-cache gcompat
  # Run a glibc binary
  ./test-glibc-binary
"

# Test kernel module
lsmod | grep solana_optimizer
cat /proc/sys/solana/hugepages_target
```

### 3. Performance Comparison

Create a comprehensive benchmark suite:

```bash
#!/bin/bash
# benchmark-talos.sh

# Memory performance
sysbench memory --memory-total-size=100G run

# CPU performance with NUMA
numactl --cpunodebind=0 --membind=0 \
  sysbench cpu --cpu-max-prime=100000 run

# Storage performance
fio --name=test \
    --ioengine=io_uring \
    --rw=randrw \
    --bs=4k \
    --numjobs=16 \
    --iodepth=64 \
    --size=10G \
    --runtime=300

# Network performance
iperf3 -c <target> -t 60 -P 8
```

## Dependencies

This task has no dependencies and can be started immediately. However, it's critical for all subsequent tasks as they will use the custom Talos image.

## Success Criteria

1. Custom Talos kernel builds successfully with all optimizations
2. Huge pages (100GB) are allocated at boot time
3. NUMA optimizations show improved memory locality
4. io_uring support is functional for storage operations
5. SR-IOV is enabled and functional for network interfaces
6. gcompat layer allows running glibc-dependent binaries
7. Performance benchmarks show >15% improvement over stock Talos
8. Custom installer image is published to container registry

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Kernel compilation failures | High | Test incrementally, maintain fallback to stock kernel |
| Performance regressions | Medium | Comprehensive benchmarking before production |
| Compatibility issues | Medium | Extensive testing in staging environment |
| Module stability | Low | Implement proper error handling and logging |

## Resources and References

- [Talos OS Documentation](https://www.talos.dev/v1.10/)
- [Linux Kernel Configuration Guide](https://www.kernel.org/doc/html/latest/admin-guide/kernel-parameters.html)
- [AMD EPYC Tuning Guide](https://www.amd.com/system/files/TechDocs/56263-EPYC-performance-tuning-app-note.pdf)
- [Solana Validator Requirements](https://docs.anza.xyz/operations/requirements)
- Architecture Document: Section 5 - Custom Kernel and Image Building

## Timeline

Estimated Duration: 1 week

- Day 1-2: Build environment setup and kernel configuration
- Day 3-4: Kernel module development and testing
- Day 5: Extension packaging and installer build
- Day 6-7: Comprehensive testing and documentation

## Next Steps

Once this task is complete, the custom Talos image will be used in:
- Task 2: Provision Cherry Servers Infrastructure
- Task 3: Deploy Talos OS Kubernetes Cluster
- All subsequent deployment tasks