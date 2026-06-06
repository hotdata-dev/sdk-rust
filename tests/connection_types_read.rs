//! Scenario: connection_types_read.
//!
//! Defined in www.hotdata.dev/api/test-scenarios.yaml — read-only. list_connection_types
//! returns the connector catalog, and get_connection_type fetches one by name
//! with its config schema. No fixtures created.

mod common;

use hotdata::apis::connection_types_api;

#[tokio::test]
async fn connection_types_read() {
    let client = skip_if_no_creds!();
    let config = client.configuration();

    let listing = connection_types_api::list_connection_types(config)
        .await
        .expect("list_connection_types should succeed");
    assert!(
        !listing.connection_types.is_empty(),
        "expected a non-empty connector catalog"
    );
    for ct in &listing.connection_types {
        assert!(!ct.name.is_empty(), "connection type missing a name");
        assert!(
            !ct.label.is_empty(),
            "connection type {} missing a label",
            ct.name
        );
    }

    // Fetch one by name and confirm the detail echoes the catalog entry.
    let first = &listing.connection_types[0];
    let detail = connection_types_api::get_connection_type(config, &first.name)
        .await
        .expect("get_connection_type should succeed");
    assert_eq!(detail.name, first.name);
    assert_eq!(detail.label, first.label);

    // A fabricated connector name must not resolve to a real type.
    let bogus = connection_types_api::get_connection_type(config, "sdkci-not-a-connector").await;
    match bogus {
        Err(err) => assert_eq!(
            common::status_of(&err),
            Some(404),
            "expected 404 for an unknown connection type, got {err:?}"
        ),
        Ok(_) => panic!("get_connection_type should 404 for an unknown connector name"),
    }
}
