# Task 4: Implement Cilium CNI with eBPF Acceleration - AI Agent Prompt

You are tasked with deploying Cilium as the Container Network Interface (CNI) for a high-performance Solana validator Kubernetes cluster. This deployment requires advanced eBPF features, custom XDP programs, and specific optimizations for blockchain network traffic.

## Context

You have:
- A working Kubernetes cluster deployed with Talos OS (from Task 3)
- Worker node with high-performance networking (25Gbps, SR-IOV capable)
- No existing CNI installed (fresh cluster)
- Kernel with eBPF support (Linux 5.x+)
- Requirements for ultra-low latency and high packet throughput

## Objective

Deploy and configure Cilium CNI with:
- eBPF datapath for maximum performance
- kube-proxy replacement for reduced latency
- Custom XDP programs for Solana traffic prioritization
- DDoS protection at kernel level
- Network observability with Hubble
- Optimized for blockchain gossip protocols

## Requirements

### 1. Cilium Deployment
- Install Cilium v1.16.x using Helm
- Enable eBPF-only mode (no iptables)
- Configure kube-proxy replacement in strict mode
- Disable BPF masquerade for Talos compatibility
- Enable native routing mode
- Configure for jumbo frames (MTU 9000)

### 2. XDP Traffic Prioritization
- Implement XDP program for Solana gossip traffic (UDP 8000-8002)
- Prioritize validator-to-validator communication
- Rate limit incoming connections
- Drop malicious traffic at kernel level
- Integrate with Cilium's datapath

### 3. Performance Optimizations
- Enable socket-level load balancing
- Configure DSR (Direct Server Return) mode
- Enable bandwidth manager
- Optimize for single-queue NICs
- Configure CPU affinity for network processing

### 4. Network Security
- Implement CiliumNetworkPolicies for Solana validator
- Allow only required ports (8000-8002 UDP, 8899-8900 TCP)
- Restrict RPC access to Jupiter API pods only
- Enable DNS policies for external connections
- Configure DDoS protection rules

### 5. Observability
- Deploy Hubble for flow visibility
- Enable Hubble metrics for Prometheus
- Configure flow export for analysis
- Set up network debugging capabilities

## Implementation Steps

1. **Pre-flight Checks**
   ```bash
   # Verify eBPF support
   mount | grep bpf
   ls /sys/fs/bpf
   
   # Check kernel version
   uname -r  # Should be 5.x or higher
   ```

2. **Deploy Cilium**
   - Create optimized Helm values file
   - Install using Helm with proper version
   - Wait for all components to be ready
   - Verify kube-proxy replacement

3. **Implement XDP Program**
   - Write XDP code for traffic classification
   - Compile to BPF bytecode
   - Load on network interfaces
   - Verify program attachment

4. **Configure Network Policies**
   - Create policies for Solana validator
   - Apply policies for Jupiter API
   - Test policy enforcement
   - Monitor dropped connections

5. **Performance Validation**
   - Test UDP throughput for gossip
   - Measure RPC latency
   - Verify bandwidth capabilities
   - Check CPU usage of eBPF programs

## Configuration Examples

### Cilium Helm Values
```yaml
kubeProxyReplacement: "true"
bpf:
  masquerade: false
  hostRouting: true
loadBalancer:
  mode: "dsr"
  acceleration:
    enabled: true
```

### XDP Program Structure
```c
SEC("xdp/solana_traffic_prio")
int solana_traffic_prio(struct xdp_md *ctx) {
    // Parse packet headers
    // Check for Solana ports
    // Apply rate limiting
    // Return XDP_PASS or XDP_DROP
}
```

### Network Policy Example
```yaml
apiVersion: cilium.io/v2
kind: CiliumNetworkPolicy
spec:
  endpointSelector:
    matchLabels:
      app: solana-validator
  ingress:
  - fromEndpoints:
    - {}
    toPorts:
    - ports:
      - port: "8000-8002"
        protocol: UDP
```

## Testing Requirements

1. **Connectivity Tests**
   - Run Cilium connectivity test suite
   - Verify pod-to-pod communication
   - Test service resolution
   - Validate external connectivity

2. **Performance Benchmarks**
   - UDP packet rate: >50k pps
   - Service latency: <1ms P99
   - Bandwidth: >20Gbps capability
   - CPU overhead: <10% for eBPF

3. **Security Validation**
   - Verify network policies block unauthorized traffic
   - Test DDoS protection triggers
   - Validate audit logs capture violations
   - Confirm no iptables rules present

4. **XDP Verification**
   - Check program loaded on interfaces
   - Monitor packet counters
   - Verify priority queuing works
   - Test rate limiting effectiveness

## Expected Outputs

1. **Running Components**
   - cilium-agent on all nodes
   - cilium-operator replicas
   - hubble-relay service
   - hubble-ui accessible

2. **Configuration State**
   - No kube-proxy pods
   - eBPF programs loaded
   - Network policies active
   - XDP attached to interfaces

3. **Metrics Available**
   - Cilium metrics in Prometheus format
   - Hubble flow metrics
   - XDP packet counters
   - Bandwidth usage stats

## Success Criteria

- [ ] All Cilium pods healthy and running
- [ ] kube-proxy successfully replaced
- [ ] XDP program processing packets
- [ ] Network policies enforced
- [ ] Hubble showing traffic flows
- [ ] Performance benchmarks met
- [ ] No iptables rules present
- [ ] SR-IOV ready for future use

## Important Considerations

1. **Talos Compatibility**: BPF masquerade must be disabled
2. **Performance**: Every optimization impacts latency
3. **Security**: Start restrictive, open as needed
4. **Debugging**: Keep Hubble data for troubleshooting
5. **Upgrades**: Plan for Cilium version updates

## Error Handling

Common issues and solutions:

1. **eBPF program fails to load**
   - Check kernel config for BPF support
   - Verify memory limits aren't hit
   - Review program for unsupported helpers

2. **Pods cannot communicate**
   - Check Cilium agent logs
   - Verify IPAM has addresses
   - Test without network policies first

3. **Performance degradation**
   - Monitor eBPF program CPU usage
   - Check for packet drops
   - Verify NIC offloads enabled

4. **XDP attachment fails**
   - Confirm driver supports XDP
   - Try generic XDP mode first
   - Check interface is up

Remember: Cilium with eBPF is powerful but complex. Test thoroughly at each step and maintain rollback capability.