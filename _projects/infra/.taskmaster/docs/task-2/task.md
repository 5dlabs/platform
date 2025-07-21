# Task 2: Provision Cherry Servers Infrastructure with Terraform

## Overview

This task implements a cost-effective, staged approach to provisioning bare metal infrastructure on Cherry Servers for a high-performance Solana validator deployment. The infrastructure includes control plane nodes and a dedicated Solana validator node with optimized storage configuration, following Infrastructure as Code principles with comprehensive validation stages before committing to the >$5,000/month production infrastructure.

## Objectives

- Develop reusable Terraform modules for Cherry Servers infrastructure
- Implement staged validation approach to minimize financial risk
- Provision 3 control plane nodes and 1 high-performance validator node
- Configure optimized storage layout with separate NVMe volumes
- Enable 25Gbps networking with SR-IOV support
- Establish remote management and monitoring capabilities
- Document and test disaster recovery procedures

## Architecture Context

According to the architecture document, this infrastructure forms the foundation for achieving near-bare-metal Solana validator performance. Key architectural decisions include:

- **Single Region Deployment**: EU-East-1 for cost efficiency (65% savings vs multi-region)
- **Vertical Scaling**: One powerful validator node vs distributed approach
- **Storage Separation**: Dedicated NVMe volumes for ledger, accounts, and snapshots
- **Memory Configuration**: Starting with 512GB DDR5 ECC, scalable to 1.5TB based on performance testing

## Staged Implementation Approach

### Stage 1: Local Development ($0/month)

Validate Terraform configurations without any infrastructure costs:

```hcl
# terraform/environments/local/main.tf
terraform {
  backend "local" {
    path = "terraform.tfstate"
  }
}

# Mock provider for local validation
provider "cherryservers" {
  auth_token = "mock-token-for-validation"
  endpoint   = "http://localhost:8080/mock-api"
}

# Validation scripts
resource "null_resource" "validate_config" {
  provisioner "local-exec" {
    command = <<-EOT
      # Validate network topology
      python3 scripts/validate_network.py
      
      # Check security compliance
      tfsec . --minimum-severity HIGH
      
      # Estimate costs
      infracost breakdown --path . --format json > cost-estimate.json
      
      # Validate against Cherry Servers quotas
      python3 scripts/check_quotas.py
    EOT
  }
}
```

### Stage 2: VM Testing (<$100/month)

Test on local virtualization infrastructure:

```yaml
# vagrant/Vagrantfile
Vagrant.configure("2") do |config|
  # Control plane nodes
  (1..3).each do |i|
    config.vm.define "control-#{i}" do |control|
      control.vm.box = "generic/ubuntu2204"
      control.vm.provider "libvirt" do |v|
        v.memory = 8192
        v.cpus = 4
        v.machine_virtual_size = 100
      end
    end
  end

  # Validator node (scaled down)
  config.vm.define "validator" do |validator|
    validator.vm.box = "generic/ubuntu2204"
    validator.vm.provider "libvirt" do |v|
      v.memory = 32768
      v.cpus = 16
      v.numa_nodes = [
        {:cpus => "0-7", :memory => "16384"},
        {:cpus => "8-15", :memory => "16384"}
      ]
      # Test storage configuration
      v.storage :file, :size => '100G', :device => 'vdb' # ledger
      v.storage :file, :size => '50G', :device => 'vdc'  # accounts
      v.storage :file, :size => '20G', :device => 'vdd'  # snapshots
    end
  end
end
```

### Stage 3: Cloud Staging (<$500/month)

Deploy equivalent infrastructure on AWS/GCP:

```hcl
# terraform/environments/staging/main.tf
module "staging_infrastructure" {
  source = "../../modules/cloud_staging"
  
  provider = "aws" # or "gcp"
  region   = "eu-central-1"
  
  control_plane_config = {
    instance_type = "t3.2xlarge"
    count         = 3
  }
  
  validator_config = {
    instance_type = "m6a.24xlarge" # 96 vCPUs, 384 GB RAM
    storage = {
      ledger = {
        type = "gp3"
        size = 1000
        iops = 16000
      }
      accounts = {
        type = "gp3"
        size = 500
        iops = 16000
      }
      snapshots = {
        type = "gp3"
        size = 200
        iops = 3000
      }
    }
  }
}
```

### Stage 4: Production Deployment (>$5,000/month)

Only proceed after all validation stages pass:

## Terraform Module Implementation

### Main Module Structure

```
terraform/
├── modules/
│   ├── cherry_servers/
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   ├── outputs.tf
│   │   └── versions.tf
│   ├── networking/
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   └── outputs.tf
│   └── storage/
│       ├── main.tf
│       ├── variables.tf
│       └── outputs.tf
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

### Cherry Servers Provider Configuration

```hcl
# modules/cherry_servers/versions.tf
terraform {
  required_version = ">= 1.5.0"
  
  required_providers {
    cherryservers = {
      source  = "cherryservers/cherryservers"
      version = "~> 1.0"
    }
  }
}

# modules/cherry_servers/main.tf
provider "cherryservers" {
  auth_token = var.cherry_servers_token
}

# Data source for available plans
data "cherryservers_plans" "available" {
  region = var.region
}

# Data source for available images
data "cherryservers_images" "talos" {
  filter {
    name   = "name"
    values = ["Talos OS Custom Solana"]
  }
}
```

### Control Plane Nodes

```hcl
# modules/cherry_servers/control_plane.tf
resource "cherryservers_server" "control_plane" {
  count = var.control_plane_count

  hostname = "${var.cluster_name}-control-${count.index + 1}"
  region   = var.region
  plan_id  = data.cherryservers_plans.available.plans[
    index(data.cherryservers_plans.available.plans[*].name, var.control_plane_plan)
  ].id
  
  image_id = data.cherryservers_images.talos.images[0].id
  
  ssh_keys = var.ssh_key_ids
  
  user_data = templatefile("${path.module}/templates/control_plane_init.yaml", {
    node_index    = count.index
    cluster_name  = var.cluster_name
    talos_version = var.talos_version
  })
  
  tags = merge(var.common_tags, {
    Name = "${var.cluster_name}-control-${count.index + 1}"
    Role = "control-plane"
    Index = count.index
  })
}

# Floating IPs for control plane
resource "cherryservers_ip" "control_plane" {
  count = var.control_plane_count
  
  region    = var.region
  type      = "floating"
  
  tags = {
    Name = "${var.cluster_name}-control-${count.index + 1}-ip"
  }
}

# Attach floating IPs
resource "cherryservers_server_ip_attachment" "control_plane" {
  count = var.control_plane_count
  
  server_id = cherryservers_server.control_plane[count.index].id
  ip_id     = cherryservers_ip.control_plane[count.index].id
}
```

### Solana Validator Node

```hcl
# modules/cherry_servers/validator.tf
resource "cherryservers_server" "validator" {
  hostname = "${var.cluster_name}-validator"
  region   = var.region
  
  # High-performance plan (AMD EPYC 9454P/9654)
  plan_id = data.cherryservers_plans.available.plans[
    index(data.cherryservers_plans.available.plans[*].name, var.validator_plan)
  ].id
  
  image_id = data.cherryservers_images.talos.images[0].id
  
  ssh_keys = var.ssh_key_ids
  
  user_data = templatefile("${path.module}/templates/validator_init.yaml", {
    cluster_name  = var.cluster_name
    numa_config   = var.numa_configuration
    huge_pages    = var.huge_pages_count
  })
  
  tags = merge(var.common_tags, {
    Name = "${var.cluster_name}-validator"
    Role = "solana-validator"
    Type = "production"
  })
}

# Additional storage volumes
resource "cherryservers_storage" "ledger" {
  size        = var.ledger_storage_size # 15TB
  description = "${var.cluster_name} Solana ledger storage"
  region      = var.region
  type        = "nvme-performance"
  
  tags = {
    Name = "${var.cluster_name}-ledger"
    Purpose = "solana-ledger"
  }
}

resource "cherryservers_storage" "accounts" {
  size        = var.accounts_storage_size # 6TB
  description = "${var.cluster_name} Solana accounts storage"
  region      = var.region
  type        = "nvme-performance"
  
  tags = {
    Name = "${var.cluster_name}-accounts"
    Purpose = "solana-accounts"
  }
}

resource "cherryservers_storage" "snapshots" {
  size        = var.snapshots_storage_size # 2TB
  description = "${var.cluster_name} Solana snapshots storage"
  region      = var.region
  type        = "nvme-standard"
  
  tags = {
    Name = "${var.cluster_name}-snapshots"
    Purpose = "solana-snapshots"
  }
}

# Attach storage to validator
resource "cherryservers_storage_attachment" "ledger" {
  storage_id = cherryservers_storage.ledger.id
  server_id  = cherryservers_server.validator.id
  device     = "/dev/nvme1n1"
}

resource "cherryservers_storage_attachment" "accounts" {
  storage_id = cherryservers_storage.accounts.id
  server_id  = cherryservers_server.validator.id
  device     = "/dev/nvme2n1"
}

resource "cherryservers_storage_attachment" "snapshots" {
  storage_id = cherryservers_storage.snapshots.id
  server_id  = cherryservers_server.validator.id
  device     = "/dev/nvme3n1"
}
```

### Network Configuration

```hcl
# modules/networking/main.tf
resource "cherryservers_network" "primary" {
  name   = "${var.cluster_name}-primary"
  region = var.region
  vlan   = var.vlan_id
  
  tags = {
    Name = "${var.cluster_name}-primary-network"
  }
}

# Configure advanced networking
resource "null_resource" "network_optimization" {
  for_each = cherryservers_server.all_nodes
  
  connection {
    type     = "ssh"
    host     = each.value.primary_ip
    user     = "root"
    private_key = file(var.ssh_private_key_path)
  }
  
  provisioner "remote-exec" {
    inline = [
      # Enable jumbo frames
      "ip link set dev eth0 mtu 9000",
      
      # Enable SR-IOV if available
      "if [ -f /sys/class/net/eth0/device/sriov_numvfs ]; then",
      "  echo 8 > /sys/class/net/eth0/device/sriov_numvfs",
      "fi",
      
      # Optimize network stack
      "sysctl -w net.core.rmem_max=268435456",
      "sysctl -w net.core.wmem_max=268435456",
      "sysctl -w net.ipv4.tcp_congestion_control=bbr"
    ]
  }
}
```

### Remote Management Configuration

```hcl
# modules/cherry_servers/ipmi.tf
resource "cherryservers_ipmi" "all_nodes" {
  for_each = merge(
    { for idx, srv in cherryservers_server.control_plane : 
      "control-${idx}" => srv.id },
    { "validator" = cherryservers_server.validator.id }
  )
  
  server_id = each.value
  enabled   = true
  
  tags = {
    Name = "${var.cluster_name}-${each.key}-ipmi"
  }
}

output "ipmi_access" {
  value = {
    for name, ipmi in cherryservers_ipmi.all_nodes : name => {
      url      = ipmi.url
      username = ipmi.username
      password = ipmi.password
    }
  }
  sensitive = true
}
```

### State Backend Configuration

```hcl
# environments/production/backend.tf
terraform {
  backend "s3" {
    bucket         = "solana-infrastructure-state"
    key            = "cherry-servers/production/terraform.tfstate"
    region         = "eu-central-1"
    encrypt        = true
    dynamodb_table = "terraform-state-lock"
    
    # Enable state locking
    lock_table = "terraform-state-lock"
  }
}

# State encryption
resource "aws_kms_key" "terraform_state" {
  description             = "Terraform state encryption key"
  deletion_window_in_days = 10
  enable_key_rotation     = true
  
  tags = {
    Name = "terraform-state-key"
  }
}
```

## Validation Scripts

### Cost Estimation Script

```python
#!/usr/bin/env python3
# scripts/cost_calculator.py

import json
import sys

CHERRY_SERVERS_PRICING = {
    "AMD EPYC 7302P": 250,  # per month
    "AMD EPYC 9454P": 1500, # per month
    "storage_nvme_performance": 0.15, # per GB
    "storage_nvme_standard": 0.08,    # per GB
    "bandwidth": 0.01, # per GB over 100TB
    "ip_floating": 5  # per IP
}

def calculate_monthly_cost(config):
    """Calculate estimated monthly infrastructure cost"""
    total = 0
    
    # Control plane nodes
    control_plane_cost = (
        CHERRY_SERVERS_PRICING[config['control_plane_plan']] * 
        config['control_plane_count']
    )
    total += control_plane_cost
    
    # Validator node
    validator_cost = CHERRY_SERVERS_PRICING[config['validator_plan']]
    total += validator_cost
    
    # Storage costs
    storage_cost = (
        (config['ledger_size_gb'] * CHERRY_SERVERS_PRICING['storage_nvme_performance']) +
        (config['accounts_size_gb'] * CHERRY_SERVERS_PRICING['storage_nvme_performance']) +
        (config['snapshots_size_gb'] * CHERRY_SERVERS_PRICING['storage_nvme_standard'])
    )
    total += storage_cost
    
    # Floating IPs
    ip_cost = CHERRY_SERVERS_PRICING['ip_floating'] * (config['control_plane_count'] + 1)
    total += ip_cost
    
    # Bandwidth (estimate)
    bandwidth_overage = max(0, config['estimated_bandwidth_tb'] - 100)
    bandwidth_cost = bandwidth_overage * 1024 * CHERRY_SERVERS_PRICING['bandwidth']
    total += bandwidth_cost
    
    return {
        'control_plane': control_plane_cost,
        'validator': validator_cost,
        'storage': storage_cost,
        'ips': ip_cost,
        'bandwidth': bandwidth_cost,
        'total': total
    }

if __name__ == "__main__":
    with open('terraform.tfvars.json') as f:
        config = json.load(f)
    
    costs = calculate_monthly_cost(config)
    print(f"Estimated Monthly Costs:")
    print(f"  Control Plane: ${costs['control_plane']:,.2f}")
    print(f"  Validator Node: ${costs['validator']:,.2f}")
    print(f"  Storage: ${costs['storage']:,.2f}")
    print(f"  IPs: ${costs['ips']:,.2f}")
    print(f"  Bandwidth: ${costs['bandwidth']:,.2f}")
    print(f"  TOTAL: ${costs['total']:,.2f}")
    
    if costs['total'] > 5500:
        print("WARNING: Costs exceed budget!")
        sys.exit(1)
```

### Network Validation Script

```python
#!/usr/bin/env python3
# scripts/validate_network.py

import ipaddress
import json
import sys

def validate_network_topology(config):
    """Validate network configuration and topology"""
    errors = []
    
    # Check subnet configuration
    try:
        network = ipaddress.ip_network(config['vpc_cidr'])
        if network.prefixlen > 24:
            errors.append("VPC CIDR too small for required nodes")
    except ValueError as e:
        errors.append(f"Invalid VPC CIDR: {e}")
    
    # Validate node IPs don't conflict
    node_ips = []
    for node in config['nodes']:
        try:
            ip = ipaddress.ip_address(node['ip'])
            if ip in node_ips:
                errors.append(f"Duplicate IP: {ip}")
            node_ips.append(ip)
        except ValueError as e:
            errors.append(f"Invalid IP for {node['name']}: {e}")
    
    # Check MTU settings
    if config.get('mtu', 1500) != 9000:
        errors.append("Jumbo frames (MTU 9000) not configured")
    
    # Validate SR-IOV settings
    if not config.get('sr_iov_enabled', False):
        errors.append("SR-IOV not enabled for validator node")
    
    return errors

if __name__ == "__main__":
    with open('network_config.json') as f:
        config = json.load(f)
    
    errors = validate_network_topology(config)
    if errors:
        print("Network validation failed:")
        for error in errors:
            print(f"  - {error}")
        sys.exit(1)
    else:
        print("Network configuration valid")
```

## Testing Strategy

### Performance Benchmarks

```bash
#!/bin/bash
# test_infrastructure.sh

echo "=== Infrastructure Performance Tests ==="

# Test storage performance
test_storage() {
    local device=$1
    local name=$2
    
    echo "Testing $name storage ($device)..."
    fio --name=test \
        --filename=$device \
        --ioengine=libaio \
        --direct=1 \
        --rw=randrw \
        --bs=4k \
        --numjobs=32 \
        --iodepth=64 \
        --size=10G \
        --runtime=60 \
        --output-format=json > ${name}_benchmark.json
}

# Test network performance
test_network() {
    echo "Testing network performance..."
    iperf3 -c $PEER_IP -t 60 -P 8 -J > network_benchmark.json
    
    # Test SR-IOV if available
    if [ -d /sys/class/net/eth0/device/virtfn0 ]; then
        echo "Testing SR-IOV performance..."
        iperf3 -c $PEER_IP -t 60 -P 8 -B $VF_IP -J > sriov_benchmark.json
    fi
}

# Test NUMA configuration
test_numa() {
    echo "Testing NUMA configuration..."
    numactl --hardware > numa_topology.txt
    
    # Test cross-NUMA vs local memory access
    numactl --cpunodebind=0 --membind=0 \
        sysbench memory --memory-total-size=10G run > numa_local.txt
    
    numactl --cpunodebind=0 --membind=1 \
        sysbench memory --memory-total-size=10G run > numa_cross.txt
}

# Run all tests
test_storage /dev/nvme0n1 "ledger"
test_storage /dev/nvme1n1 "accounts"
test_storage /dev/nvme2n1 "snapshots"
test_network
test_numa

echo "Performance tests complete. Check *_benchmark.json files for results."
```

## Disaster Recovery

### Backup Configuration

```hcl
# modules/backup/main.tf
resource "cherryservers_backup" "validator_config" {
  server_id = cherryservers_server.validator.id
  enabled   = true
  
  schedule {
    frequency = "daily"
    retention = 7
    time      = "03:00"
  }
  
  paths = [
    "/etc/talos",
    "/var/lib/solana/config"
  ]
  
  tags = {
    Name = "${var.cluster_name}-validator-config-backup"
  }
}

# Snapshot scheduling for data volumes
resource "null_resource" "snapshot_scheduler" {
  provisioner "local-exec" {
    command = <<-EOT
      # Create snapshot schedule via API
      curl -X POST https://api.cherryservers.com/v1/snapshots/schedules \
        -H "Authorization: Bearer $CHERRY_SERVERS_TOKEN" \
        -H "Content-Type: application/json" \
        -d '{
          "storage_id": "${cherryservers_storage.ledger.id}",
          "frequency": "every_6_hours",
          "retention": 4,
          "name": "ledger-snapshots"
        }'
    EOT
  }
}
```

## Documentation

### Deployment Guide

```markdown
# Cherry Servers Infrastructure Deployment Guide

## Prerequisites

1. Cherry Servers account with API token
2. Terraform >= 1.5.0
3. Python 3.8+ for validation scripts
4. SSH key pair for server access

## Deployment Steps

### 1. Local Validation

```bash
cd terraform/environments/local
terraform init
terraform plan -out=plan.tfplan

# Run validation scripts
python3 ../../scripts/cost_calculator.py
python3 ../../scripts/validate_network.py

# Security scan
tfsec .
```

### 2. VM Testing (Optional but Recommended)

```bash
cd ../vm_testing
vagrant up
terraform apply -var-file=vm.tfvars
```

### 3. Cloud Staging

```bash
cd ../cloud_staging
terraform apply -var-file=staging.tfvars

# Run performance tests
ssh ubuntu@$VALIDATOR_IP < ../../scripts/test_infrastructure.sh
```

### 4. Production Deployment

Only proceed if all previous stages pass:

```bash
cd ../production

# Final review
terraform plan -var-file=production.tfvars -out=prod.tfplan

# Apply with approval
terraform apply prod.tfplan
```

## Post-Deployment

1. Verify all nodes are accessible
2. Check IPMI access for all servers
3. Validate storage is properly attached
4. Run performance benchmarks
5. Configure monitoring and alerts
```

## Dependencies

- Task 1: Custom Talos OS image (for node provisioning)
- No other task dependencies

## Success Criteria

1. All validation stages complete without errors
2. Infrastructure costs within budget (<$5,500/month)
3. All nodes provisioned with correct specifications
4. Storage volumes attached and accessible
5. Network performance meets requirements (>20Gbps)
6. IPMI remote management functional
7. Terraform state properly managed and encrypted
8. Documentation complete and validated

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| High infrastructure costs | Critical | Staged validation approach, cost monitoring |
| Resource unavailability | High | Check quotas before deployment, have fallback regions |
| Network performance issues | Medium | SR-IOV enablement, jumbo frames configuration |
| Storage performance bottlenecks | Medium | Separate NVMe volumes, performance testing |
| Configuration drift | Low | Infrastructure as Code, state management |

## Timeline

Estimated Duration: 2 weeks

- Days 1-2: Module development and local validation
- Days 3-4: VM testing environment
- Days 5-7: Cloud staging deployment and testing
- Days 8-9: Production readiness review
- Days 10-11: Production deployment
- Days 12-14: Post-deployment validation and documentation

## Next Steps

After successful infrastructure provisioning:
- Task 3: Deploy Talos OS Kubernetes Cluster
- Task 4: Implement Cilium CNI
- Task 5: Deploy Solana Validator