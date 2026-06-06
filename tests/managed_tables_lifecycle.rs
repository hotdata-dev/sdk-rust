//! Scenario: managed_tables_lifecycle.
//!
//! Defined in www.hotdata.dev/api/test-scenarios.yaml — the heaviest scenario.
//! It ties databases, uploads, and managed tables together on a fresh scratch
//! database (whose default catalog is a managed catalog): declare a schema and a
//! table, upload a small parquet file, load it into the table, read the table
//! profile, refresh catalog metadata, purge the table cache, then delete the
//! table and the database. Self-cleaning — touches no seeded data.
//!
//! The 3-row parquet payload is a committed fixture (tests/fixtures/), so the
//! test needs no parquet writer at runtime.

mod common;

use hotdata::apis::{connections_api, databases_api, refresh_api, uploads_api};
use hotdata::models;
use std::path::Path;

#[tokio::test]
async fn managed_tables_lifecycle() {
    let client = skip_if_no_creds!();
    let config = client.configuration();

    let database_id = common::create_scratch_database(&client, "managed-tables").await;

    // The database's auto-provisioned default catalog is a managed catalog,
    // addressed through its default_connection_id.
    let connection_id = databases_api::get_database(config, &database_id)
        .await
        .expect("get_database should succeed")
        .default_connection_id;

    let schema_name = "sdkci_mt";
    let table_name = "sdkci_loaded";

    let schema = connections_api::add_managed_schema(
        config,
        &connection_id,
        models::AddManagedSchemaRequest::new(schema_name.to_string()),
    )
    .await
    .expect("add_managed_schema should succeed");
    assert_eq!(schema.schema, schema_name);

    let table = connections_api::add_managed_table(
        config,
        &connection_id,
        schema_name,
        models::AddManagedTableRequest::new(table_name.to_string()),
    )
    .await
    .expect("add_managed_table should succeed");
    assert_eq!(table.table, table_name);

    // Upload the committed 3-row parquet fixture, then load it into the table.
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sdkci_managed.parquet");
    let upload = uploads_api::upload_file(config, fixture)
        .await
        .expect("upload_file should succeed");
    assert!(!upload.id.is_empty(), "upload must return an id");

    let loaded = connections_api::load_managed_table(
        config,
        &connection_id,
        schema_name,
        table_name,
        models::LoadManagedTableRequest::new("replace".to_string(), upload.id.clone()),
    )
    .await
    .expect("load_managed_table should succeed");
    assert_eq!(loaded.schema_name, schema_name);
    assert_eq!(loaded.table_name, table_name);
    assert_eq!(loaded.row_count, 3, "fixture has 3 rows");

    // The profile is populated by an async sync triggered by the load, so it can
    // briefly 404 ("table may not be synced yet") right after load_managed_table.
    // Poll until it's ready (bounded), then assert it reflects the loaded data.
    let mut profile = None;
    for _ in 0..30 {
        match connections_api::get_table_profile(config, &connection_id, schema_name, table_name)
            .await
        {
            Ok(p) => {
                profile = Some(p);
                break;
            }
            Err(err) if common::status_of(&err) == Some(404) => {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            Err(err) => panic!("get_table_profile should succeed: {err:?}"),
        }
    }
    let profile = profile.expect("get_table_profile did not become available within 30s");
    assert_eq!(profile.schema, schema_name);
    assert_eq!(profile.table, table_name);
    assert_eq!(profile.row_count, 3);

    // Refresh catalog metadata for the managed connection.
    let mut refresh_req = models::RefreshRequest::new();
    refresh_req.connection_id = Some(Some(connection_id.clone()));
    refresh_api::refresh(config, refresh_req)
        .await
        .expect("refresh should succeed");

    // purge_table_cache and delete_managed_table both return () on success.
    connections_api::purge_table_cache(config, &connection_id, schema_name, table_name)
        .await
        .expect("purge_table_cache should succeed");
    connections_api::delete_managed_table(config, &connection_id, schema_name, table_name)
        .await
        .expect("delete_managed_table should succeed");

    // Tear down the scratch database (and its managed catalog).
    databases_api::delete_database(config, &database_id)
        .await
        .expect("delete_database should succeed");
}
