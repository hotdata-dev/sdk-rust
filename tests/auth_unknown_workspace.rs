//! Scenario: auth_unknown_workspace.
//!
//! A valid bearer token combined with a fabricated workspace id (random UUID)
//! must return a 4xx error and never leak data from another workspace. The
//! server may respond 403 (forbidden) or 404 (not found) — both are acceptable.

mod common;

use hotdata::apis::datasets_api;
use hotdata::Client;

#[tokio::test]
async fn auth_unknown_workspace() {
    let env = common::load_env();
    let Some(api_key) = env.api_key.clone() else {
        eprintln!(
            "SKIP {}: set HOTDATA_SDK_TEST_API_KEY to run this scenario",
            module_path!()
        );
        return;
    };

    let fake_workspace = format!("ws_{}", uuid::Uuid::new_v4().simple());

    // Real token, fabricated workspace id.
    let client = Client::builder()
        .api_token(api_key)
        .workspace_id(fake_workspace.clone())
        .base_url(env.api_url)
        .reqwest_client(common::test_http_client())
        .build()
        .expect("Client::build should succeed");

    let result = datasets_api::list_datasets(client.configuration(), None, None).await;
    match result {
        Err(err) => {
            let status = common::status_of(&err);
            assert!(
                matches!(status, Some(403) | Some(404)),
                "expected 403/404 for fabricated workspace {fake_workspace}, got {err:?}"
            );
        }
        Ok(_) => panic!(
            "list_datasets with fabricated workspace {fake_workspace} must not succeed \
             (potential cross-workspace leak)"
        ),
    }
}
