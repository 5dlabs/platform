# Task 1: Build Custom Talos OS Image with Solana Optimizations - AI Agent Prompt

You are tasked with building a custom Talos OS image optimized specifically for high-performance Solana validators. This is a critical infrastructure component that will serve as the foundation for a production Solana validator deployment.

## Context

You are working on a project to deploy a high-performance Solana validator on bare metal infrastructure using Talos OS. The standard Talos kernel lacks specific optimizations required for Solana's performance requirements, including huge pages support, NUMA optimizations, and compatibility layers for future validator implementations.

## Objective

Create a custom Talos OS build with kernel-level optimizations that will enable near-bare-metal performance for Solana validators while maintaining the security and immutability benefits of Talos OS.

## Requirements

### 1. Kernel Optimizations
- Configure static huge pages allocation (100GB at boot time using 2MB pages)
- Enable NUMA optimizations for AMD EPYC processors
- Add io_uring support for future Firedancer compatibility
- Enable SR-IOV for network acceleration
- Configure optimal kernel parameters for high-frequency trading workloads

### 2. Compatibility Layers
- Integrate gcompat layer for glibc compatibility
- Ensure compatibility with both current (Agave) and future (Firedancer) Solana validators

### 3. Custom Kernel Module
- Develop a kernel module for runtime performance tuning
- Implement sysctl interfaces for dynamic parameter adjustment
- Ensure compatibility with Talos OS's immutable design

### 4. Build Artifacts
- Create a custom Talos installer image
- Package all optimizations as Talos extensions
- Publish images to container registry (ghcr.io/our-org/)

## Implementation Steps

1. **Fork and Setup Build Environment**
   ```bash
   git clone https://github.com/siderolabs/talos.git
   cd talos
   # Install Docker, buildx, and other build dependencies
   ```

2. **Create Custom Kernel Configuration**
   - Copy the base AMD64 config
   - Add Solana-specific optimizations
   - Enable huge pages, NUMA, io_uring, SR-IOV
   - Configure crypto acceleration

3. **Develop Kernel Module**
   - Create `solana-optimizer` module
   - Implement runtime tuning capabilities
   - Add sysctl interfaces

4. **Package as Talos Extension**
   - Create extension Dockerfile
   - Include gcompat libraries
   - Package kernel module
   - Define extension metadata

5. **Build Custom Installer**
   - Build custom kernel
   - Build extension image
   - Create installer with extensions
   - Tag and push to registry

6. **Configure Machine Parameters**
   - Set kernel command line arguments
   - Configure sysctls for performance
   - Enable required kernel modules

## Testing Requirements

1. **Functional Testing**
   - Verify huge pages allocation (50,000 pages = 100GB)
   - Confirm NUMA node detection and configuration
   - Test io_uring functionality
   - Validate SR-IOV device enumeration

2. **Performance Testing**
   - Benchmark memory performance with sysbench
   - Test CPU performance with NUMA bindings
   - Validate storage performance with fio using io_uring
   - Measure network throughput with iperf3

3. **Compatibility Testing**
   - Test gcompat with glibc binaries
   - Verify kernel module loading and operation
   - Ensure Talos APIs work with customizations

4. **Comparison Testing**
   - Compare performance vs stock Talos kernel
   - Target: >15% improvement in memory and CPU benchmarks
   - Document all performance metrics

## Expected Outputs

1. **Container Images**
   - `ghcr.io/our-org/kernel:custom-solana-v1.10.3`
   - `ghcr.io/our-org/solana-optimizer:latest`
   - `ghcr.io/our-org/installer:custom-solana-v1.10.3`

2. **Documentation**
   - Build process documentation
   - Kernel configuration parameters
   - Performance benchmark results
   - Usage instructions

3. **Configuration Files**
   - Custom kernel config (`config-amd64-solana`)
   - Machine configuration template
   - Extension metadata

## Success Criteria

- [ ] Custom kernel builds without errors
- [ ] 100GB huge pages allocated at boot
- [ ] NUMA optimizations functional
- [ ] io_uring support verified
- [ ] SR-IOV enabled and working
- [ ] gcompat layer functional
- [ ] Performance improvement >15% over stock
- [ ] All images published to registry
- [ ] Comprehensive documentation complete

## Important Considerations

1. **Security**: Maintain Talos OS security model while adding optimizations
2. **Immutability**: Ensure all changes are compatible with immutable infrastructure
3. **Reproducibility**: Document build process for reproducible builds
4. **Compatibility**: Test with both current and planned Solana validators
5. **Performance**: Validate that optimizations provide measurable benefits

## Resources

- Talos OS source: https://github.com/siderolabs/talos
- Kernel parameters: https://www.kernel.org/doc/html/latest/admin-guide/kernel-parameters.html
- AMD EPYC tuning: https://developer.amd.com/resources/epyc-resources/
- Solana requirements: https://docs.anza.xyz/operations/requirements

## Error Handling

If you encounter build failures:
1. Check Docker and buildx installation
2. Verify kernel config syntax
3. Test with minimal changes first
4. Consult Talos build documentation

For performance issues:
1. Verify huge pages allocation
2. Check NUMA topology detection
3. Confirm kernel parameters applied
4. Review benchmark methodology

Remember: This custom image is the foundation for the entire Solana validator infrastructure. Ensure thorough testing before proceeding to subsequent tasks.