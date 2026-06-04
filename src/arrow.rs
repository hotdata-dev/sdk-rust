//! Arrow IPC helpers for `GET /v1/results/{id}`.
//!
//! The auto-generated [`crate::apis::results_api::get_result`] understands the
//! `format=arrow` query parameter but cannot decode the
//! `application/vnd.apache.arrow.stream` response body — openapi-generator picks
//! the JSON content variant for status 200 and routes Arrow bytes through the
//! JSON deserializer, which fails with an "unsupported content type" error.
//!
//! This module mirrors `hotdata/arrow.py` from the Python SDK. It builds the
//! request exactly like the generated `get_result` (same URL, user-agent,
//! `X-Workspace-Id`/`X-Session-Id` API keys, and transparent JWT-exchanged
//! bearer token via
//! [`crate::apis::configuration::Configuration::resolve_bearer_token`]), adds
//! `Accept: application/vnd.apache.arrow.stream` plus `?format=arrow`, and
//! decodes the resulting IPC stream with `arrow-ipc`.
//!
//! Two entry points are provided:
//!
//! * [`get_result_arrow`] — buffers the full IPC stream and returns all
//!   [`RecordBatch`]es (the Rust analog of pyarrow `Table`; Rust has no
//!   `Table`, so we expose `Vec<RecordBatch>` plus the resolved [`SchemaRef`]).
//! * [`stream_result_arrow`] — returns an [`ArrowBatchStream`] that decodes one
//!   [`RecordBatch`] at a time, mirroring pyarrow's
//!   `RecordBatchStreamReader`. The body is still collected once (reqwest's
//!   async body is not a blocking `Read`); decoding is then lazy per batch.
//!
//! Enable with the `arrow` cargo feature (mirrors Python's `[arrow]` extra):
//!
//! ```toml
//! hotdata = { version = "0.1", features = ["arrow"] }
//! ```

use std::fmt;
use std::io::Cursor;

use arrow_array::RecordBatch;
use arrow_ipc::reader::StreamReader;
use arrow_schema::{ArrowError as IpcArrowError, SchemaRef};
use bytes::Bytes;

use crate::apis::configuration::Configuration;
use crate::client::{SESSION_ID_HEADER, WORKSPACE_ID_HEADER};

/// The Arrow IPC stream media type negotiated with the results endpoint.
pub const ARROW_STREAM_MEDIA_TYPE: &str = "application/vnd.apache.arrow.stream";

/// Errors returned by the Arrow result helpers.
///
/// Mirrors the status mapping declared for `GET /v1/results/{id}`:
/// 202 → [`ArrowError::NotReady`], 409 → [`ArrowError::Failed`],
/// 404 → [`ArrowError::NotFound`], 400 → [`ArrowError::InvalidParams`], and any
/// other non-success status → [`ArrowError::Http`].
///
/// Marked `#[non_exhaustive]`: new variants may be added without a breaking
/// change, so downstream `match`es should carry a wildcard arm.
#[derive(Debug)]
#[non_exhaustive]
pub enum ArrowError {
    /// HTTP 202: the result exists but is still `pending`/`processing`.
    ///
    /// Poll [`crate::apis::results_api::get_result`] (or
    /// [`crate::client::Client::get_result`]) until `status == "ready"` before
    /// fetching as Arrow. `retry_after` carries the server's `Retry-After`
    /// header in seconds when present.
    NotReady {
        /// The reported result status (e.g. `pending`, `processing`).
        status: String,
        /// The result ID echoed by the server (falls back to the request ID).
        result_id: String,
        /// The `Retry-After` value in seconds, when the server set one.
        retry_after: Option<u64>,
    },
    /// HTTP 409: the result computation failed.
    Failed {
        /// The server-provided failure message, when present.
        error_message: Option<String>,
    },
    /// HTTP 404: no result with the given ID.
    NotFound,
    /// HTTP 400: invalid `offset`/`limit` (or other bad request).
    InvalidParams {
        /// The raw response body describing the problem.
        message: String,
    },
    /// Any other non-success HTTP status.
    Http {
        /// The HTTP status code.
        status: reqwest::StatusCode,
        /// The raw response body.
        body: String,
    },
    /// A transport-level error from reqwest.
    Reqwest(reqwest::Error),
    /// An Arrow IPC decode error.
    Ipc(IpcArrowError),
}

impl fmt::Display for ArrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArrowError::NotReady {
                status, result_id, ..
            } => write!(
                f,
                "result {result_id} is not ready (status={status:?}); poll get_result \
                 until status=\"ready\" before fetching as Arrow"
            ),
            ArrowError::Failed { error_message } => match error_message {
                Some(msg) => write!(f, "result failed: {msg}"),
                None => write!(f, "result failed"),
            },
            ArrowError::NotFound => write!(f, "result not found"),
            ArrowError::InvalidParams { message } => {
                write!(f, "invalid parameters: {message}")
            }
            ArrowError::Http { status, body } => {
                write!(f, "unexpected HTTP status {status}: {body}")
            }
            ArrowError::Reqwest(e) => write!(f, "transport error: {e}"),
            ArrowError::Ipc(e) => write!(f, "arrow IPC decode error: {e}"),
        }
    }
}

impl std::error::Error for ArrowError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ArrowError::Reqwest(e) => Some(e),
            ArrowError::Ipc(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for ArrowError {
    fn from(e: reqwest::Error) -> Self {
        ArrowError::Reqwest(e)
    }
}

impl From<IpcArrowError> for ArrowError {
    fn from(e: IpcArrowError) -> Self {
        ArrowError::Ipc(e)
    }
}

/// A fully-buffered Arrow result.
///
/// The Rust analog of pyarrow's `Table`: all [`RecordBatch`]es are decoded into
/// memory, alongside the resolved [`SchemaRef`] and the result metadata headers.
#[derive(Debug, Clone)]
pub struct ArrowResult {
    /// All decoded record batches, in stream order.
    pub batches: Vec<RecordBatch>,
    /// The schema of the result (shared across all batches).
    pub schema: SchemaRef,
    /// `X-Total-Row-Count`: total rows in the full result, ignoring
    /// offset/limit. `None` if the header was absent or unparseable.
    pub total_row_count: Option<i64>,
    /// The `rel="next"` URL from the RFC 5988 `Link` header, when a finite
    /// `limit` did not reach the end of the result.
    pub next_link: Option<String>,
}

impl ArrowResult {
    /// Total number of rows across all buffered batches (respects offset/limit).
    ///
    /// This is the count of rows actually returned in this page; for the full
    /// result size use [`ArrowResult::total_row_count`].
    pub fn num_rows(&self) -> usize {
        self.batches.iter().map(RecordBatch::num_rows).sum()
    }
}

/// A lazily-decoded stream of Arrow record batches.
///
/// Mirrors pyarrow's `RecordBatchStreamReader`. The response body is collected
/// once when the stream is created; batches are then decoded on demand as the
/// iterator advances. The pooled HTTP connection is already released at
/// construction time (the body has been fully drained).
pub struct ArrowBatchStream {
    reader: StreamReader<Cursor<Bytes>>,
    total_row_count: Option<i64>,
    next_link: Option<String>,
}

impl ArrowBatchStream {
    /// The schema of the stream, available before any batch is read.
    pub fn schema(&self) -> SchemaRef {
        self.reader.schema()
    }

    /// `X-Total-Row-Count` for the full result, when present.
    pub fn total_row_count(&self) -> Option<i64> {
        self.total_row_count
    }

    /// The `rel="next"` `Link` URL, when present.
    pub fn next_link(&self) -> Option<&str> {
        self.next_link.as_deref()
    }

    /// Drain the remaining batches into a single [`ArrowResult`].
    ///
    /// The Rust analog of pyarrow's `reader.read_all()`.
    pub fn read_all(mut self) -> Result<ArrowResult, ArrowError> {
        let schema = self.reader.schema();
        let total_row_count = self.total_row_count;
        let next_link = self.next_link.clone();
        let mut batches = Vec::new();
        for batch in &mut self.reader {
            batches.push(batch?);
        }
        Ok(ArrowResult {
            batches,
            schema,
            total_row_count,
            next_link,
        })
    }
}

impl Iterator for ArrowBatchStream {
    type Item = Result<RecordBatch, IpcArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.reader.next()
    }
}

/// Fetch a ready result as a fully-buffered [`ArrowResult`].
///
/// Buffers the entire Arrow IPC stream into memory before returning. Use
/// [`stream_result_arrow`] for large results where you want to iterate batches
/// without materializing them all at once.
///
/// The request is built to match the generated `get_result`: same URL, the
/// `X-Workspace-Id`/`X-Session-Id` API key headers, the user-agent, and a
/// transparently JWT-exchanged bearer token via
/// [`Configuration::resolve_bearer_token`](crate::apis::configuration::Configuration::resolve_bearer_token).
/// `Accept` and `?format=arrow` are added on top.
///
/// # Errors
///
/// * [`ArrowError::NotReady`] — HTTP 202, the result is still pending/processing.
/// * [`ArrowError::Failed`] — HTTP 409, the result computation failed.
/// * [`ArrowError::NotFound`] — HTTP 404.
/// * [`ArrowError::InvalidParams`] — HTTP 400, bad `offset`/`limit`.
/// * [`ArrowError::Http`] — any other non-success status.
/// * [`ArrowError::Reqwest`] / [`ArrowError::Ipc`] — transport / decode errors.
pub async fn get_result_arrow(
    configuration: &Configuration,
    id: &str,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<ArrowResult, ArrowError> {
    let (bytes, total_row_count, next_link) =
        fetch_arrow_bytes(configuration, id, offset, limit).await?;
    let reader = StreamReader::try_new(Cursor::new(bytes), None)?;
    let schema = reader.schema();
    let mut batches = Vec::new();
    for batch in reader {
        batches.push(batch?);
    }
    Ok(ArrowResult {
        batches,
        schema,
        total_row_count,
        next_link,
    })
}

/// Fetch a ready result as a lazily-decoded [`ArrowBatchStream`].
///
/// Mirrors Python's `stream_result_arrow`. The response body is collected once
/// (so the pooled connection is released immediately); the returned stream then
/// yields one [`RecordBatch`] per iteration without re-materializing the whole
/// result up front. Call [`ArrowBatchStream::read_all`] to collect everything.
///
/// # Errors
///
/// Same status mapping as [`get_result_arrow`].
pub async fn stream_result_arrow(
    configuration: &Configuration,
    id: &str,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<ArrowBatchStream, ArrowError> {
    let (bytes, total_row_count, next_link) =
        fetch_arrow_bytes(configuration, id, offset, limit).await?;
    let reader = StreamReader::try_new(Cursor::new(bytes), None)?;
    Ok(ArrowBatchStream {
        reader,
        total_row_count,
        next_link,
    })
}

/// Apply the `X-Workspace-Id` and `X-Session-Id` API-key headers, mirroring the
/// generated `get_result` `isKeyInHeader` blocks so a session-scoped client
/// (one built via [`crate::client::ClientBuilder::session_id`]) behaves
/// identically on the Arrow path.
fn apply_apikey_headers(
    mut req_builder: reqwest::RequestBuilder,
    configuration: &Configuration,
) -> reqwest::RequestBuilder {
    for header in [WORKSPACE_ID_HEADER, SESSION_ID_HEADER] {
        if let Some(apikey) = configuration.api_keys.get(header) {
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

/// Build the request, execute it, map the status, and return the raw IPC body
/// plus the metadata headers on success.
///
/// On every path the response body is fully drained (`resp.bytes().await`)
/// before returning, so the pooled connection is released — the reqwest analog
/// of urllib3's `release_conn`.
async fn fetch_arrow_bytes(
    configuration: &Configuration,
    id: &str,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<(Bytes, Option<i64>, Option<String>), ArrowError> {
    let uri_str = format!(
        "{}/v1/results/{id}",
        configuration.base_path,
        id = crate::apis::urlencode(id)
    );
    let mut req_builder = configuration.client.request(reqwest::Method::GET, &uri_str);

    // format=arrow takes precedence over the Accept header server-side, but we
    // send both to match the generated client and be explicit on the wire.
    req_builder = req_builder.query(&[("format", "arrow")]);
    if let Some(offset) = offset {
        req_builder = req_builder.query(&[("offset", &offset.to_string())]);
    }
    if let Some(limit) = limit {
        req_builder = req_builder.query(&[("limit", &limit.to_string())]);
    }

    req_builder = apply_apikey_headers(req_builder, configuration);

    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }

    // Transparent api_token -> JWT exchange (falls back to the static
    // bearer_access_token when no token provider is installed).
    if let Some(token) = configuration.resolve_bearer_token().await {
        req_builder = req_builder.bearer_auth(token);
    }

    req_builder = req_builder.header(reqwest::header::ACCEPT, ARROW_STREAM_MEDIA_TYPE);

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;
    let status = resp.status();

    if status == reqwest::StatusCode::OK {
        let total_row_count = parse_total_row_count(&resp);
        let next_link = parse_next_link(&resp);
        let bytes = resp.bytes().await?;
        return Ok((bytes, total_row_count, next_link));
    }

    match status {
        reqwest::StatusCode::ACCEPTED => {
            let retry_after = parse_retry_after(&resp);
            let body = resp.text().await?;
            let (result_status, result_id) = parse_status_and_id(&body, id);
            Err(ArrowError::NotReady {
                status: result_status,
                result_id,
                retry_after,
            })
        }
        reqwest::StatusCode::CONFLICT => {
            let body = resp.text().await?;
            let error_message = parse_error_message(&body);
            Err(ArrowError::Failed { error_message })
        }
        reqwest::StatusCode::NOT_FOUND => {
            // Drain the body so the connection is returned to the pool.
            let _ = resp.text().await;
            Err(ArrowError::NotFound)
        }
        reqwest::StatusCode::BAD_REQUEST => {
            let message = resp.text().await?;
            Err(ArrowError::InvalidParams { message })
        }
        other => {
            let body = resp.text().await.unwrap_or_default();
            Err(ArrowError::Http {
                status: other,
                body,
            })
        }
    }
}

fn parse_total_row_count(resp: &reqwest::Response) -> Option<i64> {
    resp.headers()
        .get("X-Total-Row-Count")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.trim().parse::<i64>().ok())
}

fn parse_retry_after(resp: &reqwest::Response) -> Option<u64> {
    resp.headers()
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.trim().parse::<u64>().ok())
}

/// Extract the `rel="next"` URL from an RFC 5988 `Link` header.
fn parse_next_link(resp: &reqwest::Response) -> Option<String> {
    let header = resp
        .headers()
        .get(reqwest::header::LINK)
        .and_then(|v| v.to_str().ok())?;
    parse_link_rel_next(header)
}

/// Parse an RFC 5988 `Link` header value and return the URL whose `rel` is
/// exactly `next`. Pulled out as a free function so it can be unit-tested.
fn parse_link_rel_next(header: &str) -> Option<String> {
    for part in header.split(',') {
        let mut segments = part.split(';');
        let url_seg = match segments.next() {
            Some(s) => s.trim(),
            None => continue,
        };
        let is_next = segments.any(|param| {
            let param = param.trim();
            // Match rel=next or rel="next" (case-insensitive rel param name).
            let lower = param.to_ascii_lowercase();
            lower == "rel=next" || lower == "rel=\"next\""
        });
        if is_next {
            let url = url_seg
                .trim_start_matches('<')
                .trim_end_matches('>')
                .to_string();
            if !url.is_empty() {
                return Some(url);
            }
        }
    }
    None
}

/// Best-effort parse of `{status, result_id}` from a JSON body (202 path).
fn parse_status_and_id(body: &str, fallback_id: &str) -> (String, String) {
    let parsed: Option<crate::models::GetResultResponse> = serde_json::from_str(body).ok();
    match parsed {
        Some(r) => (r.status, r.result_id),
        None => ("pending".to_string(), fallback_id.to_string()),
    }
}

/// Best-effort parse of `error_message` from a JSON body (409 path).
fn parse_error_message(body: &str) -> Option<String> {
    let parsed: Option<crate::models::GetResultResponse> = serde_json::from_str(body).ok();
    parsed.and_then(|r| r.error_message.flatten())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apis::configuration::ApiKey;
    use std::sync::Arc;

    /// A session-scoped client must send *both* `X-Workspace-Id` and
    /// `X-Session-Id` on the Arrow path, matching the generated `get_result`.
    /// Guards against silently dropping the session header (the prior behavior).
    #[test]
    fn apikey_headers_forward_workspace_and_session() {
        let mut configuration = Configuration::new();
        configuration.api_keys.insert(
            WORKSPACE_ID_HEADER.to_owned(),
            ApiKey {
                prefix: None,
                key: "ws-123".to_owned(),
            },
        );
        configuration.api_keys.insert(
            SESSION_ID_HEADER.to_owned(),
            ApiKey {
                prefix: None,
                key: "sess-456".to_owned(),
            },
        );

        let req_builder = configuration
            .client
            .request(reqwest::Method::GET, "https://api.hotdata.dev/v1/results/abc");
        let req = apply_apikey_headers(req_builder, &configuration)
            .build()
            .unwrap();
        let headers = req.headers();

        assert_eq!(headers.get(WORKSPACE_ID_HEADER).unwrap(), "ws-123");
        assert_eq!(headers.get(SESSION_ID_HEADER).unwrap(), "sess-456");
    }

    /// Without a session id installed, only `X-Workspace-Id` is sent — the
    /// session header is omitted rather than sent empty.
    #[test]
    fn apikey_headers_omit_absent_session() {
        let mut configuration = Configuration::new();
        configuration.api_keys.insert(
            WORKSPACE_ID_HEADER.to_owned(),
            ApiKey {
                prefix: None,
                key: "ws-123".to_owned(),
            },
        );

        let req_builder = configuration
            .client
            .request(reqwest::Method::GET, "https://api.hotdata.dev/v1/results/abc");
        let req = apply_apikey_headers(req_builder, &configuration)
            .build()
            .unwrap();
        let headers = req.headers();

        assert_eq!(headers.get(WORKSPACE_ID_HEADER).unwrap(), "ws-123");
        assert!(headers.get(SESSION_ID_HEADER).is_none());
    }

    use arrow_array::{Int64Array, StringArray};
    use arrow_ipc::writer::StreamWriter;
    use arrow_schema::{DataType, Field, Schema};

    /// Build a small two-batch Arrow IPC stream in memory: an `id: Int64` and a
    /// `name: Utf8` column.
    fn make_ipc_stream() -> (Vec<u8>, SchemaRef) {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("name", DataType::Utf8, false),
        ]));

        let batch1 = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int64Array::from(vec![1, 2, 3])),
                Arc::new(StringArray::from(vec!["a", "b", "c"])),
            ],
        )
        .unwrap();
        let batch2 = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int64Array::from(vec![4, 5])),
                Arc::new(StringArray::from(vec!["d", "e"])),
            ],
        )
        .unwrap();

        let mut buf: Vec<u8> = Vec::new();
        {
            let mut writer = StreamWriter::try_new(&mut buf, &schema).unwrap();
            writer.write(&batch1).unwrap();
            writer.write(&batch2).unwrap();
            writer.finish().unwrap();
        }
        (buf, schema)
    }

    #[test]
    fn buffered_round_trip_decodes_all_batches() {
        let (buf, schema) = make_ipc_stream();
        let reader = StreamReader::try_new(Cursor::new(Bytes::from(buf)), None).unwrap();
        let decoded_schema = reader.schema();
        let mut batches = Vec::new();
        for batch in reader {
            batches.push(batch.unwrap());
        }
        let result = ArrowResult {
            batches,
            schema: decoded_schema,
            total_row_count: Some(5),
            next_link: None,
        };

        assert_eq!(result.batches.len(), 2);
        assert_eq!(result.num_rows(), 5);
        assert_eq!(result.total_row_count, Some(5));
        assert_eq!(result.schema.fields().len(), 2);
        assert_eq!(result.schema.field(0).name(), "id");
        assert_eq!(result.schema.field(1).name(), "name");
        assert_eq!(*result.schema.field(0).data_type(), DataType::Int64);

        // Values round-trip.
        let first = &result.batches[0];
        let ids = first
            .column(0)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap();
        assert_eq!(ids.values(), &[1, 2, 3]);
        let names = first
            .column(1)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        assert_eq!(names.value(0), "a");
        assert_eq!(names.value(2), "c");

        // Schema we wrote matches the schema reported by the result.
        assert_eq!(result.schema.as_ref(), schema.as_ref());
    }

    #[test]
    fn streaming_iterator_yields_batches_lazily() {
        let (buf, _schema) = make_ipc_stream();
        let reader = StreamReader::try_new(Cursor::new(Bytes::from(buf)), None).unwrap();
        let mut stream = ArrowBatchStream {
            reader,
            total_row_count: Some(5),
            next_link: Some("https://api.hotdata.dev/v1/results/abc?offset=5".to_string()),
        };

        assert_eq!(stream.schema().fields().len(), 2);
        assert_eq!(stream.total_row_count(), Some(5));
        assert_eq!(
            stream.next_link(),
            Some("https://api.hotdata.dev/v1/results/abc?offset=5")
        );

        let b1 = stream.next().unwrap().unwrap();
        assert_eq!(b1.num_rows(), 3);
        let b2 = stream.next().unwrap().unwrap();
        assert_eq!(b2.num_rows(), 2);
        assert!(stream.next().is_none());
    }

    #[test]
    fn streaming_read_all_collects_everything() {
        let (buf, _schema) = make_ipc_stream();
        let reader = StreamReader::try_new(Cursor::new(Bytes::from(buf)), None).unwrap();
        let stream = ArrowBatchStream {
            reader,
            total_row_count: Some(5),
            next_link: None,
        };
        let result = stream.read_all().unwrap();
        assert_eq!(result.batches.len(), 2);
        assert_eq!(result.num_rows(), 5);
        assert_eq!(result.total_row_count, Some(5));
    }

    #[test]
    fn empty_stream_decodes_to_zero_batches() {
        let schema = Arc::new(Schema::new(vec![Field::new("id", DataType::Int64, false)]));
        let mut buf: Vec<u8> = Vec::new();
        {
            let mut writer = StreamWriter::try_new(&mut buf, &schema).unwrap();
            writer.finish().unwrap();
        }
        let reader = StreamReader::try_new(Cursor::new(Bytes::from(buf)), None).unwrap();
        let batches: Vec<_> = reader.collect::<Result<_, _>>().unwrap();
        assert!(batches.is_empty());
    }

    #[test]
    fn malformed_bytes_surface_as_ipc_error() {
        let garbage = Bytes::from_static(b"not an arrow stream at all");
        let err = StreamReader::try_new(Cursor::new(garbage), None)
            .err()
            .map(ArrowError::from);
        assert!(matches!(err, Some(ArrowError::Ipc(_))));
    }

    #[test]
    fn link_header_parses_rel_next() {
        let h = "<https://api.hotdata.dev/v1/results/abc?offset=100&limit=100>; rel=\"next\"";
        assert_eq!(
            parse_link_rel_next(h).as_deref(),
            Some("https://api.hotdata.dev/v1/results/abc?offset=100&limit=100")
        );

        // Multiple links; pick the one with rel="next".
        let multi = "<https://example.com/prev>; rel=\"prev\", \
                     <https://example.com/next>; rel=\"next\"";
        assert_eq!(
            parse_link_rel_next(multi).as_deref(),
            Some("https://example.com/next")
        );

        // No next relation.
        assert_eq!(
            parse_link_rel_next("<https://example.com/x>; rel=\"prev\""),
            None
        );

        // Unquoted rel value.
        assert_eq!(
            parse_link_rel_next("<https://example.com/n>; rel=next").as_deref(),
            Some("https://example.com/n")
        );
    }

    #[test]
    fn status_and_id_parse_from_json_body() {
        let body = r#"{"result_id":"res_123","status":"processing"}"#;
        let (status, id) = parse_status_and_id(body, "fallback");
        assert_eq!(status, "processing");
        assert_eq!(id, "res_123");

        // Malformed body falls back.
        let (status, id) = parse_status_and_id("not json", "fallback");
        assert_eq!(status, "pending");
        assert_eq!(id, "fallback");
    }

    #[test]
    fn error_message_parses_from_conflict_body() {
        let body = r#"{"result_id":"res_1","status":"failed","error_message":"boom"}"#;
        assert_eq!(parse_error_message(body).as_deref(), Some("boom"));

        // Absent error_message yields None.
        let body = r#"{"result_id":"res_1","status":"failed"}"#;
        assert_eq!(parse_error_message(body), None);
    }

    #[test]
    fn arrow_error_display_is_descriptive() {
        let e = ArrowError::NotReady {
            status: "pending".to_string(),
            result_id: "res_9".to_string(),
            retry_after: Some(2),
        };
        let s = e.to_string();
        assert!(s.contains("res_9"));
        assert!(s.contains("not ready"));

        assert_eq!(ArrowError::NotFound.to_string(), "result not found");
        assert!(ArrowError::Failed {
            error_message: Some("oops".into())
        }
        .to_string()
        .contains("oops"));
    }
}
