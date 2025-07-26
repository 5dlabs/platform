# Acceptance Criteria: Task 6 - Generate and Save Tool Configuration in Docs Phase

## Overview
This document defines the acceptance criteria for implementing configuration generation and storage functionality in the docs agent, enabling persistent tool configurations for code agents to consume.

## Core Requirements

### 1. Configuration Generation
- [ ] **Complete Schema**: All required fields in ProjectConfig
- [ ] **Tool Selection**: Includes local and remote tools
- [ ] **Metadata Capture**: Git info, timestamps, scores
- [ ] **Analysis Integration**: Uses project analysis results
- [ ] **Version Support**: Schema versioning implemented

### 2. Storage Implementation
- [ ] **ConfigMap Creation**: Creates project-specific ConfigMap
- [ ] **Update Support**: Can update existing configurations
- [ ] **Multiple Formats**: Stores config, analysis, and tools JSON
- [ ] **Human Summary**: Includes readable summary.txt
- [ ] **Proper Labels**: K8s labels and annotations applied

### 3. Integration
- [ ] **Docs Flow**: Integrated into main docs generation
- [ ] **Error Recovery**: Continues with defaults on failure
- [ ] **Logging**: Comprehensive logging throughout
- [ ] **Metrics**: Tracks generation time and confidence
- [ ] **Verification**: Validates saved configuration

## Technical Specifications

### 1. Configuration Schema
```json
{
  "tools": {
    "local": ["filesystem", "git"],
    "remote": ["github", "kubernetes", "postgres"],
    "tool_configs": {
      "filesystem": {
        "readonly": false,
        "allowed_directories": ["/workspace"]
      }
    }
  },
  "generated_at": "2024-01-20T10:30:00Z",
  "docs_run_id": "docs-run-abc123",
  "project_analysis": {
    "has_kubernetes": true,
    "has_database": true,
    "detected_languages": ["go", "python"],
    "project_size": "Medium"
  },
  "version": "1.0.0",
  "metadata": {
    "project_id": "my-project",
    "git_commit": "abc123def",
    "git_branch": "main",
    "total_tools_available": 25,
    "total_tools_recommended": 5,
    "confidence_score": 0.85,
    "generation_duration_ms": 342
  }
}
```

### 2. ConfigMap Structure
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: my-project-project-config
  namespace: orchestrator
  labels:
    app.kubernetes.io/name: project-config
    app.kubernetes.io/instance: my-project
    app.kubernetes.io/component: tool-config
    app.kubernetes.io/managed-by: docs-agent
    generated-by: docs-run-abc123
  annotations:
    generated-at: "2024-01-20T10:30:00Z"
    confidence-score: "0.85"
    git-commit: "abc123def"
data:
  config.json: |  # Complete configuration
  analysis.json: |  # Project analysis only
  tools.json: |  # Tool selection only
  summary.txt: |  # Human-readable summary
```

## Test Cases

### Test Case 1: Configuration Generation
```rust
#[tokio::test]
async fn test_config_generation_complete() {
    let handler = create_test_handler();
    let analysis = ProjectAnalysis {
        has_kubernetes: true,
        has_database: true,
        has_ci_cd: true,
        detected_languages: vec!["go".to_string()],
        project_size: ProjectSize::Medium,
        ..Default::default()
    };
    
    let config = handler.generate_project_config(
        Path::new("/test"),
        "test-project",
        "docs-123",
        &["kubernetes", "postgres", "github"],
        &analysis,
    ).await.unwrap();
    
    // Verify all fields populated
    assert_eq!(config.metadata.project_id, "test-project");
    assert_eq!(config.docs_run_id, "docs-123");
    assert_eq!(config.version, "1.0.0");
    assert!(config.metadata.confidence_score > 0.0);
    assert!(config.metadata.generation_duration_ms > 0);
    assert!(!config.tools.local.is_empty());
    assert!(!config.tools.remote.is_empty());
}
```

### Test Case 2: Confidence Score Calculation
```rust
#[tokio::test]
async fn test_confidence_score_calculation() {
    let handler = create_test_handler();
    
    // Perfect match scenario
    let analysis = ProjectAnalysis {
        has_kubernetes: true,
        has_database: true,
        ..Default::default()
    };
    let tool_config = ProjectToolConfig {
        local: vec![],
        remote: vec!["kubernetes".to_string(), "postgres".to_string()],
        tool_configs: HashMap::new(),
    };
    
    let score = handler.calculate_confidence_score(
        &analysis,
        &tool_config,
        &["kubernetes", "postgres", "other"]
    );
    
    assert!(score > 0.8); // High confidence for good matches
}
```

### Test Case 3: ConfigMap Storage
```rust
#[tokio::test]
async fn test_configmap_storage() {
    let handler = create_test_handler();
    let config = create_test_config();
    
    // Save configuration
    handler.save_project_config(&config).await.unwrap();
    
    // Verify ConfigMap created
    let cm = get_configmap("test-project-project-config").await.unwrap();
    
    // Check all data keys
    assert!(cm.data.unwrap().contains_key("config.json"));
    assert!(cm.data.unwrap().contains_key("analysis.json"));
    assert!(cm.data.unwrap().contains_key("tools.json"));
    assert!(cm.data.unwrap().contains_key("summary.txt"));
    
    // Verify labels
    let labels = cm.metadata.labels.unwrap();
    assert_eq!(labels.get("app.kubernetes.io/name"), Some(&"project-config".to_string()));
}
```

### Test Case 4: Update Existing Configuration
```rust
#[tokio::test]
async fn test_config_update() {
    let handler = create_test_handler();
    let config1 = create_test_config();
    
    // Save initial
    handler.save_project_config(&config1).await.unwrap();
    
    // Update with new config
    let mut config2 = config1.clone();
    config2.tools.remote.push("new-tool".to_string());
    config2.generated_at = Utc::now();
    
    handler.save_project_config(&config2).await.unwrap();
    
    // Verify update
    let cm = get_configmap("test-project-project-config").await.unwrap();
    let saved_json = cm.data.unwrap().get("tools.json").unwrap();
    let saved_tools: ProjectToolConfig = serde_json::from_str(saved_json).unwrap();
    
    assert!(saved_tools.remote.contains(&"new-tool".to_string()));
}
```

### Test Case 5: Tool-Specific Configurations
```rust
#[tokio::test]
async fn test_tool_specific_configs() {
    let handler = create_test_handler();
    let tool_config = ProjectToolConfig {
        local: vec!["filesystem".to_string(), "git".to_string()],
        remote: vec!["kubernetes".to_string()],
        tool_configs: HashMap::new(),
    };
    
    let configs = handler.generate_tool_configs(&tool_config, &analysis);
    
    // Verify filesystem config
    assert!(configs.contains_key("filesystem"));
    let fs_config = &configs["filesystem"];
    assert!(fs_config.get("allowed_directories").is_some());
    
    // Verify git config
    assert!(configs.contains_key("git"));
    let git_config = &configs["git"];
    assert_eq!(git_config.get("allow_push"), Some(&json!(false)));
}
```

### Test Case 6: Error Recovery
```rust
#[tokio::test]
async fn test_storage_retry_logic() {
    let handler = create_test_handler_with_flaky_k8s();
    let config = create_test_config();
    
    // Should retry and eventually succeed
    handler.save_project_config(&config).await.unwrap();
    
    // Verify retry attempts logged
    assert!(logs_contain("Failed to save config (attempt 1/3)"));
    assert!(logs_contain("Successfully saved project configuration"));
}
```

### Test Case 7: Integration Flow
```rust
#[tokio::test]
async fn test_complete_integration() {
    let ctx = DocsContext {
        project_id: "integration-test".to_string(),
        run_id: "docs-456".to_string(),
        project_path: create_test_project_with_files(),
        k8s_client: create_test_client(),
    };
    
    let handler = DocsHandler::new(ctx.k8s_client.clone());
    
    // Run complete flow
    handler.generate_docs(ctx).await.unwrap();
    
    // Verify configuration was saved
    let cm = get_configmap("integration-test-project-config").await.unwrap();
    assert!(cm.data.is_some());
    
    // Verify configuration is valid
    let config_json = cm.data.unwrap().get("config.json").unwrap();
    let config: ProjectConfig = serde_json::from_str(config_json).unwrap();
    assert_eq!(config.docs_run_id, "docs-456");
}
```

## Validation Checklist

### Configuration Validation
- [ ] **Schema Completeness**: All fields properly populated
- [ ] **Tool Validation**: Tools exist in available list
- [ ] **Metadata Accuracy**: Git info, timestamps correct
- [ ] **Version Format**: Follows semver format
- [ ] **Score Range**: Confidence between 0.0 and 1.0

### Storage Validation
- [ ] **ConfigMap Format**: Valid Kubernetes resource
- [ ] **Data Keys**: All expected keys present
- [ ] **JSON Validity**: All JSON fields parse correctly
- [ ] **Label Standards**: Follows K8s conventions
- [ ] **Size Limits**: Under ConfigMap size limit (1MB)

### Integration Validation
- [ ] **Flow Integration**: Works within docs generation
- [ ] **Error Handling**: Graceful degradation
- [ ] **Logging Quality**: Informative log messages
- [ ] **Performance**: Completes in reasonable time
- [ ] **Idempotency**: Can be run multiple times safely

## Performance Criteria

### 1. Generation Performance
- [ ] **Analysis Time**: < 1 second for average project
- [ ] **Config Generation**: < 500ms
- [ ] **Total Time**: < 2 seconds end-to-end

### 2. Storage Performance
- [ ] **Save Time**: < 1 second including retries
- [ ] **Retry Delay**: Exponential backoff implemented
- [ ] **Verification**: < 500ms

### 3. Resource Usage
- [ ] **Memory**: < 100MB for generation
- [ ] **CPU**: Minimal spike during generation
- [ ] **Network**: Single ConfigMap write per project

## Security Requirements

### 1. Data Security
- [ ] **No Secrets**: No sensitive data in ConfigMap
- [ ] **Git Safety**: Only commit hash, not content
- [ ] **Path Safety**: No absolute paths exposed

### 2. Access Control
- [ ] **RBAC**: Respects K8s permissions
- [ ] **Namespace**: Stays within orchestrator namespace
- [ ] **Read/Write**: Only writes own ConfigMaps

## Documentation Requirements

### 1. API Documentation
- [ ] **Public Methods**: All documented with examples
- [ ] **Data Structures**: Schema clearly defined
- [ ] **Error Cases**: All errors documented

### 2. Integration Guide
- [ ] **Setup**: How to enable configuration generation
- [ ] **Customization**: How to modify behavior
- [ ] **Troubleshooting**: Common issues and fixes

### 3. User Guide
- [ ] **ConfigMap Access**: How to view configurations
- [ ] **Understanding Output**: How to interpret fields
- [ ] **Manual Override**: How to modify if needed

## Definition of Done

✅ **Configuration Generation**
- Generates complete configurations
- Includes all metadata
- Calculates confidence scores
- Handles all project types

✅ **Storage Implementation**
- Reliably saves to ConfigMap
- Supports updates
- Includes retry logic
- Verifies saves

✅ **Integration Complete**
- Works in docs flow
- Handles errors gracefully
- Logs appropriately
- Performance acceptable

✅ **Testing Verified**
- All test cases pass
- Edge cases covered
- Integration tested
- Performance validated

✅ **Documentation Delivered**
- Code documented
- Integration guide complete
- User guide ready
- Examples provided

## Sign-off Requirements

- [ ] **Code Review**: Implementation reviewed and approved
- [ ] **Security Review**: No security concerns identified
- [ ] **Performance Test**: Meets performance criteria
- [ ] **Integration Test**: Works with real docs flow
- [ ] **Documentation Review**: All guides complete

## Notes
- Configuration is the bridge between docs and code agents
- Ensure forward compatibility with schema versioning
- Consider ConfigMap size limits for large projects
- Tool-specific configs enable future extensibility