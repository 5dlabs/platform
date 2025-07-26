use super::config::ControllerConfig;
use super::types::{Result, TaskType};
use handlebars::Handlebars;
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use tracing::debug;

// Template base path (mounted from ConfigMap)
const CLAUDE_TEMPLATES_PATH: &str = "/claude-templates";

/// Load a template file from the mounted `ConfigMap`
fn load_template(relative_path: &str) -> Result<String> {
    // Convert path separators to underscores for ConfigMap key lookup
    let configmap_key = relative_path.replace('/', "_");
    let full_path = Path::new(CLAUDE_TEMPLATES_PATH).join(&configmap_key);
    debug!(
        "Loading template from: {} (key: {})",
        full_path.display(),
        configmap_key
    );

    fs::read_to_string(&full_path).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to load template {relative_path} (key: {configmap_key}): {e}"
        ))
    })
}

/// Generate all template files for a task
pub fn generate_templates(
    task: &TaskType,
    config: &ControllerConfig,
) -> Result<BTreeMap<String, String>> {
    let mut templates = BTreeMap::new();

    // Generate container startup script
    templates.insert("container.sh".to_string(), generate_container_script(task)?);

    // Generate Claude memory
    templates.insert("CLAUDE.md".to_string(), generate_claude_memory(task)?);

    // Generate Claude settings
    templates.insert(
        "settings.json".to_string(),
        generate_claude_settings(task, config)?,
    );

    // Generate task-specific templates
    if task.is_docs() {
        // Generate docs prompt
        templates.insert("prompt.md".to_string(), generate_docs_prompt(task)?);
        // Generate tool catalog documentation from live ConfigMap
        templates.insert("tool-catalog-documentation.md".to_string(), generate_tool_catalog_documentation()?);
    } else {
        // Generate code-specific templates
        templates.insert("mcp.json".to_string(), generate_mcp_config(task, config)?);
        templates.insert(
            "client-config.json".to_string(),
            generate_client_config(task, config)?,
        );
        templates.insert(
            "coding-guidelines.md".to_string(),
            generate_coding_guidelines(task)?,
        );
        templates.insert(
            "github-guidelines.md".to_string(),
            generate_github_guidelines(task)?,
        );
        templates.insert("mcp-tools.md".to_string(), generate_mcp_tools_doc(task)?);
    }

    // Generate hook scripts
    let hook_scripts = generate_hook_scripts(task)?;
    for (filename, content) in hook_scripts {
        // Use hooks- prefix to comply with ConfigMap key constraints
        templates.insert(format!("hooks-{filename}"), content);
    }

    Ok(templates)
}

/// Generate CLAUDE.md content from memory template
fn generate_claude_memory(task: &TaskType) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template_path = if task.is_docs() {
        "docs/claude.md.hbs"
    } else {
        "code/claude.md.hbs"
    };

    let template = load_template(template_path)?;

    handlebars
        .register_template_string("claude_memory", template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register CLAUDE.md template: {e}"
            ))
        })?;

    let data = json!({
        "repository": json!({
            "url": task.repository_url(),
            "githubUser": task.github_user()
        }),
        "working_directory": task.working_directory(),
        "task_id": task.task_id(),
        "docs_repository_url": task.docs_repository_url()
    });

    handlebars.render("claude_memory", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render CLAUDE.md template: {e}"
        ))
    })
}

/// Generate Claude Code settings.json for tool permissions
fn generate_claude_settings(task: &TaskType, config: &ControllerConfig) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template_path = if task.is_docs() {
        "docs/settings.json.hbs"
    } else {
        "code/settings.json.hbs"
    };

    let template = load_template(template_path)?;

    handlebars
        .register_template_string("settings", template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register settings template: {e}"
            ))
        })?;

    let data = build_settings_template_data(task, config);

    handlebars.render("settings", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render settings template: {e}"
        ))
    })
}

/// Generate container startup script from template
fn generate_container_script(task: &TaskType) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template_path = if task.is_docs() {
        "docs/container.sh.hbs"
    } else {
        "code/container.sh.hbs"
    };

    let template = load_template(template_path)?;

    handlebars
        .register_template_string("container_script", template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register container script template: {e}"
            ))
        })?;

    // Prompt content is now embedded inline in container script - no template needed

    let data = json!({
        "repository_url": task.repository_url(),
        "github_user": task.github_user(),
        "working_directory": task.working_directory(),
        "model": task.model(),
        "service_name": task.service_name(),
        "task_id": task.task_id(),
        "source_branch": task.source_branch(),
        "docs_repository_url": task.docs_repository_url(),
        "docs_branch": task.docs_branch(),
        "docs_project_directory": task.docs_project_directory(),
        "overwrite_memory": task.overwrite_memory(),
        "continue_session": task.continue_session(),
        "user_requested": match task {
            crate::controllers::task_controller::types::TaskType::Code(cr) => cr.spec.continue_session,
            _ => false
        }
    });

    handlebars.render("container_script", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render container script template: {e}"
        ))
    })
}

/// Generate MCP configuration for implementation tasks
fn generate_mcp_config(task: &TaskType, config: &ControllerConfig) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template = load_template("code/mcp.json.hbs")?;

    handlebars
        .register_template_string("mcp", &template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register MCP template: {e}"
            ))
        })?;

    let data = build_settings_template_data(task, config);

    handlebars.render("mcp", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render MCP template: {e}"
        ))
    })
}

/// Generate MCP tools documentation based on task configuration
fn generate_mcp_tools_doc(task: &TaskType) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template = load_template("code/mcp-tools.md.hbs")?;

    handlebars
        .register_template_string("mcp_tools", template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register MCP tools template: {e}"
            ))
        })?;

    // Parse comma-separated tool strings into arrays
    let local_tools: Vec<String> = task
        .local_tools()
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    let remote_tools: Vec<String> = task
        .remote_tools()
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    let data = json!({
        "localTools": local_tools,
        "remoteTools": remote_tools,
        "service": task.service_name(),
        "task_id": task.task_id()
    });

    handlebars.render("mcp_tools", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render MCP tools template: {e}"
        ))
    })
}

/// Generate client configuration for dynamic tool selection
fn generate_client_config(task: &TaskType, config: &ControllerConfig) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template = load_template("code/client-config.json.hbs")?;

    handlebars
        .register_template_string("client_config", &template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register client config template: {e}"
            ))
        })?;

    let data = build_settings_template_data(task, config);

    handlebars.render("client_config", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render client config template: {e}"
        ))
    })
}

/// Generate coding guidelines for implementation tasks
fn generate_coding_guidelines(task: &TaskType) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template = load_template("code/coding-guidelines.md.hbs")?;

    handlebars
        .register_template_string("coding_guidelines", &template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register coding guidelines template: {e}"
            ))
        })?;

    let data = json!({
        "task_id": task.task_id(),
        "service_name": task.service_name()
    });

    handlebars.render("coding_guidelines", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render coding guidelines template: {e}"
        ))
    })
}

/// Generate GitHub guidelines for implementation tasks
fn generate_github_guidelines(task: &TaskType) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template = load_template("code/github-guidelines.md.hbs")?;

    handlebars
        .register_template_string("github_guidelines", &template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register GitHub guidelines template: {e}"
            ))
        })?;

    let data = json!({
        "task_id": task.task_id(),
        "service_name": task.service_name(),
        "github_user": task.github_user()
    });

    handlebars.render("github_guidelines", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render GitHub guidelines template: {e}"
        ))
    })
}

/// Generate docs prompt for documentation generation tasks
fn generate_docs_prompt(task: &TaskType) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template = load_template("docs/prompt.md.hbs")?;

    handlebars
        .register_template_string("docs_prompt", &template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register docs prompt template: {e}"
            ))
        })?;

    let data = json!({
        "task_id": task.task_id(),
        "service_name": task.service_name(),
        "github_user": task.github_user(),
        "working_directory": task.working_directory(),
        "repository_url": task.repository_url()
    });

    handlebars.render("docs_prompt", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render docs prompt template: {e}"
        ))
    })
}

/// Build template data for settings/MCP/client config templates
fn build_settings_template_data(task: &TaskType, config: &ControllerConfig) -> serde_json::Value {
    let mut data = json!({
        "task_id": task.task_id(),
        "service_name": task.service_name(),
        "model": task.model(),
        "github_user": task.github_user(),
        "repository": {
            "url": task.repository_url(),
            "githubUser": task.github_user()
        },
        "working_directory": task.working_directory(),
        "agent_tools_override": config.permissions.agent_tools_override,
        "permissions": {
            "allow": config.permissions.allow,
            "deny": config.permissions.deny
        },
        "telemetry": {
            "enabled": config.telemetry.enabled,
            "otlpEndpoint": config.telemetry.otlp_endpoint,
            "otlpProtocol": config.telemetry.otlp_protocol,
            "logs_endpoint": config.telemetry.logs_endpoint,
            "logs_protocol": config.telemetry.logs_protocol
        }
    });

    // Add retry information for code tasks
    if !task.is_docs() {
        let retry_data = json!({
            "context_version": task.context_version(),
            "prompt_modification": task.prompt_modification(),
            "session_id": task.session_id()
        });
        data["retry"] = retry_data;

        // Add tool configuration
        let (local_tools, remote_tools) = parse_tool_configuration(task);
        data["tools"] = json!({
            "local": local_tools,
            "remote": remote_tools
        });

        // Add docs repository info
        if let Some(docs_url) = task.docs_repository_url() {
            data["docs_repository_url"] = json!(docs_url);
        }
    }

    data
}

/// Parse tool configuration into local and remote tool lists
fn parse_tool_configuration(task: &TaskType) -> (Vec<String>, Vec<String>) {
    let local_tools = task
        .local_tools()
        .map(|tools| tools.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    let remote_tools = task
        .remote_tools()
        .map(|tools| tools.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    (local_tools, remote_tools)
}

/// Generate hook scripts from the hooks directory
fn generate_hook_scripts(task: &TaskType) -> Result<BTreeMap<String, String>> {
    let mut hook_scripts = BTreeMap::new();
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    // Get hook templates based on task type
    let hook_templates = get_hook_templates(task)?;

    // Prepare template data
    let data = json!({
        "task_id": task.task_id(),
        "service_name": task.service_name(),
        "repository": json!({
            "url": task.repository_url(),
            "githubUser": task.github_user()
        }),
        "working_directory": task.working_directory(),
        "attempts": task.retry_count() + 1, // retry_count + 1 = attempt number
        "is_docs_generation": task.is_docs(),
        "docs_repository_url": task.docs_repository_url()
    });

    // Process each hook template
    for (hook_name, template_content) in hook_templates {
        handlebars
            .register_template_string(&hook_name, &template_content)
            .map_err(|e| {
                crate::controllers::task_controller::types::Error::ConfigError(format!(
                    "Failed to register hook template {hook_name}: {e}"
                ))
            })?;

        let rendered = handlebars.render(&hook_name, &data).map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to render hook template {hook_name}: {e}"
            ))
        })?;

        // Remove .hbs extension for the final filename
        let filename = hook_name.strip_suffix(".hbs").unwrap_or(&hook_name);
        hook_scripts.insert(filename.to_string(), rendered);
    }

    Ok(hook_scripts)
}

/// Get all hook templates for a specific task type by scanning the filesystem
fn get_hook_templates(task: &TaskType) -> Result<Vec<(String, String)>> {
    let hooks_prefix = match task {
        TaskType::Docs(_) => "docs_hooks_",
        TaskType::Code(_) => "code_hooks_",
    };

    debug!("Scanning for hook templates with prefix: {}", hooks_prefix);

    let mut templates = Vec::new();

    // Read the ConfigMap directory and find files with the hook prefix
    match std::fs::read_dir(CLAUDE_TEMPLATES_PATH) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                        // Check if this is a hook template for our task type
                        if filename.starts_with(hooks_prefix) && filename.ends_with(".hbs") {
                            // Extract just the hook filename (remove prefix and convert back)
                            let hook_name = filename.strip_prefix(hooks_prefix).unwrap_or(filename);

                            match fs::read_to_string(&path) {
                                Ok(content) => {
                                    debug!(
                                        "Loaded hook template: {} (from {})",
                                        hook_name, filename
                                    );
                                    templates.push((hook_name.to_string(), content));
                                }
                                Err(e) => {
                                    debug!("Failed to load hook template {}: {}", filename, e);
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            debug!(
                "Templates directory {} not found or not accessible: {}",
                CLAUDE_TEMPLATES_PATH, e
            );
            // Don't fail - hooks are optional
        }
    }

    Ok(templates)
}

/// Generate tool catalog documentation from mounted ConfigMap file
fn generate_tool_catalog_documentation() -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template = load_template("docs/tool-catalog-documentation.md.hbs")?;

    handlebars
        .register_template_string("tool_catalog", &template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register tool catalog template: {e}"
            ))
        })?;

    // Read tool catalog data from mounted ConfigMap file
    let tool_catalog_data = get_tool_catalog_data_from_file()?;

    let data = json!({
        "tool_catalog": tool_catalog_data
    });

    handlebars.render("tool_catalog", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render tool catalog template: {e}"
        ))
    })
}

/// Read tool catalog data from mounted ConfigMap file
fn get_tool_catalog_data_from_file() -> Result<serde_json::Value> {
    use std::fs;

    // Read the tool catalog JSON from the mounted ConfigMap
    let tool_catalog_json = fs::read_to_string("/tool-catalog/tool-catalog.json")
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to read tool catalog file: {e}"
            ))
        })?;

    // Parse the JSON
    serde_json::from_str(&tool_catalog_json).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to parse tool catalog JSON: {e}"
        ))
    })
}
