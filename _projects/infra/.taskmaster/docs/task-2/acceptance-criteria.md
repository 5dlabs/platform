# Task 2: Provision Cherry Servers Infrastructure - Acceptance Criteria

## Overview

This document defines comprehensive acceptance criteria for the Cherry Servers infrastructure provisioning task. Given the significant financial commitment (>$5,000/month), each validation stage must be thoroughly completed before proceeding to the next.

## Stage-Gate Acceptance Criteria

### Stage 1: Local Development ($0 cost)

#### Code Quality and Structure

- [ ] **Terraform modules properly structured**
  ```
  terraform/
  ├── modules/
  │   ├── cherry_servers/
  │   │   ├── main.tf
  │   │   ├── variables.tf
  │   │   ├── outputs.tf
  │   │   └── versions.tf (provider requirements)
  │   ├── networking/
  │   └── storage/
  └── environments/
      ├── local/
      ├── vm_testing/
      ├── cloud_staging/
      └── production/
  ```

- [ ] **All Terraform code passes validation**
  ```bash
  terraform fmt -check -recursive .
  terraform init
  terraform validate
  # All commands must succeed
  ```

- [ ] **Security scanning passes**
  ```bash
  tfsec . --minimum-severity HIGH
  # No HIGH or CRITICAL vulnerabilities
  
  checkov -d . --framework terraform
  # No failing checks for security best practices
  ```

#### Cost Validation

- [ ] **Cost estimation script functional**
  ```bash
  python3 scripts/cost_calculator.py
  # Output shows:
  # - Control Plane: ~$750/month (3 nodes)
  # - Validator Node: ~$3,500/month
  # - Storage: ~$800/month
  # - Total: <$5,500/month
  ```

- [ ] **Cost breakdown documented**
  - Per-component costs identified
  - Bandwidth estimates included
  - Growth projections calculated

#### Network Validation

- [ ] **Network topology validated**
  ```bash
  python3 scripts/validate_network.py
  # Checks pass for:
  # - No IP conflicts
  # - Sufficient subnet size
  # - MTU 9000 configured
  # - SR-IOV enabled
  ```

#### Documentation

- [ ] **Module documentation complete**
  - README for each module
  - Variable descriptions
  - Example usage
  - Output descriptions

### Stage 2: VM Testing (<$100/month)

#### Test Environment Setup

- [ ] **VM environment operational**
  ```bash
  vagrant status
  # All VMs running:
  # control-1: running
  # control-2: running
  # control-3: running
  # validator: running
  ```

- [ ] **Mock Cherry Servers API functional**
  ```bash
  curl http://localhost:8080/mock-api/v1/servers
  # Returns mock server list
  ```

#### Resource Provisioning Tests

- [ ] **Terraform apply succeeds in test environment**
  ```bash
  cd terraform/environments/vm_testing
  terraform apply -auto-approve
  # Apply complete! Resources: X added, 0 changed, 0 destroyed
  ```

- [ ] **All resources created with correct specifications**
  - Control plane nodes: 3 instances
  - Validator node: 1 instance with NUMA configuration
  - Storage volumes: 3 separate volumes attached
  - Network configuration: Applied successfully

#### Configuration Validation

- [ ] **NUMA configuration verified**
  ```bash
  vagrant ssh validator -c "numactl --hardware"
  # Shows 2 NUMA nodes with correct CPU/memory distribution
  ```

- [ ] **Storage layout correct**
  ```bash
  vagrant ssh validator -c "lsblk"
  # Shows:
  # vdb - 100G (ledger simulation)
  # vdc - 50G (accounts simulation)
  # vdd - 20G (snapshots simulation)
  ```

### Stage 3: Cloud Staging (<$500/month)

#### Infrastructure Deployment

- [ ] **Staging environment provisioned successfully**
  ```bash
  cd terraform/environments/cloud_staging
  terraform apply -var-file=staging.tfvars
  # All resources created without errors
  ```

- [ ] **Instance specifications match requirements**
  ```bash
  # AWS Example
  aws ec2 describe-instances --instance-ids $VALIDATOR_ID
  # Shows: m6a.24xlarge (96 vCPUs, 384 GB RAM)
  ```

#### Performance Benchmarks

- [ ] **Storage performance meets targets**
  ```bash
  # Ledger volume
  fio --name=ledger-test --filename=/mnt/ledger/test \
      --ioengine=libaio --direct=1 --rw=randrw \
      --bs=4k --numjobs=32 --iodepth=64 --size=10G
  # Results: >300K IOPS
  
  # Accounts volume  
  fio --name=accounts-test --filename=/mnt/accounts/test \
      --ioengine=libaio --direct=1 --rw=randrw \
      --bs=4k --numjobs=32 --iodepth=64 --size=10G
  # Results: >500K IOPS
  ```

- [ ] **Network performance validated**
  ```bash
  iperf3 -c $PEER_IP -t 60 -P 8
  # Results: >20 Gbps throughput
  # Packet loss: <0.01%
  ```

#### Cost Tracking

- [ ] **Actual costs align with estimates**
  ```bash
  # Cloud provider cost report
  aws ce get-cost-and-usage \
    --time-period Start=2024-01-01,End=2024-01-31 \
    --granularity DAILY
  # Daily cost: <$17 (for <$500/month)
  ```

#### Disaster Recovery Testing

- [ ] **Backup procedures functional**
  - Configuration backups created
  - Snapshot schedules working
  - Restore tested successfully

- [ ] **Infrastructure recreation tested**
  ```bash
  terraform destroy -auto-approve
  terraform apply -auto-approve
  # Infrastructure recreated successfully
  # Time to full deployment: <30 minutes
  ```

### Stage 4: Production Deployment (>$5,000/month)

#### Pre-Deployment Checklist

- [ ] **Go/No-Go criteria met**
  - [ ] All previous stages passed
  - [ ] Budget approval documented
  - [ ] Cherry Servers quotas confirmed
  - [ ] API access verified
  - [ ] Team training completed
  - [ ] Runbooks prepared
  - [ ] Rollback plan documented

- [ ] **Cherry Servers API access verified**
  ```bash
  curl -H "Authorization: Bearer $CHERRY_SERVERS_TOKEN" \
    https://api.cherryservers.com/v1/plans?region=EU-East-1
  # Returns available plans including required specs
  ```

#### Production Infrastructure

- [ ] **All nodes provisioned correctly**
  ```bash
  # Via Cherry Servers API or console
  # Control Plane Nodes:
  # - 3x AMD EPYC 7302P, 64GB RAM, 1TB NVMe
  # Validator Node:
  # - 1x AMD EPYC 9454P, 512GB DDR5 ECC, custom storage
  ```

- [ ] **Storage volumes attached and verified**
  ```bash
  ssh root@$VALIDATOR_IP "lsblk"
  # nvme0n1: 15TB (ledger)
  # nvme1n1: 6TB (accounts)
  # nvme2n1: 2TB (snapshots)
  ```

- [ ] **Network configuration applied**
  ```bash
  ssh root@$VALIDATOR_IP "ip link show"
  # eth0: mtu 9000
  
  ssh root@$VALIDATOR_IP "ls /sys/class/net/eth0/device/virtfn*"
  # Shows 8 virtual functions (SR-IOV enabled)
  ```

#### Remote Management

- [ ] **IPMI access functional for all nodes**
  ```bash
  ipmitool -I lanplus -H $IPMI_IP -U $IPMI_USER -P $IPMI_PASS power status
  # Chassis Power is on
  ```

- [ ] **Console access verified**
  - Web console accessible
  - SOL (Serial over LAN) working
  - Power control tested

#### Performance Validation

- [ ] **Production benchmarks meet or exceed staging**
  ```bash
  # Run full benchmark suite
  ./scripts/test_infrastructure.sh
  
  # Minimum requirements:
  # Storage IOPS: Ledger >300K, Accounts >500K
  # Network throughput: >20Gbps
  # Network latency: <0.5ms between nodes
  # NUMA local vs remote: >30% performance difference
  ```

#### Operational Readiness

- [ ] **Monitoring integration points configured**
  - Node metrics exporters installed
  - IPMI monitoring configured
  - Network statistics available
  - Storage metrics accessible

- [ ] **Backup systems operational**
  ```bash
  # Verify backup job created
  curl -H "Authorization: Bearer $TOKEN" \
    https://api.cherryservers.com/v1/backup-jobs
  # Shows configured backup schedules
  ```

- [ ] **Documentation validated**
  - Deployment guide tested by team member
  - Runbooks cover common scenarios
  - Disaster recovery plan validated

## Terraform State Management

- [ ] **Remote state configured and tested**
  ```bash
  # State backend configured
  grep -q "backend \"s3\"" terraform/environments/production/backend.tf
  
  # State locking functional
  terraform plan
  # Another terminal: terraform plan
  # Should show "Error acquiring the state lock"
  ```

- [ ] **State encryption enabled**
  ```bash
  aws s3api get-bucket-encryption \
    --bucket solana-infrastructure-state
  # Shows encryption configuration
  ```

## Cost Control Measures

- [ ] **Cost alerts configured**
  ```bash
  # Monthly budget alert set at $5,000
  # Daily spend alerts if >$200/day
  # Anomaly detection enabled
  ```

- [ ] **Resource tagging complete**
  ```bash
  # All resources tagged with:
  # - Environment
  # - Project
  # - Owner
  # - CostCenter
  ```

## Security Compliance

- [ ] **No credentials in code**
  ```bash
  git grep -i "password\|secret\|token" --exclude="*.md"
  # No results
  ```

- [ ] **Network security validated**
  - Only required ports open
  - DDoS protection enabled
  - Access control lists configured

- [ ] **Audit logging enabled**
  - API calls logged
  - Infrastructure changes tracked
  - Access logs configured

## Acceptance Test Suite

Create and run comprehensive acceptance test:

```bash
#!/bin/bash
# acceptance_test.sh

echo "Running Infrastructure Acceptance Tests..."

# Function to check and report
check() {
    if eval "$2"; then
        echo "✓ $1"
    else
        echo "✗ $1"
        exit 1
    fi
}

# Stage-specific tests
case $STAGE in
    "local")
        check "Terraform valid" "terraform validate"
        check "Costs under budget" "python3 scripts/cost_calculator.py | grep -q 'TOTAL: \$[0-9]*\.[0-9]*' && [ \$(python3 scripts/cost_calculator.py | grep TOTAL | awk -F'$' '{print int(\$2)}') -lt 5500 ]"
        check "Security scan clean" "tfsec . --minimum-severity HIGH --no-error"
        ;;
    "vm_testing")
        check "VMs running" "vagrant status | grep -c running | grep -q 4"
        check "Terraform applies" "terraform apply -auto-approve"
        ;;
    "cloud_staging")
        check "Performance targets met" "./scripts/test_infrastructure.sh && grep -q 'PASS' benchmark_results.txt"
        check "Costs tracking" "[ \$(cat daily_cost.txt) -lt 17 ]"
        ;;
    "production")
        check "All nodes accessible" "for ip in \$NODE_IPS; do ssh root@\$ip hostname || exit 1; done"
        check "Storage attached" "ssh root@\$VALIDATOR_IP 'lsblk | grep -c nvme' | grep -q 3"
        check "IPMI working" "ipmitool -I lanplus -H \$IPMI_IP -U \$IPMI_USER -P \$IPMI_PASS power status"
        ;;
esac

echo "All acceptance tests passed for stage: $STAGE"
```

## Sign-off Requirements

### Technical Sign-off
- [ ] Infrastructure team lead approval
- [ ] Network team validation
- [ ] Security team review

### Financial Sign-off
- [ ] Budget owner approval
- [ ] Cost center allocation confirmed
- [ ] Monthly spend approved

### Operational Sign-off
- [ ] Operations team trained
- [ ] Runbooks reviewed
- [ ] On-call rotation configured

## Definition of Done

The task is considered complete when:

1. All 4 validation stages completed successfully
2. Production infrastructure deployed and validated
3. All performance benchmarks met or exceeded
4. Cost tracking and controls in place
5. Remote management fully functional
6. Documentation complete and validated
7. Team training completed
8. All sign-offs obtained

## Rollback Criteria

If critical issues arise:

1. Terraform destroy plan ready
2. Maximum rollback time: 15 minutes
3. Cost implications documented
4. Communication plan activated

This acceptance criteria ensures safe, validated progression through each stage with clear gates before committing to expensive production infrastructure.