//! Scenario: saved_queries_read.
//!
//! Defined in www.hotdata.dev/api/test-scenarios.yaml — read-focused companion
//! to saved_query_versioning. Create a saved query, confirm it appears in
//! list_saved_queries, fetch it by id with get_saved_query, then delete it.

mod common;

use hotdata::apis::saved_queries_api;
use hotdata::models;

#[tokio::test]
async fn saved_queries_read() {
    let client = skip_if_no_creds!();
    let config = client.configuration();

    let name = common::sdkci_name("savedq-read");
    let mut create_req =
        models::CreateSavedQueryRequest::new(name.clone(), "SELECT 1 AS x".to_string());
    create_req.description = Some(Some("sdkci read test".to_string()));
    let created = saved_queries_api::create_saved_query(config, create_req)
        .await
        .expect("create_saved_query should succeed");
    assert!(
        !created.id.is_empty(),
        "created saved query must have an id"
    );

    let listing = saved_queries_api::list_saved_queries(config, Some(100), Some(0))
        .await
        .expect("list_saved_queries should succeed");
    assert!(
        listing.queries.iter().any(|q| q.id == created.id),
        "created saved query {} not present in list_saved_queries",
        created.id
    );

    let fetched = saved_queries_api::get_saved_query(config, &created.id)
        .await
        .expect("get_saved_query should succeed");
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, name);
    assert_eq!(fetched.sql, "SELECT 1 AS x");

    saved_queries_api::delete_saved_query(config, &created.id)
        .await
        .expect("delete_saved_query should succeed");

    let after_delete = saved_queries_api::get_saved_query(config, &created.id).await;
    match after_delete {
        Err(err) => assert_eq!(
            common::status_of(&err),
            Some(404),
            "expected 404 after delete, got {err:?}"
        ),
        Ok(_) => panic!("get_saved_query should fail with 404 after delete"),
    }
}
