//! Arrow in one call from `POST /v1/query`.
//!
//! [`crate::Client::query_to_arrow`] runs a query and then fetches its saved
//! result as Arrow: two round trips at least. The query endpoint can answer with
//! an Arrow IPC stream itself, when asked with
//! `Accept: application/vnd.apache.arrow.stream`, and sends the values the JSON
//! body would carry (run id, result id, row count, timing, warning, rollup
//! outcome) as response headers. [`crate::Client::query_arrow`] makes that one
//! request and returns the batches with those values, as [`ArrowQueryResult`] —
//! mirroring `QueryApi.query_arrow` from the Python SDK.
//!
//! Two behaviors match [`crate::query`]:
//!
//! * **429 retry**, on the same [`RetryPolicy`].
//! * **Large results.** A result too large to send inline is never sent as
//!   partial Arrow; the server answers 202 with an `X-Result-Id` header. With
//!   [`QueryConfig::auto_follow`] on, the saved result is awaited and read from
//!   the Results API, under the [`QueryConfig::max_auto_rows`] guard. With it
//!   off, the call returns [`ResultError::NotInline`] carrying the ids.
//!
//! This module is hand-written and listed in `.openapi-generator-ignore`, so it
//! survives client regeneration.

// Same rationale as `crate::query`: `QueryError` carries the generated `Error<T>`
// by value, like every generated op.
#![allow(clippy::result_large_err)]

use std::io::Cursor;
use std::time::Instant;

use arrow_array::RecordBatch;
use arrow_ipc::reader::StreamReader;
use arrow_schema::SchemaRef;
use reqwest::header::HeaderMap;
use reqwest::StatusCode;

use crate::apis::configuration::Configuration;
use crate::apis::query_api::QueryError as GeneratedQueryError;
use crate::apis::{query_runs_api, Error, ResponseContent};
use crate::arrow::{open_result_arrow, ArrowError, ARROW_STREAM_MEDIA_TYPE};
use crate::http::{backoff_delay, parse_retry_after};
use crate::models::{AsyncQueryResponse, QueryRequest, RollupUsage};
use crate::query::{
    apply_apikey_headers, is_retryable_submit_error, wait_for_result, QueryConfig, QueryError,
    ResultError, RetryPolicy, TooLargeKind,
};

/// A query result as Arrow, with the values the JSON body would carry.
#[derive(Debug, Clone)]
pub struct ArrowQueryResult {
    /// All record batches of the full result, in stream order.
    pub batches: Vec<RecordBatch>,
    /// The schema of the result (shared across all batches).
    pub schema: SchemaRef,
    /// ID of the query run.
    pub query_run_id: Option<String>,
    /// ID of the saved result, when one was saved.
    pub result_id: Option<String>,
    /// Rows in the full result.
    pub total_row_count: Option<i64>,
    /// Query execution time in milliseconds, when the server reported it.
    pub execution_time_ms: Option<i64>,
    /// A problem that did not fail the query, if any.
    pub warning: Option<String>,
    /// Whether a rollup answered the query. `None` when no table the query read
    /// has a rollup, or rollups were turned off for the query.
    pub rollup: Option<RollupUsage>,
    /// `true` when the result was too large to send inline and was read from
    /// the Results API instead (extra round trips).
    pub followed: bool,
}

impl ArrowQueryResult {
    /// Total number of rows across all batches.
    pub fn num_rows(&self) -> usize {
        self.batches.iter().map(RecordBatch::num_rows).sum()
    }
}

/// What one `POST /v1/query` asking for Arrow came back with.
enum Submission {
    /// 200: the rows, inline.
    Inline(ArrowQueryResult),
    /// 202: the run ticket, and the saved result's id when the server named one.
    Deferred {
        ticket: AsyncQueryResponse,
        result_id: Option<String>,
    },
}

/// Execute a query against `config` and return the rows as Arrow, retrying on
/// 429 and following a result too large to send inline, per `qc`.
pub(crate) async fn execute_query_arrow(
    config: &Configuration,
    request: QueryRequest,
    x_database_id: Option<&str>,
    qc: &QueryConfig,
) -> Result<ArrowQueryResult, QueryError> {
    // Same up-front rejection as `execute_query`: this path returns rows.
    if request.r#async == Some(true) {
        return Err(QueryError::AsyncRequested);
    }
    match submit_with_retry(config, &request, x_database_id, &qc.retry).await? {
        Submission::Inline(result) => Ok(result),
        // A 202 with no result to read is a plain async acknowledgement.
        Submission::Deferred {
            ticket,
            result_id: None,
        } => Err(QueryError::Async(Box::new(ticket))),
        Submission::Deferred {
            ticket,
            result_id: Some(result_id),
        } => {
            if !qc.auto_follow {
                return Err(ResultError::NotInline {
                    result_id,
                    query_run_id: ticket.query_run_id,
                }
                .into());
            }
            // The follow-up reads are scoped to the query's database: the header
            // value when present, otherwise the request-body `database_id`.
            let effective_db = x_database_id
                .map(str::to_owned)
                .or_else(|| request.database_id.clone().flatten())
                .unwrap_or_default();
            follow(config, ticket.query_run_id, result_id, &effective_db, qc).await
        }
    }
}

/// Submit the query, retrying HTTP 429 and pre-response connection resets on
/// the same budget as `crate::query::submit_with_retry`.
async fn submit_with_retry(
    config: &Configuration,
    request: &QueryRequest,
    x_database_id: Option<&str>,
    retry: &RetryPolicy,
) -> Result<Submission, QueryError> {
    let start = Instant::now();
    let mut attempt: u32 = 0;
    loop {
        attempt += 1;
        let resp = match send_query(config, request, x_database_id).await {
            Ok(resp) => resp,
            Err(e) => {
                if attempt <= retry.max_retries && is_retryable_submit_error(&e) {
                    let delay = backoff_delay(retry, attempt, None);
                    if start.elapsed() + delay <= retry.deadline {
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                }
                return Err(QueryError::Submit(e));
            }
        };

        let status = resp.status();
        if status == StatusCode::OK {
            return read_inline(resp).await.map(Submission::Inline);
        }

        // Everything else is JSON.
        let retry_after = parse_retry_after(&resp);
        let result_id = header_str(resp.headers(), "X-Result-Id");
        let body = resp
            .text()
            .await
            .map_err(|e| QueryError::Submit(Error::from(e)))?;
        crate::http_log::log_response_body(&body);

        if status == StatusCode::ACCEPTED {
            let ticket: AsyncQueryResponse =
                serde_json::from_str(&body).map_err(|e| QueryError::Submit(Error::from(e)))?;
            return Ok(Submission::Deferred { ticket, result_id });
        }

        let error = || {
            let entity: Option<GeneratedQueryError> = serde_json::from_str(&body).ok();
            Error::ResponseError(ResponseContent {
                status,
                content: body.clone(),
                entity,
            })
        };
        if status != StatusCode::TOO_MANY_REQUESTS {
            return Err(QueryError::Submit(error()));
        }

        // HTTP 429 OVERLOADED: retry within the count and deadline budgets.
        let overloaded = |attempts| QueryError::Overloaded {
            attempts,
            source: Box::new(error()),
        };
        if attempt > retry.max_retries {
            return Err(overloaded(attempt));
        }
        let delay = backoff_delay(retry, attempt, retry_after);
        if start.elapsed() + delay > retry.deadline {
            return Err(overloaded(attempt));
        }
        tokio::time::sleep(delay).await;
    }
}

/// Build and send one `POST /v1/query` asking for Arrow. Same request as
/// `crate::query::send_query`, plus the `Accept` header.
async fn send_query(
    config: &Configuration,
    request: &QueryRequest,
    x_database_id: Option<&str>,
) -> Result<reqwest::Response, Error<GeneratedQueryError>> {
    let uri_str = format!("{}/v1/query", config.base_path);
    let mut req_builder = config.client.request(reqwest::Method::POST, &uri_str);

    if let Some(ref user_agent) = config.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(param_value) = x_database_id {
        req_builder = req_builder.header("X-Database-Id", param_value.to_string());
    }
    req_builder = apply_apikey_headers(req_builder, config);
    if let Some(token) = config.resolve_bearer_token().await {
        req_builder = req_builder.bearer_auth(token);
    }
    req_builder = req_builder
        .header(reqwest::header::ACCEPT, ARROW_STREAM_MEDIA_TYPE)
        .json(request);

    let req = req_builder.build()?;
    crate::http_log::log_request(&req);
    let resp = config.client.execute(req).await?;
    crate::http_log::log_response_status(resp.status());
    Ok(resp)
}

/// Decode a 200 Arrow body and pick the result's values off its headers.
async fn read_inline(resp: reqwest::Response) -> Result<ArrowQueryResult, QueryError> {
    let headers = resp.headers().clone();
    let bytes = resp.bytes().await.map_err(ArrowError::from)?;
    let reader = StreamReader::try_new(Cursor::new(bytes), None).map_err(ArrowError::from)?;
    let schema = reader.schema();
    let mut batches = Vec::new();
    for batch in reader {
        batches.push(batch.map_err(ArrowError::from)?);
    }
    Ok(ArrowQueryResult {
        batches,
        schema,
        query_run_id: header_str(&headers, "X-Query-Run-Id"),
        result_id: header_str(&headers, "X-Result-Id"),
        total_row_count: header_i64(&headers, "X-Total-Row-Count"),
        execution_time_ms: header_i64(&headers, "X-Execution-Time-Ms"),
        warning: header_str(&headers, "X-Query-Warning"),
        rollup: header_str(&headers, "X-Rollup").and_then(|v| parse_rollup(&v)),
        followed: false,
    })
}

/// Wait for the saved result, then read all of it as Arrow, enforcing the row
/// guard.
async fn follow(
    config: &Configuration,
    query_run_id: String,
    result_id: String,
    x_database_id: &str,
    qc: &QueryConfig,
) -> Result<ArrowQueryResult, QueryError> {
    wait_for_result(config, &result_id, x_database_id, &qc.poll).await?;

    // The run record carries what the inline headers would have: the total,
    // timing, warning and rollup outcome. Best effort: the rows matter more.
    let run = query_runs_api::get_query_run(config, &query_run_id, x_database_id)
        .await
        .ok();
    let total = run.as_ref().and_then(|r| r.row_count.flatten());
    log::info!(
        target: "hotdata::query",
        "following result {} ({} rows) as Arrow for query run {}",
        result_id,
        total.map(|t| t.to_string()).unwrap_or_else(|| "unknown".to_owned()),
        query_run_id,
    );

    let too_large = |observed: u64, limit: u64| ResultError::TooLarge {
        result_id: result_id.clone(),
        kind: TooLargeKind::Rows,
        observed,
        limit,
    };
    if let (Some(max), Some(total)) = (qc.max_auto_rows, total) {
        if total > 0 && total as u64 > max {
            return Err(too_large(total as u64, max).into());
        }
    }

    // Ask for one row past the guard at most: enough to detect the overflow
    // when the total is unknown, without downloading an oversized result. The
    // stream decodes off the socket, so rows are counted as they arrive.
    let limit = qc
        .max_auto_rows
        .map(|max| i64::try_from(max.saturating_add(1)).unwrap_or(i64::MAX));
    let mut stream = open_result_arrow(config, &result_id, x_database_id, None, limit).await?;
    let schema = stream.schema();
    let mut batches = Vec::new();
    let mut fetched: u64 = 0;
    while let Some(batch) = stream.next_batch().await? {
        fetched += batch.num_rows() as u64;
        if let Some(max) = qc.max_auto_rows {
            if fetched > max {
                return Err(too_large(fetched, max).into());
            }
        }
        batches.push(batch);
    }

    Ok(ArrowQueryResult {
        batches,
        schema,
        total_row_count: total.or(i64::try_from(fetched).ok()),
        execution_time_ms: run.as_ref().and_then(|r| r.execution_time_ms.flatten()),
        warning: run
            .as_ref()
            .and_then(|r| r.warning_message.clone().flatten()),
        rollup: run.as_ref().and_then(|r| r.rollup.flatten()),
        query_run_id: Some(query_run_id),
        result_id: Some(result_id),
        followed: true,
    })
}

fn header_str(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
}

fn header_i64(headers: &HeaderMap, name: &str) -> Option<i64> {
    header_str(headers, name).and_then(|v| v.trim().parse().ok())
}

/// The `X-Rollup` value as a [`RollupUsage`]. A value this client does not know
/// yet is dropped: the answer is the same either way, so it must not fail the
/// query.
fn parse_rollup(value: &str) -> Option<RollupUsage> {
    serde_json::from_value(serde_json::Value::String(value.trim().to_owned())).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apis::configuration::ApiKey;
    use crate::client::{Client, WORKSPACE_ID_HEADER};
    use crate::query::PollPolicy;
    use arrow_array::Int64Array;
    use arrow_ipc::writer::StreamWriter;
    use arrow_schema::{DataType, Field, Schema};
    use serde_json::json;
    use std::sync::Arc;
    use std::time::Duration;
    use wiremock::matchers::{body_partial_json, header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_client(base_url: &str, qc: QueryConfig) -> Client {
        let mut configuration = Configuration {
            base_path: base_url.to_owned(),
            user_agent: Some("hotdata-rust-test".to_owned()),
            bearer_access_token: Some("test-bearer".to_owned()),
            ..Configuration::default()
        };
        configuration.api_keys.insert(
            WORKSPACE_ID_HEADER.to_owned(),
            ApiKey {
                prefix: None,
                key: "ws_test".to_owned(),
            },
        );
        Client::from_configuration(configuration).with_query_config(qc)
    }

    /// A fast, deterministic config: tiny backoffs, no jitter.
    fn fast_config() -> QueryConfig {
        QueryConfig {
            retry: RetryPolicy {
                max_retries: 2,
                base_backoff: Duration::from_millis(1),
                max_backoff: Duration::from_millis(5),
                deadline: Duration::from_secs(30),
                jitter: 0.0,
            },
            poll: PollPolicy {
                base_backoff: Duration::from_millis(1),
                max_backoff: Duration::from_millis(5),
                deadline: Duration::from_secs(30),
                page_size: 2,
            },
            ..QueryConfig::default()
        }
    }

    /// An Arrow IPC stream of one `n: Int64` column holding 1, 2, 3.
    fn ipc_bytes() -> Vec<u8> {
        let schema = Arc::new(Schema::new(vec![Field::new("n", DataType::Int64, false)]));
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(Int64Array::from(vec![1, 2, 3]))],
        )
        .unwrap();
        let mut buf: Vec<u8> = Vec::new();
        {
            let mut writer = StreamWriter::try_new(&mut buf, &schema).unwrap();
            writer.write(&batch).unwrap();
            writer.finish().unwrap();
        }
        buf
    }

    fn arrow_response() -> ResponseTemplate {
        ResponseTemplate::new(200).set_body_raw(ipc_bytes(), ARROW_STREAM_MEDIA_TYPE)
    }

    fn req() -> QueryRequest {
        let mut request = QueryRequest::new("SELECT n FROM t".to_owned());
        request.dialect = Some(Some("clickhouse".to_owned()));
        request
    }

    /// Mount the 202 "too large to inline" answer to the query.
    async fn mount_too_large(server: &MockServer, result_id: Option<&str>) {
        let mut response = ResponseTemplate::new(202).set_body_json(json!({
            "query_run_id": "qrun1",
            "status": "running",
            "status_url": "/v1/query-runs/qrun1",
            "reason": "the result is larger than an inline response can hold",
        }));
        if let Some(id) = result_id {
            response = response.insert_header("X-Result-Id", id);
        }
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .respond_with(response)
            .mount(server)
            .await;
    }

    async fn mount_ready(server: &MockServer) {
        Mock::given(method("GET"))
            .and(path("/v1/results/rslt1"))
            .and(query_param("limit", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "result_id": "rslt1", "status": "ready"
            })))
            .mount(server)
            .await;
    }

    async fn mount_query_run(server: &MockServer, row_count: i64) {
        Mock::given(method("GET"))
            .and(path("/v1/query-runs/qrun1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "qrun1",
                "created_at": "2026-01-01T00:00:00Z",
                "snapshot_id": "snap1",
                "sql_hash": "h",
                "sql_text": "SELECT 1",
                "status": "succeeded",
                "result_id": "rslt1",
                "row_count": row_count,
                "execution_time_ms": 42,
                "rollup": "used",
            })))
            .mount(server)
            .await;
    }

    /// Mount the saved result as Arrow, matched only at the given `limit`.
    async fn mount_result_arrow(server: &MockServer, limit: &str) {
        Mock::given(method("GET"))
            .and(path("/v1/results/rslt1"))
            .and(query_param("format", "arrow"))
            .and(query_param("limit", limit))
            .respond_with(arrow_response())
            .mount(server)
            .await;
    }

    #[tokio::test]
    async fn returns_batches_and_header_values() {
        let server = MockServer::start().await;
        // Matches only a request that asks for Arrow, in the caller's dialect,
        // scoped to the database: anything else 404s and fails the test.
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .and(header("Accept", ARROW_STREAM_MEDIA_TYPE))
            .and(header("X-Database-Id", "db1"))
            .and(body_partial_json(json!({"dialect": "clickhouse"})))
            .respond_with(
                arrow_response()
                    .insert_header("X-Query-Run-Id", "qrun1")
                    .insert_header("X-Result-Id", "rslt1")
                    .insert_header("X-Total-Row-Count", "3")
                    .insert_header("X-Execution-Time-Ms", "7")
                    .insert_header("X-Query-Warning", "careful")
                    .insert_header("X-Rollup", "not_matched"),
            )
            .expect(1)
            .mount(&server)
            .await;

        let client = test_client(&server.uri(), fast_config());
        let out = client.query_arrow(req(), Some("db1")).await.unwrap();

        assert_eq!(out.num_rows(), 3);
        assert_eq!(out.schema.field(0).name(), "n");
        assert_eq!(out.query_run_id.as_deref(), Some("qrun1"));
        assert_eq!(out.result_id.as_deref(), Some("rslt1"));
        assert_eq!(out.total_row_count, Some(3));
        assert_eq!(out.execution_time_ms, Some(7));
        assert_eq!(out.warning.as_deref(), Some("careful"));
        assert_eq!(out.rollup, Some(RollupUsage::NotMatched));
        assert!(!out.followed);
    }

    #[tokio::test]
    async fn absent_and_unknown_headers_are_none() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .respond_with(arrow_response().insert_header("X-Rollup", "some_new_value"))
            .mount(&server)
            .await;
        let client = test_client(&server.uri(), fast_config());
        let out = client.query_arrow(req(), None).await.unwrap();
        assert_eq!(out.rollup, None);
        assert_eq!(out.warning, None);
        assert_eq!(out.result_id, None);
    }

    #[tokio::test]
    async fn retries_429_then_succeeds() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .respond_with(
                ResponseTemplate::new(429)
                    .insert_header("Retry-After", "0")
                    .set_body_json(json!({"error": {"code": "OVERLOADED", "message": "busy"}})),
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .respond_with(arrow_response())
            .mount(&server)
            .await;
        let client = test_client(&server.uri(), fast_config());
        let out = client.query_arrow(req(), None).await.unwrap();
        assert_eq!(out.num_rows(), 3);
    }

    #[tokio::test]
    async fn exhausted_429_is_overloaded() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .respond_with(
                ResponseTemplate::new(429)
                    .insert_header("Retry-After", "0")
                    .set_body_json(json!({"error": {"code": "OVERLOADED", "message": "busy"}})),
            )
            .expect(3) // initial + 2 retries
            .mount(&server)
            .await;
        let client = test_client(&server.uri(), fast_config());
        match client.query_arrow(req(), None).await {
            Err(QueryError::Overloaded { attempts, .. }) => assert_eq!(attempts, 3),
            other => panic!("expected Overloaded, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn timeout_propagates_with_status() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .respond_with(
                ResponseTemplate::new(408)
                    .set_body_json(json!({"error": {"code": "TIMEOUT", "message": "too slow"}})),
            )
            .mount(&server)
            .await;
        let client = test_client(&server.uri(), fast_config());
        match client.query_arrow(req(), None).await {
            Err(QueryError::Submit(Error::ResponseError(rc))) => {
                assert_eq!(rc.status, StatusCode::REQUEST_TIMEOUT);
                assert!(rc.content.contains("too slow"));
            }
            other => panic!("expected Submit(408), got {other:?}"),
        }
    }

    #[tokio::test]
    async fn async_request_is_rejected_before_any_request() {
        let server = MockServer::start().await;
        let client = test_client(&server.uri(), fast_config());
        let mut request = req();
        request.r#async = Some(true);
        assert!(matches!(
            client.query_arrow(request, None).await,
            Err(QueryError::AsyncRequested)
        ));
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn too_large_follows_to_results_api() {
        let server = MockServer::start().await;
        mount_too_large(&server, Some("rslt1")).await;
        mount_ready(&server).await;
        mount_query_run(&server, 3).await;
        // Default guard is 1,000,000 rows: the read asks for one past it.
        mount_result_arrow(&server, "1000001").await;

        let client = test_client(&server.uri(), fast_config());
        let out = client.query_arrow(req(), Some("db1")).await.unwrap();

        assert!(out.followed);
        assert_eq!(out.num_rows(), 3);
        assert_eq!(out.query_run_id.as_deref(), Some("qrun1"));
        assert_eq!(out.result_id.as_deref(), Some("rslt1"));
        assert_eq!(out.total_row_count, Some(3));
        assert_eq!(out.execution_time_ms, Some(42));
        assert_eq!(out.rollup, Some(RollupUsage::Used));
        // Every follow-up is scoped to the query's database.
        for request in server.received_requests().await.unwrap() {
            assert_eq!(request.headers.get("X-Database-Id").unwrap(), "db1");
        }
    }

    #[tokio::test]
    async fn follow_row_guard_from_known_total() {
        let server = MockServer::start().await;
        mount_too_large(&server, Some("rslt1")).await;
        mount_ready(&server).await;
        mount_query_run(&server, 10).await;
        // No Arrow mock: the rows must never be fetched.

        let qc = fast_config().with_max_auto_rows(Some(5));
        let client = test_client(&server.uri(), qc);
        match client.query_arrow(req(), Some("db1")).await {
            Err(QueryError::Result(ResultError::TooLarge {
                observed, limit, ..
            })) => {
                assert_eq!((observed, limit), (10, 5));
            }
            other => panic!("expected TooLarge, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn follow_row_guard_holds_when_run_lookup_fails() {
        let server = MockServer::start().await;
        mount_too_large(&server, Some("rslt1")).await;
        mount_ready(&server).await;
        // No query-run mock: the lookup 404s, so the total is unknown and the
        // guard has to act on the rows as they arrive. limit = guard + 1.
        mount_result_arrow(&server, "3").await;

        let qc = fast_config().with_max_auto_rows(Some(2));
        let client = test_client(&server.uri(), qc);
        match client.query_arrow(req(), Some("db1")).await {
            Err(QueryError::Result(ResultError::TooLarge {
                observed, limit, ..
            })) => {
                assert_eq!((observed, limit), (3, 2));
            }
            other => panic!("expected TooLarge, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn follow_survives_failed_run_lookup() {
        let server = MockServer::start().await;
        mount_too_large(&server, Some("rslt1")).await;
        mount_ready(&server).await;
        mount_result_arrow(&server, "1000001").await;

        let client = test_client(&server.uri(), fast_config());
        let out = client.query_arrow(req(), Some("db1")).await.unwrap();
        assert_eq!(out.num_rows(), 3);
        assert_eq!(out.total_row_count, Some(3));
        assert_eq!(out.rollup, None);
    }

    #[tokio::test]
    async fn too_large_without_auto_follow() {
        let server = MockServer::start().await;
        mount_too_large(&server, Some("rslt1")).await;
        // Instance config follows; the per-call one turns it off.
        let client = test_client(&server.uri(), fast_config());
        let qc = client.query_config().clone().with_auto_follow(false);
        match client.query_arrow_with(req(), Some("db1"), &qc).await {
            Err(QueryError::Result(ResultError::NotInline {
                result_id,
                query_run_id,
            })) => {
                assert_eq!(result_id, "rslt1");
                assert_eq!(query_run_id, "qrun1");
            }
            other => panic!("expected NotInline, got {other:?}"),
        }
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn deferred_without_result_id_is_async() {
        let server = MockServer::start().await;
        mount_too_large(&server, None).await;
        let client = test_client(&server.uri(), fast_config());
        match client.query_arrow(req(), Some("db1")).await {
            Err(QueryError::Async(ticket)) => assert_eq!(ticket.query_run_id, "qrun1"),
            other => panic!("expected Async, got {other:?}"),
        }
    }
}
