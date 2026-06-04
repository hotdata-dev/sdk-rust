//! Scenario: saved_query_versioning.
//!
//! Create a saved query, update its SQL twice, confirm
//! list_saved_query_versions reflects the edits, and confirm
//! execute_saved_query runs the latest SQL.

mod common;

use hotdata::apis::saved_queries_api;
use hotdata::models;

#[tokio::test]
async fn saved_query_versioning() {
    let client = skip_if_no_creds!();
    let config = client.configuration();

    let name = common::sdkci_name("savedq-versioning");

    let mut create_req =
        models::CreateSavedQueryRequest::new(name.clone(), "SELECT 1 AS x".to_string());
    create_req.description = Some(Some("sdkci versioning test".to_string()));
    let created = saved_queries_api::create_saved_query(config, create_req)
        .await
        .expect("create_saved_query should succeed");
    assert_eq!(created.latest_version, 1);
    assert_eq!(created.sql, "SELECT 1 AS x");

    let mut update2 = models::UpdateSavedQueryRequest::new();
    update2.sql = Some(Some("SELECT 2 AS x".to_string()));
    let v2 = saved_queries_api::update_saved_query(config, &created.id, update2)
        .await
        .expect("update_saved_query (v2) should succeed");
    assert_eq!(v2.latest_version, 2);
    assert_eq!(v2.sql, "SELECT 2 AS x");

    let mut update3 = models::UpdateSavedQueryRequest::new();
    update3.sql = Some(Some("SELECT 3 AS x".to_string()));
    let v3 = saved_queries_api::update_saved_query(config, &created.id, update3)
        .await
        .expect("update_saved_query (v3) should succeed");
    assert_eq!(v3.latest_version, 3);
    assert_eq!(v3.sql, "SELECT 3 AS x");

    let versions = saved_queries_api::list_saved_query_versions(config, &created.id, None, None)
        .await
        .expect("list_saved_query_versions should succeed");
    assert_eq!(versions.saved_query_id, created.id);
    assert!(versions.count >= 3, "expected at least 3 versions");
    let version_numbers: std::collections::HashSet<i32> =
        versions.versions.iter().map(|v| v.version).collect();
    for expected in [1, 2, 3] {
        assert!(
            version_numbers.contains(&expected),
            "expected version {expected} in {:?}",
            {
                let mut sorted: Vec<i32> = version_numbers.iter().copied().collect();
                sorted.sort_unstable();
                sorted
            }
        );
    }

    // Executing a saved query runs SQL, so it needs a database scope (passed
    // via the required X-Database-Id parameter) and a non-null request body —
    // the generated client serializes `None` as JSON `null`, which the server
    // rejects, so send an empty `ExecuteSavedQueryRequest`.
    let database_id = common::shared_database_id(&client).await;
    let executed = saved_queries_api::execute_saved_query(
        config,
        &created.id,
        &database_id,
        Some(models::ExecuteSavedQueryRequest::new()),
    )
    .await
    .expect("execute_saved_query should succeed");
    assert_eq!(executed.row_count, 1);
    assert_eq!(
        executed.rows,
        vec![vec![serde_json::json!(3)]],
        "execute_saved_query should run the latest SQL (SELECT 3)"
    );

    // Cleanup.
    let _ = saved_queries_api::delete_saved_query(config, &created.id).await;
}
