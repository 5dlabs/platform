//! Simple output formatting for the CLI

#![allow(clippy::disallowed_macros)]

use colored::Colorize;

/// Simple output manager for consistent formatting
pub struct OutputManager;

impl OutputManager {
    pub fn new() -> Self {
        Self
    }

    pub fn info(&self, message: &str) {
        println!("{} {}", "INFO:".blue().bold(), message);
    }

    pub fn success(&self, message: &str) {
        println!("{} {}", "✓".green().bold(), message);
    }

    pub fn error(&self, message: &str) {
        eprintln!("{} {}", "✗".red().bold(), message);
    }
}
