use hotdata::apis::configuration::{ApiKey, Configuration};
use hotdata::apis::workspaces_api;
use hotdata::models;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn error_model_deserializes() {
    let e: models::Error = serde_json::from_str(r#"{"error":"not_found"}"#).unwrap();
    assert_eq!(e.error, "not_found");
}

#[test]
fn configuration_matches_python_sdk_usage() {
    let mut c = Configuration::new();
    c.bearer_access_token = Some("YOUR_ACCESS_TOKEN".into());
    c.api_key = Some(ApiKey {
        prefix: None,
        key: "YOUR_WORKSPACE_ID".into(),
    });
    assert!(c.bearer_access_token.is_some());
}

#[tokio::test]
async fn list_workspaces_hits_api_and_deserializes() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/workspaces"))
        .and(header("authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"ok":true,"workspaces":[{"public_id":"ws_1","name":"Demo","active":true,"favorite":false,"provision_status":"ready","namespace":"demo"}]}"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let mut config = Configuration::new();
    config.base_path = server.uri();
    config.bearer_access_token = Some("test-token".into());

    let body = workspaces_api::list_workspaces(&config, None).await.unwrap();
    assert!(body.ok);
    assert_eq!(body.workspaces.len(), 1);
    assert_eq!(body.workspaces[0].public_id, "ws_1");
    assert_eq!(body.workspaces[0].name, "Demo");
}
