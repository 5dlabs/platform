//! Request handlers for the orchestrator service

pub mod code_handler;
pub mod common;
pub mod docs_handler;
pub mod tool_discovery;

pub use code_handler::submit_code_task;
pub use common::{ApiResponse, AppError, AppState};
pub use docs_handler::generate_docs;
