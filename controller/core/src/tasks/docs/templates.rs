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
}
