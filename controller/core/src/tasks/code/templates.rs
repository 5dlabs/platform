use crate::crds::CodeRun;
use crate::tasks::config::ControllerConfig;
use crate::tasks::types::Result;
use handlebars::Handlebars;
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use tracing::debug;

// Template base path (mounted from ConfigMap)
const CLAUDE_TEMPLATES_PATH: &str = "/claude-templates";

pub struct CodeTemplateGenerator;

impl CodeTemplateGenerator {
    /// Generate all template files for a code task
    pub fn generate_all_templates(
        code_run: &CodeRun,
        config: &ControllerConfig,
    ) -> Result<BTreeMap<String, String>> {
        let mut templates = BTreeMap::new();

        // Generate core code templates
        templates.insert(
            "container.sh".to_string(),
            Self::generate_container_script(code_run)?,
        );
        templates.insert(
            "CLAUDE.md".to_string(),
            Self::generate_claude_memory(code_run)?,
        );
        templates.insert(
            "settings.json".to_string(),
            Self::generate_claude_settings(code_run, config)?,
        );

        // Generate code-specific templates
        templates.insert(
            "mcp.json".to_string(),
            Self::generate_mcp_config(code_run, config)?,
        );

        templates.insert(
            "coding-guidelines.md".to_string(),
            Self::generate_coding_guidelines(code_run)?,
        );
        templates.insert(
            "github-guidelines.md".to_string(),
            Self::generate_github_guidelines(code_run)?,
        );

        // Generate hook scripts
        let hook_scripts = Self::generate_hook_scripts(code_run)?;
        for (filename, content) in hook_scripts {
            // Use hooks- prefix to comply with ConfigMap key constraints
            templates.insert(format!("hooks-{filename}"), content);
        }

        // Generate agent system prompt if github_app is specified
        if let Some(github_app) = &code_run.spec.github_app {
            if let Some(system_prompt) = Self::get_agent_system_prompt(github_app) {
                let agent_key = Self::get_agent_key(github_app);
                templates.insert(
                    format!("agents_{agent_key}_system-prompt.md"),
                    system_prompt,
                );
            }
        }

        Ok(templates)
    }

    fn generate_container_script(code_run: &CodeRun) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template("code/container.sh.hbs")?;

        handlebars
            .register_template_string("container_script", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register container script template: {e}"
                ))
            })?;

        let context = json!({
            "task_id": code_run.spec.task_id,
            "service": code_run.spec.service,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_branch": code_run.spec.docs_branch,
            "working_directory": Self::get_working_directory(code_run),
            "continue_session": Self::get_continue_session(code_run),
            "overwrite_memory": code_run.spec.overwrite_memory,
            "docs_project_directory": code_run.spec.docs_project_directory.as_deref().unwrap_or(""),
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
            "model": code_run.spec.model,
        });

        handlebars
            .render("container_script", &context)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to render container script: {e}"
                ))
            })
    }

    fn generate_claude_memory(code_run: &CodeRun) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template("code/claude.md.hbs")?;

        handlebars
            .register_template_string("claude_memory", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register CLAUDE.md template: {e}"
                ))
            })?;

        let context = json!({
            "task_id": code_run.spec.task_id,
            "service": code_run.spec.service,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_branch": code_run.spec.docs_branch,
            "working_directory": Self::get_working_directory(code_run),
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
            "model": code_run.spec.model,
            "context_version": code_run.spec.context_version,
        });

        handlebars.render("claude_memory", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!("Failed to render CLAUDE.md: {e}"))
        })
    }

    fn generate_claude_settings(code_run: &CodeRun, config: &ControllerConfig) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template("code/settings.json.hbs")?;

        handlebars
            .register_template_string("claude_settings", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register settings.json template: {e}"
                ))
            })?;

        let context = json!({
            "model": code_run.spec.model,
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
            "api_key_secret_name": config.secrets.api_key_secret_name,
            "api_key_secret_key": config.secrets.api_key_secret_key,
            "working_directory": code_run.spec.working_directory.as_deref().unwrap_or(".")
        });

        handlebars.render("claude_settings", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!("Failed to render settings.json: {e}"))
        })
    }

    fn generate_mcp_config(_code_run: &CodeRun, _config: &ControllerConfig) -> Result<String> {
        // MCP config is currently static, so just load and return the template content
        Self::load_template("code/mcp.json.hbs")
    }

    fn generate_coding_guidelines(code_run: &CodeRun) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template("code/coding-guidelines.md.hbs")?;

        handlebars
            .register_template_string("coding_guidelines", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register coding-guidelines.md template: {e}"
                ))
            })?;

        let context = json!({
            "service": code_run.spec.service,
            "working_directory": Self::get_working_directory(code_run),
        });

        handlebars
            .render("coding_guidelines", &context)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to render coding-guidelines.md: {e}"
                ))
            })
    }

    fn generate_github_guidelines(code_run: &CodeRun) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template("code/github-guidelines.md.hbs")?;

        handlebars
            .register_template_string("github_guidelines", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register github-guidelines.md template: {e}"
                ))
            })?;

        let context = json!({
            "service": code_run.spec.service,
            "working_directory": Self::get_working_directory(code_run),
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
        });

        handlebars
            .render("github_guidelines", &context)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to render github-guidelines.md: {e}"
                ))
            })
    }

    fn generate_hook_scripts(code_run: &CodeRun) -> Result<BTreeMap<String, String>> {
        let mut hook_scripts = BTreeMap::new();
        let hooks_prefix = "code_hooks_";

        debug!(
            "Scanning for code hook templates with prefix: {}",
            hooks_prefix
        );

        // Read the ConfigMap directory and find files with the hook prefix
        match std::fs::read_dir(CLAUDE_TEMPLATES_PATH) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                            // Check if this is a hook template for code
                            if filename.starts_with(hooks_prefix) && filename.ends_with(".hbs") {
                                // Extract just the hook filename (remove prefix)
                                let hook_name =
                                    filename.strip_prefix(hooks_prefix).unwrap_or(filename);

                                match std::fs::read_to_string(&path) {
                                    Ok(template_content) => {
                                        debug!(
                                            "Loaded code hook template: {} (from {})",
                                            hook_name, filename
                                        );

                                        let mut handlebars = Handlebars::new();
                                        handlebars.set_strict_mode(false);

                                        if let Err(e) = handlebars
                                            .register_template_string("hook", template_content)
                                        {
                                            debug!(
                                                "Failed to register hook template {}: {}",
                                                hook_name, e
                                            );
                                            continue;
                                        }

                                        let context = json!({
                                            "task_id": code_run.spec.task_id,
                                            "service": code_run.spec.service,
                                            "repository_url": code_run.spec.repository_url,
                                            "docs_repository_url": code_run.spec.docs_repository_url,
                                            "working_directory": Self::get_working_directory(code_run),
                                            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
                                        });

                                        match handlebars.render("hook", &context) {
                                            Ok(rendered_script) => {
                                                // Remove .hbs extension for the final filename
                                                let script_name = hook_name
                                                    .strip_suffix(".hbs")
                                                    .unwrap_or(hook_name);
                                                hook_scripts.insert(
                                                    script_name.to_string(),
                                                    rendered_script,
                                                );
                                            }
                                            Err(e) => {
                                                debug!(
                                                    "Failed to render code hook script {}: {}",
                                                    hook_name, e
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        debug!(
                                            "Failed to load code hook template {}: {}",
                                            filename, e
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                debug!("Failed to read templates directory: {}", e);
            }
        }

        Ok(hook_scripts)
    }

    /// Get working directory (defaults to service name if not specified)
    fn get_working_directory(code_run: &CodeRun) -> &str {
        match &code_run.spec.working_directory {
            Some(wd) if !wd.is_empty() => wd,
            _ => &code_run.spec.service,
        }
    }

    /// Get continue session flag - true for retries or user-requested continuation
    fn get_continue_session(code_run: &CodeRun) -> bool {
        // Continue if it's a retry attempt OR user explicitly requested it
        let retry_count = code_run
            .status
            .as_ref()
            .map_or(0, |s| s.retry_count.unwrap_or(0));
        retry_count > 0 || code_run.spec.continue_session
    }

    /// Load a template file from the mounted ConfigMap
    fn load_template(relative_path: &str) -> Result<String> {
        // Convert path separators to underscores for ConfigMap key lookup
        let configmap_key = relative_path.replace('/', "_");
        let full_path = Path::new(CLAUDE_TEMPLATES_PATH).join(&configmap_key);
        debug!(
            "Loading code template from: {} (key: {})",
            full_path.display(),
            configmap_key
        );

        fs::read_to_string(&full_path).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to load code template {relative_path} (key: {configmap_key}): {e}"
            ))
        })
    }

    /// Get the agent key from the GitHub App name
    fn get_agent_key(github_app: &str) -> String {
        match github_app {
            "5DLabs-Morgan" => "morgan".to_string(),
            "5DLabs-Rex" => "rex".to_string(),
            "5DLabs-Blaze" => "blaze".to_string(),
            "5DLabs-Cipher" => "cipher".to_string(),
            "5DLabs-Scout" => "scout".to_string(),
            "5DLabs-Ziggy" => "ziggy".to_string(),
            _ => github_app.to_lowercase().replace("-", "_"),
        }
    }

    /// Get the system prompt for a specific agent with enhanced Rust expertise
    fn get_agent_system_prompt(github_app: &str) -> Option<String> {
        match github_app {
            "5DLabs-Blaze" => Some(
                r#"---
name: Blaze
description: Performance Engineer & Rust Expert
# tools: omitted to inherit all available tools
---

You are Blaze, a Performance Engineer, Optimization Specialist, and Rust Expert at 5D Labs.

## Core Mission
Excel in your specialized domain while collaborating effectively with the broader AI agent team. You bring deep expertise and strategic thinking to every challenge.

## Rust Expertise
You are a Rust programming language expert with comprehensive knowledge of:

### Language Fundamentals
- **Ownership & Borrowing**: Master of Rust's ownership system, lifetimes, and the borrow checker
- **Type System**: Deep understanding of traits, generics, associated types, and type inference
- **Memory Safety**: Expertise in writing safe, concurrent code without data races
- **Error Handling**: Proficient with Result<T, E>, Option<T>, and custom error types

### Advanced Rust Patterns
- **Async/Await**: Expert in tokio, async-std, and futures for asynchronous programming
- **Unsafe Rust**: Know when and how to use unsafe blocks responsibly
- **Macros**: Skilled in declarative and procedural macros for code generation
- **Smart Pointers**: Proficient with Box<T>, Rc<T>, Arc<T>, RefCell<T>, and Mutex<T>

### Performance Optimization
- **Zero-Cost Abstractions**: Leverage Rust's compile-time optimizations
- **SIMD & Parallelism**: Use rayon, crossbeam, and SIMD intrinsics for parallel processing
- **Memory Layout**: Optimize struct layouts, use repr attributes effectively
- **Profiling & Benchmarking**: Expert with criterion, flamegraph, and perf tools

### Ecosystem & Best Practices
- **Cargo & Crates**: Proficient with cargo workspaces, features, and build scripts
- **Testing**: Comprehensive unit, integration, and property-based testing
- **Documentation**: Write excellent rustdoc comments with examples
- **Clippy & Rustfmt**: Enforce best practices and consistent formatting
- **Popular Crates**: Deep knowledge of serde, tokio, actix, diesel, sqlx, axum, and more

### Kubernetes & Cloud Native Rust
- **kube-rs**: Expert in Kubernetes operators and controllers in Rust
- **gRPC & REST**: Building high-performance APIs with tonic and axum
- **Observability**: Implementing metrics, tracing, and logging with tokio-metrics, tracing, and slog

## Working Style
- Write idiomatic, performant Rust code that leverages the language's strengths
- Always consider memory safety, concurrency, and performance implications
- Use appropriate error handling patterns and provide helpful error messages
- Write comprehensive tests and benchmarks for critical code paths
- Document code thoroughly with examples and safety considerations
- Follow Rust API guidelines and naming conventions

## Collaboration
When working with other agents or on existing codebases:
- Provide clear explanations of Rust-specific concepts to non-Rust developers
- Suggest Rust-idiomatic solutions while respecting existing patterns
- Help migrate code to Rust where appropriate for performance gains
- Share knowledge about Rust best practices and common pitfalls"#.to_string()
            ),
            "5DLabs-Rex" => Some(
                r#"---
name: Rex
description: Senior Backend Architect & Systems Engineer with Rust Expertise
# tools: omitted to inherit all available tools
---

You are Rex, a Senior Backend Architect & Systems Engineer at 5D Labs.

## Core Mission
Excel in your specialized domain while collaborating effectively with the broader AI agent team. You bring deep expertise and strategic thinking to every challenge.

## Rust Backend Expertise
As a backend architect, you have deep expertise in Rust for systems programming:

### Systems Programming
- **Low-Level Control**: Expert in systems programming, FFI, and interop with C/C++
- **Network Programming**: Proficient with tokio, async networking, and protocol implementation
- **Database Drivers**: Experience with diesel, sqlx, and custom database integrations
- **Message Queues**: Implementing high-performance message passing with crossbeam channels

### Backend Architecture
- **Microservices**: Design and implement microservices with clear boundaries
- **API Design**: RESTful and gRPC APIs using axum, actix-web, and tonic
- **Event-Driven Systems**: Implement event sourcing and CQRS patterns in Rust
- **Service Mesh**: Integration with Istio, Linkerd, and service discovery

### Performance & Reliability
- **Load Balancing**: Implement custom load balancers and connection pooling
- **Caching Strategies**: Use Redis, in-memory caches, and memoization effectively
- **Circuit Breakers**: Implement resilience patterns for distributed systems
- **Monitoring**: Comprehensive metrics and tracing for production systems

## Working Style
- Design scalable, maintainable backend systems
- Focus on reliability, observability, and operational excellence
- Write clear, well-documented code with proper error handling
- Consider security implications in all design decisions"#.to_string()
            ),
            "5DLabs-Morgan" => Some(
                r#"---
name: Morgan
description: AI Documentation Specialist | Product Manager at 5D Labs
# tools: omitted to inherit all available tools
---

You are Morgan, a meticulous AI Product Manager and Documentation Specialist at 5D Labs.

## Core Mission
Transform ideas into actionable plans and comprehensive documentation. You excel at creating clear, structured documentation that serves as the foundation for successful implementation.

## Documentation Excellence
- Write clear, concise, and comprehensive documentation
- Create detailed task breakdowns and implementation plans
- Maintain consistency across all project documentation
- Ensure technical accuracy while keeping content accessible

## Task Master Expertise
- Expert in Task Master workflows and best practices
- Create well-structured task hierarchies with clear dependencies
- Write detailed implementation instructions and test strategies
- Maintain task documentation throughout the project lifecycle"#.to_string()
            ),
            "5DLabs-Cipher" => Some(
                r#"---
name: Cipher
description: Security Engineer & Code Analysis Specialist
# tools: omitted to inherit all available tools
---

You are Cipher, a Security Engineer & Code Analysis Specialist at 5D Labs.

## Core Mission
Excel in your specialized domain while collaborating effectively with the broader AI agent team. You bring deep expertise in security and code quality.

## Security Expertise
- **Vulnerability Assessment**: Identify and remediate security vulnerabilities
- **Secure Coding**: Implement security best practices in all code
- **Authentication & Authorization**: Design secure auth systems
- **Cryptography**: Proper use of encryption and hashing
- **Compliance**: Ensure adherence to security standards and regulations

## Code Analysis
- **Static Analysis**: Use tools and manual review to identify issues
- **Code Quality**: Enforce high standards for maintainability and reliability
- **Performance Analysis**: Identify bottlenecks and optimization opportunities
- **Dependency Security**: Audit and update dependencies for security"#.to_string()
            ),
            _ => None,
        }
    }
}
