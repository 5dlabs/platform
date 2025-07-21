//! Orchestrator core library
//!
//! This crate provides the core functionality for the unified orchestration service,
//! including Kubernetes client wrapper, job orchestration, and request handling.

pub mod config;
pub mod controllers;
pub mod crds;
pub mod handlers;
pub mod k8s;
pub mod tool_mapping;
// Re-export commonly used types
pub use config::ControllerConfig;
pub use controllers::run_taskrun_controller;
pub use crds::{TaskRun, TaskRunSpec, TaskRunStatus};
pub use k8s::{K8sClient, K8sError, K8sResult};
pub use tool_mapping::{ToolMappingConfig, ToolCategory, PresetConfig};
