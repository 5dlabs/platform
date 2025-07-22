//! Simple output formatting for the CLI

use anyhow::Result;
use colored::*;

/// Simple output manager for consistent formatting
pub struct OutputManager;

impl OutputManager {
    pub fn new() -> Self {
        Self
    }

    pub fn info(&self, message: &str) -> Result<()> {
        println!("{} {}", "INFO:".blue().bold(), message);
        Ok(())
    }

    pub fn success(&self, message: &str) -> Result<()> {
        println!("{} {}", "✓".green().bold(), message);
        Ok(())
    }

    pub fn warning(&self, message: &str) -> Result<()> {
        println!("{} {}", "⚠".yellow().bold(), message);
        Ok(())
    }

    pub fn error(&self, message: &str) -> Result<()> {
        eprintln!("{} {}", "✗".red().bold(), message);
        Ok(())
    }
}