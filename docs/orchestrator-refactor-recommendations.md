# Orchestrator Refactor Recommendations

This document outlines recommended refactoring improvements for the Orchestrator codebase, based on our analysis of current patterns and architectural decisions.

## Overview

The current orchestrator architecture is **fundamentally sound** and follows Kubernetes best practices. These recommendations focus on **incremental improvements** rather than major architectural changes.

---

## ✅ **Completed Improvements**

### **1. Magic Number Elimination** ✅ DONE
- **Issue**: Hard-coded `task_id == 999999` throughout the codebase
- **Solution**: Added constants and helper function
- **Benefit**: Clear intent, single source of truth, better maintainability

```rust
// Before
if tr.spec.task_id == 999999 {

// After
const DOCS_GENERATION_TASK_ID: u32 = 999999;
fn is_docs_generation(tr: &TaskRun) -> bool {
    tr.spec.task_id == DOCS_GENERATION_TASK_ID
}
if is_docs_generation(&tr) {
```

---

## 🎯 **Recommended Improvements**

### **Priority 1: High Impact, Low Risk**

#### **1.1 Settings Generation Refactoring**
**Effort**: 2-3 hours | **Risk**: Low | **Impact**: High

**Current Issue**: The `generate_claude_settings()` function is doing too many things:
- Job type detection
- Permission building
- Environment variable construction
- Model selection
- Hook configuration

**Recommended Solution**:
```rust
// Break down into focused functions
fn generate_claude_settings(tr: &TaskRun, config: &ControllerConfig) -> Result<String> {
    let job_type = determine_job_type(tr);
    let permissions = build_permissions(tr, job_type);
    let environment = build_environment(tr, config, job_type);
    let model = select_model(tr, job_type);
    let hooks = build_hooks(tr, job_type);

    let settings = ClaudeSettings {
        permissions,
        env: environment,
        model,
        cleanup_period_days: get_cleanup_period(job_type),
        include_co_authored_by: true,
        hooks,
    };

    serde_json::to_string_pretty(&settings).map_err(Error::SerializationError)
}

fn build_permissions(tr: &TaskRun, job_type: JobType) -> PermissionSettings {
    match job_type {
        JobType::DocsGeneration => build_docs_permissions(),
        JobType::Implementation => build_implementation_permissions(tr),
    }
}

fn build_environment(tr: &TaskRun, config: &ControllerConfig, job_type: JobType) -> EnvironmentSettings {
    let mut env = base_environment_settings();

    if config.telemetry.enabled {
        env.extend(telemetry_settings(config));
    }

    match job_type {
        JobType::DocsGeneration => env.extend(docs_environment_settings()),
        JobType::Implementation => env.extend(implementation_environment_settings(tr)),
    }

    env
}
```

**Benefits**:
- Single responsibility functions
- Easier testing of individual components
- Better error handling and debugging
- Cleaner code organization

#### **1.2 Job Type Enum Introduction**
**Effort**: 3-4 hours | **Risk**: Medium | **Impact**: High

**Current Issue**: Using magic number to differentiate job types

**Recommended Solution**:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum JobType {
    #[serde(rename = "docs_generation")]
    DocsGeneration {
        target_tasks: Option<Vec<u32>>,
        force_overwrite: bool,
    },
    #[serde(rename = "task_implementation")]
    TaskImplementation {
        task_id: u32,
        retry_config: Option<RetryConfig>,
    },
}

pub struct TaskRunSpec {
    pub job_type: JobType,
    pub service_name: String,
    pub agent_name: String,
    pub model: String,
    pub context_version: u32,
    pub markdown_files: Vec<MarkdownFile>,
    pub agent_tools: Vec<AgentTool>,
    pub repository: Option<RepositorySpec>,
}

impl JobType {
    pub fn is_docs_generation(&self) -> bool {
        matches!(self, JobType::DocsGeneration { .. })
    }

    pub fn cleanup_period_days(&self) -> u32 {
        match self {
            JobType::DocsGeneration { .. } => 3,
            JobType::TaskImplementation { .. } => 7,
        }
    }

    pub fn default_model(&self) -> &'static str {
        match self {
            JobType::DocsGeneration { .. } => "claude-opus-4-20250514",
            JobType::TaskImplementation { .. } => "sonnet",
        }
    }
}
```

**Migration Strategy**:
1. Add JobType enum alongside existing task_id field
2. Update API handlers to populate both fields
3. Update controller to use JobType when available, fallback to task_id
4. Deprecate task_id field in favor of job_type
5. Remove task_id field in next major version

**Benefits**:
- Type-safe job differentiation
- Eliminates magic numbers entirely
- Supports future job types without code changes
- Better API documentation and validation
- Enables job-type-specific configuration

#### **1.3 Configuration Structs**
**Effort**: 1-2 hours | **Risk**: Low | **Impact**: Medium

**Current Issue**: Settings generation uses ad-hoc JSON construction

**Recommended Solution**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeSettings {
    pub permissions: PermissionSettings,
    pub env: BTreeMap<String, String>,
    pub model: String,
    #[serde(rename = "cleanupPeriodDays")]
    pub cleanup_period_days: u32,
    #[serde(rename = "includeCoAuthoredBy")]
    pub include_co_authored_by: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<HookSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSettings {
    pub allow: Vec<String>,
    pub deny: Vec<String>,
    #[serde(rename = "defaultMode")]
    pub default_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookSettings {
    #[serde(rename = "onStop")]
    pub on_stop: String,
}
```

**Benefits**:
- Type safety for settings generation
- Better IDE support and autocomplete
- Compile-time validation of settings structure
- Easier testing with known types

### **Priority 2: Medium Impact, Low Risk**

#### **2.1 Template Organization**
**Effort**: 1-2 hours | **Risk**: Low | **Impact**: Medium

**Current Issue**: Multiple template constants at module level

**Recommended Solution**:
```rust
pub struct Templates {
    prep_job: &'static str,
    docs_prep_job: &'static str,
    main_container: &'static str,
    docs_container: &'static str,
    docs_hook: &'static str,
}

impl Templates {
    pub const fn new() -> Self {
        Self {
            prep_job: include_str!("../../templates/prep-job.sh.hbs"),
            docs_prep_job: include_str!("../../templates/docs-generation-prep-job.sh.hbs"),
            main_container: include_str!("../../templates/main-container.sh.hbs"),
            docs_container: include_str!("../../templates/docs-generation-container.sh.hbs"),
            docs_hook: include_str!("../../templates/stop-hook-docs-pr.sh.hbs"),
        }
    }

    pub fn get_prep_template(&self, job_type: &JobType) -> &'static str {
        match job_type {
            JobType::DocsGeneration { .. } => self.docs_prep_job,
            JobType::TaskImplementation { .. } => self.prep_job,
        }
    }

    pub fn get_container_template(&self, job_type: &JobType) -> &'static str {
        match job_type {
            JobType::DocsGeneration { .. } => self.docs_container,
            JobType::TaskImplementation { .. } => self.main_container,
        }
    }
}
```

#### **2.2 Error Handling Improvements**
**Effort**: 2-3 hours | **Risk**: Low | **Impact**: Medium

**Current Issue**: Generic error types and inconsistent error handling

**Recommended Solution**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum ControllerError {
    #[error("Kubernetes API error: {0}")]
    KubeError(#[from] kube::Error),

    #[error("Template rendering failed: {template} - {source}")]
    TemplateError { template: String, source: handlebars::RenderError },

    #[error("Invalid task configuration: {0}")]
    InvalidConfig(String),

    #[error("Settings generation failed: {0}")]
    SettingsError(String),

    #[error("Resource not found: {resource_type} '{name}'")]
    ResourceNotFound { resource_type: String, name: String },

    #[error("Serialization failed: {0}")]
    SerializationError(#[from] serde_json::Error),
}

// Context-aware error creation
impl ControllerError {
    pub fn template_error(template: &str, source: handlebars::RenderError) -> Self {
        Self::TemplateError {
            template: template.to_string(),
            source,
        }
    }

    pub fn resource_not_found(resource_type: &str, name: &str) -> Self {
        Self::ResourceNotFound {
            resource_type: resource_type.to_string(),
            name: name.to_string(),
        }
    }
}
```

### **Priority 3: Future Considerations**

#### **3.1 Separate CRDs (Future)**
**Effort**: 6-8 hours | **Risk**: Medium | **Impact**: High

**When to Consider**: If docs generation becomes significantly more complex

**Potential Solution**:
```rust
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(group = "orchestrator.io", version = "v1", kind = "DocsGenerationRun")]
pub struct DocsGenerationRunSpec {
    pub repository: RepositorySpec,
    pub working_directory: String,
    pub target_tasks: Option<Vec<u32>>,
    pub model: String,
    pub force: bool,
    pub output_format: DocsOutputFormat,
}

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(group = "orchestrator.io", version = "v1", kind = "TaskImplementationRun")]
pub struct TaskImplementationRunSpec {
    pub task_id: u32,
    pub service_name: String,
    pub agent_name: String,
    pub model: String,
    pub markdown_files: Vec<MarkdownFile>,
    pub agent_tools: Vec<AgentTool>,
    pub repository: Option<RepositorySpec>,
    pub retry_config: Option<RetryConfig>,
}
```

**Benefits**:
- Perfect separation of concerns
- Type-safe fields specific to each job type
- Independent lifecycle management
- Cleaner API surface

**Drawbacks**:
- More complexity
- Duplicate controller logic
- Migration complexity

#### **3.2 Plugin Architecture (Future)**
**Effort**: 12-16 hours | **Risk**: High | **Impact**: High

**When to Consider**: When adding 3+ job types or external integrations

**Concept**:
```rust
pub trait JobHandler {
    type Spec: DeserializeOwned + Serialize + Clone;
    type Status: DeserializeOwned + Serialize + Clone;

    fn job_type(&self) -> &'static str;
    fn create_resources(&self, spec: &Self::Spec) -> Result<Vec<Resource>>;
    fn check_status(&self, spec: &Self::Spec) -> Result<Self::Status>;
    fn cleanup(&self, spec: &Self::Spec) -> Result<()>;
}

pub struct DocsGenerationHandler;
impl JobHandler for DocsGenerationHandler {
    type Spec = DocsGenerationSpec;
    type Status = DocsGenerationStatus;
    // ... implementation
}
```

---

## 🚀 **Implementation Roadmap**

### **Phase 1: Foundation (Week 1)**
1. ✅ Magic number elimination (DONE)
2. Settings generation refactoring
3. Configuration structs

### **Phase 2: Type Safety (Week 2)**
1. Job type enum introduction
2. Template organization
3. Error handling improvements

### **Phase 3: Future Enhancements (Month 2+)**
1. Evaluate need for separate CRDs
2. Consider plugin architecture if complexity grows

---

## 📊 **Risk Assessment**

### **Low Risk Improvements**
- Settings generation refactoring
- Configuration structs
- Template organization
- Error handling improvements

### **Medium Risk Changes**
- Job type enum (requires API changes)
- Separate CRDs (new complexity)

### **High Risk Changes**
- Plugin architecture (major refactoring)

---

## 🎯 **Success Metrics**

### **Code Quality**
- Reduced cyclomatic complexity in `generate_claude_settings()`
- Elimination of magic numbers
- Improved test coverage

### **Maintainability**
- Faster onboarding for new developers
- Easier addition of new job types
- Clearer error messages and debugging

### **Type Safety**
- Compile-time validation of settings
- Reduced runtime configuration errors
- Better IDE support

---

## 📝 **Next Steps**

1. **Start with Priority 1 items** - High impact, low risk
2. **Implement incrementally** - One improvement at a time
3. **Test thoroughly** - Each change should have comprehensive tests
4. **Document changes** - Update this document as improvements are completed
5. **Gather feedback** - Review with team after each phase

The current architecture is solid. These improvements will make it even better while maintaining the excellent foundation you've built.