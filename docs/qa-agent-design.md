# QA Agent Design Document

## 🎯 **Overview**

The QA Agent is an autonomous quality assurance system that monitors pull requests, deploys applications to test environments, validates functionality and code quality, and provides automated feedback to development teams. It operates as a feedback loop until both functionality and quality standards are met.

## 🏗️ **Architecture**

### **Core Principles**
- **Pull/Polling Model**: 30-second intervals per repository
- **Service-Based Isolation**: Dedicated namespaces per service (`qa-{service-name}`)
- **Functionality-First**: Deploy and test functionality before code quality
- **Self-Healing**: Automatically fixes deployment issues
- **Human-Gated Merging**: Can approve PRs but never merge (CTO-only)
- **Feedback Loop**: Continuous iteration until both functionality + quality pass

### **Integration Points**
- **Tasks System**: References acceptance criteria via task IDs
- **MCP Quality Server**: Pluggable code quality analysis
- **GitHub API**: PR management, comments, approvals
- **Kubernetes**: Deployment orchestration and namespace management
- **Code Agent**: Triggers remediation work via PR comments

## 📋 **QAAgentRun CRD Specification**

```yaml
apiVersion: agents.platform/v1
kind: QAAgentRun
metadata:
  name: qa-simple-api
  namespace: orchestrator
spec:
  # Repository Configuration
  repository: "5dlabs/simple-api"
  service: "simple-api"           # Creates qa-simple-api namespace
  githubUser: "pm0-5dlabs"
  branches: ["main", "feature/*"]
  pollInterval: "30s"

  # Testing Configuration
  tasksAcceptanceCriteria: "task-123"  # Reference to Tasks system
  deploymentChart: "./helm"            # Path to Helm chart in repo

  # Image Strategy
  imageRegistry: "ghcr.io/5dlabs"
  buildAndPush: true                   # Build fresh image per PR

  # Testing Phases (executed in order)
  phases:
    - "functionality"                  # Deploy + functional tests first
    - "quality"                        # Code quality via MCP server
    - "approval"                       # Auto-approve if both pass

  # Self-Healing & Alerting
  autoFixDeployments: true
  alertCTOOnFailure: true             # Stretch goal: Alert on critical issues
  maxRetryAttempts: 3

  # Actions
  autoApprove: true                   # Approve PRs that pass all phases
  autoMerge: false                    # NEVER merge (human-only gate)

  # State Tracking
  lastCheckedCommit: ""
  lastQArun: ""
  currentPhase: ""
```

## 🔄 **Workflow Sequence**

### **1. Detection Phase**
- Poll GitHub every 30 seconds for:
  - New pull requests on monitored branches
  - New commits on existing pull requests
  - Merge conflicts requiring resolution
- Track state using `lastCheckedCommit` to avoid duplicate processing

### **2. Build & Push Phase**
```bash
# Build application image
docker build -t ${imageRegistry}/${service}:pr-${PR_NUMBER} .
docker push ${imageRegistry}/${service}:pr-${PR_NUMBER}
```

### **3. Deployment Phase**
- Deploy to service-specific namespace: `qa-${service}`
- Use Helm chart from repository: `helm upgrade qa-${service} ./helm`
- Override image tag to use PR-specific build
- **Self-Healing**: If deployment fails, analyze and fix issues automatically

### **4. Functionality Testing Phase**
- Execute acceptance criteria from Tasks system
- Validate endpoints, core functionality, integration points
- **Priority**: Must pass before proceeding to quality phase

### **5. Quality Analysis Phase**
- Call quality MCP server for code review
- Check coding standards, architecture patterns, security issues
- Generate detailed quality report

### **6. Results & Actions Phase**
- **Both Pass**: Auto-approve PR, comment with success details
- **Functionality Fails**: Comment with issues, trigger Code agent to fix
- **Quality Fails**: Comment with quality issues, trigger Code agent to improve
- **Deployment Fails**: Attempt auto-fix, alert CTO if unable to resolve

### **7. Merge Conflict Detection**
- Detect merge conflicts during polling
- Comment: "⚠️ Merge conflict detected. @Code-Agent please resolve conflicts with main branch"
- Trigger Code agent to resolve conflicts and continue

## 🛠️ **Implementation Components**

### **QA Agent Container**
```dockerfile
# Base image with GitHub CLI, Docker, Helm, kubectl
FROM ghcr.io/5dlabs/qa-agent:latest

# QA-specific tools and MCP client
COPY qa-agent.py /app/
COPY quality-checkers/ /app/checkers/
```

### **QA Templates Structure**
```
claude-templates/qa/
├── claude.md.hbs           # QA agent personality and instructions
├── container.sh.hbs        # QA execution script
├── settings.json.hbs       # QA-specific Claude settings
└── hooks/
    ├── pre-deployment.sh.hbs
    ├── post-deployment.sh.hbs
    └── quality-check.sh.hbs
```

### **Controller Logic**
- **QA Controller**: Manages QAAgentRun lifecycle
- **Namespace Management**: Creates/manages `qa-{service}` namespaces
- **State Tracking**: Maintains polling state and retry counts
- **Integration**: Interfaces with GitHub API and Tasks system

## 🔧 **Configuration Management**

### **Quality Standards**
```yaml
qualityStandards:
  level: "strict"                    # strict|standard|minimal
  mcpServer: "rust-quality-server"
  checkers:
    - "security-scan"
    - "code-complexity"
    - "test-coverage"
    - "documentation"
```

### **Deployment Configuration**
```yaml
deployment:
  namespace: "qa-{service}"
  helmChart: "./helm"
  valueOverrides:
    image:
      tag: "pr-{PR_NUMBER}"
    resources:
      limits:
        cpu: "500m"
        memory: "512Mi"
```

## 🚨 **Error Handling & Alerting**

### **Self-Healing Scenarios**
- **Helm deployment failures**: Analyze and fix chart issues
- **Resource constraints**: Adjust resource requests/limits
- **Dependency issues**: Install missing dependencies
- **Configuration errors**: Fix common misconfigurations

### **CTO Alerting (Stretch Goal)**
- **Critical deployment failures** after max retries
- **Security vulnerabilities** found during quality checks
- **System-wide issues** affecting multiple services
- **Code agent unresponsive** to QA feedback

## 📊 **Success Metrics**

### **Functionality Metrics**
- Deployment success rate
- Functional test pass rate
- Time to detect issues

### **Quality Metrics**
- Code quality score trends
- Security vulnerability detection
- Technical debt accumulation

### **Process Metrics**
- Average time from PR to approval
- Merge conflict detection/resolution time
- Human intervention frequency

## 🚀 **Implementation Phases**

### **Phase 1: Core QA Agent**
- [ ] Create QAAgentRun CRD
- [ ] Implement basic polling and PR detection
- [ ] Build deployment automation
- [ ] Basic functionality testing

### **Phase 2: Quality Integration**
- [ ] Integrate MCP quality server
- [ ] Implement quality scoring
- [ ] Add quality-based approval logic

### **Phase 3: Advanced Features**
- [ ] Self-healing deployment fixes
- [ ] CTO alerting system
- [ ] Advanced merge conflict detection
- [ ] Performance and security testing

### **Phase 4: Optimization**
- [ ] Multi-service orchestration
- [ ] Advanced quality metrics
- [ ] Predictive failure detection
- [ ] Integration with CI/CD pipelines

## 🎛️ **Configuration Examples**

### **Simple API Service**
```yaml
apiVersion: agents.platform/v1
kind: QAAgentRun
metadata:
  name: qa-simple-api
spec:
  repository: "5dlabs/simple-api"
  service: "simple-api"
  tasksAcceptanceCriteria: "task-456"
  deploymentChart: "./helm"
  autoApprove: true
```

### **Complex Microservice**
```yaml
apiVersion: agents.platform/v1
kind: QAAgentRun
metadata:
  name: qa-trader-service
spec:
  repository: "5dlabs/trader"
  service: "trader"
  tasksAcceptanceCriteria: "task-789"
  deploymentChart: "./k8s/helm"
  qualityStandards:
    level: "strict"
    mcpServer: "financial-quality-server"
  maxRetryAttempts: 5
  alertCTOOnFailure: true
```

## 🔮 **Future Enhancements**

- **Multi-Environment Testing**: Test across dev, staging, prod-like environments
- **Performance Testing**: Automated load and performance testing
- **Security Scanning**: Integrated vulnerability scanning
- **Compliance Checking**: Automated compliance validation
- **Cross-Service Integration Testing**: Test service interactions
- **Rollback Automation**: Automatic rollback on failure detection