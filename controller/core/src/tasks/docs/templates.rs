use crate::crds::DocsRun;
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

pub struct DocsTemplateGenerator;

impl DocsTemplateGenerator {
    /// Generate all template files for a docs task
    pub fn generate_all_templates(
        docs_run: &DocsRun,
        config: &ControllerConfig,
    ) -> Result<BTreeMap<String, String>> {
        let mut templates = BTreeMap::new();

        // Generate core docs templates
        templates.insert(
            "container.sh".to_string(),
            Self::generate_container_script(docs_run)?,
        );
        templates.insert(
            "CLAUDE.md".to_string(),
            Self::generate_claude_memory(docs_run)?,
        );
        templates.insert(
            "settings.json".to_string(),
            Self::generate_claude_settings(docs_run, config)?,
        );
        templates.insert(
            "prompt.md".to_string(),
            Self::generate_docs_prompt(docs_run)?,
        );

        // Generate hook scripts
        let hook_scripts = Self::generate_hook_scripts(docs_run)?;
        for (filename, content) in hook_scripts {
            // Use hooks- prefix to comply with ConfigMap key constraints
            templates.insert(format!("hooks-{filename}"), content);
        }

        // Generate agent system prompt if github_app is specified
        if let Some(github_app) = &docs_run.spec.github_app {
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

    fn generate_container_script(docs_run: &DocsRun) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template("docs/container.sh.hbs")?;

        handlebars
            .register_template_string("container_script", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register container script template: {e}"
                ))
            })?;

        let context = json!({
            "repository_url": docs_run.spec.repository_url,
            "source_branch": docs_run.spec.source_branch,
            "working_directory": docs_run.spec.working_directory,
            "github_app": docs_run.spec.github_app.as_deref().unwrap_or(""),
            "model": docs_run.spec.model.as_deref().unwrap_or(""),
            "service_name": "docs-generator",
            "include_codebase": docs_run.spec.include_codebase.unwrap_or(false)
        });

        handlebars
            .render("container_script", &context)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to render container script: {e}"
                ))
            })
    }

    fn generate_claude_memory(docs_run: &DocsRun) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template("docs/claude.md.hbs")?;

        handlebars
            .register_template_string("claude_memory", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register CLAUDE.md template: {e}"
                ))
            })?;

        let context = json!({
            "repository_url": docs_run.spec.repository_url,
            "source_branch": docs_run.spec.source_branch,
            "working_directory": docs_run.spec.working_directory,
            "github_app": docs_run.spec.github_app.as_deref().unwrap_or(""),
            "model": docs_run.spec.model.as_deref().unwrap_or(""),
            "service_name": "docs-generator"
        });

        handlebars.render("claude_memory", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!("Failed to render CLAUDE.md: {e}"))
        })
    }

    fn generate_claude_settings(docs_run: &DocsRun, config: &ControllerConfig) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template("docs/settings.json.hbs")?;

        handlebars
            .register_template_string("claude_settings", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register settings.json template: {e}"
                ))
            })?;

        // Debug logging to trace model value
        let model_value = docs_run.spec.model.as_deref().unwrap_or("");
        tracing::info!(
            "🐛 DEBUG: DocsRun template - model from spec: {:?}",
            docs_run.spec.model
        );
        tracing::info!(
            "🐛 DEBUG: DocsRun template - model value for template: {}",
            model_value
        );

        let context = json!({
            "model": model_value,
            "github_app": docs_run.spec.github_app.as_deref().unwrap_or(""),
            "api_key_secret_name": config.secrets.api_key_secret_name,
            "api_key_secret_key": config.secrets.api_key_secret_key,
            "working_directory": &docs_run.spec.working_directory
        });

        handlebars.render("claude_settings", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!("Failed to render settings.json: {e}"))
        })
    }

    fn generate_docs_prompt(docs_run: &DocsRun) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template("docs/prompt.md.hbs")?;

        handlebars
            .register_template_string("docs_prompt", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register docs prompt template: {e}"
                ))
            })?;

        // Load toolman catalog for embedding in prompt
        let catalog_data = Self::load_toolman_catalog_data()?;
        let catalog_markdown = Self::render_toolman_catalog_markdown(&catalog_data)?;

        let context = json!({
            "repository_url": docs_run.spec.repository_url,
            "source_branch": docs_run.spec.source_branch,
            "working_directory": docs_run.spec.working_directory,
            "service_name": "docs-generator",
            "toolman_catalog_markdown": catalog_markdown,
            "include_codebase": docs_run.spec.include_codebase.unwrap_or(false)
        });

        handlebars.render("docs_prompt", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!("Failed to render docs prompt: {e}"))
        })
    }

    // Removed generate_toolman_catalog - catalog is now embedded as markdown in prompt

    fn load_toolman_catalog_data() -> Result<serde_json::Value> {
        const TOOLMAN_CATALOG_PATH: &str = "/toolman-catalog/tool-catalog.json";

        match fs::read_to_string(TOOLMAN_CATALOG_PATH) {
            Ok(catalog_json) => serde_json::from_str(&catalog_json).map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to parse toolman catalog JSON: {e}"
                ))
            }),
            Err(e) => {
                debug!(
                    "Toolman catalog not found at {}: {}",
                    TOOLMAN_CATALOG_PATH, e
                );
                // Return empty catalog structure if toolman ConfigMap is not available
                Ok(json!({
                    "local": {},
                    "remote": {},
                    "last_updated": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                }))
            }
        }
    }

    fn count_total_tools(catalog_data: &serde_json::Value) -> u32 {
        let mut count = 0;

        if let Some(local) = catalog_data.get("local").and_then(|v| v.as_object()) {
            for server in local.values() {
                if let Some(tools) = server.get("tools").and_then(|v| v.as_array()) {
                    count += tools.len() as u32;
                }
            }
        }

        if let Some(remote) = catalog_data.get("remote").and_then(|v| v.as_object()) {
            for server in remote.values() {
                if let Some(tools) = server.get("tools").and_then(|v| v.as_array()) {
                    count += tools.len() as u32;
                }
            }
        }

        count
    }

    fn render_toolman_catalog_markdown(catalog_data: &serde_json::Value) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        // Register json helper for proper JSON serialization
        handlebars.register_helper(
            "json",
            Box::new(
                |h: &handlebars::Helper,
                 _: &Handlebars,
                 _: &handlebars::Context,
                 _: &mut handlebars::RenderContext,
                 out: &mut dyn handlebars::Output|
                 -> handlebars::HelperResult {
                    let param =
                        h.param(0)
                            .ok_or(handlebars::RenderErrorReason::ParamNotFoundForIndex(
                                "json", 0,
                            ))?;
                    let json_str = serde_json::to_string(param.value())
                        .map_err(|e| handlebars::RenderErrorReason::NestedError(Box::new(e)))?;
                    out.write(&json_str)?;
                    Ok(())
                },
            ),
        );

        let template = Self::load_template("docs/toolman-catalog.md.hbs")?;

        handlebars
            .register_template_string("toolman_catalog_markdown", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register toolman catalog markdown template: {e}"
                ))
            })?;

        let context = json!({
            "toolman_catalog": catalog_data,
            "generated_timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            "total_tool_count": Self::count_total_tools(catalog_data)
        });

        handlebars
            .render("toolman_catalog_markdown", &context)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to render toolman catalog markdown: {e}"
                ))
            })
    }

    fn generate_hook_scripts(docs_run: &DocsRun) -> Result<BTreeMap<String, String>> {
        let mut hook_scripts = BTreeMap::new();
        let hooks_prefix = "docs_hooks_";

        debug!(
            "Scanning for docs hook templates with prefix: {}",
            hooks_prefix
        );

        // Read the ConfigMap directory and find files with the hook prefix
        match std::fs::read_dir(CLAUDE_TEMPLATES_PATH) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                            // Check if this is a hook template for docs
                            if filename.starts_with(hooks_prefix) && filename.ends_with(".hbs") {
                                // Extract just the hook filename (remove prefix)
                                let hook_name =
                                    filename.strip_prefix(hooks_prefix).unwrap_or(filename);

                                match std::fs::read_to_string(&path) {
                                    Ok(template_content) => {
                                        debug!(
                                            "Loaded docs hook template: {} (from {})",
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
                                            "repository_url": docs_run.spec.repository_url,
                                            "source_branch": docs_run.spec.source_branch,
                                            "working_directory": docs_run.spec.working_directory,
                                            "github_app": docs_run.spec.github_app.as_deref().unwrap_or(""),
                                            "service_name": "docs-generator"
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
                                                    "Failed to render docs hook script {}: {}",
                                                    hook_name, e
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        debug!(
                                            "Failed to load docs hook template {}: {}",
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

    /// Load a template file from the mounted ConfigMap
    fn load_template(relative_path: &str) -> Result<String> {
        // Convert path separators to underscores for ConfigMap key lookup
        let configmap_key = relative_path.replace('/', "_");
        let full_path = Path::new(CLAUDE_TEMPLATES_PATH).join(&configmap_key);
        debug!(
            "Loading docs template from: {} (key: {})",
            full_path.display(),
            configmap_key
        );

        fs::read_to_string(&full_path).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to load docs template {relative_path} (key: {configmap_key}): {e}"
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
