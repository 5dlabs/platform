//! Clean documentation generation request structure

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsRequest {
    /// Git repository URL
    pub repository_url: String,

    /// Working directory within the repository
    pub working_directory: String,

    /// Claude model to use (sonnet, opus)
    pub model: String,

    /// GitHub username for authentication
    pub github_user: String,

    /// Source branch (auto-detected)
    pub source_branch: String,
}
