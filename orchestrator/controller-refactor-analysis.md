# TaskRun Controller Refactor Analysis

## Executive Summary

The current TaskRun controller (`orchestrator-core/src/controllers/taskrun.rs`, 2032 lines) is a sophisticated Kubernetes controller that handles both documentation generation and code implementation tasks. We need to refactor it to work with our new `DocsRun` and `CodeRun` CRDs while preserving all critical functionality.

**Key Findings:**
- ✅ **Rich template system** - Very well-designed handlebars templating system
- ✅ **Solid Kubernetes patterns** - Proper finalizers, status management, job orchestration
- ✅ **Comprehensive functionality** - Handles SSH/HTTPS auth, hook scripts, tool configuration
- ⚠️ **Task type detection** - Currently uses magic number (999999) for docs detection
- ⚠️ **Single CRD complexity** - Handles two different workflows in one resource type

---

## 🔍 Detailed Line-by-Line Analysis

### **Core Controller Infrastructure (Lines 1-180)**

#### ✅ **KEEP - Essential Kubernetes Controller Patterns**
```rust
// Lines 45-65: Error handling and types
#[derive(Debug, thiserror::Error)]
pub enum Error { ... }
type Result<T, E = Error> = std::result::Result<T, E>;

// Lines 66-140: PVC Management
async fn ensure_pvc_exists(...) -> Result<()>

// Lines 141-178: Controller setup and configuration loading
pub async fn run_taskrun_controller(...) -> Result<()>
```

**Analysis:**
- Well-structured error types with proper error propagation
- PVC management is essential for implementation tasks (workspace persistence)
- Controller setup follows Kubernetes best practices
- Configuration loading from ConfigMap is flexible and needed

**Action:** Keep all of this, just update to handle both DocsRun and CodeRun

---

### **Main Reconciliation Logic (Lines 179-305)**

#### ✅ **KEEP - Core Orchestration Pattern**
```rust
// Lines 179-239: Main reconcile function
async fn reconcile(tr: Arc<TaskRun>, ctx: Arc<Context>) -> Result<Action>

// Lines 240-304: Create/update logic
async fn reconcile_create_or_update(...)
```

**Analysis:**
- Proper finalizer handling for cleanup
- Status monitoring and updates
- Resource creation orchestration
- 30-second requeue interval for monitoring

**Issues to Fix:**
- Lines 32-43: Hard-coded docs detection using `DOCS_GENERATION_TASK_ID: u32 = 999999`
- Lines 250-254: Special case handling for docs vs implementation

**Action:** Replace task type detection with CRD type checking (`DocsRun` vs `CodeRun`)

---

### **Job Creation and Management (Lines 305-477)**

#### ✅ **KEEP - Sophisticated Job Orchestration**
```rust
// Lines 305-376: Claude job creation
async fn create_claude_job(...)

// Lines 377-435: Job status monitoring
async fn monitor_job_status(...)

// Lines 436-477: Resource cleanup
async fn cleanup_resources(...)
```

**Analysis:**
- Handles job creation, monitoring, and failure recovery
- Proper cleanup when TaskRuns are deleted
- Retry logic for failed jobs
- Status propagation from Jobs to TaskRuns

**Critical Features:**
- Job deadline configuration (2 hours for implementation, 4 hours for docs)
- Automatic job recreation on failure
- Version-based job cleanup (removes older attempts)

**Action:** Keep all functionality, adapt to new CRD types

---

### **Status Management (Lines 477-550)**

#### ✅ **KEEP - Comprehensive Status Tracking**
```rust
// Lines 477-503: Basic status updates
async fn update_status(...)

// Lines 504-550: Detailed status updates
async fn update_status_with_details(...)
```

**Analysis:**
- Preserves attempt counters across updates
- Tracks job names and ConfigMap names for debugging
- RFC3339 timestamps for audit trails
- Proper Kubernetes status subresource usage

**Action:** Keep all, just update field names to match new CRD status structures

---

### **ConfigMap Building (Lines 550-700)**

#### ✅ **KEEP - Sophisticated Template System**
```rust
// Lines 550-650: Main ConfigMap builder
fn build_configmap(tr: &TaskRun, name: &str, config: &ControllerConfig) -> Result<ConfigMap>
```

**Analysis:**
- Generates all necessary files from templates:
  - `CLAUDE.md` (agent memory/context)
  - `settings-local.json` / `claude-settings.json` (tool permissions)
  - `mcp.json` (MCP server configuration)
  - `client-config.json` (tool selection)
  - `coding-guidelines.md` / `github-guidelines.md` (documentation)
  - Hook scripts (`.stop-hook-docs-pr.sh`, `stop-commit.sh`, `early-test.sh`)

**Critical Features:**
- Template-based file generation using Handlebars
- Different file sets for docs vs implementation
- Retry-aware (preserves existing CLAUDE.md on retry attempts)
- Proper ConfigMap key naming (Kubernetes restrictions)

**Action:** Keep entire system, just update task type detection logic

---

### **Kubernetes Job Building (Lines 700-950)**

#### ✅ **KEEP - Advanced Container Orchestration**
```rust
// Lines 700-850: Job specification builder
fn build_claude_job(...) -> Result<Job>

// Lines 850-950: Container startup script generation
fn build_agent_startup_script(...) -> Result<String, Error>
```

**Analysis:**
- Handles both SSH and HTTPS Git authentication
- Dynamic volume mounting (PVC for implementation, emptyDir for docs)
- Proper security contexts and image pull secrets
- Environment variable injection from secrets
- Template-based container startup scripts

**Critical Features:**
- SSH key mounting for Git operations
- GitHub PAT injection from user-specific secrets
- Working directory configuration
- Model override handling
- Tool configuration passing

**Action:** Keep all functionality, very well designed

---

### **Environment Variable Building (Lines 1400-1500)**

#### ✅ **KEEP - Clean Environment Management**
```rust
// Lines 1400-1500: Container environment setup
fn build_env_vars(tr: &TaskRun, config: &ControllerConfig) -> Vec<serde_json::Value>
```

**Analysis:**
- Essential environment variables only (most config via settings.json)
- Proper secret references for API keys and GitHub tokens
- User-specific secret name resolution
- Clean separation between container env and Claude settings

**Action:** Keep as-is, very clean design

---

### **Template Generation System (Lines 950-1800)**

#### ✅ **KEEP - Comprehensive Template Engine**
```rust
// Lines 950-1050: Claude memory generation
fn generate_claude_memory(...)

// Lines 1050-1200: Settings.json generation
fn generate_claude_settings(...)

// Lines 1200-1400: MCP and client config generation
fn generate_mcp_config(...), fn generate_client_config(...)

// Lines 1600-1800: Hook script generation
fn generate_hook_scripts(...)
```

**Analysis:**
- Very sophisticated template system using Handlebars
- Supports different templates for docs vs implementation
- Handles tool configuration (minimal/default/advanced presets)
- Agent tools permission translation
- Dynamic hook script generation

**Critical Templates:**
- `DOCS_CLAUDE_TEMPLATE` / `IMPLEMENTATION_CLAUDE_TEMPLATE` - Agent memory
- `DOCS_SETTINGS_TEMPLATE` / `IMPLEMENTATION_SETTINGS_TEMPLATE` - Tool permissions
- `IMPLEMENTATION_MCP_TEMPLATE` - MCP server configuration
- Hook templates for PR creation and testing

**Action:** Keep entire system - this is very well designed

---

### **Test Suite (Lines 2000+)**

#### ✅ **KEEP - Comprehensive Test Coverage**
```rust
// Lines 2000+: Extensive test suite
mod tests { ... }
```

**Analysis:**
- Tests all major functions individually
- Integration tests for template rendering
- Tool configuration validation
- Error handling verification
- Template conditional logic testing

**Action:** Keep all tests, update to use new CRD types

---

## 🎯 Refactoring Strategy

### **Phase 1: Minimal Adaptation (Recommended)**

#### **Replace Task Type Detection Logic**
```rust
// OLD (Lines 32-43):
const DOCS_GENERATION_TASK_ID: u32 = 999999;
fn is_docs_generation(tr: &TaskRun) -> bool {
    tr.spec.task_id == DOCS_GENERATION_TASK_ID
}

// NEW:
enum TaskType {
    Docs(Arc<DocsRun>),
    Code(Arc<CodeRun>),
}

impl TaskType {
    fn is_docs(&self) -> bool {
        matches!(self, TaskType::Docs(_))
    }
}
```

#### **Update Function Signatures**
```rust
// OLD:
async fn reconcile(tr: Arc<TaskRun>, ctx: Arc<Context>) -> Result<Action>

// NEW:
async fn reconcile_docs(dr: Arc<DocsRun>, ctx: Arc<Context>) -> Result<Action>
async fn reconcile_code(cr: Arc<CodeRun>, ctx: Arc<Context>) -> Result<Action>
async fn reconcile_common(task: TaskType, ctx: Arc<Context>) -> Result<Action>
```

#### **Update Template Data Extraction**
```rust
// Create common trait for extracting template data
trait TaskSpec {
    fn task_id(&self) -> Option<u32>;
    fn service_name(&self) -> &str;
    fn repository(&self) -> Option<&RepositorySpec>;
    fn working_directory(&self) -> Option<&str>;
    fn model(&self) -> &str;
    fn github_user(&self) -> &str;
    // ... other common fields
}

impl TaskSpec for DocsRunSpec { ... }
impl TaskSpec for CodeRunSpec { ... }
```

### **Phase 2: Clean Up and Modernize**

#### **Remove Deprecated Logic**
- ✅ Remove `DOCS_GENERATION_TASK_ID` constant
- ✅ Remove magic number task type detection
- ✅ Update naming from "TaskRun" to "unified controller"

#### **Improve Error Messages**
- ✅ Update error messages to mention DocsRun/CodeRun instead of TaskRun
- ✅ Add better debugging info for which CRD type failed

#### **Update Tests**
- ✅ Convert all tests to use DocsRun/CodeRun
- ✅ Add tests for new unified controller logic
- ✅ Test cross-CRD functionality

---

## 📋 Implementation Checklist

### **Critical Features to Preserve**

#### **Core Orchestration**
- [ ] Kubernetes controller pattern (reconcile loops)
- [ ] Finalizer-based cleanup
- [ ] Status management and propagation
- [ ] Error handling and retry logic
- [ ] Configuration loading from ConfigMap

#### **Resource Management**
- [ ] PVC creation and management (for CodeRun)
- [ ] ConfigMap generation with templates
- [ ] Kubernetes Job creation and monitoring
- [ ] Resource cleanup on deletion

#### **Authentication and Security**
- [ ] SSH key mounting for Git operations
- [ ] GitHub PAT injection from secrets
- [ ] User-specific secret name resolution
- [ ] Proper security contexts

#### **Template System**
- [ ] Handlebars template rendering
- [ ] CLAUDE.md memory file generation
- [ ] settings.json tool permission configuration
- [ ] MCP configuration generation
- [ ] Hook script generation
- [ ] Tool configuration presets (minimal/default/advanced)

#### **Advanced Features**
- [ ] Retry attempt handling
- [ ] Context version management
- [ ] Prompt modification support
- [ ] Working directory configuration
- [ ] Model override handling
- [ ] Git branch management

### **Items to Update**

#### **CRD Integration**
- [ ] Replace TaskRun references with DocsRun/CodeRun
- [ ] Update status field mappings
- [ ] Add missing fields to new CRDs (conditions, configmap_name, etc.)
- [ ] Update API field extraction

#### **Task Type Detection**
- [ ] Remove magic number detection (999999)
- [ ] Implement CRD-based type detection
- [ ] Update template selection logic
- [ ] Update resource naming

#### **Function Signatures**
- [ ] Update all function signatures to accept new CRD types
- [ ] Create common trait or enum for shared functionality
- [ ] Update test suite to use new types

### **Testing Strategy**
- [ ] Run existing test suite against new CRD types
- [ ] Add integration tests for DocsRun/CodeRun workflows
- [ ] Test template rendering with new CRD fields
- [ ] Verify resource cleanup works properly
- [ ] Test status updates and monitoring

---

## 🚀 Recommended Approach

### **Option 1: Gradual Migration (Recommended)**
1. **Keep existing controller temporarily** - comment out broken imports
2. **Create new unified controller file** - `orchestrator-core/src/controllers/unified.rs`
3. **Copy and adapt existing logic** - preserve all functionality
4. **Test thoroughly** - ensure no functionality loss
5. **Replace old controller** - once new one is proven

### **Option 2: In-Place Refactor (Risky)**
1. **Update existing controller file** - could break during transition
2. **Fix compilation issues incrementally** - higher risk of introducing bugs
3. **Test after each change** - more complex rollback if issues arise

### **Why Option 1 is Better:**
- ✅ **Zero risk of losing functionality** - old controller remains as reference
- ✅ **Easier testing** - can compare old vs new behavior
- ✅ **Faster iteration** - don't need to fix everything at once
- ✅ **Better understanding** - forces us to understand every line we copy

---

## 🎯 Next Steps

1. **Review this analysis** - ensure we haven't missed any critical functionality
2. **Choose refactoring approach** - I recommend Option 1 (gradual migration)
3. **Create new controller file** - start with basic structure
4. **Copy core functionality** - begin with reconcile loop and basic job creation
5. **Test incrementally** - ensure each piece works before moving to the next

**Estimated effort:**
- New controller structure: 2-3 hours
- Template system migration: 3-4 hours
- Testing and validation: 2-3 hours
- **Total: 7-10 hours** (but spread across multiple sessions for proper testing)

The existing controller is very well-designed. Our main job is adaptation, not reimplementation.