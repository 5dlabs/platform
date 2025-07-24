use super::types::TaskType;
use serde_json::json;

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
            "defaultMode": 0o600,
            "items": [
                {
                    "key": "ssh-privatekey",
                    "path": "id_ed25519",
                    "mode": 0o600
                },
                {
                    "key": "ssh-publickey",
                    "path": "id_ed25519.pub",
                    "mode": 0o644
                }
            ]
        }
    })]
}
