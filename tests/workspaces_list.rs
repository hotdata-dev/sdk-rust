//! Scenario: workspaces_list.
//!
//! List workspaces and confirm the seeded HOTDATA_SDK_TEST_WORKSPACE_ID is
//! present. Read-only — never creates or deletes workspaces against prod.

mod common;

use hotdata::apis::workspaces_api;

#[tokio::test]
async fn workspaces_list() {
    let client = skip_if_no_creds!();
    let config = client.configuration();
    let env = common::load_env();
    let workspace_id = env.workspace_id.expect("creds checked by skip macro");

    let response = workspaces_api::list_workspaces(config, None)
        .await
        .expect("list_workspaces should succeed");
    assert!(response.ok, "list_workspaces response not ok");

    let found = response
        .workspaces
        .iter()
        .any(|w| w.public_id == workspace_id);
    assert!(
        found,
        "expected seeded workspace {workspace_id} in list, got {:?}",
        response
            .workspaces
            .iter()
            .map(|w| &w.public_id)
            .collect::<Vec<_>>()
    );
}
