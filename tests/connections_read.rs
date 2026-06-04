//! Scenario: connections_read.
//!
//! Read-only lifecycle ops on the seeded connection — get, list, health check,
//! and cache purge. Does not create or delete connections in prod (would
//! require real datastore credentials in CI secrets).

mod common;

use hotdata::apis::connections_api;

#[tokio::test]
async fn connections_read() {
    let (client, connection_id) = skip_if_no_connection!();
    let config = client.configuration();

    let detail = connections_api::get_connection(config, &connection_id)
        .await
        .expect("get_connection should succeed");
    assert_eq!(detail.id, connection_id);
    assert!(!detail.source_type.is_empty(), "expected a source_type");
    assert!(!detail.name.is_empty(), "expected a connection name");

    let listing = connections_api::list_connections(config)
        .await
        .expect("list_connections should succeed");
    assert!(
        listing.connections.iter().any(|c| c.id == connection_id),
        "seeded connection {connection_id} not in list_connections"
    );

    let health = connections_api::check_connection_health(config, &connection_id)
        .await
        .expect("check_connection_health should succeed");
    assert_eq!(health.connection_id, connection_id);
    assert!(
        health.healthy,
        "seeded connection unhealthy: {:?}",
        health.error
    );

    // purge_connection_cache returns () on success.
    connections_api::purge_connection_cache(config, &connection_id)
        .await
        .expect("purge_connection_cache should succeed");
}
