//! Scenario: results_arrow (flagship).
//!
//! Submit a small query, poll until the result is ready, then fetch it as Arrow
//! IPC via `Accept: application/vnd.apache.arrow.stream` and `?format=arrow`.
//! Verifies the buffered variant round-trips schema and values, the streaming
//! variant yields the same RecordBatches, and offset/limit pagination forwards.
//!
//! Gated behind the `arrow` feature (mirrors sdk-python's `pyarrow`
//! importorskip). With the feature off this test binary compiles to an empty
//! set, so `cargo test` (default features) passes.

#![cfg(feature = "arrow")]

mod common;

use std::time::{Duration, Instant};

use arrow_array::cast::AsArray;
use arrow_array::types::Int64Type;
use arrow_array::{Array, RecordBatch};
use hotdata::apis::query_runs_api;
use hotdata::{models, QueryOutcome};

const POLL_TIMEOUT: Duration = Duration::from_secs(60);
const POLL_INTERVAL: Duration = Duration::from_secs(1);

fn is_terminal(status: &str) -> bool {
    matches!(status, "succeeded" | "failed" | "cancelled")
}

/// Flatten a single-column Int64 across all batches into a Vec<i64>.
fn int_column(batches: &[RecordBatch], name: &str) -> Vec<i64> {
    let mut out = Vec::new();
    for batch in batches {
        let idx = batch
            .schema()
            .index_of(name)
            .unwrap_or_else(|_| panic!("column {name} missing from batch schema"));
        let col = batch.column(idx).as_primitive::<Int64Type>();
        for i in 0..col.len() {
            out.push(col.value(i));
        }
    }
    out
}

/// Flatten a single-column Utf8 across all batches into a Vec<String>.
fn str_column(batches: &[RecordBatch], name: &str) -> Vec<String> {
    let mut out = Vec::new();
    for batch in batches {
        let idx = batch
            .schema()
            .index_of(name)
            .unwrap_or_else(|_| panic!("column {name} missing from batch schema"));
        let col = batch.column(idx).as_string::<i32>();
        for i in 0..col.len() {
            out.push(col.value(i).to_string());
        }
    }
    out
}

fn total_rows(batches: &[RecordBatch]) -> usize {
    batches.iter().map(|b| b.num_rows()).sum()
}

#[tokio::test]
async fn results_arrow() {
    let client = skip_if_no_creds!();
    let config = client.configuration();

    // Submit asynchronously (mirrors sdk-python) and scope to the shared
    // `sdkci-shared` database via the `database_id` body field — queries
    // require a database or the server returns 400 "a database is required".
    let database_id = common::shared_database_id(&client).await;
    // ORDER BY makes the row order deterministic — a bare UNION ALL has no
    // guaranteed order, so the [1, 2] / offset assertions below would be flaky.
    let mut request = models::QueryRequest::new(
        "SELECT 1 AS x, 'hello' AS msg UNION ALL SELECT 2, 'world' ORDER BY x".to_string(),
    );
    request.r#async = Some(true);
    request.async_after_ms = Some(Some(1000));
    request.database_id = Some(Some(database_id));
    // `submit_query` recovers the run id whether the query ran inline (HTTP 200)
    // or went async (HTTP 202); the enhanced `client.query` reports a 202 as
    // `QueryError::Async`, so the async submission path uses `submit_query`.
    let outcome = client
        .submit_query(request, None)
        .await
        .expect("submit_query should succeed");
    let query_run_id = match outcome {
        QueryOutcome::Inline(resp) => resp.query_run_id,
        QueryOutcome::Submitted(resp) => resp.query_run_id,
        other => panic!("unexpected query outcome: {other:?}"),
    };
    assert!(!query_run_id.is_empty(), "expected a query_run_id");

    // Poll the run to a terminal/succeeded state and capture its result_id.
    let deadline = Instant::now() + POLL_TIMEOUT;
    let mut run: Option<models::QueryRunInfo> = None;
    while Instant::now() < deadline {
        let current = query_runs_api::get_query_run(config, &query_run_id)
            .await
            .expect("get_query_run should succeed");
        let terminal = is_terminal(&current.status);
        run = Some(current);
        if terminal {
            break;
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
    let run = run.expect("expected at least one get_query_run response");
    assert_eq!(
        run.status, "succeeded",
        "expected succeeded, got {}: {:?}",
        run.status, run.error_message
    );
    let result_id = run
        .result_id
        .flatten()
        .expect("succeeded run must expose a result_id");

    // Wait for the result to reach `ready` before fetching as Arrow —
    // get_result_arrow returns ArrowError::NotReady on a 202.
    let deadline = Instant::now() + POLL_TIMEOUT;
    let mut ready = false;
    while Instant::now() < deadline {
        let result = client
            .get_result(&result_id)
            .await
            .expect("get_result should succeed");
        if result.status == "ready" {
            ready = true;
            break;
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
    assert!(ready, "result {result_id} never became ready");

    // Buffered: full set of RecordBatches.
    let buffered = client
        .get_result_arrow(&result_id, None, None)
        .await
        .expect("get_result_arrow should succeed");
    assert_eq!(total_rows(&buffered.batches), 2, "expected 2 rows");

    let columns: Vec<String> = buffered
        .schema
        .fields()
        .iter()
        .map(|f| f.name().to_string())
        .collect();
    assert!(
        columns.iter().any(|c| c == "x") && columns.iter().any(|c| c == "msg"),
        "expected columns x and msg, got {columns:?}"
    );
    assert_eq!(int_column(&buffered.batches, "x"), vec![1, 2]);
    assert_eq!(
        str_column(&buffered.batches, "msg"),
        vec!["hello".to_string(), "world".to_string()]
    );
    // X-Total-Row-Count is present when status is ready.
    assert_eq!(
        buffered.total_row_count,
        Some(2),
        "expected X-Total-Row-Count of 2"
    );

    // Streaming: same data via the per-batch iterator.
    let stream = client
        .stream_result_arrow(&result_id, None, None)
        .await
        .expect("stream_result_arrow should succeed");
    let streamed: Vec<RecordBatch> = stream
        .read_all()
        .expect("streaming reader should decode all batches")
        .batches;
    assert_eq!(total_rows(&streamed), 2);
    assert_eq!(int_column(&streamed, "x"), vec![1, 2]);
    assert_eq!(
        str_column(&streamed, "msg"),
        vec!["hello".to_string(), "world".to_string()]
    );

    // Pagination forwards correctly: offset=1, limit=1 -> just the second row.
    let sliced = client
        .get_result_arrow(&result_id, Some(1), Some(1))
        .await
        .expect("get_result_arrow with offset/limit should succeed");
    assert_eq!(total_rows(&sliced.batches), 1);
    assert_eq!(int_column(&sliced.batches, "x"), vec![2]);
}
