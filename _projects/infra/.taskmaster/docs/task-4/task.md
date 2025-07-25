# Task 4: Implement Cilium CNI with eBPF Acceleration

## Overview

This task focuses on deploying Cilium as the Container Network Interface (CNI) for the Kubernetes cluster, with specific optimizations for Solana validator workloads. Cilium's eBPF-based datapath provides superior performance, reduced latency, and advanced network security features essential for high-frequency blockchain operations.

## Objectives

- Deploy Cilium CNI with eBPF optimizations enabled
- Replace kube-proxy for reduced latency and improved performance
- Implement XDP programs for Solana traffic prioritization
- Configure application-level rate limiting for DDoS protection
- Enable SR-IOV support for future hardware acceleration
- Set up Hubble for comprehensive network observability
- Apply network policies for security hardening

## Architecture Context

According to the architecture document, Cilium CNI with eBPF acceleration is critical for:

- **Ultra-low Latency**: Direct kernel datapath bypassing iptables
- **Traffic Prioritization**: XDP programs for Solana gossip traffic (UDP 8000-8002)
- **DDoS Protection**: Application-aware rate limiting at kernel level
- **Performance**: Native load balancing without kube-proxy overhead
- **Observability**: Deep network visibility with Hubble

## Implementation Details

### 1. Pre-deployment Preparation

Verify cluster readiness for Cilium:

```bash
#!/bin/bash
# pre-cilium-check.sh

# Check kernel version supports eBPF
kubectl get nodes -o wide
# Kernel should be 5.x or higher

# Verify no existing CNI
kubectl get pods -n kube-system | grep -E "calico|flannel|weave"
# Should return empty

# Check for existing network policies
kubectl get networkpolicies -A
# Note any existing policies for migration

# Verify eBPF support on nodes
for node in $(kubectl get nodes -o name | cut -d/ -f2); do
  echo "Checking $node..."
  kubectl debug node/$node -it --image=ubuntu -- \
    bash -c "mount | grep -E 'bpf|cgroup2'"
done
```

### 2. Cilium Helm Values Configuration

Create comprehensive Helm values for production deployment:

```yaml
# cilium-values.yaml
# Cilium v1.16.x configuration for Solana validator

# Core Configuration
kubeProxyReplacement: "true"
k8sServiceHost: "10.0.1.10"  # Control plane VIP
k8sServicePort: "6443"

# eBPF Configuration
bpf:
  # Disable masquerade for Talos compatibility
  masquerade: false
  # Enable native routing for performance
  hostRouting: true
  # Prefilter for enhanced performance
  preallocateMaps: true
  # Enable bandwidth manager
  bandwidthManager:
    enabled: true
  # Bypass iptables for maximum performance
  bypassIPTables: true

# Load Balancer Configuration
loadBalancer:
  # Use eBPF for load balancing
  mode: "dsr"
  # Enable Maglev for consistent hashing
  algorithm: "maglev"
  # Native acceleration
  acceleration:
    enabled: true

# IPAM Configuration
ipam:
  mode: "kubernetes"
  operator:
    clusterPoolIPv4PodCIDRList: 
      - "10.244.0.0/16"

# Hubble Observability
hubble:
  enabled: true
  relay:
    enabled: true
  ui:
    enabled: true
    ingress:
      enabled: false
  metrics:
    enableOpenMetrics: true
    enabled:
      - dns
      - drop
      - tcp
      - flow
      - port-distribution
      - icmp
      - httpV2:exemplars=true;labelsContext=source_ip,source_namespace,source_workload,destination_ip,destination_namespace,destination_workload,traffic_direction

# Enable IPv4 and disable IPv6
ipv4:
  enabled: true
ipv6:
  enabled: false

# Node Port Configuration
nodePort:
  enabled: true
  bindProtection: true
  enableHealthCheck: true

# Host Services
hostServices:
  enabled: true

# External IPs
externalIPs:
  enabled: true

# Host Port
hostPort:
  enabled: true

# Encryption (optional, impacts performance)
encryption:
  enabled: false

# Operator Configuration
operator:
  replicas: 2
  prometheus:
    enabled: true
    serviceMonitor:
      enabled: true
  resources:
    requests:
      cpu: 100m
      memory: 128Mi
    limits:
      cpu: 1000m
      memory: 1Gi

# Agent Configuration
agent:
  prometheus:
    enabled: true
    serviceMonitor:
      enabled: true
  resources:
    requests:
      cpu: 500m
      memory: 512Mi
    limits:
      cpu: 2000m
      memory: 2Gi

# MTU Configuration for Jumbo Frames
mtu: 8950  # Account for encapsulation overhead

# Enable bandwidth manager
bandwidthManager:
  enabled: true

# Socket-level load balancing
socketLB:
  enabled: true

# Enable local redirect policy
localRedirectPolicy: true

# Advanced Features
egressGateway:
  enabled: false  # Not needed for single-region
wellKnownIdentities:
  enabled: true
policyEnforcementMode: "default"

# Debug and Monitoring
debug:
  enabled: false
monitor:
  aggregation: medium

# Cluster Mesh (disabled for single cluster)
clustermesh:
  enable: false
```

### 3. Deploy Cilium

Install Cilium using Helm:

```bash
#!/bin/bash
# deploy-cilium.sh

# Add Cilium Helm repository
helm repo add cilium https://helm.cilium.io/
helm repo update

# Create namespace
kubectl create namespace cilium-system || true

# Install Cilium
helm upgrade --install cilium cilium/cilium \
  --version 1.16.5 \
  --namespace kube-system \
  --values cilium-values.yaml \
  --wait

# Wait for Cilium to be ready
echo "Waiting for Cilium pods to be ready..."
kubectl -n kube-system wait --for=condition=ready pod -l k8s-app=cilium --timeout=300s

# Verify installation
cilium status --wait
```

### 4. XDP Program for Solana Traffic

Implement XDP program for traffic prioritization:

```c
// solana-xdp.c
#include <linux/bpf.h>
#include <linux/if_ether.h>
#include <linux/ip.h>
#include <linux/ipv6.h>
#include <linux/udp.h>
#include <linux/tcp.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_endian.h>

#define SOLANA_GOSSIP_PORT_START 8000
#define SOLANA_GOSSIP_PORT_END   8002
#define SOLANA_RPC_PORT          8899
#define SOLANA_RPC_WEBSOCKET     8900

// Map for rate limiting
struct {
    __uint(type, BPF_MAP_TYPE_LRU_HASH);
    __type(key, __u32);    // Source IP
    __type(value, __u64);  // Packet count
    __uint(max_entries, 100000);
} rate_limit_map SEC(".maps");

// Map for priority marking
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __type(key, __u16);    // Port number
    __type(value, __u8);   // Priority level
    __uint(max_entries, 10);
} priority_map SEC(".maps");

static __always_inline int parse_ip_header(struct xdp_md *ctx,
                                          struct ethhdr **eth,
                                          struct iphdr **ip) {
    void *data = (void *)(long)ctx->data;
    void *data_end = (void *)(long)ctx->data_end;
    
    *eth = data;
    if ((void *)(*eth + 1) > data_end)
        return -1;
        
    if ((*eth)->h_proto != bpf_htons(ETH_P_IP))
        return -1;
        
    *ip = (void *)(*eth + 1);
    if ((void *)(*ip + 1) > data_end)
        return -1;
        
    return 0;
}

SEC("xdp/solana_traffic_prio")
int solana_traffic_prio(struct xdp_md *ctx) {
    struct ethhdr *eth;
    struct iphdr *ip;
    
    if (parse_ip_header(ctx, &eth, &ip) < 0)
        return XDP_PASS;
    
    void *data_end = (void *)(long)ctx->data_end;
    
    // Handle UDP traffic
    if (ip->protocol == IPPROTO_UDP) {
        struct udphdr *udp = (void *)ip + (ip->ihl * 4);
        if ((void *)(udp + 1) > data_end)
            return XDP_PASS;
            
        __u16 dest_port = bpf_ntohs(udp->dest);
        
        // Prioritize Solana gossip traffic
        if (dest_port >= SOLANA_GOSSIP_PORT_START && 
            dest_port <= SOLANA_GOSSIP_PORT_END) {
            
            // Rate limiting check
            __u32 src_ip = ip->saddr;
            __u64 *packet_count = bpf_map_lookup_elem(&rate_limit_map, &src_ip);
            
            if (packet_count) {
                __sync_fetch_and_add(packet_count, 1);
                // Rate limit: 10000 packets per entry lifetime
                if (*packet_count > 10000) {
                    return XDP_DROP;
                }
            } else {
                __u64 initial_count = 1;
                bpf_map_update_elem(&rate_limit_map, &src_ip, 
                                   &initial_count, BPF_ANY);
            }
            
            // Mark packet for priority handling
            // This would integrate with TC for actual QoS
            return XDP_PASS;
        }
    }
    
    // Handle TCP traffic (RPC)
    if (ip->protocol == IPPROTO_TCP) {
        struct tcphdr *tcp = (void *)ip + (ip->ihl * 4);
        if ((void *)(tcp + 1) > data_end)
            return XDP_PASS;
            
        __u16 dest_port = bpf_ntohs(tcp->dest);
        
        // Handle RPC traffic
        if (dest_port == SOLANA_RPC_PORT || 
            dest_port == SOLANA_RPC_WEBSOCKET) {
            // Apply different rate limiting for RPC
            return XDP_PASS;
        }
    }
    
    return XDP_PASS;
}

char _license[] SEC("license") = "GPL";
```

Compile and load the XDP program:

```bash
#!/bin/bash
# compile-load-xdp.sh

# Install dependencies
apt-get update
apt-get install -y clang llvm libelf-dev gcc-multilib

# Compile XDP program
clang -O2 -g -target bpf -c solana-xdp.c -o solana-xdp.o

# Create ConfigMap for XDP program
kubectl create configmap solana-xdp-program \
  --from-file=solana-xdp.o \
  -n kube-system

# Deploy XDP loader DaemonSet
cat <<EOF | kubectl apply -f -
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: solana-xdp-loader
  namespace: kube-system
spec:
  selector:
    matchLabels:
      name: solana-xdp-loader
  template:
    metadata:
      labels:
        name: solana-xdp-loader
    spec:
      hostNetwork: true
      hostPID: true
      tolerations:
      - operator: Exists
      containers:
      - name: xdp-loader
        image: cilium/cilium:v1.16.5
        command:
        - /bin/bash
        - -c
        - |
          # Load XDP program on main interface
          IFACE=\$(ip route | grep default | awk '{print \$5}' | head -1)
          ip link set dev \$IFACE xdpgeneric obj /xdp/solana-xdp.o sec xdp/solana_traffic_prio
          
          # Keep container running
          sleep infinity
        securityContext:
          privileged: true
        volumeMounts:
        - name: xdp-program
          mountPath: /xdp
        - name: bpf-maps
          mountPath: /sys/fs/bpf
          mountPropagation: Bidirectional
      volumes:
      - name: xdp-program
        configMap:
          name: solana-xdp-program
      - name: bpf-maps
        hostPath:
          path: /sys/fs/bpf
          type: DirectoryOrCreate
EOF
```

### 5. Network Policies

Apply Solana-specific network policies:

```yaml
# solana-network-policies.yaml
---
apiVersion: cilium.io/v2
kind: CiliumNetworkPolicy
metadata:
  name: solana-validator-policy
  namespace: default
spec:
  endpointSelector:
    matchLabels:
      app: solana-validator
  ingress:
  # Allow gossip traffic
  - fromEndpoints:
    - {}
    toPorts:
    - ports:
      - port: "8000"
        protocol: UDP
      - port: "8001"
        protocol: UDP
      - port: "8002"
        protocol: UDP
  
  # Allow RPC from Jupiter API only
  - fromEndpoints:
    - matchLabels:
        app: jupiter-api
    toPorts:
    - ports:
      - port: "8899"
        protocol: TCP
      - port: "8900"
        protocol: TCP
  
  # Allow metrics scraping
  - fromEndpoints:
    - matchLabels:
        app: prometheus
    toPorts:
    - ports:
      - port: "9090"
        protocol: TCP
  
  egress:
  # Allow all outbound (Solana needs to connect to many peers)
  - toEndpoints:
    - {}
  - toFQDNs:
    - matchPattern: "*"
  - toCIDR:
    - 0.0.0.0/0

---
apiVersion: cilium.io/v2
kind: CiliumNetworkPolicy
metadata:
  name: jupiter-api-policy
  namespace: default
spec:
  endpointSelector:
    matchLabels:
      app: jupiter-api
  ingress:
  # Allow external traffic
  - fromCIDR:
    - 0.0.0.0/0
    toPorts:
    - ports:
      - port: "80"
        protocol: TCP
      - port: "443"
        protocol: TCP
  
  egress:
  # Allow connection to Solana validator
  - toEndpoints:
    - matchLabels:
        app: solana-validator
    toPorts:
    - ports:
      - port: "8899"
        protocol: TCP
      - port: "8900"
        protocol: TCP
  
  # Allow external API calls
  - toFQDNs:
    - matchName: "api.mainnet-beta.solana.com"
    - matchPattern: "*.jup.ag"
```

### 6. Configure Hubble UI

Deploy Hubble UI for network observability:

```bash
# Enable Hubble UI
kubectl apply -f - <<EOF
apiVersion: v1
kind: Service
metadata:
  name: hubble-ui
  namespace: kube-system
spec:
  type: NodePort
  selector:
    k8s-app: hubble-ui
  ports:
  - port: 80
    targetPort: 8081
    nodePort: 31234
EOF

# Create port-forward for local access
kubectl port-forward -n kube-system service/hubble-ui 8080:80 &

echo "Hubble UI available at http://localhost:8080"
```

### 7. Performance Validation

Create comprehensive performance tests:

```bash
#!/bin/bash
# cilium-performance-test.sh

echo "=== Cilium Performance Validation ==="

# Test 1: Measure kube-proxy replacement performance
echo "Testing service resolution performance..."
kubectl run test-client --image=nicolaka/netshoot --rm -it -- \
  bash -c "for i in {1..1000}; do curl -s kubernetes.default >/dev/null; done"

# Test 2: UDP performance for gossip
echo "Testing UDP performance..."
kubectl run udp-server --image=subfuzion/netcat --rm -it -- \
  nc -u -l 8001 &

kubectl run udp-client --image=subfuzion/netcat --rm -it -- \
  bash -c "yes | head -n 1000000 | nc -u udp-server 8001"

# Test 3: Network policy enforcement
echo "Testing network policy enforcement..."
# This should fail due to policy
kubectl run blocked-client --image=nicolaka/netshoot --rm -it -- \
  curl -m 5 solana-validator:8899 || echo "Correctly blocked"

# Test 4: XDP program verification
echo "Verifying XDP program loaded..."
kubectl exec -n kube-system ds/solana-xdp-loader -- \
  ip link show | grep xdp

# Test 5: Bandwidth measurement
echo "Testing network bandwidth..."
kubectl apply -f - <<EOF
apiVersion: v1
kind: Pod
metadata:
  name: iperf-server
spec:
  containers:
  - name: iperf
    image: networkstatic/iperf3
    args: ["-s"]
  nodeSelector:
    kubernetes.io/hostname: solana-validator
EOF

sleep 10

kubectl run iperf-client --rm -it --image=networkstatic/iperf3 -- \
  iperf3 -c iperf-server -t 30 -P 8

kubectl delete pod iperf-server
```

## Testing Strategy

### 1. Functional Testing

```bash
# Verify Cilium status
cilium status

# Check connectivity
cilium connectivity test

# Verify eBPF programs loaded
cilium bpf list

# Check Hubble status
hubble status
```

### 2. Performance Benchmarks

```bash
# Latency test
kubectl run latency-test --rm -it --image=nicolaka/netshoot -- \
  bash -c "for i in {1..100}; do 
    time curl -s kubernetes.default:443 >/dev/null 2>&1
  done | grep real | awk '{sum+=\$2} END {print sum/NR}'"

# Throughput test
# Should achieve >20Gbps between nodes
```

### 3. Security Validation

```bash
# Test network policy enforcement
# Create test pods and verify connectivity restrictions
```

## Dependencies

- Task 3: Kubernetes cluster must be deployed and healthy
- Kernel must support eBPF (5.x+)
- Nodes must have sufficient CPU for eBPF processing

## Success Criteria

1. Cilium pods running on all nodes
2. kube-proxy successfully replaced
3. XDP program loaded and functional
4. Network policies enforced correctly
5. Hubble UI accessible and showing flows
6. UDP performance >50k packets/sec for gossip
7. Service latency <1ms P99
8. Bandwidth tests show >20Gbps capability

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| eBPF compatibility issues | High | Test thoroughly, have fallback to iptables mode |
| Performance regression | Medium | Comprehensive benchmarking before production |
| XDP program bugs | Medium | Extensive testing, gradual rollout |
| Network policy misconfig | High | Test policies in staging first |

## Resources and References

- [Cilium Documentation](https://docs.cilium.io/en/stable/)
- [eBPF Programming Guide](https://ebpf.io/)
- [XDP Tutorial](https://github.com/xdp-project/xdp-tutorial)
- [Cilium Performance Tuning](https://docs.cilium.io/en/stable/operations/performance/)
- Architecture Document: Section 6 - Cilium CNI with eBPF

## Timeline

Estimated Duration: 2 days

- Day 1: Deploy Cilium, configure eBPF features, implement XDP
- Day 2: Network policies, performance testing, observability setup

## Next Steps

Once this task is complete:
- Task 5: Deploy Solana validator with network optimizations
- Configure advanced Cilium features as needed
- Set up network monitoring dashboards