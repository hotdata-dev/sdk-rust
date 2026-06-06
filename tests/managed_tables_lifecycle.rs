//! Scenario: managed_tables_lifecycle.
//!
//! Defined in www.hotdata.dev/api/test-scenarios.yaml — create a database (which
//! auto-provisions a managed catalog), declare a schema and table on it, upload a
//! small parquet file, load it into the table via load_managed_table, then delete
//! the table and the database. Self-cleaning — touches no seeded data.
//!
//! refresh / get_table_profile / purge_table_cache are deliberately NOT exercised
//! here: runtimedb rejects all three against a managed catalog (they are valid
//! only for real source connections, covered by source_table_refresh_profile).
//! The 3-row parquet payload is a committed fixture (tests/fixtures/), so the test
//! needs no parquet writer at runtime.

mod common;

use hotdata::apis::{connections_api, databases_api, uploads_api};
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

    // Delete the managed table, then tear down the scratch database (and catalog).
    connections_api::delete_managed_table(config, &connection_id, schema_name, table_name)
        .await
        .expect("delete_managed_table should succeed");
    databases_api::delete_database(config, &database_id)
        .await
        .expect("delete_database should succeed");
}
