# Task 4: Implement Cilium CNI with eBPF Acceleration - Acceptance Criteria

## Overview

This document defines the acceptance criteria for deploying Cilium CNI with eBPF acceleration optimized for Solana validator network traffic. All criteria must be met before considering the network infrastructure ready for production workloads.

## Functional Acceptance Criteria

### 1. Cilium Deployment Status

- [ ] **All Cilium components operational**
  ```bash
  kubectl -n kube-system get pods -l k8s-app=cilium
  # Expected: All pods Running with 1/1 READY
  
  kubectl -n kube-system get pods -l name=cilium-operator
  # Expected: 2 replicas Running
  
  cilium status
  # Expected output:
  # KVStore: Ok
  # Kubernetes: Ok
  # Cilium: Ok
  # Nodes: 4/4 reachable
  # Cluster Pods: X/X managed by Cilium
  ```

- [ ] **kube-proxy successfully replaced**
  ```bash
  # No kube-proxy pods should exist
  kubectl -n kube-system get pods | grep kube-proxy
  # Expected: No results
  
  # Verify Cilium is handling services
  cilium service list
  # Should show kubernetes.default and other services
  
  # Check iptables is not used
  iptables -t nat -L | grep -i kube
  # Expected: No kube-related rules
  ```

- [ ] **eBPF programs loaded**
  ```bash
  cilium bpf list
  # Expected: Multiple eBPF programs listed including:
  # - bpf_lxc
  # - bpf_sock
  # - bpf_lb
  # - bpf_host
  
  # Verify BPF filesystem
  mount | grep bpf
  # Expected: bpffs mounted on /sys/fs/bpf
  ```

### 2. XDP Program Deployment

- [ ] **XDP program compiled and loaded**
  ```bash
  # Check XDP attachment on interfaces
  kubectl exec -n kube-system ds/solana-xdp-loader -- \
    ip link show | grep xdp
  # Expected: "xdpgeneric" or "xdpdrv" shown on main interface
  
  # Verify BPF maps created
  kubectl exec -n kube-system ds/solana-xdp-loader -- \
    ls /sys/fs/bpf/ | grep rate_limit
  # Expected: rate_limit_map present
  ```

- [ ] **XDP program processing packets**
  ```bash
  # Check packet statistics
  kubectl exec -n kube-system ds/solana-xdp-loader -- \
    bpftool map dump name rate_limit_map
  # Should show entries for active connections
  
  # Monitor XDP statistics
  kubectl exec -n kube-system ds/solana-xdp-loader -- \
    bpftool prog show | grep xdp
  # Should show program ID and statistics
  ```

### 3. Network Configuration

- [ ] **Jumbo frames configured**
  ```bash
  # Check MTU on Cilium interfaces
  kubectl exec -n kube-system ds/cilium -- \
    ip link show | grep -E "cilium_host|cilium_net" | grep mtu
  # Expected: mtu 8950 or similar (accounting for overhead)
  
  # Verify on host interfaces
  kubectl debug node/solana-validator -it --image=nicolaka/netshoot -- \
    ip link show eth0 | grep mtu
  # Expected: mtu 9000
  ```

- [ ] **Native routing enabled**
  ```bash
  cilium config view | grep -E "routing-mode|tunnel"
  # Expected:
  # routing-mode: native
  # tunnel: disabled
  ```

- [ ] **Bandwidth manager active**
  ```bash
  cilium config view | grep bandwidth
  # Expected: enable-bandwidth-manager: true
  
  # Check BPF bandwidth maps
  cilium bpf bandwidth list
  # Should show bandwidth entries
  ```

### 4. Network Policies

- [ ] **Policies applied and enforced**
  ```bash
  kubectl get ciliumnetworkpolicies -A
  # Expected:
  # - solana-validator-policy
  # - jupiter-api-policy
  
  # Verify policy enforcement
  cilium policy get
  # Should show imported policies with endpoints
  ```

- [ ] **Policy enforcement validation**
  ```bash
  # Test allowed connection (should succeed)
  kubectl run test-allowed --rm -it --image=nicolaka/netshoot -- \
    nc -zv -u solana-validator.default.svc.cluster.local 8000
  # Expected: Connection successful
  
  # Test blocked connection (should fail)
  kubectl run test-blocked --rm -it --image=nicolaka/netshoot -- \
    curl -m 5 solana-validator.default.svc.cluster.local:8899
  # Expected: Connection timeout
  ```

### 5. Hubble Observability

- [ ] **Hubble components running**
  ```bash
  kubectl -n kube-system get pods -l k8s-app=hubble-relay
  # Expected: Running
  
  kubectl -n kube-system get pods -l k8s-app=hubble-ui
  # Expected: Running (if UI enabled)
  
  hubble status
  # Expected: Healthcheck successful
  ```

- [ ] **Flow visibility working**
  ```bash
  hubble observe --last 10
  # Should show recent network flows
  
  hubble observe --protocol udp --port 8000-8002
  # Should show Solana gossip traffic when active
  ```

## Performance Acceptance Criteria

### 1. Service Resolution Performance

- [ ] **kube-proxy replacement performance**
  ```bash
  # Measure service resolution latency
  kubectl run perf-test --rm -it --image=nicolaka/netshoot -- bash -c "
    for i in {1..100}; do
      time curl -s kubernetes.default:443/healthz >/dev/null
    done 2>&1 | grep real | awk '{sum+=\$2; count++} END {print sum/count*1000 \"ms average\"}'
  "
  # Expected: <1ms average latency
  ```

### 2. UDP Performance (Gossip Traffic)

- [ ] **High packet rate handling**
  ```bash
  # Deploy UDP test pods
  kubectl apply -f - <<EOF
  apiVersion: v1
  kind: Pod
  metadata:
    name: udp-server
    labels:
      app: udp-test
  spec:
    containers:
    - name: server
      image: subfuzion/netcat
      command: ["nc", "-u", "-l", "-k", "8001"]
  EOF
  
  # Test packet rate
  kubectl run udp-client --rm -it --image=nicolaka/netshoot -- \
    bash -c "timeout 10s hping3 -2 -p 8001 -i u100 --flood udp-server"
  # Expected: >50,000 pps without drops
  ```

### 3. TCP Throughput

- [ ] **Bandwidth capability verification**
  ```bash
  # Deploy iperf server on worker node
  kubectl apply -f - <<EOF
  apiVersion: v1
  kind: Pod
  metadata:
    name: iperf-server
  spec:
    nodeSelector:
      kubernetes.io/hostname: solana-validator
    containers:
    - name: iperf
      image: networkstatic/iperf3
      args: ["-s"]
  EOF
  
  # Run bandwidth test
  kubectl run iperf-client --rm -it --image=networkstatic/iperf3 -- \
    iperf3 -c iperf-server -t 30 -P 8
  # Expected: >20Gbps throughput
  
  kubectl delete pod iperf-server
  ```

### 4. eBPF Overhead

- [ ] **CPU usage acceptable**
  ```bash
  # Monitor eBPF program CPU usage
  kubectl top nodes
  # Note baseline CPU usage
  
  # Generate high traffic load
  # Re-check CPU usage
  # Expected: <10% increase due to eBPF processing
  ```

### 5. Latency Measurements

- [ ] **P99 latency targets**
  ```bash
  # Deploy latency test
  kubectl apply -f - <<EOF
  apiVersion: v1
  kind: Service
  metadata:
    name: echo-service
  spec:
    selector:
      app: echo
    ports:
    - port: 8080
  ---
  apiVersion: v1
  kind: Pod
  metadata:
    name: echo-server
    labels:
      app: echo
  spec:
    containers:
    - name: echo
      image: hashicorp/http-echo
      args: ["-text=hello", "-listen=:8080"]
  EOF
  
  # Measure latency distribution
  kubectl run latency-test --rm -it --image=jordi/ab -- \
    ab -n 10000 -c 10 http://echo-service:8080/
  # Expected P99: <1ms
  
  kubectl delete pod echo-server
  kubectl delete service echo-service
  ```

## Security Acceptance Criteria

### 1. Network Isolation

- [ ] **Default deny policy active**
  ```bash
  # Test pod without policy
  kubectl run isolated-test --image=nginx
  kubectl run test-client --rm -it --image=nicolaka/netshoot -- \
    curl -m 5 isolated-test
  # Should timeout if default deny is active
  kubectl delete pod isolated-test
  ```

- [ ] **Audit logs capturing violations**
  ```bash
  # Check for policy violation logs
  kubectl logs -n kube-system -l k8s-app=cilium | grep -i "denied"
  # Should show denied connections
  ```

### 2. DDoS Protection

- [ ] **Rate limiting functional**
  ```bash
  # Generate high packet rate from single source
  # Monitor XDP drop statistics
  kubectl exec -n kube-system ds/solana-xdp-loader -- \
    bpftool prog show | grep -A5 xdp | grep dropped
  # Should show increasing drop count during attack
  ```

### 3. SR-IOV Readiness

- [ ] **SR-IOV compatible configuration**
  ```bash
  cilium config view | grep devices
  # Should exclude VF devices from management
  
  # Verify on worker node
  kubectl debug node/solana-validator -it --image=nicolaka/netshoot -- \
    ls /sys/class/net/*/device/sriov_*
  # SR-IOV sysfs entries should be accessible
  ```

## Operational Acceptance Criteria

### 1. Monitoring Integration

- [ ] **Prometheus metrics available**
  ```bash
  # Check Cilium metrics
  kubectl exec -n kube-system ds/cilium -- \
    curl -s localhost:9962/metrics | grep cilium_
  # Should return Cilium metrics
  
  # Check Hubble metrics
  kubectl exec -n kube-system deployment/hubble-relay -- \
    curl -s localhost:9965/metrics | grep hubble_
  # Should return Hubble metrics
  ```

- [ ] **ServiceMonitor objects created**
  ```bash
  kubectl get servicemonitor -n kube-system | grep cilium
  # Should show:
  # - cilium-agent
  # - cilium-operator
  # - hubble-relay
  ```

### 2. Troubleshooting Capabilities

- [ ] **Cilium CLI functional**
  ```bash
  # Install Cilium CLI
  cilium version
  cilium status
  cilium connectivity test
  # All commands should work
  ```

- [ ] **BPF maps accessible**
  ```bash
  cilium bpf list
  cilium map list
  cilium endpoint list
  # Should show comprehensive BPF state
  ```

### 3. High Availability

- [ ] **Operator HA configured**
  ```bash
  kubectl -n kube-system get deployment cilium-operator
  # Expected: 2 replicas
  
  # Test failover
  kubectl -n kube-system delete pod -l name=cilium-operator
  # New pods should start, no service disruption
  ```

## Test Suite Execution

```bash
#!/bin/bash
# cilium-acceptance-test.sh

set -e

echo "Running Cilium CNI Acceptance Tests..."

# Function to check and report
check() {
    if eval "$2"; then
        echo "✓ $1"
    else
        echo "✗ $1"
        exit 1
    fi
}

# Component checks
check "Cilium pods running" "kubectl -n kube-system get pods -l k8s-app=cilium --no-headers | grep -v Running | wc -l | grep -q '^0$'"
check "No kube-proxy" "! kubectl -n kube-system get pods | grep -q kube-proxy"
check "XDP loaded" "kubectl exec -n kube-system ds/solana-xdp-loader -- ip link show | grep -q xdp"

# Performance checks
echo "Testing service latency..."
LATENCY=$(kubectl run lat-test --rm -it --image=nicolaka/netshoot --restart=Never -- \
  bash -c "time curl -s kubernetes.default:443/healthz >/dev/null 2>&1" | grep real | awk '{print $2}')
check "Service latency <1ms" "[ '${LATENCY%m*}' -lt 1 ]"

# Network policy checks
check "Network policies exist" "kubectl get ciliumnetworkpolicies -A --no-headers | wc -l | grep -qv '^0$'"

# Hubble checks
check "Hubble relay running" "kubectl -n kube-system get pods -l k8s-app=hubble-relay --no-headers | grep -q Running"

echo "All Cilium acceptance tests passed!"
```

## Definition of Done

The task is considered complete when:

1. All functional acceptance criteria are met
2. Performance benchmarks achieve targets
3. Security policies are enforced correctly
4. Monitoring and observability are operational
5. XDP program is processing Solana traffic
6. Network policies provide proper isolation
7. Documentation is complete and accurate
8. Team is trained on Cilium operations

## Rollback Plan

If critical issues arise:

1. Save current Cilium configuration
2. Document all custom XDP programs
3. Prepare iptables-mode configuration
4. Test rollback in staging first
5. Keep 24-hour traffic logs for analysis

This acceptance criteria ensures Cilium CNI is properly configured for high-performance Solana validator operations with security and observability.