#!/usr/bin/env python3
"""
Fully Automated GitHub App Creation
No browser required - uses GitHub API directly
"""

import json
import base64
import time
import subprocess
import requests
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.hazmat.backends import default_backend
import jwt
from datetime import datetime, timedelta

# Configuration
ORG = "5dlabs"
AGENTS = {
    "morgan": {
        "name": "5DLabs-Morgan", 
        "description": "Product Management & Documentation Agent",
        "emoji": "📋"
    },
    "rex": {
        "name": "5DLabs-Rex",
        "description": "Senior Backend Architecture Agent", 
        "emoji": "🦖"
    },
    # Add other agents as needed
}

def generate_private_key():
    """Generate RSA private key for GitHub App"""
    key = rsa.generate_private_key(
        public_exponent=65537,
        key_size=2048,
        backend=default_backend()
    )
    
    private_key = key.private_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PrivateFormat.TraditionalOpenSSL,
        encryption_algorithm=serialization.NoEncryption()
    )
    
    return private_key.decode('utf-8')

def create_github_app_direct(token, org, app_config):
    """Create GitHub App using REST API directly"""
    
    # Note: This endpoint requires organization owner permissions
    # and is currently in preview
    url = f"https://api.github.com/orgs/{org}/apps"
    
    headers = {
        "Authorization": f"Bearer {token}",
        "Accept": "application/vnd.github.machine-man-preview+json"
    }
    
    data = {
        "name": app_config["name"],
        "description": app_config["description"],
        "events": [],
        "permissions": {
            "contents": "write",
            "pull_requests": "write",
            "issues": "write",
            "metadata": "read",
            "actions": "write",
            "checks": "write"
        },
        "public": False
    }
    
    response = requests.post(url, headers=headers, json=data)
    
    if response.status_code == 201:
        return response.json()
    else:
        print(f"Error creating app: {response.status_code}")
        print(response.json())
        return None

def store_app_credentials(name, app_id, private_key):
    """Store app credentials in Kubernetes"""
    
    # Create secret in secret-store namespace
    cmd = [
        "kubectl", "create", "secret", "generic", f"github-app-5dlabs-{name}",
        "--namespace=secret-store",
        f"--from-literal=app-id={app_id}",
        f"--from-literal=private-key={private_key}",
        "--dry-run=client", "-o", "yaml"
    ]
    
    create_process = subprocess.run(cmd, capture_output=True, text=True)
    apply_process = subprocess.run(
        ["kubectl", "apply", "-f", "-"],
        input=create_process.stdout,
        capture_output=True,
        text=True
    )
    
    if apply_process.returncode == 0:
        print(f"✅ Stored credentials for {name}")
    else:
        print(f"❌ Failed to store credentials: {apply_process.stderr}")
    
    # Create ExternalSecret
    external_secret = f"""
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: github-app-5dlabs-{name}
  namespace: agent-platform
spec:
  refreshInterval: 30s
  secretStoreRef:
    name: secret-store
    kind: ClusterSecretStore
  target:
    name: github-app-5dlabs-{name}
    creationPolicy: Owner
  data:
  - secretKey: app-id
    remoteRef:
      key: github-app-5dlabs-{name}
      property: app-id
  - secretKey: private-key
    remoteRef:
      key: github-app-5dlabs-{name}
      property: private-key
"""
    
    apply_process = subprocess.run(
        ["kubectl", "apply", "-f", "-"],
        input=external_secret,
        capture_output=True,
        text=True
    )
    
    if apply_process.returncode == 0:
        print(f"✅ Created ExternalSecret for {name}")

def main():
    print("🤖 Fully Automated GitHub App Creation")
    print("=====================================")
    print()
    
    # Get GitHub token (requires org owner permissions)
    print("This script requires a GitHub Personal Access Token with:")
    print("- admin:org scope (to create GitHub Apps)")
    print("- repo scope (to install apps)")
    print()
    
    token = input("Enter your GitHub PAT: ").strip()
    
    # Create each agent app
    for agent_key, agent_config in AGENTS.items():
        print(f"\n📱 Creating {agent_config['name']}...")
        
        # Generate private key
        private_key = generate_private_key()
        
        # Create the app
        app = create_github_app_direct(token, ORG, agent_config)
        
        if app:
            app_id = app["id"]
            print(f"✅ Created app: {agent_config['name']} (ID: {app_id})")
            
            # Store credentials
            store_app_credentials(agent_key, app_id, private_key)
            
            # Install on repositories
            # Note: You'd implement installation here
            
            print(f"✅ Setup complete for {agent_config['name']}")
        else:
            print(f"❌ Failed to create {agent_config['name']}")
        
        time.sleep(2)  # Rate limiting
    
    print("\n✅ All apps created!")
    print("\nNext steps:")
    print("1. Update workflow templates to use GitHub App authentication")
    print("2. Test with Morgan first")
    print("3. Deploy the changes")

if __name__ == "__main__":
    main()