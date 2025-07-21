# Task 2: Provision Cherry Servers Infrastructure with Terraform - AI Agent Prompt

You are tasked with developing Terraform modules to provision bare metal infrastructure on Cherry Servers for a high-performance Solana validator deployment. This is a critical task that requires careful cost management through a staged validation approach before committing to expensive production infrastructure.

## Context

You are implementing infrastructure for a production Solana validator that requires:
- High-performance bare metal servers with specific hardware requirements
- Optimized storage configuration with separate NVMe volumes
- Advanced networking with 25Gbps and SR-IOV support
- Single region deployment (EU-East-1) for cost efficiency
- Staged validation to minimize financial risk (>$5,000/month production cost)

## Objective

Create comprehensive Terraform modules that provision Cherry Servers infrastructure following a staged approach:
1. Local validation ($0 cost)
2. VM testing (<$100/month)
3. Cloud staging (<$500/month)
4. Production deployment (>$5,000/month)

Only proceed to each stage after the previous one validates successfully.

## Requirements

### Infrastructure Components

1. **Control Plane Nodes (3x)**
   - AMD EPYC 7302P or equivalent
   - 64GB RAM
   - 1TB NVMe storage
   - Standard networking

2. **Solana Validator Node (1x)**
   - AMD EPYC 9454P or 9654 (64 cores, 128 threads)
   - 512GB DDR5 ECC RAM (upgradeable to 1.5TB)
   - Storage configuration:
     - 15TB NVMe Gen5 for ledger (ext4)
     - 6TB NVMe Gen5 for accounts (ext4)
     - 2TB NVMe Gen4 for snapshots (btrfs)
   - 25Gbps networking with SR-IOV support

3. **Network Configuration**
   - Jumbo frames (MTU 9000)
   - SR-IOV with 8 virtual functions
   - DDoS protection enabled
   - Floating IPs for all nodes

4. **Remote Management**
   - IPMI access for all servers
   - Automated backup configuration
   - Monitoring integration points

### Terraform Module Structure

Create modular, reusable Terraform code:

```
terraform/
├── modules/
│   ├── cherry_servers/
│   ├── networking/
│   └── storage/
├── environments/
│   ├── local/
│   ├── vm_testing/
│   ├── cloud_staging/
│   └── production/
└── scripts/
    ├── validate_network.py
    ├── check_quotas.py
    └── cost_calculator.py
```

## Implementation Steps

### Stage 1: Local Development (Immediate)

1. **Create Cherry Servers Provider Module**
   ```hcl
   # Configure provider with mock endpoint for testing
   provider "cherryservers" {
     auth_token = var.auth_token
     endpoint   = var.api_endpoint # Allow override for mocking
   }
   ```

2. **Develop Validation Scripts**
   - Cost calculator to estimate monthly expenses
   - Network topology validator
   - Security compliance checker
   - Quota verification script

3. **Implement Pre-commit Hooks**
   - Terraform fmt and validate
   - tfsec security scanning
   - Cost threshold checks

### Stage 2: VM Testing

1. **Create VM Test Environment**
   - Use Vagrant or similar for local VMs
   - Simulate Cherry Servers API responses
   - Test NUMA configuration
   - Validate storage layout

2. **Test Resource Templates**
   - Verify Terraform resource creation
   - Test user data scripts
   - Validate network configuration

### Stage 3: Cloud Staging

1. **Deploy to AWS/GCP**
   - Use equivalent instance types
   - Test complete deployment pipeline
   - Benchmark performance
   - Validate disaster recovery

2. **Performance Testing**
   - Storage: >300K IOPS ledger, >500K IOPS accounts
   - Network: >20Gbps throughput
   - Latency: <0.5ms between nodes

### Stage 4: Production Deployment

1. **Final Validation Checklist**
   - [ ] All previous stages passed
   - [ ] Cost estimates confirmed (<$5,500/month)
   - [ ] Cherry Servers quotas available
   - [ ] Security review completed
   - [ ] Team training done
   - [ ] Runbooks prepared

2. **Phased Rollout**
   - Deploy control plane nodes first
   - Validate cluster formation
   - Deploy validator node
   - Attach storage volumes
   - Configure networking

## Code Examples

### Main Terraform Module

```hcl
module "solana_infrastructure" {
  source = "./modules/cherry_servers"
  
  region         = "EU-East-1"
  cluster_name   = var.cluster_name
  
  control_plane_count = 3
  control_plane_plan  = "e3-medium-x86"
  
  validator_plan = "e3-max-x86" # AMD EPYC 9454P
  validator_memory = "512GB"    # Start with 512GB
  
  storage_config = {
    ledger = {
      size = 15360  # 15TB
      type = "nvme-performance"
      fs   = "ext4"
    }
    accounts = {
      size = 6144   # 6TB
      type = "nvme-performance"
      fs   = "ext4"
    }
    snapshots = {
      size = 2048   # 2TB
      type = "nvme-standard"
      fs   = "btrfs"
    }
  }
  
  network_config = {
    bandwidth = "25Gbps"
    sr_iov_enabled = true
    vf_count = 8
    mtu = 9000
  }
  
  tags = {
    Environment = var.environment
    Project     = "solana-validator"
    ManagedBy   = "terraform"
  }
}
```

### Validation Script Example

```python
def validate_deployment_readiness(stage):
    """Validate readiness for next deployment stage"""
    checks = {
        'local': [
            check_terraform_syntax,
            check_cost_estimate,
            check_security_compliance
        ],
        'vm_testing': [
            check_vm_deployment,
            check_resource_templates,
            check_numa_config
        ],
        'cloud_staging': [
            check_performance_benchmarks,
            check_failover_procedures,
            check_cost_tracking
        ],
        'production': [
            check_api_access,
            check_quotas,
            check_approvals
        ]
    }
    
    for check in checks[stage]:
        result = check()
        if not result.passed:
            raise ValidationError(f"Check failed: {result.message}")
    
    return True
```

## Testing Requirements

### For Each Stage

1. **Local Development**
   - Terraform plan runs without errors
   - Cost estimates are within budget
   - Security scans pass
   - Network topology is valid

2. **VM Testing**
   - All resources create successfully
   - Storage layout is correct
   - Network configuration applies
   - NUMA settings work

3. **Cloud Staging**
   - Performance meets targets
   - Costs align with estimates
   - Failover works correctly
   - Monitoring is functional

4. **Production**
   - All infrastructure provisions correctly
   - Performance benchmarks pass
   - Remote management works
   - Disaster recovery tested

## Expected Outputs

1. **Terraform Modules**
   - Reusable Cherry Servers modules
   - Network configuration module
   - Storage management module
   - Backup configuration module

2. **Validation Scripts**
   - Cost calculator
   - Network validator
   - Security scanner
   - Quota checker

3. **Documentation**
   - Deployment guide
   - Module documentation
   - Runbooks for common operations
   - Disaster recovery procedures

4. **Test Results**
   - Performance benchmarks from each stage
   - Cost analysis reports
   - Security scan results
   - Validation checklists

## Success Criteria

- [ ] All 4 validation stages complete successfully
- [ ] Monthly costs confirmed under $5,500
- [ ] Infrastructure provisions without errors
- [ ] All performance targets met
- [ ] Remote management functional
- [ ] State management configured
- [ ] Documentation complete and tested
- [ ] Team trained on operations

## Important Considerations

1. **Cost Management**: This infrastructure costs >$5,000/month. The staged approach is critical to avoid expensive mistakes.

2. **State Management**: Implement remote state with locking from the beginning to prevent conflicts.

3. **Security**: Never commit credentials. Use environment variables or secure secret management.

4. **Idempotency**: Ensure all Terraform code is idempotent and can be run multiple times safely.

5. **Rollback Plan**: Document how to destroy infrastructure quickly if issues arise.

## Error Handling

For common issues:

1. **API Rate Limits**: Implement exponential backoff
2. **Resource Unavailability**: Check quotas before applying
3. **Network Conflicts**: Validate IP ranges don't overlap
4. **State Corruption**: Regular state backups, use locking

Remember: Each validation stage is a critical gate. Do not proceed to the next stage until all checks pass. The financial implications of mistakes at the production stage are significant.