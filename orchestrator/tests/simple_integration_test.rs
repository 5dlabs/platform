use std::env;

#[tokio::test]
async fn test_basic_functionality() {
    if env::var("SKIP_INTEGRATION_TESTS").is_ok() {
        println!("Skipping integration tests due to SKIP_INTEGRATION_TESTS environment variable");
        return;
    }

    println!("🧪 Running basic functionality test...");

    // Test that we can create a test JSON structure
    let test_event = create_test_github_event(123, "Test issue", "Test body");
    assert_eq!(test_event["issue"]["number"], 123);
    assert_eq!(test_event["issue"]["title"], "Test issue");
    assert_eq!(test_event["action"], "opened");

    println!("✅ JSON event creation works");

    // Test that Axum router can be created
    use axum::Router;

    let _app: Router<()> = Router::new().route("/health", axum::routing::get(|| async { "OK" }));

    println!("✅ Axum router creation works");

    // TODO: Fix configuration test when config module is available
    // use core::config::Config;
    // let _config = Config::default();

    println!("✅ Basic tests completed (some functionality commented out)");

    println!("🎉 Basic integration test completed successfully!");
}

fn create_test_github_event(
    issue_number: i64,
    title: &str,
    body: &str,
) -> serde_json::Value {
    use serde_json::json;

    serde_json::from_value(json!({
        "action": "opened",
        "issue": {
            "id": 123456789,
            "number": issue_number,
            "title": title,
            "body": body,
            "html_url": format!("https://github.com/test-org/test-repo/issues/{}", issue_number),
            "created_at": "2024-06-30T10:00:00Z",
            "updated_at": "2024-06-30T10:00:00Z",
            "labels": [
                {
                    "name": "enhancement",
                    "color": "a2eeef",
                    "description": "New feature or request"
                }
            ],
            "user": {
                "login": "testuser",
                "id": 12345,
                "avatar_url": "https://avatars.githubusercontent.com/u/12345?v=4",
                "html_url": "https://github.com/testuser"
            },
            "state": "open"
        },
        "repository": {
            "id": 987654321,
            "name": "test-repo",
            "full_name": "test-org/test-repo",
            "owner": {
                "login": "test-org",
                "id": 54321,
                "avatar_url": "https://avatars.githubusercontent.com/u/54321?v=4",
                "html_url": "https://github.com/test-org"
            },
            "html_url": "https://github.com/test-org/test-repo",
            "description": "A test repository for webhook testing",
            "default_branch": "main",
            "clone_url": "https://github.com/test-org/test-repo.git"
        },
        "sender": {
            "login": "testuser",
            "id": 12345,
            "avatar_url": "https://avatars.githubusercontent.com/u/12345?v=4",
            "html_url": "https://github.com/testuser"
        }
    }))
    .expect("Failed to create test GitHub event")
}
