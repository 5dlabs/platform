use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct CodebaseAnalysis {
    pub overview: ProjectOverview,
    pub components: Vec<Component>,
    pub apis: Vec<ApiDefinition>,
    pub configurations: Vec<ConfigFile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectOverview {
    pub name: String,
    pub description: String,
    pub architecture: String,
    pub technologies: Vec<String>,
    pub statistics: ProjectStatistics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectStatistics {
    pub rust_crates: usize,
    pub total_rs_files: usize,
    pub total_lines_of_code: usize,
    pub config_files: usize,
    pub components_analyzed: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub path: String,
    pub component_type: ComponentType,
    pub source_files: Vec<SourceFile>,
    pub dependencies: Vec<String>,
    pub description: String,
    pub line_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ComponentType {
    RustBinary,
    RustLibrary,
    HelmChart,
    KubernetesConfig,
    Documentation,
    Scripts,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceFile {
    pub path: String,
    pub file_type: String,
    pub line_count: usize,
    pub key_definitions: Vec<String>,
    pub content: Option<String>, // Only included if include_source is true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiDefinition {
    pub name: String,
    pub file_path: String,
    pub endpoints: Vec<ApiEndpoint>,
    pub data_models: Vec<DataModel>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiEndpoint {
    pub method: String,
    pub path: String,
    pub handler: String,
    pub line_number: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataModel {
    pub name: String,
    pub model_type: String,
    pub fields: Vec<String>,
    pub file_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigFile {
    pub name: String,
    pub path: String,
    pub config_type: String,
    pub content: Option<String>,
}

pub struct CodebaseAnalyzer {
    workspace_root: PathBuf,
    include_source: bool,
}

impl CodebaseAnalyzer {
    pub fn new(workspace_root: PathBuf, include_source: bool) -> Self {
        Self {
            workspace_root,
            include_source,
        }
    }

        pub fn analyze(&self) -> Result<CodebaseAnalysis> {
        println!("🔍 Analyzing codebase at: {}", self.workspace_root.display());

        let mut overview = self.analyze_project_overview()?;
        let components = self.analyze_components()?;
        let apis = self.analyze_apis()?;
        let configurations = self.analyze_configurations()?;

        // Update the components analyzed count
        overview.statistics.components_analyzed = components.len();

        Ok(CodebaseAnalysis {
            overview,
            components,
            apis,
            configurations,
        })
    }

    fn analyze_project_overview(&self) -> Result<ProjectOverview> {
        println!("📋 Analyzing project overview...");

        let readme_path = self.workspace_root.join("README.md");
        let mut description = String::new();
        let mut name = "Unknown Project".to_string();

        if readme_path.exists() {
            let content = fs::read_to_string(&readme_path)?;
            if let Some(first_line) = content.lines().find(|line| !line.starts_with('#') && !line.trim().is_empty()) {
                description = first_line.to_string();
            }
            if let Some(header) = content.lines().find(|line| line.starts_with("# ")) {
                name = header.trim_start_matches("# ").to_string();
            }
        }

        // Calculate statistics
        let rust_crates = self.count_cargo_files()?;
        let (total_rs_files, total_lines_of_code) = self.count_rust_files()?;
        let config_files = self.count_config_files()?;

        Ok(ProjectOverview {
            name,
            description,
            architecture: "Kubernetes-based orchestrator with MCP integration".to_string(),
            technologies: vec![
                "Rust".to_string(),
                "Kubernetes".to_string(),
                "Helm".to_string(),
                "MCP".to_string(),
                "Docker".to_string(),
            ],
            statistics: ProjectStatistics {
                rust_crates,
                total_rs_files,
                total_lines_of_code,
                config_files,
                components_analyzed: 0, // Will be set correctly in analyze()
            },
        })
    }

    fn analyze_components(&self) -> Result<Vec<Component>> {
        println!("🔧 Analyzing components...");

        let mut components = Vec::new();

        // Analyze Rust components
        self.analyze_rust_components(&mut components)?;

        // Analyze infrastructure components
        self.analyze_infra_components(&mut components)?;

        println!("✅ Found {} components", components.len());
        Ok(components)
    }

    fn analyze_rust_components(&self, components: &mut Vec<Component>) -> Result<()> {
        let cargo_files = self.find_files_by_name("Cargo.toml")?;

        for cargo_path in cargo_files {
            if cargo_path.to_string_lossy().contains("target/") {
                continue;
            }

            let component_dir = cargo_path.parent().unwrap();
            let rel_path = component_dir.strip_prefix(&self.workspace_root)
                .unwrap_or(component_dir)
                .to_string_lossy()
                .to_string();

            let cargo_content = fs::read_to_string(&cargo_path)?;
            let component_name = self.extract_cargo_name(&cargo_content)
                .unwrap_or_else(|| component_dir.file_name().unwrap().to_string_lossy().to_string());

            let component_type = if component_dir.join("src/main.rs").exists() {
                ComponentType::RustBinary
            } else {
                ComponentType::RustLibrary
            };

            let source_files = self.analyze_rust_source_files(component_dir)?;
            let dependencies = self.extract_dependencies(&cargo_content);
            let line_count = source_files.iter().map(|f| f.line_count).sum();

            components.push(Component {
                name: component_name,
                path: rel_path,
                component_type,
                source_files,
                dependencies,
                description: self.extract_description(&cargo_content),
                line_count,
            });
        }

        Ok(())
    }

    fn analyze_rust_source_files(&self, component_dir: &Path) -> Result<Vec<SourceFile>> {
        let mut source_files = Vec::new();

        self.walk_directory(component_dir, &mut |path| {
            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(content) = fs::read_to_string(path) {
                    let rel_path = path.strip_prefix(component_dir)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .to_string();

                    let line_count = content.lines().count();
                    let key_definitions = self.extract_key_definitions(&content);

                    source_files.push(SourceFile {
                        path: rel_path,
                        file_type: "rust".to_string(),
                        line_count,
                        key_definitions,
                        content: if self.include_source { Some(content) } else { None },
                    });
                }
            }
        })?;

        Ok(source_files)
    }

    fn analyze_infra_components(&self, components: &mut Vec<Component>) -> Result<()> {
        let infra_components = vec![
            ("helm-charts", "infra/charts", ComponentType::HelmChart),
            ("kubernetes-config", "infra/cluster-config", ComponentType::KubernetesConfig),
            ("scripts", "infra/scripts", ComponentType::Scripts),
            ("documentation", "docs", ComponentType::Documentation),
        ];

        for (name, path, comp_type) in infra_components {
            let full_path = self.workspace_root.join(path);
            if full_path.exists() {
                let source_files = self.analyze_config_files(&full_path)?;
                let line_count = source_files.iter().map(|f| f.line_count).sum();

                components.push(Component {
                    name: name.to_string(),
                    path: path.to_string(),
                    component_type: comp_type,
                    source_files,
                    dependencies: Vec::new(),
                    description: format!("{} configuration and files", name),
                    line_count,
                });
            }
        }

        Ok(())
    }

    fn analyze_config_files(&self, dir: &Path) -> Result<Vec<SourceFile>> {
        let mut source_files = Vec::new();

        self.walk_directory(dir, &mut |path| {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if matches!(ext, "yaml" | "yml" | "toml" | "json" | "md" | "sh") {
                    if let Ok(content) = fs::read_to_string(path) {
                        let rel_path = path.strip_prefix(dir)
                            .unwrap_or(path)
                            .to_string_lossy()
                            .to_string();

                        let line_count = content.lines().count();

                        source_files.push(SourceFile {
                            path: rel_path,
                            file_type: ext.to_string(),
                            line_count,
                            key_definitions: Vec::new(),
                            content: if self.include_source { Some(content) } else { None },
                        });
                    }
                }
            }
        })?;

        Ok(source_files)
    }

        fn analyze_apis(&self) -> Result<Vec<ApiDefinition>> {
        println!("🌐 Analyzing API surface...");

        let mut apis = Vec::new();

        // Look for route definitions in main.rs files and handler files
        let mut api_files = Vec::new();

        // Find main.rs files that define routes
        let main_files = self.find_files_by_name("main.rs")?;
        for file in main_files {
            if file.to_string_lossy().contains("orchestrator") && !file.to_string_lossy().contains("target") {
                api_files.push(file);
            }
        }

        // Find handler files
        let handler_files = self.find_files_in_path("handlers")?;
        for file in handler_files {
            if file.extension().and_then(|s| s.to_str()) == Some("rs") {
                api_files.push(file);
            }
        }

        for file in api_files {
            let content = fs::read_to_string(&file)?;
            let endpoints = self.extract_api_endpoints(&content);

            if !endpoints.is_empty() {
                let file_name = if file.file_name().unwrap().to_string_lossy() == "main.rs" {
                    "routes".to_string()
                } else {
                    file.file_stem().unwrap().to_string_lossy().to_string()
                };

                apis.push(ApiDefinition {
                    name: file_name,
                    file_path: file.strip_prefix(&self.workspace_root)
                        .unwrap_or(&file)
                        .to_string_lossy()
                        .to_string(),
                    endpoints,
                    data_models: Vec::new(), // Could be enhanced
                });
            }
        }

        Ok(apis)
    }

    fn analyze_configurations(&self) -> Result<Vec<ConfigFile>> {
        println!("⚙️  Analyzing configurations...");

        let mut configs = Vec::new();

        let extensions = vec!["yaml", "yml", "toml", "json"];
        for ext in extensions {
            let files = self.find_files_by_extension(ext)?;
            for file in files {
                let rel_path = file.strip_prefix(&self.workspace_root)
                    .unwrap_or(&file)
                    .to_string_lossy()
                    .to_string();

                if rel_path.contains("target/") || rel_path.contains(".git/") {
                    continue;
                }

                let content = if self.include_source {
                    fs::read_to_string(&file).ok()
                } else {
                    None
                };

                configs.push(ConfigFile {
                    name: file.file_name().unwrap().to_string_lossy().to_string(),
                    path: rel_path,
                    config_type: ext.to_string(),
                    content,
                });
            }
        }

        Ok(configs)
    }

    // Helper methods
    fn count_cargo_files(&self) -> Result<usize> {
        Ok(self.find_files_by_name("Cargo.toml")?
            .into_iter()
            .filter(|p| !p.to_string_lossy().contains("target/"))
            .count())
    }

    fn count_rust_files(&self) -> Result<(usize, usize)> {
        let rust_files = self.find_files_by_extension("rs")?;
        let file_count = rust_files.len();
        let mut total_lines = 0;

        for file in rust_files {
            if let Ok(content) = fs::read_to_string(&file) {
                total_lines += content.lines().count();
            }
        }

        Ok((file_count, total_lines))
    }

    fn count_config_files(&self) -> Result<usize> {
        let mut count = 0;
        for ext in &["yaml", "yml", "toml", "json"] {
            count += self.find_files_by_extension(ext)?.len();
        }
        Ok(count)
    }

    fn find_files_by_name(&self, name: &str) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.walk_directory(&self.workspace_root, &mut |path| {
            if path.file_name().and_then(|s| s.to_str()) == Some(name) {
                files.push(path.to_path_buf());
            }
        })?;
        Ok(files)
    }

    fn find_files_by_extension(&self, ext: &str) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.walk_directory(&self.workspace_root, &mut |path| {
            if path.extension().and_then(|s| s.to_str()) == Some(ext) {
                files.push(path.to_path_buf());
            }
        })?;
        Ok(files)
    }

    fn find_files_in_path(&self, subpath: &str) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.walk_directory(&self.workspace_root, &mut |path| {
            if path.to_string_lossy().contains(subpath) && path.is_file() {
                files.push(path.to_path_buf());
            }
        })?;
        Ok(files)
    }

    fn walk_directory<F>(&self, dir: &Path, callback: &mut F) -> Result<()>
    where
        F: FnMut(&Path),
    {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    if name == "target" || name == ".git" || name.starts_with('.') {
                        continue;
                    }
                }

                callback(&path);

                if path.is_dir() {
                    self.walk_directory(&path, callback)?;
                }
            }
        }
        Ok(())
    }

    fn extract_cargo_name(&self, content: &str) -> Option<String> {
        for line in content.lines() {
            if line.starts_with("name =") {
                return line.split('=').nth(1)
                    .map(|s| s.trim().trim_matches('"').to_string());
            }
        }
        None
    }

    fn extract_dependencies(&self, content: &str) -> Vec<String> {
        let mut deps = Vec::new();
        let mut in_deps_section = false;

        for line in content.lines() {
            if line.starts_with("[dependencies]") {
                in_deps_section = true;
                continue;
            }
            if line.starts_with('[') && in_deps_section {
                break;
            }
            if in_deps_section && line.contains('=') && !line.starts_with('#') {
                if let Some(dep_name) = line.split('=').next() {
                    deps.push(dep_name.trim().to_string());
                }
            }
        }

        deps
    }

    fn extract_description(&self, content: &str) -> String {
        for line in content.lines() {
            if line.starts_with("description =") {
                return line.split('=').nth(1)
                    .unwrap_or("")
                    .trim()
                    .trim_matches('"')
                    .to_string();
            }
        }
        "No description available".to_string()
    }

    fn extract_key_definitions(&self, content: &str) -> Vec<String> {
        let mut definitions = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().starts_with("pub struct")
                || line.trim().starts_with("pub enum")
                || line.trim().starts_with("pub fn")
                || line.trim().starts_with("impl ")
                || line.trim().starts_with("pub trait") {
                definitions.push(format!("{}:{}", line_num + 1, line.trim()));
            }
        }

        definitions
    }

        fn extract_api_endpoints(&self, content: &str) -> Vec<ApiEndpoint> {
        let mut endpoints = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Look for axum route definitions
            if trimmed.contains(".route(") {
                // Handle .route("/path", method(handler)) syntax
                if let Some(path) = self.extract_route_path(trimmed) {
                    let method = if trimmed.contains("post(") { "POST" }
                    else if trimmed.contains("get(") { "GET" }
                    else if trimmed.contains("put(") { "PUT" }
                    else if trimmed.contains("delete(") { "DELETE" }
                    else { "ANY" };

                    endpoints.push(ApiEndpoint {
                        method: method.to_string(),
                        path,
                        handler: trimmed.to_string(),
                        line_number: line_num + 1,
                    });
                }
            }
            // Also look for direct method calls like .get("/path", handler)
            else if trimmed.contains(".get(") || trimmed.contains(".post(")
                || trimmed.contains(".put(") || trimmed.contains(".delete(") {
                if let Some(method) = self.extract_http_method(trimmed) {
                    if let Some(path) = self.extract_route_path(trimmed) {
                        endpoints.push(ApiEndpoint {
                            method,
                            path,
                            handler: trimmed.to_string(),
                            line_number: line_num + 1,
                        });
                    }
                }
            }
        }

        endpoints
    }

    fn extract_http_method(&self, line: &str) -> Option<String> {
        if line.contains(".get(") { Some("GET".to_string()) }
        else if line.contains(".post(") { Some("POST".to_string()) }
        else if line.contains(".put(") { Some("PUT".to_string()) }
        else if line.contains(".delete(") { Some("DELETE".to_string()) }
        else { Some("ROUTE".to_string()) }
    }

    fn extract_route_path(&self, line: &str) -> Option<String> {
        // Look for quoted strings that look like routes (start with /)
        let mut start_pos = 0;
        while let Some(start) = line[start_pos..].find('"') {
            let actual_start = start_pos + start;
            if let Some(end) = line[actual_start + 1..].find('"') {
                let path = &line[actual_start + 1..actual_start + 1 + end];
                if path.starts_with('/') {
                    return Some(path.to_string());
                }
                start_pos = actual_start + 1 + end + 1;
            } else {
                break;
            }
        }
        None
    }

    pub fn generate_modular_markdown(&self, analysis: &CodebaseAnalysis, output_dir: &str) -> Result<()> {
        println!("📝 Generating modular markdown documentation...");

        let output_path = Path::new(output_dir);
        fs::create_dir_all(output_path)?;

        // Generate master index
        self.generate_index_file(analysis, output_path)?;

        // Generate component files
        for component in &analysis.components {
            self.generate_component_file(component, output_path)?;
        }

        // Generate API documentation
        if !analysis.apis.is_empty() {
            self.generate_api_file(&analysis.apis, output_path)?;
        }

        // Generate configuration summary
        self.generate_config_file(&analysis.configurations, output_path)?;

        println!("✅ Modular documentation generated in: {}", output_dir);
        Ok(())
    }

    fn generate_index_file(&self, analysis: &CodebaseAnalysis, output_path: &Path) -> Result<()> {
        let index_file = output_path.join("README.md");
        let mut content = String::new();

        content.push_str(&format!("# {} - Codebase Analysis\n\n", analysis.overview.name));
        content.push_str(&format!("**Generated:** {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        content.push_str("## Overview\n\n");
        content.push_str(&format!("**Description:** {}\n\n", analysis.overview.description));
        content.push_str(&format!("**Architecture:** {}\n\n", analysis.overview.architecture));
        content.push_str(&format!("**Technologies:** {}\n\n", analysis.overview.technologies.join(", ")));

        content.push_str("## Statistics\n\n");
        let stats = &analysis.overview.statistics;
        content.push_str(&format!("- **Rust Crates:** {}\n", stats.rust_crates));
        content.push_str(&format!("- **Rust Files:** {}\n", stats.total_rs_files));
        content.push_str(&format!("- **Lines of Code:** {}\n", stats.total_lines_of_code));
        content.push_str(&format!("- **Config Files:** {}\n", stats.config_files));
        content.push_str(&format!("- **Components:** {}\n\n", analysis.components.len()));

        content.push_str("## Components\n\n");
        for component in &analysis.components {
            content.push_str(&format!("- [{}](./{}.md) - `{}` ({} lines)\n",
                component.name,
                component.name.replace(' ', "-").to_lowercase(),
                component.path,
                component.line_count
            ));
        }

        if !analysis.apis.is_empty() {
            content.push_str("\n- [API Surface](./api-surface.md) - REST endpoints and data models\n");
        }
        content.push_str("- [Configurations](./configurations.md) - All configuration files\n");

        fs::write(index_file, content)?;
        Ok(())
    }

    fn generate_component_file(&self, component: &Component, output_path: &Path) -> Result<()> {
        let filename = format!("{}.md", component.name.replace(' ', "-").to_lowercase());
        let file_path = output_path.join(filename);
        let mut content = String::new();

        content.push_str(&format!("# {} Analysis\n\n", component.name));
        content.push_str(&format!("**Path:** `{}`\n", component.path));
        content.push_str(&format!("**Type:** {:?}\n", component.component_type));
        content.push_str(&format!("**Lines of Code:** {}\n", component.line_count));
        content.push_str(&format!("**Description:** {}\n\n", component.description));

        if !component.dependencies.is_empty() {
            content.push_str("## Dependencies\n\n");
            for dep in &component.dependencies {
                content.push_str(&format!("- {}\n", dep));
            }
            content.push_str("\n");
        }

        content.push_str("## Source Files\n\n");
        for source_file in &component.source_files {
            content.push_str(&format!("### {} ({} lines)\n\n", source_file.path, source_file.line_count));

            if !source_file.key_definitions.is_empty() {
                content.push_str("**Key Definitions:**\n```rust\n");
                for def in &source_file.key_definitions {
                    content.push_str(&format!("{}\n", def));
                }
                content.push_str("```\n\n");
            }

            if let Some(file_content) = &source_file.content {
                content.push_str("**Full Content:**\n```");
                content.push_str(&source_file.file_type);
                content.push_str("\n");
                content.push_str(file_content);
                content.push_str("\n```\n\n");
            }
        }

        fs::write(file_path, content)?;
        Ok(())
    }

    fn generate_api_file(&self, apis: &[ApiDefinition], output_path: &Path) -> Result<()> {
        let file_path = output_path.join("api-surface.md");
        let mut content = String::new();

        content.push_str("# API Surface Analysis\n\n");

        for api in apis {
            content.push_str(&format!("## {} ({})\n\n", api.name, api.file_path));

            if !api.endpoints.is_empty() {
                content.push_str("### Endpoints\n\n");
                for endpoint in &api.endpoints {
                    content.push_str(&format!("- **{}** `{}` - Line {}\n",
                        endpoint.method, endpoint.path, endpoint.line_number));
                    content.push_str(&format!("  ```rust\n  {}\n  ```\n\n", endpoint.handler));
                }
            }
        }

        fs::write(file_path, content)?;
        Ok(())
    }

    fn generate_config_file(&self, configs: &[ConfigFile], output_path: &Path) -> Result<()> {
        let file_path = output_path.join("configurations.md");
        let mut content = String::new();

        content.push_str("# Configuration Files\n\n");

        let mut configs_by_type: HashMap<String, Vec<&ConfigFile>> = HashMap::new();
        for config in configs {
            configs_by_type.entry(config.config_type.clone())
                .or_insert_with(Vec::new)
                .push(config);
        }

        for (config_type, type_configs) in configs_by_type {
            content.push_str(&format!("## {} Files\n\n", config_type.to_uppercase()));

            for config in type_configs {
                content.push_str(&format!("### {} ({})\n\n", config.name, config.path));

                if let Some(file_content) = &config.content {
                    content.push_str("```");
                    content.push_str(&config.config_type);
                    content.push_str("\n");
                    content.push_str(file_content);
                    content.push_str("\n```\n\n");
                }
            }
        }

        fs::write(file_path, content)?;
        Ok(())
    }

    pub fn generate_single_markdown(&self, analysis: &CodebaseAnalysis) -> Result<String> {
        let mut content = String::new();

        content.push_str(&format!("# {} - Complete Codebase Analysis\n\n", analysis.overview.name));
        content.push_str(&format!("**Generated:** {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        content.push_str("## Project Overview\n\n");
        content.push_str(&format!("**Description:** {}\n\n", analysis.overview.description));
        content.push_str(&format!("**Architecture:** {}\n\n", analysis.overview.architecture));
        content.push_str(&format!("**Technologies:** {}\n\n", analysis.overview.technologies.join(", ")));

        content.push_str("## Components\n\n");
        for component in &analysis.components {
            content.push_str(&format!("### {} ({})\n\n", component.name, component.path));
            content.push_str(&format!("**Type:** {:?} | **Lines:** {}\n\n", component.component_type, component.line_count));
            content.push_str(&format!("{}\n\n", component.description));

            for source_file in &component.source_files {
                if let Some(file_content) = &source_file.content {
                    content.push_str(&format!("#### {}\n\n", source_file.path));
                    content.push_str("```");
                    content.push_str(&source_file.file_type);
                    content.push_str("\n");
                    content.push_str(file_content);
                    content.push_str("\n```\n\n");
                }
            }
        }

        Ok(content)
    }
}