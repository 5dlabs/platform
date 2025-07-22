//! Orchestrator core library
//!
//! This crate provides the core functionality for the unified orchestration service,
//! including Kubernetes client wrapper, job orchestration, and request handling.

pub mod controllers;
pub mod crds;
pub mod handlers;

// Re-export commonly used types
pub use controllers::task_controller::ControllerConfig;
pub use crds::{CodeRun, CodeRunSpec, CodeRunStatus, DocsRun, DocsRunSpec, DocsRunStatus};
pub use handlers::*;
