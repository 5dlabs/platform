use handlebars::Handlebars;
use serde_json::json;
use super::config::ControllerConfig;
use super::types::{Result, TaskType};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use tracing::debug;

// Template base path (mounted from ConfigMap)
const CLAUDE_TEMPLATES_PATH: &str = "/claude-templates";

/// Load a template file from the mounted ConfigMap
fn load_template(relative_path: &str) -> Result<String> {
    // Convert path separators to underscores for ConfigMap key lookup
    let configmap_key = relative_path.replace('/', "_");
    let full_path = Path::new(CLAUDE_TEMPLATES_PATH).join(&configmap_key);
    debug!("Loading template from: {} (key: {})", full_path.display(), configmap_key);

    fs::read_to_string(&full_path)
        .map_err(|e| crate::controllers::task_controller::types::Error::ConfigError(
            format!("Failed to load template {} (key: {}): {}", relative_path, configmap_key, e)
        ))
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
    templates.insert("settings.json".to_string(), generate_claude_settings(task, config)?);

    // Generate MCP config (code tasks only)
    if !task.is_docs() {
        templates.insert("mcp.json".to_string(), generate_mcp_config(task, config)?);
        templates.insert("client-config.json".to_string(), generate_client_config(task, config)?);
        templates.insert("coding-guidelines.md".to_string(), generate_coding_guidelines(task)?);
        templates.insert("github-guidelines.md".to_string(), generate_github_guidelines(task)?);
    }

    // Generate hook scripts
    let hook_scripts = generate_hook_scripts(task)?;
    for (filename, content) in hook_scripts {
        // Use hooks- prefix to comply with ConfigMap key constraints
        templates.insert(format!("hooks-{}", filename), content);
    }

    Ok(templates)
}

/// Generate CLAUDE.md content from memory template
fn generate_claude_memory(task: &TaskType) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template_path = if task.is_docs() {
        "docs/CLAUDE.md.hbs"
    } else {
        "code/CLAUDE.md.hbs"
    };

    let template = load_template(template_path)?;

    handlebars
        .register_template_string("claude_memory", template)
        .map_err(|e| crate::controllers::task_controller::types::Error::ConfigError(format!("Failed to register CLAUDE.md template: {e}")))?;

    let data = json!({
        "repository": json!({
            "url": task.repository_url(),
            "branch": task.branch(),
            "githubUser": task.github_user()
        }),
        "working_directory": task.working_directory(),
        "task_id": task.task_id(),
        "platform_repository_url": task.platform_repository_url()
    });

    handlebars
        .render("claude_memory", &data)
        .map_err(|e| crate::controllers::task_controller::types::Error::ConfigError(format!("Failed to render CLAUDE.md template: {e}")))
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
        .map_err(|e| crate::controllers::task_controller::types::Error::ConfigError(format!("Failed to register settings template: {e}")))?;

    let data = build_settings_template_data(task, config);

    handlebars
        .render("settings", &data)
        .map_err(|e| crate::controllers::task_controller::types::Error::ConfigError(format!("Failed to render settings template: {e}")))
}

/// Generate prompt content from template
fn generate_prompt(task: &TaskType) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template_path = if task.is_docs() {
        "docs/prompt.hbs"
    } else {
        "code/prompt.hbs"
    };

    let template = load_template(template_path)?;

    handlebars
        .register_template_string("prompt", template)
        .map_err(|e| crate::controllers::task_controller::types::Error::ConfigError(format!("Failed to register prompt template: {e}")))?;

    let data = json!({
        "repository_url": task.repository_url(),
        "source_branch": task.branch(),
        "branch": task.branch(),
        "github_user": task.github_user(),
        "working_directory": task.working_directory(),
        "model": task.model(),
        "service_name": task.service_name(),
        "task_id": task.task_id(),
        "platform_repository_url": task.platform_repository_url()
    });

    handlebars
        .render("prompt", &data)
        .map_err(|e| crate::controllers::task_controller::types::Error::ConfigError(format!("Failed to render prompt template: {e}")))
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
        .map_err(|e| crate::controllers::task_controller::types::Error::ConfigError(format!("Failed to register container script template: {e}")))?;

    // Generate the prompt content first
    let prompt_content = generate_prompt(task)?;

    let data = json!({
        "repository_url": task.repository_url(),
        "source_branch": task.branch(),
        "branch": task.branch(),
        "github_user": task.github_user(),
        "working_directory": task.working_directory(),
        "model": task.model(),
        "service_name": task.service_name(),
        "task_id": task.task_id(),
        "platform_repository_url": task.platform_repository_url(),
        "prompt_content": prompt_content  // Add the rendered prompt content
    });

    handlebars
        .render("container_script", &data)
        .map_err(|e| crate::controllers::task_controller::types::Error::ConfigError(format!("Failed to render container script template: {e}")))
}

/// Generate MCP configuration for implementation tasks
fn generate_mcp_config(task: &TaskType, config: &ControllerConfig) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template = load_template("code/mcp.json.hbs")?;

    handlebars
        .register_template_string("mcp", &template)
        .map_err(|e| crate::controllers::task_controller::types::Error::ConfigError(format!("Failed to register MCP template: {e}")))?;

    let data = build_settings_template_data(task, config);

    handlebars
        .render("mcp", &data)
        .map_err(|e| crate::controllers::task_controller::types::Error::ConfigError(format!("Failed to render MCP template: {e}")))
}

/// Generate client configuration for dynamic tool selection
fn generate_client_config(task: &TaskType, config: &ControllerConfig) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template = load_template("code/client-config.json.hbs")?;

    handlebars
        .register_template_string("client_config", &template)
        .map_err(|e| crate::controllers::task_controller::types::Error::ConfigError(format!("Failed to register client config template: {e}")))?;

    let data = build_settings_template_data(task, config);

    handlebars
        .render("client_config", &data)
        .map_err(|e| crate::controllers::task_controller::types::Error::ConfigError(format!("Failed to render client config template: {e}")))
}

/// Generate coding guidelines for implementation tasks
fn generate_coding_guidelines(task: &TaskType) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template = load_template("code/coding-guidelines.md.hbs")?;

    handlebars
        .register_template_string("coding_guidelines", &template)
        .map_err(|e| crate::controllers::task_controller::types::Error::ConfigError(format!("Failed to register coding guidelines template: {e}")))?;

    let data = json!({
        "task_id": task.task_id(),
        "service_name": task.service_name()
    });

    handlebars
        .render("coding_guidelines", &data)
        .map_err(|e| crate::controllers::task_controller::types::Error::ConfigError(format!("Failed to render coding guidelines template: {e}")))
}

/// Generate GitHub guidelines for implementation tasks
fn generate_github_guidelines(task: &TaskType) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template = load_template("code/github-guidelines.md.hbs")?;

    handlebars
        .register_template_string("github_guidelines", &template)
        .map_err(|e| crate::controllers::task_controller::types::Error::ConfigError(format!("Failed to register GitHub guidelines template: {e}")))?;

    let data = json!({
        "task_id": task.task_id(),
        "service_name": task.service_name(),
        "github_user": task.github_user(),
        "branch": task.branch()
    });

    handlebars
        .render("github_guidelines", &data)
        .map_err(|e| crate::controllers::task_controller::types::Error::ConfigError(format!("Failed to render GitHub guidelines template: {e}")))
}

/// Build template data for settings/MCP/client config templates
fn build_settings_template_data(task: &TaskType, config: &ControllerConfig) -> serde_json::Value {
    let mut data = json!({
        "task_id": task.task_id(),
        "service_name": task.service_name(),
        "model": task.model(),
        "github_user": task.github_user(),
        "tool_config": task.tool_config(),
        "repository": {
            "url": task.repository_url(),
            "branch": task.branch(),
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
            "prompt_mode": task.prompt_mode(),
            "session_id": task.session_id()
        });
        data["retry"] = retry_data;

        // Add tool configuration
        let (local_tools, remote_tools) = parse_tool_configuration(task);
        data["tools"] = json!({
            "local": local_tools,
            "remote": remote_tools
        });

        // Add platform repository info
        if let Some(platform_url) = task.platform_repository_url() {
            data["platform_repository_url"] = json!(platform_url);
        }
    }

    data
}

/// Parse tool configuration into local and remote tool lists
fn parse_tool_configuration(task: &TaskType) -> (Vec<String>, Vec<String>) {
    let local_tools = task.local_tools()
        .map(|tools| tools.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    let remote_tools = task.remote_tools()
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
            "branch": task.branch(),
            "githubUser": task.github_user()
        }),
        "working_directory": task.working_directory(),
        "attempts": task.retry_count() + 1, // retry_count + 1 = attempt number
        "is_docs_generation": task.is_docs(),
        "platform_repository_url": task.platform_repository_url()
    });

    // Process each hook template
    for (hook_name, template_content) in hook_templates {
        handlebars
            .register_template_string(&hook_name, &template_content)
            .map_err(|e| crate::controllers::task_controller::types::Error::ConfigError(format!("Failed to register hook template {}: {}", hook_name, e)))?;

        let rendered = handlebars
            .render(&hook_name, &data)
            .map_err(|e| crate::controllers::task_controller::types::Error::ConfigError(format!("Failed to render hook template {}: {}", hook_name, e)))?;

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
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                            // Check if this is a hook template for our task type
                            if filename.starts_with(hooks_prefix) && filename.ends_with(".hbs") {
                                // Extract just the hook filename (remove prefix and convert back)
                                let hook_name = filename.strip_prefix(hooks_prefix)
                                    .unwrap_or(filename);

                                match fs::read_to_string(&path) {
                                    Ok(content) => {
                                        debug!("Loaded hook template: {} (from {})", hook_name, filename);
                                        templates.push((hook_name.to_string(), content));
                                    },
                                    Err(e) => {
                                        debug!("Failed to load hook template {}: {}", filename, e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        Err(e) => {
            debug!("Templates directory {} not found or not accessible: {}", CLAUDE_TEMPLATES_PATH, e);
            // Don't fail - hooks are optional
        }
    }

    Ok(templates)
}