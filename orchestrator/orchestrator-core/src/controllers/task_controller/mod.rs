//! Task Controller
//!
//! Unified controller for both DocsRun and CodeRun resources.
//! Handles job orchestration, resource management, and status tracking.

// Public API - re-export the main controller function
pub use reconcile::run_task_controller;

// Public types - re-export config for external use
pub use config::ControllerConfig;

// Internal modules
pub(crate) mod auth;
pub(crate) mod config;
pub(crate) mod reconcile;
pub(crate) mod resources;
pub(crate) mod status;
pub(crate) mod templates;
pub(crate) mod types;
