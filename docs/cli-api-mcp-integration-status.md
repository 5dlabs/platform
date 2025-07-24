# CLI/API/MCP Integration Status & Outstanding Work

## 🎯 **Current State Summary**

Our orchestrator now has **two solid CRD deployments** (CodeRun and DocsRun) with good template integration and session management. However, there are significant **mismatches between CLI, API, and MCP interfaces** that need to be addressed for full feature parity.

## 📊 **Complete CRD Schema Analysis**

### **CodeRun CRD - Full Field Specification**
```rust
// === CORE TASK IDENTIFICATION ===
task_id: u32                           // Task ID to implement
service: String                        // Target service name

// === REPOSITORY CONFIGURATION ===
repository_url: String                 // Target project repo (where work happens)
docs_repository_url: String            // Docs repo (where Task Master definitions come from)
docs_project_directory: Option<String> // Project dir within docs repo (e.g. "_projects/simple-api")
working_directory: Option<String>      // Working dir within target repo (defaults to service name)
docs_branch: String                    // Docs branch to use (default: "main")

// === AI MODEL & USER CONFIG ===
model: String                          // Claude model to use (sonnet, opus)
github_user: String                    // GitHub username for authentication and commits

// === MCP TOOLS CONFIGURATION ===
local_tools: Option<String>            // Local MCP tools/servers (comma-separated)
remote_tools: Option<String>           // Remote MCP tools/servers (comma-separated)

// === SESSION & CONTEXT MANAGEMENT ===
context_version: u32                   // Context version for retry attempts (incremented on retries)
prompt_modification: Option<String>    // Additional context for retry attempts
continue_session: bool                 // Whether to continue previous session (auto-continue or user-requested)
overwrite_memory: bool                 // Whether to overwrite memory before starting

// === ENVIRONMENT VARIABLES ===
env: HashMap<String, String>           // Environment variables to set in container
env_from_secrets: Vec<SecretEnvVar>    // Environment variables from Kubernetes secrets
```

### **DocsRun CRD - Full Field Specification**
```rust
// === REPOSITORY CONFIGURATION ===
repository_url: String                 // Target repository URL
working_directory: String              // Working directory within repository
source_branch: String                  // Source branch for documentation generation

// === AI MODEL & USER CONFIG ===
model: String                          // Claude model to use
github_user: String                    // GitHub username for authentication and commits
```

### **Field Cleanup Decisions**

#### **🗑️ Fields to REMOVE:**

1. **`prompt_mode: String`** - ✅ **DECISION: Remove**
   - Only "append" mode is used, "replace" mode is unnecessary

2. **`tool_config: String`** - ✅ **DECISION: Remove**
   - Will dynamically construct tool config from `local_tools` and `remote_tools`
   - No need for presets when we have granular control

#### **✅ Fields to KEEP (All Others):**
- **Core**: `task_id`, `service`, `repository_url`, `docs_repository_url`
- **Model & User**: `model`, `github_user`
- **Tools**: `local_tools`, `remote_tools` (comma-separated is fine for now)
- **Session Management**: `context_version`, `continue_session`, `overwrite_memory`, `prompt_modification`
- **Environment**: `env`, `env_from_secrets`
- **Dev Flexibility** (keep for now): `docs_project_directory`, `working_directory`, `docs_branch`

#### **📝 Implementation Notes:**
- `docs_project_directory`, `working_directory`, `docs_branch` retained for development flexibility
- Tool configuration will be dynamically built from individual tool flags
- Future simplification possible once dev needs stabilize

### **Field Support Matrix**

| Field | CLI | API | MCP | CRD | Notes |
|-------|-----|-----|-----|-----|-------|
| task_id | ✅ | ✅ | ✅ | ✅ | |
| service | ✅ | ✅ | ✅ | ✅ | |
| repository_url | ✅ | ✅ | ✅ | ✅ | |
| docs_repository_url | ✅ | ✅ | ❌ | ✅ | MCP uses `platform_repository_url` |
| docs_project_directory | ✅ | ✅ | ❌ | ✅ | **Missing from MCP** |
| working_directory | ✅ | ✅ | ✅ | ✅ | |
| model | ✅ | ✅ | ✅ | ✅ | |
| github_user | ✅ | ✅ | ✅ | ✅ | |
| local_tools | ✅ | ✅ | ✅ | ✅ | |
| remote_tools | ✅ | ✅ | ✅ | ✅ | |
| context_version | ❌ | ❌ | ❌ | ✅ | **Missing from all external interfaces** |
| prompt_modification | ❌ | ❌ | ✅ | ✅ | **Missing from CLI/API** |
| docs_branch | ❌ | ❌ | ❌ | ✅ | **Missing from all external interfaces** |
| continue_session | ❌ | ❌ | ❌ | ✅ | **Missing from all external interfaces** |
| overwrite_memory | ❌ | ❌ | ❌ | ✅ | **Missing from all external interfaces** |
| env | ❌ | ❌ | ❌ | ✅ | **Missing from all external interfaces** |
| env_from_secrets | ❌ | ❌ | ❌ | ✅ | **Missing from all external interfaces** |

## 🚨 **Critical Issues Identified**

### **1. CLI Outdated Branch Handling**
- **Issue**: CLI still has `branch` field but CRD removed it
- **Impact**: CLI will fail when trying to submit code tasks
- **Fix Required**: Remove `branch` field from CLI and use auto-generated feature branches

### **2. MCP Tool Schema Mismatch**
- **Issue**: MCP uses `platform_repository_url` but CRD expects `docs_repository_url`
- **Issue**: MCP missing many CRD fields (see matrix above)
- **Impact**: MCP tools cannot create tasks with full feature set
- **Fix Required**: Update MCP schema to match CRD exactly

### **3. Missing Advanced Features in External Interfaces**
- **Issue**: Critical fields like `continue_session`, `overwrite_memory`, `env`, `env_from_secrets` not exposed
- **Impact**: Cannot test or use advanced features via CLI/MCP
- **Fix Required**: Add missing fields to all interfaces

### **4. Local File Processing Status**
- **Status**: ✅ CLI local file processing (DocsGenerator) appears intact
- **Features**: Auto-commits taskmaster changes, creates docs structure, git detection
- **Validation Needed**: Ensure it still works with current workflow

## 📋 **Outstanding Work Plan**

### **Phase 1: Field Alignment (Priority 1)**

#### **1.1 Update CLI CodeRequest Model**
```rust
// Remove from CLI:
pub branch: String  // ❌ REMOVE - no longer used

// Add to CLI:
pub docs_repository_url: String         // Was missing, now required
pub docs_project_directory: Option<String>
pub context_version: Option<u32>        // For retry scenarios
pub prompt_modification: Option<String> // For retry scenarios
pub prompt_mode: Option<String>         // Default to "append"
pub docs_branch: Option<String>         // Default to "main"
pub continue_session: Option<bool>      // Default to false
pub overwrite_memory: Option<bool>      // Default to false
pub env: Option<HashMap<String, String>>
pub env_from_secrets: Option<Vec<SecretEnvVar>>
```

#### **1.2 Update API Handler**
```rust
// Add mapping for all new fields in code_handler.rs:
context_version: request.context_version.unwrap_or(1),
prompt_modification: request.prompt_modification,
prompt_mode: request.prompt_mode.unwrap_or("append".to_string()),
docs_branch: request.docs_branch.unwrap_or("main".to_string()),
continue_session: request.continue_session.unwrap_or(false),
overwrite_memory: request.overwrite_memory.unwrap_or(false),
env: request.env.unwrap_or_default(),
env_from_secrets: request.env_from_secrets.unwrap_or_default(),
```

#### **1.3 Update MCP Tool Schema**
```rust
// Fix field name:
"docs_repository_url" (not "platform_repository_url")

// Add missing fields:
"docs_project_directory"
"context_version"
"prompt_modification"
"prompt_mode"
"docs_branch"
"continue_session"
"overwrite_memory"
"env"
"env_from_secrets"
```

### **Phase 2: Enhanced CLI Features (Priority 2)**

#### **2.1 Add CLI Flags for Advanced Features**
```bash
# Environment variables
--env KEY=VALUE
--env-from-secret name:secretName:secretKey

# Session management
--continue-session
--overwrite-memory
--context-version 2

# Prompt modification (for retries)
--prompt-modification "Additional instructions"
--prompt-mode replace|append

# Docs branch
--docs-branch feature/updates
```

#### **2.2 CLI Auto-Detection Enhancements**
- Auto-detect if continuing previous task (check for existing feature branch)
- Auto-increment context version for retries
- Smart defaults for session continuation

### **Phase 3: Testing & Validation (Priority 3)**

#### **3.1 Comprehensive Integration Testing**
1. **CLI → API → CRD**: Test all field propagation
2. **MCP → API → CRD**: Test tool calls create proper resources
3. **Local File Processing**: Validate DocsGenerator still works
4. **Advanced Features**: Test env vars, session continuation, memory management

#### **3.2 Backward Compatibility**
- Ensure existing simple workflows continue to work
- Provide sensible defaults for all new fields
- Graceful handling of old vs new CLI versions

## 🔧 **Specific Implementation Tasks**

### **Task 1: Schema Cleanup & Fix CLI Issues (Immediate)**
```rust
// === CRD FIELD REMOVAL ===
// orchestrator-core/src/crds/coderun.rs - Remove prompt_mode and tool_config fields
// Remove default_prompt_mode() and default_tool_config() functions
// Update container template logic to dynamically build tool config

// === CLI BRANCH FIXES ===
// orchestrator-cli/src/main.rs - Remove branch field
// orchestrator-common/src/models/code_request.rs - Remove branch field
// orchestrator-cli/src/commands.rs - Remove branch usage
```

### **Task 2: Add Missing CLI Fields**
```rust
// Update CodeRequest struct with all CRD fields
// Add CLI argument parsing for new fields
// Update command construction in handle_code_command
```

### **Task 3: Fix MCP Tool Schema**
```rust
// orchestrator-mcp/src/tools.rs
// Update get_submit_implementation_task_schema() with correct field names
// Add all missing field definitions
```

### **Task 4: Update API Handler**
```rust
// orchestrator-core/src/handlers/code_handler.rs
// Map all new request fields to CRD spec
// Ensure backward compatibility with defaults
```

### **Task 5: Validate Local Processing**
```rust
// Test orchestrator-cli/src/docs_generator.rs
// Ensure auto-commit, git detection, docs structure creation still works
// Test with current .taskmaster structure
```

## 🎯 **Success Criteria**

1. **✅ All CRD fields accessible via CLI and MCP**
2. **✅ No compilation errors in any component**
3. **✅ Existing workflows continue to work (backward compatibility)**
4. **✅ New advanced features (env vars, session management) functional**
5. **✅ MCP tools can create tasks with full feature parity**
6. **✅ CLI local file processing intact and working**

## ⚠️ **Risk Assessment**

### **High Risk**
- **CLI Breaking Changes**: Removing `branch` field will break existing users
  - *Mitigation*: Provide clear migration guide and backward compatibility warnings

### **Medium Risk**
- **MCP Schema Changes**: Updating tool schemas might affect existing integrations
  - *Mitigation*: Test thoroughly with Cursor integration

### **Low Risk**
- **API Changes**: Adding optional fields with defaults should be safe
- **Local File Processing**: Core logic appears unchanged

## 🚀 **Recommended Implementation Order**

1. **Schema cleanup** (remove prompt_mode, tool_config from CRD) - 45 min
2. **Fix CLI compilation** (remove branch field) - 30 min
3. **Update MCP tool schemas** (field alignment) - 45 min
4. **Add missing CLI fields** (full feature support) - 2 hours
5. **Update API handler** (complete mapping) - 1 hour
6. **Testing & validation** (comprehensive) - 2 hours
7. **Documentation updates** - 1 hour

**Total Estimated Time: ~7.5 hours**

---

## ✅ **Design Review Complete - Ready for Implementation**

### **📋 Field Cleanup Decisions Made:**
- **REMOVE**: `prompt_mode` (only "append" used)
- **REMOVE**: `tool_config` (will build dynamically from individual tools)
- **KEEP**: All other fields including dev flexibility fields

### **🎯 Clear Implementation Path:**
- ✅ Complete CRD schema documented and reviewed
- ✅ Field-by-field gap analysis completed
- ✅ Cleanup decisions finalized
- ✅ Implementation tasks prioritized
- ✅ Time estimates provided

### **🚦 Ready to Start Work:**
The CLI/API/MCP integration work is now fully scoped and ready for implementation. All prerequisites completed:
- Discovery deep dive ✅
- Schema documentation ✅
- Field cleanup decisions ✅
- Implementation planning ✅

*This document represents the current state as of the discovery analysis and field cleanup decisions. All outstanding work should be completed before moving to QA agent implementation.*