//! Scenario: databases_lifecycle.
//!
//! Defined in www.hotdata.dev/api/test-scenarios.yaml — create a database
//! (metadata-only grouping), read it, confirm it appears in list_databases,
//! declare a schema and a table on its managed catalog, then delete it and
//! verify it's gone.

mod common;

use hotdata::apis::databases_api;
use hotdata::models;

#[tokio::test]
async fn databases_lifecycle() {
    let client = skip_if_no_creds!();
    let config = client.configuration();

    // A uniquely-named sdkci-* database we create and tear down here.
    let database_id = common::create_scratch_database(&client, "databases-lifecycle").await;

    let fetched = databases_api::get_database(config, &database_id)
        .await
        .expect("get_database should succeed");
    assert_eq!(fetched.id, database_id);

    let listing = databases_api::list_databases(config, None, None, None)
        .await
        .expect("list_databases should succeed");
    assert!(
        listing.databases.iter().any(|d| d.id == database_id),
        "created database {database_id} not present in list_databases"
    );

    // Declare a schema on the database's managed catalog. Identifiers are
    // SQL-scoped, so use underscore-only names (the database is unique per run,
    // so a fixed schema/table name can't collide across tests).
    let schema_name = "sdkci_schema";
    let schema = databases_api::add_database_schema(
        config,
        &database_id,
        models::AddManagedSchemaRequest::new(schema_name.to_string()),
    )
    .await
    .expect("add_database_schema should succeed");
    assert_eq!(schema.schema, schema_name);
    assert!(
        !schema.connection_id.is_empty(),
        "schema should report its managed-catalog connection"
    );

    // Declare a table on that schema.
    let table_name = "sdkci_table";
    let table = databases_api::add_database_table(
        config,
        &database_id,
        schema_name,
        models::AddManagedTableRequest::new(table_name.to_string()),
    )
    .await
    .expect("add_database_table should succeed");
    assert_eq!(table.table, table_name);
    assert_eq!(table.schema, schema_name);

    // Delete the database and assert it's gone.
    databases_api::delete_database(config, &database_id)
        .await
        .expect("delete_database should succeed");

    let after_delete = databases_api::get_database(config, &database_id).await;
    match after_delete {
        Err(err) => assert_eq!(
            common::status_of(&err),
            Some(404),
            "expected 404 after delete, got {err:?}"
        ),
        Ok(_) => panic!("get_database should fail with 404 after delete"),
    }
}
