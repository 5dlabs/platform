//! Webhook payload models for various sources

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Generic webhook payload wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub source: super::RequestSource,
    pub headers: HashMap<String, String>,
    pub body: Value,
}

/// GitHub webhook payload for issue events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubWebhookPayload {
    pub action: String,
    pub issue: Option<GitHubIssue>,
    pub pull_request: Option<GitHubPullRequest>,
    pub repository: GitHubRepository,
    pub sender: GitHubUser,
}

/// GitHub issue structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssue {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub labels: Vec<GitHubLabel>,
    pub created_at: String,
    pub updated_at: String,
    pub user: GitHubUser,
}

/// GitHub pull request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubPullRequest {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub head: GitHubRef,
    pub base: GitHubRef,
    pub created_at: String,
    pub updated_at: String,
    pub user: GitHubUser,
}

/// GitHub repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub owner: GitHubUser,
    pub private: bool,
    pub default_branch: String,
}

/// GitHub user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
    #[serde(rename = "type")]
    pub user_type: String,
}

/// GitHub label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubLabel {
    pub name: String,
    pub color: String,
}

/// GitHub ref (branch/tag)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRef {
    pub label: String,
    pub ref_field: String,
    pub sha: String,
}

/// Grafana alert webhook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrafanaAlert {
    pub status: String,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub values: HashMap<String, f64>,
    #[serde(rename = "startsAt")]
    pub starts_at: String,
    #[serde(rename = "endsAt")]
    pub ends_at: Option<String>,
    #[serde(rename = "generatorURL")]
    pub generator_url: String,
}

/// Grafana webhook payload wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrafanaWebhookPayload {
    pub receiver: String,
    pub status: String,
    pub alerts: Vec<GrafanaAlert>,
    #[serde(rename = "groupLabels")]
    pub group_labels: HashMap<String, String>,
    #[serde(rename = "commonLabels")]
    pub common_labels: HashMap<String, String>,
    #[serde(rename = "commonAnnotations")]
    pub common_annotations: HashMap<String, String>,
}

/// PM Agent webhook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PmAgentPayload {
    pub action: String,
    pub project_id: String,
    pub task: PmTaskData,
}

/// PM Agent task data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PmTaskData {
    pub id: String,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub priority: String,
    pub status: String,
    pub assigned_to: Option<String>,
    pub metadata: HashMap<String, Value>,
}

/// Discord webhook payload (via relay)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordPayload {
    pub channel_id: String,
    pub user_id: String,
    pub username: String,
    pub command: String,
    pub args: Vec<String>,
    pub message_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_webhook_deserialization() {
        let json = r#"{
            "action": "opened",
            "issue": {
                "id": 123,
                "number": 42,
                "title": "Test Issue",
                "body": "Test body",
                "state": "open",
                "labels": [],
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z",
                "user": {
                    "login": "testuser",
                    "id": 456,
                    "type": "User"
                }
            },
            "repository": {
                "id": 789,
                "name": "test-repo",
                "full_name": "org/test-repo",
                "owner": {
                    "login": "org",
                    "id": 999,
                    "type": "Organization"
                },
                "private": false,
                "default_branch": "main"
            },
            "sender": {
                "login": "testuser",
                "id": 456,
                "type": "User"
            }
        }"#;

        let payload: GitHubWebhookPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.action, "opened");
        assert!(payload.issue.is_some());
        assert_eq!(payload.issue.unwrap().number, 42);
    }

    #[test]
    fn test_grafana_alert_deserialization() {
        let json = r#"{
            "receiver": "webhook",
            "status": "firing",
            "alerts": [{
                "status": "firing",
                "labels": {
                    "alertname": "HighErrorRate",
                    "task_id": "123"
                },
                "annotations": {
                    "summary": "High error rate detected"
                },
                "values": {
                    "error_rate": 0.45
                },
                "startsAt": "2024-01-01T00:00:00Z",
                "endsAt": null,
                "generatorURL": "http://grafana/alert"
            }],
            "groupLabels": {},
            "commonLabels": {},
            "commonAnnotations": {}
        }"#;

        let payload: GrafanaWebhookPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.status, "firing");
        assert_eq!(payload.alerts.len(), 1);
        assert_eq!(
            payload.alerts[0].labels.get("task_id"),
            Some(&"123".to_string())
        );
    }
}
