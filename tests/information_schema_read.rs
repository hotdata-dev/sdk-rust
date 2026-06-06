//! Scenario: information_schema_read.
//!
//! Defined in www.hotdata.dev/api/test-scenarios.yaml — read-only. The
//! information_schema endpoint returns catalog/schema/table metadata visible to
//! the workspace; scoped to the seeded connection, verify its tables appear.

mod common;

use hotdata::apis::information_schema_api;

#[tokio::test]
async fn information_schema_read() {
    let (client, connection_id) = skip_if_no_connection!();
    let config = client.configuration();

    let resp = information_schema_api::information_schema(
        config,
        Some(&connection_id), // scope to the seeded connection
        None,                 // schema
        None,                 // table
        Some(true),           // include_columns
        Some(50),             // limit
        None,                 // cursor
    )
    .await
    .expect("information_schema should succeed");

    // `count` reflects the number of tables in this page.
    assert_eq!(
        resp.count as usize,
        resp.tables.len(),
        "count ({}) should match the returned tables ({})",
        resp.count,
        resp.tables.len()
    );

    if resp.tables.is_empty() {
        eprintln!(
            "information_schema_read: seeded connection has no synced tables yet \
             (empty information_schema tolerated)"
        );
        return;
    }

    for t in &resp.tables {
        assert!(!t.schema.is_empty(), "table missing a schema name");
        assert!(!t.table.is_empty(), "table missing a table name");
        assert!(
            !t.connection.is_empty(),
            "table {}.{} missing its owning connection",
            t.schema,
            t.table
        );
        // We asked for columns; synced tables should expose them.
        if t.synced {
            let columns = t.columns.as_ref().and_then(|c| c.as_ref());
            assert!(
                columns.is_some_and(|c| !c.is_empty()),
                "synced table {}.{} should report columns when include_columns=true",
                t.schema,
                t.table
            );
        }
    }
}
