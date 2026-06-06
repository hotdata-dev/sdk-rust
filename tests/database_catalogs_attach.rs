//! Scenario: database_catalogs_attach.
//!
//! Defined in www.hotdata.dev/api/test-scenarios.yaml — create a scratch
//! database, attach the seeded connection as a catalog (with an alias), confirm
//! it's reachable via the database's attachments, then detach it and delete the
//! database. Reversible and idempotent — never mutates the connection itself.

mod common;

use hotdata::apis::databases_api;
use hotdata::models;

#[tokio::test]
async fn database_catalogs_attach() {
    let (client, connection_id) = skip_if_no_connection!();
    let config = client.configuration();

    let database_id = common::create_scratch_database(&client, "dbcat-attach").await;
    let alias = "sdkci_catalog";

    let mut attach = models::AttachDatabaseCatalogRequest::new(connection_id.clone());
    attach.alias = Some(Some(alias.to_string()));
    databases_api::attach_database_catalog(config, &database_id, attach)
        .await
        .expect("attach_database_catalog should succeed");

    let attached = databases_api::get_database(config, &database_id)
        .await
        .expect("get_database should succeed after attach");
    let found = attached
        .attachments
        .iter()
        .find(|a| a.connection_id == connection_id);
    let found = found.unwrap_or_else(|| {
        panic!("attached connection {connection_id} not present in database attachments")
    });
    assert_eq!(
        found.alias.as_ref().and_then(|a| a.as_deref()),
        Some(alias),
        "attachment alias should round-trip"
    );

    databases_api::detach_database_catalog(config, &database_id, &connection_id)
        .await
        .expect("detach_database_catalog should succeed");

    let detached = databases_api::get_database(config, &database_id)
        .await
        .expect("get_database should succeed after detach");
    assert!(
        !detached
            .attachments
            .iter()
            .any(|a| a.connection_id == connection_id),
        "connection {connection_id} should be gone from attachments after detach"
    );

    // Tear down the scratch database.
    databases_api::delete_database(config, &database_id)
        .await
        .expect("delete_database should succeed");
}
