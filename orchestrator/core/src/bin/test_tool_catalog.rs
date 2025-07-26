#!/usr/bin/env cargo
//! Tool catalog template testing utility
//!
//! Usage: cargo run --bin test_tool_catalog

#![allow(clippy::disallowed_macros)]

use handlebars::Handlebars;
use serde_json::json;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Tool Catalog Template...\n");

    // Read the actual tool catalog data
    let tool_catalog_json = fs::read_to_string("/tmp/tool-catalog.json")
        .expect("Failed to read /tmp/tool-catalog.json - run kubectl get configmap toolman-tool-catalog -n orchestrator -o jsonpath='{.data.tool-catalog\\.json}' > /tmp/tool-catalog.json first");

    let tool_catalog: serde_json::Value = serde_json::from_str(&tool_catalog_json)?;

    println!("📊 Loaded tool catalog with {} remote servers and {} local servers",
        tool_catalog.get("remote").and_then(|r| r.as_object()).map(|o| o.len()).unwrap_or(0),
        tool_catalog.get("local").and_then(|l| l.as_object()).map(|o| o.len()).unwrap_or(0)
    );

    // Initialize handlebars engine
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    // Load the template
    let template_path = "../infra/charts/orchestrator/claude-templates/docs/tool-catalog-documentation.md.hbs";
    let template_content = fs::read_to_string(template_path)
        .expect("Failed to read tool catalog template");

    println!("📄 Loaded template from: {}", template_path);

    // Register the template
    handlebars.register_template_string("tool_catalog", &template_content)?;

    // Prepare template data
    let data = json!({
        "tool_catalog": tool_catalog
    });

    // Render the template
    let result = handlebars.render("tool_catalog", &data)?;

    // Write the result to a file for inspection
    fs::write("/tmp/tool-catalog-documentation.md", &result)?;

    println!("✅ Tool catalog documentation generated successfully!");
    println!("📝 Output written to: /tmp/tool-catalog-documentation.md");
    println!("\n🔍 Preview (first 20 lines):");

    for (i, line) in result.lines().take(20).enumerate() {
        println!("{:2}: {}", i + 1, line);
    }

    if result.lines().count() > 20 {
        println!("... ({} more lines)", result.lines().count() - 20);
    }

    Ok(())
}