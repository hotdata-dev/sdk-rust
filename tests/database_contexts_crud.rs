//! Scenario: database_contexts_crud.
//!
//! Defined in www.hotdata.dev/api/test-scenarios.yaml — create a scratch
//! database, upsert a named context document, read it back, confirm it appears
//! in list_database_contexts, delete the context, then delete the database.
//! Upserting the same name twice verifies replace-on-write.

mod common;

use hotdata::apis::{database_context_api, databases_api};
use hotdata::models;

#[tokio::test]
async fn database_contexts_crud() {
    let client = skip_if_no_creds!();
    let config = client.configuration();

    let database_id = common::create_scratch_database(&client, "dbctx-crud").await;
    let context_name = "sdkci_context";

    // Initial upsert.
    let initial = database_context_api::upsert_database_context(
        config,
        &database_id,
        models::UpsertDatabaseContextRequest::new(
            "first revision".to_string(),
            context_name.to_string(),
        ),
    )
    .await
    .expect("upsert_database_context (create) should succeed");
    assert_eq!(initial.context.name, context_name);
    assert_eq!(initial.context.content, "first revision");

    // Upsert the same name again — replace-on-write, not a duplicate.
    database_context_api::upsert_database_context(
        config,
        &database_id,
        models::UpsertDatabaseContextRequest::new(
            "second revision".to_string(),
            context_name.to_string(),
        ),
    )
    .await
    .expect("upsert_database_context (replace) should succeed");

    let fetched = database_context_api::get_database_context(config, &database_id, context_name)
        .await
        .expect("get_database_context should succeed");
    assert_eq!(fetched.context.name, context_name);
    assert_eq!(
        fetched.context.content, "second revision",
        "upsert with an existing name should replace the content"
    );

    let listing = database_context_api::list_database_contexts(config, &database_id)
        .await
        .expect("list_database_contexts should succeed");
    let matches: Vec<_> = listing
        .contexts
        .iter()
        .filter(|c| c.name == context_name)
        .collect();
    assert_eq!(
        matches.len(),
        1,
        "expected exactly one context named {context_name} (replace, not append), got {}",
        matches.len()
    );

    database_context_api::delete_database_context(config, &database_id, context_name)
        .await
        .expect("delete_database_context should succeed");

    let after_delete =
        database_context_api::get_database_context(config, &database_id, context_name).await;
    match after_delete {
        Err(err) => assert_eq!(
            common::status_of(&err),
            Some(404),
            "expected 404 after context delete, got {err:?}"
        ),
        Ok(_) => panic!("get_database_context should fail with 404 after delete"),
    }

    // Tear down the scratch database.
    databases_api::delete_database(config, &database_id)
        .await
        .expect("delete_database should succeed");
}
