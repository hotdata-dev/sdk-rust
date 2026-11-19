//! Scenario: query_arrow.
//!
//! Run a query with `Client::query_arrow` and check the rows come back as Arrow
//! in one call, with the run and result ids the server sends as response
//! headers; that a non-default dialect is accepted; and that a result too large
//! to send inline is followed to the Results API, or refused when auto-follow
//! is off.
//!
//! Gated behind the `arrow` feature, like `results_arrow`.

#![cfg(feature = "arrow")]

mod common;

use arrow_array::cast::AsArray;
use arrow_array::types::Int64Type;
use arrow_array::RecordBatch;
use hotdata::{models, QueryError, ResultError};

/// Flatten a single Int64 column across all batches.
fn int_column(batches: &[RecordBatch], name: &str) -> Vec<i64> {
    let mut out = Vec::new();
    for batch in batches {
        let idx = batch.schema().index_of(name).expect("column present");
        let col = batch.column(idx).as_primitive::<Int64Type>();
        out.extend(col.values().iter().copied());
    }
    out
}

/// A result well past what an inline response holds.
const LARGE_SQL: &str =
    "SELECT value AS n, repeat('x', 200) AS pad FROM generate_series(1, 300000)";

#[tokio::test]
async fn query_arrow() {
    let client = skip_if_no_creds!();
    let database_id = common::shared_database_id(&client).await;

    let request = models::QueryRequest::new(
        "SELECT 1 AS x, 'hello' AS msg UNION ALL SELECT 2, 'world' ORDER BY x".to_string(),
    );
    let out = client
        .query_arrow(request, Some(&database_id))
        .await
        .expect("query_arrow should succeed");

    assert_eq!(int_column(&out.batches, "x"), vec![1, 2]);
    assert!(out.schema.field_with_name("msg").is_ok());
    assert!(out.query_run_id.is_some(), "expected X-Query-Run-Id");
    assert_eq!(out.total_row_count, Some(2));
    assert!(!out.followed);
    // No table read, so no rollup outcome to report.
    assert_eq!(out.rollup, None);
}

#[tokio::test]
async fn query_arrow_clickhouse_dialect() {
    let client = skip_if_no_creds!();
    let database_id = common::shared_database_id(&client).await;

    let mut request = models::QueryRequest::new("SELECT toInt64(41) + 1 AS n".to_string());
    request.dialect = Some(Some("clickhouse".to_string()));
    let out = client
        .query_arrow(request, Some(&database_id))
        .await
        .expect("a ClickHouse-dialect query should succeed");
    assert_eq!(int_column(&out.batches, "n"), vec![42]);
}

#[tokio::test]
async fn query_arrow_follows_large_result() {
    let client = skip_if_no_creds!();
    let database_id = common::shared_database_id(&client).await;

    let out = client
        .query_arrow(
            models::QueryRequest::new(LARGE_SQL.to_string()),
            Some(&database_id),
        )
        .await
        .expect("a large result should be followed");
    assert!(out.followed);
    assert_eq!(out.num_rows(), 300_000);
    assert_eq!(out.total_row_count, Some(300_000));
    assert!(out.result_id.is_some());

    // Auto-follow off: the ids come back instead of the rows.
    let qc = client.query_config().clone().with_auto_follow(false);
    let err = client
        .query_arrow_with(
            models::QueryRequest::new(LARGE_SQL.to_string()),
            Some(&database_id),
            &qc,
        )
        .await
        .expect_err("a large result is not sent inline");
    assert!(
        matches!(err, QueryError::Result(ResultError::NotInline { .. })),
        "expected NotInline, got {err:?}"
    );
}
