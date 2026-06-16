//! Scenario: query_async_polling.
//!
//! Submit a query, poll get_query_run until terminal status, fetch results, and
//! verify list_query_runs / list_results surface the run.
//!
//! `submit_query` surfaces both response shapes: a fast query comes back inline
//! (HTTP 200) with a `query_run_id`, a slower one as an async acknowledgement
//! (HTTP 202). Either way we recover the `query_run_id` and drive the full
//! polling loop against `get_query_run` to exercise the async surface and to be
//! robust to the run not being instantly terminal. (The enhanced `client.query`
//! is the synchronous-results path and reports a 202 as `QueryError::Async`, so
//! the async path uses `submit_query`.)

mod common;

use std::time::{Duration, Instant};

use hotdata::apis::query_runs_api;
use hotdata::{models, QueryOutcome};

const POLL_TIMEOUT: Duration = Duration::from_secs(60);
const POLL_INTERVAL: Duration = Duration::from_secs(1);

fn is_terminal(status: &str) -> bool {
    matches!(status, "succeeded" | "failed" | "cancelled")
}

#[tokio::test]
async fn query_async_polling() {
    let client = skip_if_no_creds!();
    let config = client.configuration();

    // Submit asynchronously (mirrors sdk-python): `async=true` with a small
    // `async_after_ms` exercises the async submission path and returns a
    // query_run_id to poll. Queries require a database scope, so target the
    // shared `sdkci-shared` database via the `database_id` body field —
    // otherwise the server returns 400 "a database is required".
    let database_id = common::shared_database_id(&client).await;
    let mut request = models::QueryRequest::new("SELECT 1 AS x".to_string());
    request.r#async = Some(true);
    request.async_after_ms = Some(Some(1000));
    request.database_id = Some(Some(database_id));
    let outcome = client
        .submit_query(request, None)
        .await
        .expect("submit_query should succeed");
    // The run id is on both outcomes — inline (ran sync) or async submission.
    let query_run_id = match outcome {
        QueryOutcome::Inline(resp) => resp.query_run_id,
        QueryOutcome::Submitted(resp) => resp.query_run_id,
        other => panic!("unexpected query outcome: {other:?}"),
    };
    assert!(!query_run_id.is_empty(), "expected a query_run_id");

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
    assert!(
        is_terminal(&run.status),
        "query {query_run_id} did not reach terminal status within {POLL_TIMEOUT:?}; \
         last status was {}",
        run.status
    );
    assert_eq!(
        run.status, "succeeded",
        "expected succeeded, got {}: {:?}",
        run.status, run.error_message
    );
    assert_eq!(run.row_count, Some(Some(1)));

    let runs_listing = query_runs_api::list_query_runs(config, Some(50), None, None, None)
        .await
        .expect("list_query_runs should succeed");
    assert!(
        runs_listing.query_runs.iter().any(|r| r.id == query_run_id),
        "query run {query_run_id} not surfaced by list_query_runs"
    );

    if let Some(Some(result_id)) = run.result_id {
        let result = client
            .get_result(&result_id)
            .await
            .expect("get_result should succeed");
        assert_eq!(result.result_id, result_id);
        assert!(
            matches!(result.status.as_str(), "ready" | "processing"),
            "unexpected result status {}",
            result.status
        );
        if result.status == "ready" {
            assert_eq!(result.row_count, Some(Some(1)));
            assert_eq!(result.rows, Some(Some(vec![vec![serde_json::json!(1)]])));
        }

        // ResultInfo (list_results) exposes the id as `id`, not `result_id`.
        let results_listing = client
            .list_results(Some(50), None)
            .await
            .expect("list_results should succeed");
        assert!(
            results_listing.results.iter().any(|r| r.id == result_id),
            "result {result_id} not surfaced by list_results"
        );
    }
}
