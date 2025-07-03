//! Kubernetes client and utilities module

pub mod client;

pub use client::{K8sClient, K8sError, K8sResult};
