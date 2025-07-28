//! Task Controller
//!
//! Separated controllers for `DocsRun` and `CodeRun` resources.
//! Handles job orchestration, resource management, and status tracking.

// Public API - re-export the main controller function
pub use controller::run_task_controller;

// Public types - re-export config for external use
pub use config::ControllerConfig;

// New separated modules
pub(crate) mod code_controller;
pub(crate) mod code_resources;
pub(crate) mod code_status;
pub(crate) mod code_templates;
pub(crate) mod config;
pub(crate) mod controller;
pub(crate) mod docs_controller;
pub(crate) mod docs_resources;
pub(crate) mod docs_status;
pub(crate) mod docs_templates;
pub(crate) mod types;

// Old modules have been successfully removed during refactor
