//! Enhanced query helpers for `POST /v1/query` (#640 / #688).
//!
//! The auto-generated [`crate::apis::query_api::query`] speaks the raw contract.
//! This module layers two behaviors the bounded-memory query contract introduces
//! on top of it, transparently — mirroring `hotdata/query.py` from the Python
//! SDK so all SDKs behave comparably:
//!
//! * **429 admission-shedding retry.** Under concurrent load the server may shed
//!   a query with HTTP 429 + `Retry-After` (error code `OVERLOADED`).
//!   [`execute_query`] retries automatically — honoring `Retry-After` when
//!   present, otherwise bounded exponential backoff with jitter — under an
//!   overall deadline budget. When the budget or retry count is exhausted it
//!   returns [`QueryError::Overloaded`], distinct from a generic 503 so callers
//!   can branch on overload.
//!
//! * **Transparent truncation auto-follow.** A large result comes back with
//!   `truncated = true` and only a bounded preview in `rows`; the full result is
//!   persisted out-of-band under `result_id` (#640). [`execute_query`] polls that
//!   result to `ready` and materializes the full row set into the returned
//!   [`QueryResponse`]. Configurable [`QueryConfig::max_auto_rows`] /
//!   [`QueryConfig::max_auto_bytes`] guards mean an unbounded result is never
//!   silently pulled into client memory — past a guard it returns
//!   [`ResultError::TooLarge`], pointing callers at the streaming Arrow API
//!   ([`crate::arrow`]).
//!
//! # Choosing a query method
//!
//! [`Client`](crate::Client) exposes a few entry points; pick by what you need:
//!
//! | Method | Use when |
//! |--------|----------|
//! | [`query`](crate::Client::query) | The common case: run SQL, get rows back (429 retry + auto-follow applied). |
//! | [`query_in`](crate::Client::query_in) | Same, but scope the query to a database (`X-Database-Id`). |
//! | [`query_preview`](crate::Client::query_preview) | You want only the bounded inline preview — skip auto-follow for this call. |
//! | [`query_with`](crate::Client::query_with) | Per-call control: a custom [`QueryConfig`] (and optional database scope). |
//! | [`submit_query`](crate::Client::submit_query) | Asynchronous submission (`async = true`): returns an acknowledgement to poll. |
//! | [`query_to_arrow`](crate::Client::query_to_arrow) | Run a query and fetch its persisted result as Arrow in one call. |
//! | `client.queries().execute()` / [`apis::query_api::query`](crate::apis::query_api::query) | The raw generated op: 429 retry applies (per `Configuration::retry`), but without auto-follow. |
//!
//! Instance defaults come from [`ClientBuilder::query_config`](crate::ClientBuilder::query_config);
//! tweak one for a single call by chaining [`QueryConfig`]'s `with_*` setters off
//! [`Client::query_config`](crate::Client::query_config) and passing the result to
//! [`query_with`](crate::Client::query_with).
//!
//! This module is hand-written and listed in `.openapi-generator-ignore`, so it
//! survives client regeneration.

// `QueryError` carries the generated `Error<T>` (status + body String + typed
// entity) by value in its `Submit`/`Poll` variants, exactly as every generated
// `apis::*` op returns `Result<_, Error<T>>`. Boxing it here only would make the
// error surface inconsistent with the rest of the SDK for no real benefit, so we
// accept clippy's large-Err lint for this module.
#![allow(clippy::result_large_err)]

use std::time::{Duration, Instant};

use reqwest::StatusCode;
use serde_json::Value;

use crate::apis::configuration::Configuration;
use crate::apis::query_api::QueryError as GeneratedQueryError;
use crate::apis::results_api::GetResultError;
use crate::apis::{query_runs_api, results_api, Error, ResponseContent};
use crate::client::{SESSION_ID_HEADER, WORKSPACE_ID_HEADER};
use crate::http::{backoff_delay, is_pre_response_transport_error, parse_retry_after};
use crate::models::{AsyncQueryResponse, QueryRequest, QueryResponse, ResultsFormatQuery};
use crate::status::ResultStatus;

/// HTTP 429: too many concurrent queries (admission shedding). The server tags
/// the body with error code [`OVERLOADED_ERROR_CODE`]; retry keys off the status
/// code since 429 is unambiguous and the body is not always parsed first.
const HTTP_TOO_MANY_REQUESTS: StatusCode = StatusCode::TOO_MANY_REQUESTS;

/// The error code the server tags a shed query body with (HTTP 429). Exposed for
/// callers that want to assert on it; retry itself keys off the status code.
pub const OVERLOADED_ERROR_CODE: &str = "OVERLOADED";

/// Default ceiling on rows auto-follow will materialize into memory. Large
/// enough that ordinary results follow transparently, small enough that an
/// accidental `SELECT *` over a huge table fails loudly instead of OOMing the
/// client. `None` opts into unbounded materialization on that axis.
pub const DEFAULT_MAX_AUTO_ROWS: u64 = 1_000_000;

/// Default ceiling on estimated bytes auto-follow will materialize. A client-RAM
/// guard (estimated from page content), distinct from the server's inline
/// preview cap. `None` opts into unbounded materialization on that axis.
pub const DEFAULT_MAX_AUTO_BYTES: u64 = 64 * 1024 * 1024;

/// Controls 429 (`OVERLOADED`) retry behavior.
#[derive(Debug, Clone, Copy)]
pub struct RetryPolicy {
    /// Maximum retry attempts after the initial request.
    pub max_retries: u32,
    /// Base delay for exponential backoff when the server sends no `Retry-After`.
    pub base_backoff: Duration,
    /// Cap on any single backoff delay (does NOT cap an honored `Retry-After`).
    pub max_backoff: Duration,
    /// Overall budget across all attempts; once exceeded (or a computed delay
    /// would push past it) retries stop.
    pub deadline: Duration,
    /// Fraction of the delay added as random jitter, to avoid a thundering herd
    /// of retries hitting the freed admission slot at once.
    pub jitter: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        RetryPolicy {
            max_retries: 5,
            base_backoff: Duration::from_millis(500),
            max_backoff: Duration::from_secs(30),
            deadline: Duration::from_secs(120),
            jitter: 0.5,
        }
    }
}

/// Controls result-lifecycle polling and full-result pagination during
/// truncation auto-follow.
#[derive(Debug, Clone, Copy)]
pub struct PollPolicy {
    /// Initial poll interval; doubles each poll up to `max_backoff`.
    pub base_backoff: Duration,
    /// Cap on the poll interval.
    pub max_backoff: Duration,
    /// Overall budget for a result to reach `ready`.
    pub deadline: Duration,
    /// Rows fetched per page when paginating the full result.
    pub page_size: i32,
}

impl Default for PollPolicy {
    fn default() -> Self {
        PollPolicy {
            base_backoff: Duration::from_millis(500),
            max_backoff: Duration::from_secs(5),
            deadline: Duration::from_secs(120),
            page_size: 50_000,
        }
    }
}

/// Per-instance (and per-call) configuration for the enhanced query: retry,
/// polling, and the auto-follow guards.
///
/// Obtain the instance default via [`Client::query_config`](crate::Client::query_config)
/// and clone-with-override for a per-call tweak:
///
/// ```no_run
/// # use hotdata::{Client, QueryConfig};
/// # async fn run(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
/// // Disable auto-follow for one call — return the bounded preview unchanged.
/// let cfg = QueryConfig { auto_follow: false, ..client.query_config().clone() };
/// let preview = client
///     .query_with(hotdata::models::QueryRequest::new("select 1".into()), None, &cfg)
///     .await?;
/// # let _ = preview;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct QueryConfig {
    /// 429 retry policy.
    pub retry: RetryPolicy,
    /// Result polling / pagination policy.
    pub poll: PollPolicy,
    /// When `true` (default), a truncated response is followed to its full row
    /// set; when `false`, the bounded preview is returned unchanged.
    pub auto_follow: bool,
    /// Row ceiling for auto-follow; `None` = unbounded on the row axis.
    pub max_auto_rows: Option<u64>,
    /// Estimated-byte ceiling for auto-follow; `None` = unbounded on the byte axis.
    pub max_auto_bytes: Option<u64>,
}

impl Default for QueryConfig {
    fn default() -> Self {
        QueryConfig {
            retry: RetryPolicy::default(),
            poll: PollPolicy::default(),
            auto_follow: true,
            max_auto_rows: Some(DEFAULT_MAX_AUTO_ROWS),
            max_auto_bytes: Some(DEFAULT_MAX_AUTO_BYTES),
        }
    }
}

impl QueryConfig {
    /// Set [`auto_follow`](Self::auto_follow), returning the updated config. Pair
    /// with [`Client::query_config`](crate::Client::query_config) for a per-call
    /// tweak: `client.query_config().clone().with_auto_follow(false)`.
    #[must_use]
    pub fn with_auto_follow(mut self, auto_follow: bool) -> Self {
        self.auto_follow = auto_follow;
        self
    }

    /// Set [`max_auto_rows`](Self::max_auto_rows) (`None` = unbounded), returning
    /// the updated config.
    #[must_use]
    pub fn with_max_auto_rows(mut self, max_auto_rows: Option<u64>) -> Self {
        self.max_auto_rows = max_auto_rows;
        self
    }

    /// Set [`max_auto_bytes`](Self::max_auto_bytes) (`None` = unbounded),
    /// returning the updated config.
    #[must_use]
    pub fn with_max_auto_bytes(mut self, max_auto_bytes: Option<u64>) -> Self {
        self.max_auto_bytes = max_auto_bytes;
        self
    }

    /// Set the [`RetryPolicy`](Self::retry), returning the updated config.
    #[must_use]
    pub fn with_retry(mut self, retry: RetryPolicy) -> Self {
        self.retry = retry;
        self
    }

    /// Set the [`PollPolicy`](Self::poll), returning the updated config.
    #[must_use]
    pub fn with_poll(mut self, poll: PollPolicy) -> Self {
        self.poll = poll;
        self
    }
}

/// Which axis an auto-follow size guard tripped on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TooLargeKind {
    /// The row-count ceiling (`max_auto_rows`) was exceeded.
    Rows,
    /// The estimated-byte ceiling (`max_auto_bytes`) was exceeded.
    Bytes,
}

impl TooLargeKind {
    fn knob(self) -> &'static str {
        match self {
            TooLargeKind::Rows => "max_auto_rows",
            TooLargeKind::Bytes => "max_auto_bytes",
        }
    }
}

/// Result-lifecycle errors raised while auto-following a truncated result.
///
/// Mirrors the Python `ResultError` family (`Failed` / `Timeout` / `TooLarge` /
/// `Incomplete` / `Unavailable`) so callers can handle the whole family with one
/// `match` arm via [`QueryError::Result`].
///
/// Marked `#[non_exhaustive]`: new variants may be added without a breaking
/// change.
#[derive(Debug)]
#[non_exhaustive]
pub enum ResultError {
    /// The followed result reached terminal status `failed` (delivered as HTTP
    /// 409 by the server).
    Failed {
        /// The result id that failed.
        result_id: String,
        /// The server-provided failure message, when present.
        error_message: Option<String>,
    },
    /// The result did not reach `ready` within the poll deadline.
    Timeout {
        /// The result id being awaited.
        result_id: String,
        /// The last status observed before timing out.
        status: String,
        /// How long was waited before giving up.
        deadline: Duration,
    },
    /// Auto-follow would materialize more than the guard allows, on either axis.
    /// Stream the result instead via
    /// [`Client::stream_result_arrow`](crate::Client::stream_result_arrow), or
    /// raise (or set to `None`) the relevant guard.
    TooLarge {
        /// The result id that exceeded the guard.
        result_id: String,
        /// Which axis tripped (rows or bytes).
        kind: TooLargeKind,
        /// The offending count (row count or estimated bytes).
        observed: u64,
        /// The ceiling it exceeded.
        limit: u64,
    },
    /// Pagination could not retrieve the full result — the server returned an
    /// empty page before the known total was reached. Surfaced instead of
    /// silently returning a partial result.
    Incomplete {
        /// The result id whose pagination stalled.
        result_id: String,
        /// Rows fetched before the empty page.
        fetched: i64,
        /// Rows expected (the known total).
        expected: i64,
    },
    /// The response was truncated but carried no `result_id` to follow
    /// (persistence failed — see the response `warning`).
    Unavailable {
        /// The server-provided warning explaining why persistence was skipped.
        warning: Option<String>,
    },
}

impl std::fmt::Display for ResultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResultError::Failed {
                result_id,
                error_message,
            } => match error_message {
                Some(msg) => write!(f, "result {result_id} failed: {msg}"),
                None => write!(f, "result {result_id} failed"),
            },
            ResultError::Timeout {
                result_id,
                status,
                deadline,
            } => write!(
                f,
                "result {result_id} did not become ready within {deadline:?} \
                 (last status: {status})"
            ),
            ResultError::TooLarge {
                result_id,
                kind,
                observed,
                limit,
            } => {
                let desc = match kind {
                    TooLargeKind::Bytes => format!("~{observed} bytes (limit {limit})"),
                    TooLargeKind::Rows => format!("{observed} rows (limit {limit})"),
                };
                write!(
                    f,
                    "result {result_id} exceeds the auto-materialize limit: {desc}. \
                     Stream it with Client::stream_result_arrow, or raise (or set to \
                     None) {}.",
                    kind.knob()
                )
            }
            ResultError::Incomplete {
                result_id,
                fetched,
                expected,
            } => write!(
                f,
                "result {result_id} pagination stalled: fetched {fetched} of {expected} \
                 rows before the server returned an empty page"
            ),
            ResultError::Unavailable { warning } => {
                write!(
                    f,
                    "query result is truncated but no result_id is available to fetch \
                     the full result"
                )?;
                if let Some(w) = warning {
                    write!(f, ": {w}")?;
                }
                write!(f, ". Re-run with auto_follow disabled to use the preview.")
            }
        }
    }
}

impl std::error::Error for ResultError {}

/// Error returned by the enhanced query ([`Client::query`](crate::Client::query)
/// / [`execute_query`]).
///
/// Marked `#[non_exhaustive]`: new variants may be added without a breaking
/// change, so downstream `match`es should carry a wildcard arm.
#[derive(Debug)]
#[non_exhaustive]
pub enum QueryError {
    /// HTTP 429 (`OVERLOADED`) retries were exhausted (by retry count or by the
    /// deadline budget). Distinct from a generic 503 so callers can treat
    /// transient admission shedding differently from a hard resource error.
    Overloaded {
        /// How many requests were made before giving up.
        attempts: u32,
        /// The last 429 response, for context.
        source: Box<Error<GeneratedQueryError>>,
    },
    /// Query submission failed with a non-429 status, or a transport/decode
    /// error occurred. Carries the same error shape as the generated `query` op,
    /// so all non-overload errors propagate unchanged.
    Submit(Error<GeneratedQueryError>),
    /// [`query`](crate::Client::query) was called on a request with
    /// `async = true`. That opts into an asynchronous submission whose
    /// acknowledgement the synchronous-results path cannot return, so the request
    /// is rejected up front (before any round-trip). Use
    /// [`Client::submit_query`](crate::Client::submit_query) for async queries.
    AsyncRequested,
    /// The server fell back to asynchronous execution (HTTP 202) — e.g. a request
    /// with `async_after_ms` whose synchronous attempt timed out — so there are
    /// no inline rows to return. The acknowledgement (carrying the
    /// `query_run_id` to poll) is passed through untouched; or call
    /// [`Client::submit_query`](crate::Client::submit_query) directly.
    Async(Box<AsyncQueryResponse>),
    /// An API error while polling or paginating the result during auto-follow
    /// (other than the 409 that signals a failed result).
    Poll(Error<GetResultError>),
    /// A result-lifecycle error during auto-follow. Match this to handle the
    /// whole [`ResultError`] family at once.
    Result(ResultError),
}

impl From<ResultError> for QueryError {
    fn from(e: ResultError) -> Self {
        QueryError::Result(e)
    }
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryError::Overloaded { attempts, .. } => write!(
                f,
                "server overloaded: HTTP 429 OVERLOADED, gave up after {attempts} attempt(s)"
            ),
            QueryError::Submit(e) => write!(f, "query submission failed: {e}"),
            QueryError::AsyncRequested => write!(
                f,
                "query() is the synchronous-results path, but this request set \
                 async=true; use submit_query() to drive an async submission"
            ),
            QueryError::Async(resp) => write!(
                f,
                "query was submitted asynchronously (query_run_id={}); use submit_query \
                 to drive an async submission",
                resp.query_run_id
            ),
            QueryError::Poll(e) => write!(f, "failed to follow truncated result: {e}"),
            QueryError::Result(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for QueryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            QueryError::Overloaded { source, .. } => Some(source.as_ref()),
            QueryError::Submit(e) => Some(e),
            QueryError::Poll(e) => Some(e),
            QueryError::Result(e) => Some(e),
            QueryError::Async(_) | QueryError::AsyncRequested => None,
        }
    }
}

/// Either response shape `POST /v1/query` can return synchronously.
enum Submission {
    Inline(QueryResponse),
    Async(AsyncQueryResponse),
}

/// Raw outcome of one `POST /v1/query` round-trip: the status, the parsed
/// `Retry-After` (seconds) when present, and the body text.
struct RawResponse {
    status: StatusCode,
    retry_after: Option<Duration>,
    body: String,
}

/// Execute a query against `config`, retrying on 429 and auto-following
/// truncation per `qc`.
///
/// Mirrors `hotdata.query.QueryApi.query` from the Python SDK. The request is
/// built wire-identically to the generated `query` op (same workspace/session
/// scope headers, bearer auth, `base_path`/`/v1` join, JSON body); building it
/// here rather than calling the generated op lets the retry path read the
/// `Retry-After` header (the generated op discards response headers).
pub(crate) async fn execute_query(
    config: &Configuration,
    request: QueryRequest,
    x_database_id: Option<&str>,
    qc: &QueryConfig,
) -> Result<QueryResponse, QueryError> {
    // Reject an explicit async submission up front, before a wasted round-trip:
    // query() returns rows, not an acknowledgement, so async=true is a caller
    // mistake — point them at submit_query rather than submitting successfully
    // and then reporting that success as an error.
    if request.r#async == Some(true) {
        return Err(QueryError::AsyncRequested);
    }
    match submit_with_retry(config, &request, x_database_id, &qc.retry).await? {
        Submission::Async(async_resp) => Err(QueryError::Async(Box::new(async_resp))),
        Submission::Inline(resp) => {
            if !qc.auto_follow || !resp.truncated {
                Ok(resp)
            } else {
                materialize_full(config, resp, qc).await
            }
        }
    }
}

/// Submit the query, retrying HTTP 429 (and pre-response connection resets) per
/// `retry` until success, the retry count is exhausted, or the deadline budget
/// would be exceeded.
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
        let raw = match send_query(config, request, x_database_id).await {
            Ok(raw) => raw,
            // A pre-response connection error (e.g. a stale keep-alive socket
            // reset before this POST reached the server) is safe to retry on the
            // same budget as a 429: the server did no work, so the retry can't
            // double-execute. Any other submission error propagates unchanged.
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

        if raw.status != HTTP_TOO_MANY_REQUESTS {
            return interpret_response(raw);
        }

        // HTTP 429 OVERLOADED: retry within the count and deadline budgets.
        if attempt > retry.max_retries {
            return Err(overloaded(attempt, raw));
        }
        let delay = backoff_delay(retry, attempt, raw.retry_after);
        if start.elapsed() + delay > retry.deadline {
            return Err(overloaded(attempt, raw));
        }
        tokio::time::sleep(delay).await;
    }
}

/// Map a non-429 raw response to a [`Submission`] (200 inline / 202 async) or a
/// propagated submission error.
fn interpret_response(raw: RawResponse) -> Result<Submission, QueryError> {
    if raw.status == StatusCode::ACCEPTED {
        let async_resp: AsyncQueryResponse =
            serde_json::from_str(&raw.body).map_err(|e| QueryError::Submit(Error::from(e)))?;
        return Ok(Submission::Async(async_resp));
    }
    if raw.status.is_success() {
        let resp: QueryResponse =
            serde_json::from_str(&raw.body).map_err(|e| QueryError::Submit(Error::from(e)))?;
        return Ok(Submission::Inline(resp));
    }
    // Any other status: propagate with the generated op's error shape.
    let entity: Option<GeneratedQueryError> = serde_json::from_str(&raw.body).ok();
    Err(QueryError::Submit(Error::ResponseError(ResponseContent {
        status: raw.status,
        content: raw.body,
        entity,
    })))
}

/// Build the [`QueryError::Overloaded`] error from the last 429 response.
fn overloaded(attempts: u32, raw: RawResponse) -> QueryError {
    let entity: Option<GeneratedQueryError> = serde_json::from_str(&raw.body).ok();
    QueryError::Overloaded {
        attempts,
        source: Box::new(Error::ResponseError(ResponseContent {
            status: raw.status,
            content: raw.body,
            entity,
        })),
    }
}

/// Is a failed submission a pre-response connection error (stale keep-alive
/// reset before the request reached the server)? Only transport (`Reqwest`)
/// errors so classified are retried; a body/decode failure on this path is read
/// from `resp.text()` after a status arrived and is excluded by
/// [`is_pre_response_transport_error`].
fn is_retryable_submit_error(err: &Error<GeneratedQueryError>) -> bool {
    matches!(err, Error::Reqwest(e) if is_pre_response_transport_error(e))
}

/// Build and send one `POST /v1/query`, returning the status, parsed
/// `Retry-After`, and body. Mirrors the generated op's request construction.
async fn send_query(
    config: &Configuration,
    request: &QueryRequest,
    x_database_id: Option<&str>,
) -> Result<RawResponse, Error<GeneratedQueryError>> {
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
    req_builder = req_builder.json(request);

    let req = req_builder.build()?;
    crate::http_log::log_request(&req);
    let resp = config.client.execute(req).await?;
    let status = resp.status();
    crate::http_log::log_response_status(status);
    let retry_after = parse_retry_after(&resp);
    let body = resp.text().await?;
    crate::http_log::log_response_body(&body);
    Ok(RawResponse {
        status,
        retry_after,
        body,
    })
}

/// Apply the `X-Workspace-Id` / `X-Session-Id` API-key headers, mirroring the
/// generated op's `isKeyInHeader` blocks.
fn apply_apikey_headers(
    mut req_builder: reqwest::RequestBuilder,
    config: &Configuration,
) -> reqwest::RequestBuilder {
    for header in [WORKSPACE_ID_HEADER, SESSION_ID_HEADER] {
        if let Some(apikey) = config.api_keys.get(header) {
            let key = apikey.key.clone();
            let value = match apikey.prefix {
                Some(ref prefix) => format!("{prefix} {key}"),
                None => key,
            };
            req_builder = req_builder.header(header, value);
        }
    }
    req_builder
}

/// Poll `GET /v1/results/{id}` until the result is `ready`, returning the ready
/// response. Polls with `limit=0` so the readiness check fetches status only — a
/// `ready` result would otherwise return its full (unbounded) row set on every
/// poll, materializing the whole result into memory before the size guards act.
pub(crate) async fn wait_for_result(
    config: &Configuration,
    result_id: &str,
    poll: &PollPolicy,
) -> Result<crate::models::GetResultResponse, QueryError> {
    let start = Instant::now();
    let mut delay = poll.base_backoff;
    loop {
        // offset=None, limit=0 (status only), default format.
        let result = match results_api::get_result(config, result_id, None, Some(0), None).await {
            Ok(result) => result,
            // A failed result is delivered as HTTP 409: the generated client
            // raises on any non-2xx rather than returning status="failed". The
            // GetResultResponse body — and its error_message — rides on the
            // typed Status409 entity.
            Err(Error::ResponseError(rc)) if rc.status == StatusCode::CONFLICT => {
                let error_message = match rc.entity {
                    Some(GetResultError::Status409(r)) => r.error_message.flatten(),
                    _ => None,
                };
                return Err(ResultError::Failed {
                    result_id: result_id.to_owned(),
                    error_message,
                }
                .into());
            }
            Err(e) => return Err(QueryError::Poll(e)),
        };

        match ResultStatus::parse(&result.status) {
            ResultStatus::Ready => return Ok(result),
            // Defensive: a failure also surfaced via a 2xx body.
            ResultStatus::Failed => {
                return Err(ResultError::Failed {
                    result_id: result_id.to_owned(),
                    error_message: result.error_message.flatten(),
                }
                .into())
            }
            // pending / processing / unknown: back off and poll again, in budget.
            _ => {}
        }
        if start.elapsed() + delay > poll.deadline {
            return Err(ResultError::Timeout {
                result_id: result_id.to_owned(),
                status: result.status,
                deadline: poll.deadline,
            }
            .into());
        }
        tokio::time::sleep(delay).await;
        delay = (delay * 2).min(poll.max_backoff);
    }
}

/// Follow a truncated preview to its full row set, enforcing the size guards.
async fn materialize_full(
    config: &Configuration,
    mut preview: QueryResponse,
    qc: &QueryConfig,
) -> Result<QueryResponse, QueryError> {
    let result_id = match preview.result_id.clone().flatten() {
        Some(id) => id,
        None => {
            return Err(ResultError::Unavailable {
                warning: preview.warning.clone().flatten(),
            }
            .into())
        }
    };

    wait_for_result(config, &result_id, &qc.poll).await?;

    let total = authoritative_total(config, &preview).await;
    // Auto-follow does extra round-trips (poll + paginate) and materializes the
    // full result; log it so the hidden work behind one query() call is
    // observable without being noisy (info, not a warning).
    log::info!(
        target: "hotdata::query",
        "auto-following truncated result {} ({} rows) for query run {}",
        result_id,
        total.map(|t| t.to_string()).unwrap_or_else(|| "unknown".to_owned()),
        preview.query_run_id,
    );

    // Pre-check: if the total is known and exceeds the row guard, fail before
    // fetching anything.
    if let (Some(max), Some(total)) = (qc.max_auto_rows, total) {
        if total > 0 && total as u64 > max {
            return Err(ResultError::TooLarge {
                result_id,
                kind: TooLargeKind::Rows,
                observed: total as u64,
                limit: max,
            }
            .into());
        }
    }

    let rows = fetch_all_rows(config, &result_id, total, qc).await?;

    // Replace the bounded preview with the full row set. `truncated` /
    // `total_row_count` stay as the server reported them so the caller can still
    // see the inline body had been truncated; backfill the total if it was null.
    preview.rows = rows;
    if preview.total_row_count.flatten().is_none() {
        if let Some(t) = total {
            preview.total_row_count = Some(Some(t));
        }
    }
    Ok(preview)
}

/// The grand total row count. `total_row_count` is null while a truncated result
/// is still persisting, so fall back to the query-run record, which carries the
/// authoritative count once the run succeeds; else unknown.
async fn authoritative_total(config: &Configuration, preview: &QueryResponse) -> Option<i64> {
    if let Some(t) = preview.total_row_count.flatten() {
        return Some(t);
    }
    match query_runs_api::get_query_run(config, &preview.query_run_id).await {
        Ok(run) => run.row_count.flatten(),
        Err(_) => None,
    }
}

/// Paginate `GET /v1/results/{id}?format=json` from offset 0, accumulating every
/// row and enforcing both guards incrementally.
///
/// The inline preview is discarded and the result is re-fetched from offset 0:
/// the preview is not guaranteed to be an ordered prefix of the persisted result
/// (an unordered query could duplicate/skip rows if spliced), so re-fetching
/// guarantees consistency.
async fn fetch_all_rows(
    config: &Configuration,
    result_id: &str,
    total: Option<i64>,
    qc: &QueryConfig,
) -> Result<Vec<Vec<Value>>, QueryError> {
    let page_size = effective_page_size(qc.poll.page_size);
    let mut rows: Vec<Vec<Value>> = Vec::new();
    let mut byte_estimate: u64 = 0;
    let mut offset: i64 = 0;
    loop {
        let page = results_api::get_result(
            config,
            result_id,
            Some(checked_offset(offset, result_id)?),
            Some(page_size),
            Some(ResultsFormatQuery::Json),
        )
        .await
        .map_err(QueryError::Poll)?;
        let batch = page.rows.flatten().unwrap_or_default();
        let batch_len = batch.len();

        // Known total but the server returned nothing more: surface the gap
        // rather than silently returning a partial result.
        if let Some(total) = total {
            if batch.is_empty() && offset < total {
                return Err(ResultError::Incomplete {
                    result_id: result_id.to_owned(),
                    fetched: offset,
                    expected: total,
                }
                .into());
            }
        }

        if qc.max_auto_bytes.is_some() {
            byte_estimate += estimate_rows_bytes(&batch);
        }
        rows.extend(batch);

        // Enforce both guards during pagination, in case the total was unknown
        // up front (total_row_count null, query-run lookup failed).
        if let Some(max) = qc.max_auto_rows {
            if rows.len() as u64 > max {
                return Err(ResultError::TooLarge {
                    result_id: result_id.to_owned(),
                    kind: TooLargeKind::Rows,
                    observed: rows.len() as u64,
                    limit: max,
                }
                .into());
            }
        }
        if let Some(max) = qc.max_auto_bytes {
            if byte_estimate > max {
                return Err(ResultError::TooLarge {
                    result_id: result_id.to_owned(),
                    kind: TooLargeKind::Bytes,
                    observed: byte_estimate,
                    limit: max,
                }
                .into());
            }
        }

        offset += batch_len as i64;
        if let Some(total) = total {
            // When the total is known, completion is offset >= total; a short
            // (non-empty) page just means keep paging.
            if offset >= total {
                break;
            }
            continue;
        }
        // Total unknown: a short/empty page is the end of the stream.
        if batch_len < page_size as usize {
            break;
        }
    }
    Ok(rows)
}

/// The effective pagination page size: at least 1. A configured 0 (or negative)
/// `page_size` would request `limit <= 0`, receive an empty page, and — with the
/// total unknown — never advance the offset or hit the short-page break, looping
/// forever. Clamping guarantees each page makes progress (one row per page is
/// pathologically slow but terminates; the default is 50_000).
fn effective_page_size(page_size: i32) -> i32 {
    page_size.max(1)
}

/// Convert the accumulating `i64` page offset to the `i32` the results API
/// accepts. A result with more than `i32::MAX` rows cannot be paginated as JSON
/// (and materializing that into client memory would be absurd regardless), so
/// refuse it with a [`ResultError::TooLarge`] rather than silently wrapping the
/// offset and re-fetching the wrong pages.
fn checked_offset(offset: i64, result_id: &str) -> Result<i32, ResultError> {
    i32::try_from(offset).map_err(|_| ResultError::TooLarge {
        result_id: result_id.to_owned(),
        kind: TooLargeKind::Rows,
        observed: offset.max(0) as u64,
        limit: i32::MAX as u64,
    })
}

/// Rough in-memory size of a page of rows, used only for the byte guard.
///
/// Sums stringified cell lengths plus small per-cell/per-row overhead. A
/// conservative ceiling *signal*, not exact accounting — good enough to stop a
/// wide result before it exhausts RAM, cheap enough to run per page.
fn estimate_rows_bytes(batch: &[Vec<Value>]) -> u64 {
    let mut total: u64 = 0;
    for row in batch {
        for cell in row {
            total += cell_len(cell) + 2;
        }
        total += 2;
    }
    total
}

/// Approximate the stringified length of a single JSON cell.
fn cell_len(v: &Value) -> u64 {
    match v {
        Value::Null => 4, // "null"
        Value::String(s) => s.chars().count() as u64,
        other => other.to_string().len() as u64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apis::configuration::ApiKey;
    use crate::client::Client;
    #[cfg(unix)]
    use crate::test_support::reset_then_ok_server;
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Build a client pointed at `base_url` with a static bearer (no JWT mint)
    /// and a near-instant retry/poll config so the timing-sensitive tests run
    /// fast and deterministically (jitter = 0).
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
                max_retries: 5,
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

    fn preview_json(truncated: bool, result_id: Option<&str>, total: Option<i64>) -> Value {
        let mut body = json!({
            "columns": ["x"],
            "execution_time_ms": 1,
            "nullable": [false],
            "preview_row_count": 1,
            "query_run_id": "qrun1",
            "row_count": 1,
            "rows": [[1]],
            "truncated": truncated,
        });
        if let Some(id) = result_id {
            body["result_id"] = json!(id);
        }
        if let Some(t) = total {
            body["total_row_count"] = json!(t);
        }
        body
    }

    fn req() -> QueryRequest {
        QueryRequest::new("SELECT 1 AS x".to_owned())
    }

    // --- 429 retry -----------------------------------------------------------

    #[tokio::test]
    async fn passthrough_when_not_truncated() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .respond_with(ResponseTemplate::new(200).set_body_json(preview_json(
                false,
                Some("rslt1"),
                None,
            )))
            .mount(&server)
            .await;
        // No results endpoint mounted: any follow would 404/panic the assertion.

        let client = test_client(&server.uri(), fast_config());
        let resp = client.query(req()).await.expect("query should succeed");
        assert_eq!(resp.query_run_id, "qrun1");
        assert!(!resp.truncated);
    }

    /// A POST query whose pooled connection is reset before the request reaches
    /// the server must be retried transparently and then succeed — the server
    /// did no work, so the retry can't double-execute (#63).
    #[cfg(unix)]
    #[tokio::test]
    async fn query_retries_pre_response_reset_then_succeeds() {
        use std::sync::atomic::Ordering;
        let body = preview_json(false, Some("rslt1"), None).to_string();
        let (base, conns) = reset_then_ok_server(1, body);
        let client = test_client(&base, fast_config());
        let resp = client
            .query(req())
            .await
            .expect("pre-response reset on POST /v1/query should be retried, then succeed");
        assert_eq!(resp.query_run_id, "qrun1");
        // 1 reset + 1 success = 2 connections reached the wire.
        assert_eq!(conns.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn non_429_error_propagates_as_submit() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .respond_with(ResponseTemplate::new(400).set_body_json(json!({
                "error": {"code": "BAD_REQUEST", "message": "nope"}
            })))
            .mount(&server)
            .await;

        let client = test_client(&server.uri(), fast_config());
        let err = client.query(req()).await.expect_err("should error");
        match err {
            QueryError::Submit(Error::ResponseError(rc)) => {
                assert_eq!(rc.status, StatusCode::BAD_REQUEST);
            }
            other => panic!("expected Submit ResponseError, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn retry_then_succeed_honoring_retry_after() {
        let server = MockServer::start().await;
        // Two 429s with Retry-After: 0, then a 200.
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .respond_with(
                ResponseTemplate::new(429)
                    .insert_header("Retry-After", "0")
                    .set_body_json(json!({"error": {"code": "OVERLOADED"}})),
            )
            .up_to_n_times(2)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .respond_with(ResponseTemplate::new(200).set_body_json(preview_json(
                false,
                Some("rslt1"),
                None,
            )))
            .mount(&server)
            .await;

        let client = test_client(&server.uri(), fast_config());
        let resp = client
            .query(req())
            .await
            .expect("should succeed after retries");
        assert_eq!(resp.query_run_id, "qrun1");
    }

    #[tokio::test]
    async fn retries_exhausted_by_max_retries() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .respond_with(
                ResponseTemplate::new(429).set_body_json(json!({"error": {"code": "OVERLOADED"}})),
            )
            .mount(&server)
            .await;

        let qc = QueryConfig {
            retry: RetryPolicy {
                max_retries: 2,
                base_backoff: Duration::from_millis(1),
                max_backoff: Duration::from_millis(2),
                deadline: Duration::from_secs(30),
                jitter: 0.0,
            },
            ..fast_config()
        };
        let client = test_client(&server.uri(), qc);
        let err = client
            .query(req())
            .await
            .expect_err("should exhaust retries");
        match err {
            // 1 initial + 2 retries = attempt 3 trips the `> max_retries` guard.
            QueryError::Overloaded { attempts, .. } => assert_eq!(attempts, 3),
            other => panic!("expected Overloaded, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn retries_exhausted_by_deadline() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .respond_with(
                ResponseTemplate::new(429)
                    .insert_header("Retry-After", "100")
                    .set_body_json(json!({"error": {"code": "OVERLOADED"}})),
            )
            .mount(&server)
            .await;

        // Retry-After is 100s but the deadline budget is tiny, so the very first
        // computed delay overshoots and retries stop immediately.
        let qc = QueryConfig {
            retry: RetryPolicy {
                max_retries: 10,
                base_backoff: Duration::from_millis(1),
                max_backoff: Duration::from_secs(1),
                deadline: Duration::from_millis(10),
                jitter: 0.0,
            },
            ..fast_config()
        };
        let client = test_client(&server.uri(), qc);
        let err = client
            .query(req())
            .await
            .expect_err("should exhaust deadline");
        assert!(matches!(err, QueryError::Overloaded { attempts: 1, .. }));
    }

    // --- truncation auto-follow ---------------------------------------------

    #[tokio::test]
    async fn auto_follow_materializes_full_rows() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .respond_with(ResponseTemplate::new(200).set_body_json(preview_json(
                true,
                Some("rslt1"),
                Some(3),
            )))
            .mount(&server)
            .await;
        // Readiness poll (limit=0) -> ready.
        Mock::given(method("GET"))
            .and(path("/v1/results/rslt1"))
            .and(query_param("limit", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "result_id": "rslt1", "status": "ready"
            })))
            .mount(&server)
            .await;
        // Page 1 (offset=0, page_size=2).
        Mock::given(method("GET"))
            .and(path("/v1/results/rslt1"))
            .and(query_param("offset", "0"))
            .and(query_param("limit", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "result_id": "rslt1", "status": "ready", "rows": [[1], [2]]
            })))
            .mount(&server)
            .await;
        // Page 2 (offset=2).
        Mock::given(method("GET"))
            .and(path("/v1/results/rslt1"))
            .and(query_param("offset", "2"))
            .and(query_param("limit", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "result_id": "rslt1", "status": "ready", "rows": [[3]]
            })))
            .mount(&server)
            .await;

        let client = test_client(&server.uri(), fast_config());
        let resp = client
            .query(req())
            .await
            .expect("auto-follow should succeed");
        assert_eq!(resp.rows.len(), 3);
        assert_eq!(resp.rows[2], vec![json!(3)]);
        // truncated stays true; total backfilled / preserved.
        assert!(resp.truncated);
        assert_eq!(resp.total_row_count.flatten(), Some(3));
    }

    #[tokio::test]
    async fn auto_follow_falls_back_to_query_run_total() {
        let server = MockServer::start().await;
        // total_row_count null while persisting.
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .respond_with(ResponseTemplate::new(200).set_body_json(preview_json(
                true,
                Some("rslt1"),
                None,
            )))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/v1/results/rslt1"))
            .and(query_param("limit", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "result_id": "rslt1", "status": "ready"
            })))
            .mount(&server)
            .await;
        // Authoritative total comes from the query-run record.
        Mock::given(method("GET"))
            .and(path("/v1/query-runs/qrun1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "created_at": "2026-06-15T00:00:00Z",
                "id": "qrun1",
                "snapshot_id": "snap1",
                "sql_hash": "h",
                "sql_text": "SELECT 1 AS x",
                "status": "succeeded",
                "row_count": 1
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/v1/results/rslt1"))
            .and(query_param("offset", "0"))
            .and(query_param("limit", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "result_id": "rslt1", "status": "ready", "rows": [[1]]
            })))
            .mount(&server)
            .await;

        let client = test_client(&server.uri(), fast_config());
        let resp = client
            .query(req())
            .await
            .expect("auto-follow should succeed");
        assert_eq!(resp.rows.len(), 1);
        assert_eq!(resp.total_row_count.flatten(), Some(1));
    }

    #[tokio::test]
    async fn auto_follow_rows_guard_pre_check() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .respond_with(ResponseTemplate::new(200).set_body_json(preview_json(
                true,
                Some("rslt1"),
                Some(1000),
            )))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/v1/results/rslt1"))
            .and(query_param("limit", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "result_id": "rslt1", "status": "ready"
            })))
            .mount(&server)
            .await;

        let qc = QueryConfig {
            max_auto_rows: Some(10),
            ..fast_config()
        };
        let client = test_client(&server.uri(), qc);
        let err = client
            .query(req())
            .await
            .expect_err("should trip row guard");
        match err {
            QueryError::Result(ResultError::TooLarge {
                kind: TooLargeKind::Rows,
                observed,
                limit,
                ..
            }) => {
                assert_eq!(observed, 1000);
                assert_eq!(limit, 10);
            }
            other => panic!("expected TooLarge rows, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn auto_follow_bytes_guard_during_pagination() {
        let server = MockServer::start().await;
        // Unknown total so the pre-check is skipped and pagination runs.
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .respond_with(ResponseTemplate::new(200).set_body_json(preview_json(
                true,
                Some("rslt1"),
                None,
            )))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/v1/results/rslt1"))
            .and(query_param("limit", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "result_id": "rslt1", "status": "ready"
            })))
            .mount(&server)
            .await;
        // No query-run row_count -> total unknown.
        Mock::given(method("GET"))
            .and(path("/v1/query-runs/qrun1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "created_at": "2026-06-15T00:00:00Z",
                "id": "qrun1",
                "snapshot_id": "snap1",
                "sql_hash": "h",
                "sql_text": "SELECT 1 AS x",
                "status": "succeeded"
            })))
            .mount(&server)
            .await;
        // A full page of wide rows so the byte estimate trips on the first page.
        Mock::given(method("GET"))
            .and(path("/v1/results/rslt1"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "result_id": "rslt1", "status": "ready",
                "rows": [["aaaaaaaaaa"], ["bbbbbbbbbb"]]
            })))
            .mount(&server)
            .await;

        let qc = QueryConfig {
            max_auto_bytes: Some(8),
            ..fast_config()
        };
        let client = test_client(&server.uri(), qc);
        let err = client
            .query(req())
            .await
            .expect_err("should trip byte guard");
        assert!(matches!(
            err,
            QueryError::Result(ResultError::TooLarge {
                kind: TooLargeKind::Bytes,
                ..
            })
        ));
    }

    #[tokio::test]
    async fn auto_follow_failed_result_409() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .respond_with(ResponseTemplate::new(200).set_body_json(preview_json(
                true,
                Some("rslt1"),
                Some(3),
            )))
            .mount(&server)
            .await;
        // Readiness poll returns HTTP 409 (failed result).
        Mock::given(method("GET"))
            .and(path("/v1/results/rslt1"))
            .and(query_param("limit", "0"))
            .respond_with(ResponseTemplate::new(409).set_body_json(json!({
                "result_id": "rslt1", "status": "failed", "error_message": "boom"
            })))
            .mount(&server)
            .await;

        let client = test_client(&server.uri(), fast_config());
        let err = client
            .query(req())
            .await
            .expect_err("failed result should error");
        match err {
            QueryError::Result(ResultError::Failed { error_message, .. }) => {
                assert_eq!(error_message.as_deref(), Some("boom"));
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn auto_follow_missing_result_id_is_unavailable() {
        let server = MockServer::start().await;
        let mut body = preview_json(true, None, None);
        body["result_id"] = Value::Null; // explicit null = persistence failed
        body["warning"] = json!("catalog registration failed");
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&server)
            .await;

        let client = test_client(&server.uri(), fast_config());
        let err = client
            .query(req())
            .await
            .expect_err("missing result_id should error");
        match err {
            QueryError::Result(ResultError::Unavailable { warning }) => {
                assert_eq!(warning.as_deref(), Some("catalog registration failed"));
            }
            other => panic!("expected Unavailable, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn auto_follow_off_returns_preview() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .respond_with(ResponseTemplate::new(200).set_body_json(preview_json(
                true,
                Some("rslt1"),
                Some(3),
            )))
            .mount(&server)
            .await;
        // No results endpoint mounted: opting out must not touch it.

        let qc = QueryConfig {
            auto_follow: false,
            ..fast_config()
        };
        let client = test_client(&server.uri(), qc);
        let resp = client
            .query(req())
            .await
            .expect("preview should pass through");
        assert!(resp.truncated);
        assert_eq!(resp.rows.len(), 1); // bounded preview, not followed
    }

    #[tokio::test]
    async fn auto_follow_incomplete_pagination() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .respond_with(ResponseTemplate::new(200).set_body_json(preview_json(
                true,
                Some("rslt1"),
                Some(5),
            )))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/v1/results/rslt1"))
            .and(query_param("limit", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "result_id": "rslt1", "status": "ready"
            })))
            .mount(&server)
            .await;
        // First page returns 2 rows, then an empty page before reaching total=5.
        Mock::given(method("GET"))
            .and(path("/v1/results/rslt1"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "result_id": "rslt1", "status": "ready", "rows": [[1], [2]]
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/v1/results/rslt1"))
            .and(query_param("offset", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "result_id": "rslt1", "status": "ready", "rows": []
            })))
            .mount(&server)
            .await;

        let client = test_client(&server.uri(), fast_config());
        let err = client
            .query(req())
            .await
            .expect_err("stalled pagination should error");
        match err {
            QueryError::Result(ResultError::Incomplete {
                fetched, expected, ..
            }) => {
                assert_eq!(fetched, 2);
                assert_eq!(expected, 5);
            }
            other => panic!("expected Incomplete, got {other:?}"),
        }
    }

    // --- helpers -------------------------------------------------------------

    #[test]
    fn retry_after_parses_integer_seconds() {
        // We exercise parse_retry_after indirectly via backoff honoring.
        let retry = RetryPolicy {
            jitter: 0.0,
            ..RetryPolicy::default()
        };
        let d = backoff_delay(&retry, 1, Some(Duration::from_secs(7)));
        assert_eq!(d, Duration::from_secs(7)); // honored exactly, jitter 0
    }

    #[test]
    fn backoff_is_exponential_without_retry_after() {
        let retry = RetryPolicy {
            base_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(100),
            jitter: 0.0,
            ..RetryPolicy::default()
        };
        assert_eq!(backoff_delay(&retry, 1, None), Duration::from_secs(1));
        assert_eq!(backoff_delay(&retry, 2, None), Duration::from_secs(2));
        assert_eq!(backoff_delay(&retry, 3, None), Duration::from_secs(4));
    }

    #[test]
    fn byte_estimate_is_positive_and_grows() {
        let small = estimate_rows_bytes(&[vec![json!(1)]]);
        let big = estimate_rows_bytes(&[vec![json!("aaaaaaaaaa")]]);
        assert!(small > 0);
        assert!(big > small);
    }

    #[test]
    fn checked_offset_refuses_overflow() {
        // In range: passed through unchanged.
        assert_eq!(checked_offset(0, "r").unwrap(), 0);
        assert_eq!(checked_offset(i32::MAX as i64, "r").unwrap(), i32::MAX);
        // Past i32::MAX: refused as TooLarge(rows) instead of silently wrapping
        // the offset to a negative number and re-fetching the wrong pages.
        match checked_offset(i32::MAX as i64 + 1, "rslt1") {
            Err(ResultError::TooLarge {
                kind: TooLargeKind::Rows,
                observed,
                result_id,
                ..
            }) => {
                assert_eq!(observed, i32::MAX as u64 + 1);
                assert_eq!(result_id, "rslt1");
            }
            other => panic!("expected TooLarge rows, got {other:?}"),
        }
    }

    #[test]
    fn page_size_is_clamped_to_make_progress() {
        // The pagination loop, with the total unknown, terminates only on a page
        // shorter than `page_size`. A page_size of 0 (or negative) would request
        // limit<=0, get an empty page, and never satisfy `batch_len < page_size`
        // — an infinite loop. Clamping to >=1 guarantees forward progress.
        assert_eq!(effective_page_size(0), 1);
        assert_eq!(effective_page_size(-5), 1);
        assert_eq!(effective_page_size(50_000), 50_000);
    }

    #[test]
    fn query_config_with_setters_override_fields() {
        let cfg = QueryConfig::default()
            .with_auto_follow(false)
            .with_max_auto_rows(None)
            .with_max_auto_bytes(Some(123));
        assert!(!cfg.auto_follow);
        assert_eq!(cfg.max_auto_rows, None);
        assert_eq!(cfg.max_auto_bytes, Some(123));
        // Untouched fields keep their defaults.
        assert_eq!(cfg.retry.max_retries, RetryPolicy::default().max_retries);
    }

    #[tokio::test]
    async fn async_request_is_rejected_before_any_request() {
        // Point at an unroutable address: if query() hit the wire it would return
        // a Submit transport error. AsyncRequested proves it short-circuited
        // before any round-trip.
        let client = test_client("http://127.0.0.1:1", fast_config());
        let mut request = req();
        request.r#async = Some(true);
        let err = client
            .query(request)
            .await
            .expect_err("async=true must be rejected by query()");
        assert!(matches!(err, QueryError::AsyncRequested));
    }

    #[tokio::test]
    async fn query_preview_does_not_follow_truncation() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/query"))
            .respond_with(ResponseTemplate::new(200).set_body_json(preview_json(
                true,
                Some("rslt1"),
                Some(3),
            )))
            .mount(&server)
            .await;
        // No results endpoint mounted: query_preview must not touch it.

        let client = test_client(&server.uri(), fast_config());
        let resp = client
            .query_preview(req())
            .await
            .expect("preview should pass through");
        assert!(resp.truncated);
        assert_eq!(resp.rows.len(), 1);
    }
}
