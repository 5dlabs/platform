use serde_json::json;
use super::types::TaskType;

/// Generate SSH key volume configuration if needed
pub fn generate_ssh_volumes(task: &TaskType) -> Vec<serde_json::Value> {
    if !task.uses_ssh() {
        return vec![];
    }

    let ssh_secret_name = task.ssh_secret_name();

    vec![json!({
        "name": "ssh-key",
        "secret": {
            "secretName": ssh_secret_name,
            "defaultMode": 0o600
        }
    })]
}

/// Generate SSH-related environment variables (minimal - most auth via mounted keys)
pub fn generate_ssh_env_vars(task: &TaskType) -> Vec<serde_json::Value> {
    if !task.uses_ssh() {
        return vec![];
    }

    // Set SSH key path and configure Git for SSH
    vec![
        json!({
            "name": "SSH_KEY_PATH",
            "value": "/ssh-keys/id_ed25519"
        }),
        json!({
            "name": "GIT_SSH_COMMAND",
            "value": "ssh -i /ssh-keys/id_ed25519 -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no"
        }),
    ]
}