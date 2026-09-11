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
//! `X-Workspace-Id` API key, and the bearer credential resolved via
//! [`crate::apis::configuration::Configuration::resolve_bearer_token`]),
//! adds `Accept: application/vnd.apache.arrow.stream` plus `?format=arrow`, and
//! decodes the resulting IPC stream with `arrow-ipc`.
//!
//! Three entry points are provided:
//!
//! * [`get_result_arrow`] — buffers the full IPC stream and returns all
//!   [`RecordBatch`]es (the Rust analog of pyarrow `Table`; Rust has no
//!   `Table`, so we expose `Vec<RecordBatch>` plus the resolved [`SchemaRef`]).
//! * [`stream_result_arrow`] — returns an [`ArrowBatchStream`] that decodes one
//!   [`RecordBatch`] at a time, mirroring pyarrow's
//!   `RecordBatchStreamReader`. The body is still collected once (reqwest's
//!   async body is not a blocking `Read`); decoding is then lazy per batch.
//! * [`open_result_arrow`] — returns an [`ArrowResultStream`] that decodes
//!   straight off the socket, pulling body chunks as batches are asked for.
//!   Peak memory is one record batch rather than the whole result, which makes
//!   it the entry point for a result larger than memory. The trade is that the
//!   pooled connection stays checked out until the stream is drained.
//!
//! Enable with the `arrow` cargo feature (mirrors Python's `[arrow]` extra):
//!
//! ```toml
//! hotdata = { version = "0.1", features = ["arrow"] }
//! ```

use std::fmt;
use std::io::Cursor;

use arrow_array::RecordBatch;
use arrow_buffer::Buffer;
use arrow_ipc::reader::{StreamDecoder, StreamReader};
use arrow_schema::{ArrowError as IpcArrowError, SchemaRef};
use bytes::Bytes;

use crate::apis::configuration::Configuration;
use crate::client::WORKSPACE_ID_HEADER;

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
/// `X-Workspace-Id` API key header, the user-agent, and the bearer credential
/// resolved via
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
    x_database_id: &str,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<ArrowResult, ArrowError> {
    let (bytes, total_row_count, next_link) =
        fetch_arrow_bytes(configuration, id, x_database_id, offset, limit).await?;
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
    x_database_id: &str,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<ArrowBatchStream, ArrowError> {
    let (bytes, total_row_count, next_link) =
        fetch_arrow_bytes(configuration, id, x_database_id, offset, limit).await?;
    let reader = StreamReader::try_new(Cursor::new(bytes), None)?;
    Ok(ArrowBatchStream {
        reader,
        total_row_count,
        next_link,
    })
}

/// A [`RecordBatch`] stream decoded straight off the HTTP response body.
///
/// The difference from [`stream_result_arrow`] is where the bytes live. That one
/// collects the whole body first and then decodes lazily, so peak memory scales
/// with the size of the result. This one pulls body chunks from the socket as
/// batches are asked for, so peak memory is bounded by a single record batch —
/// which is what makes a result larger than memory readable at all.
///
/// The cost is that the pooled connection stays checked out until the stream is
/// drained or dropped. Dropping early abandons the download; the endpoint is
/// streamed end-to-end server-side, so the server stops producing when the
/// client goes away.
///
/// The schema is resolved during [`open_result_arrow`], so it is available
/// before the first batch — a caller writing a header row does not have to read
/// data to learn the column names.
#[derive(Debug)]
pub struct ArrowResultStream {
    source: ChunkSource,
    decoder: StreamDecoder,
    /// Bytes pulled from the socket that the decoder has not consumed yet.
    ///
    /// Empty whenever [`StreamDecoder::decode`] last returned `None`: it
    /// consumes the buffer it is given before asking for more, and only returns
    /// early — with bytes left over — when it has a batch to hand back.
    pending: Buffer,
    /// A batch the schema read in [`open_result_arrow`] decoded as a side
    /// effect, handed out by the first [`next_batch`](Self::next_batch) call so
    /// it is not dropped.
    buffered: Option<RecordBatch>,
    schema: SchemaRef,
    total_row_count: Option<i64>,
    next_link: Option<String>,
    done: bool,
}

impl ArrowResultStream {
    /// The schema of the result, known before any batch is read.
    pub fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }

    /// `X-Total-Row-Count`: rows in the full result, ignoring offset/limit.
    pub fn total_row_count(&self) -> Option<i64> {
        self.total_row_count
    }

    /// The `rel="next"` `Link` URL, when a finite `limit` did not reach the end.
    pub fn next_link(&self) -> Option<&str> {
        self.next_link.as_deref()
    }

    /// Decode the next [`RecordBatch`], pulling more body chunks as needed.
    ///
    /// Returns `Ok(None)` once the stream is exhausted. A body that ends in the
    /// middle of an IPC message is an error ([`StreamDecoder::finish`]), not a
    /// silent short read — a truncated download must not look like a complete
    /// result.
    pub async fn next_batch(&mut self) -> Result<Option<RecordBatch>, ArrowError> {
        if let Some(batch) = self.buffered.take() {
            return Ok(Some(batch));
        }
        if self.done {
            return Ok(None);
        }
        loop {
            if !self.pending.is_empty() {
                if let Some(batch) = self.decoder.decode(&mut self.pending)? {
                    return Ok(Some(batch));
                }
            }
            match self.source.next_chunk().await? {
                // `pending` is empty here (see the field comment), so this
                // replaces rather than discards. `Buffer::from(Bytes)` is a
                // refcount bump, not a copy.
                Some(bytes) => self.pending = Buffer::from(bytes),
                None => {
                    // `finish` first: a body that ended mid-message must keep
                    // erroring. Setting `done` before it would make a second
                    // call return `Ok(None)`, so a caller that logged the error
                    // and read on would see a truncated download as a clean end
                    // of stream. An exhausted body keeps yielding `None`, so the
                    // loop reaches `finish` again on every later call.
                    self.decoder.finish()?;
                    self.done = true;
                    return Ok(None);
                }
            }
        }
    }

    /// Drain the remaining batches into a single [`ArrowResult`].
    ///
    /// Materializes the whole result, so it defeats the point of this reader for
    /// a large one; it exists for callers that streamed only to avoid the double
    /// buffering of [`get_result_arrow`] (body bytes plus decoded batches).
    pub async fn read_all(mut self) -> Result<ArrowResult, ArrowError> {
        let schema = self.schema.clone();
        let total_row_count = self.total_row_count;
        let next_link = self.next_link.clone();
        let mut batches = Vec::new();
        while let Some(batch) = self.next_batch().await? {
            batches.push(batch);
        }
        Ok(ArrowResult {
            batches,
            schema,
            total_row_count,
            next_link,
        })
    }
}

/// Open a ready result as an [`ArrowResultStream`] that decodes off the socket.
///
/// Use this for a result too large to hold in memory: peak memory is one record
/// batch, where [`get_result_arrow`] and [`stream_result_arrow`] both scale with
/// the full result size.
///
/// The request is built by the same code as [`get_result_arrow`], so the two are
/// identical on the wire. The schema is read before returning, which costs one
/// body chunk.
///
/// # Errors
///
/// Same status mapping as [`get_result_arrow`], plus [`ArrowError::Ipc`] if the
/// body ends before a schema message arrives.
pub async fn open_result_arrow(
    configuration: &Configuration,
    id: &str,
    x_database_id: &str,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<ArrowResultStream, ArrowError> {
    let req = build_result_request(configuration, id, x_database_id, offset, limit).await?;
    crate::http_log::log_request(&req);
    let resp = configuration.client.execute(req).await?;
    let status = resp.status();
    crate::http_log::log_response_status(status);

    if status != reqwest::StatusCode::OK {
        return Err(map_error_response(id, resp).await);
    }

    let total_row_count = parse_total_row_count(&resp);
    let next_link = parse_next_link(&resp);

    open_from_source(ChunkSource::Body(resp), total_row_count, next_link).await
}

/// Where an [`ArrowResultStream`] pulls its bytes from.
///
/// The body variant is the only one in production. The test variant exists so
/// the pump below — the part that has to survive a chunk boundary landing inside
/// an IPC message — can be driven with chosen splits instead of whatever
/// chunking a socket happens to produce.
#[derive(Debug)]
enum ChunkSource {
    Body(reqwest::Response),
    #[cfg(test)]
    Chunks(std::vec::IntoIter<Bytes>),
}

impl ChunkSource {
    async fn next_chunk(&mut self) -> Result<Option<Bytes>, ArrowError> {
        match self {
            ChunkSource::Body(resp) => Ok(resp.chunk().await?),
            #[cfg(test)]
            ChunkSource::Chunks(chunks) => Ok(chunks.next()),
        }
    }
}

/// Read the schema off `source`, then hand back a stream positioned to decode
/// batches.
///
/// The schema is the first thing an Arrow IPC stream carries, so this normally
/// consumes one chunk — but a chunk boundary can fall inside the schema message,
/// and a single chunk can also carry the first batch along with it, which is why
/// a decoded batch is stashed rather than dropped.
async fn open_from_source(
    mut source: ChunkSource,
    total_row_count: Option<i64>,
    next_link: Option<String>,
) -> Result<ArrowResultStream, ArrowError> {
    let mut decoder = StreamDecoder::new();
    let mut pending = Buffer::from(Bytes::new());
    let mut buffered = None;
    let schema = loop {
        if !pending.is_empty() {
            if let Some(batch) = decoder.decode(&mut pending)? {
                buffered = Some(batch);
            }
        }
        if let Some(schema) = decoder.schema() {
            break schema;
        }
        match source.next_chunk().await? {
            Some(bytes) => pending = Buffer::from(bytes),
            None => {
                // An empty 200 body, or one cut short before the schema. Report
                // it rather than presenting a zero-column result as complete.
                decoder.finish()?;
                return Err(ArrowError::Ipc(IpcArrowError::IpcError(
                    "result body ended before an Arrow schema message".to_string(),
                )));
            }
        }
    };

    Ok(ArrowResultStream {
        source,
        decoder,
        pending,
        buffered,
        schema,
        total_row_count,
        next_link,
        done: false,
    })
}

/// Apply the `X-Workspace-Id` API-key header, mirroring the generated
/// `get_result` `isKeyInHeader` block so the Arrow path is scoped identically.
fn apply_apikey_headers(
    mut req_builder: reqwest::RequestBuilder,
    configuration: &Configuration,
) -> reqwest::RequestBuilder {
    if let Some(apikey) = configuration.api_keys.get(WORKSPACE_ID_HEADER) {
        let key = apikey.key.clone();
        let value = match apikey.prefix {
            Some(ref prefix) => format!("{prefix} {key}"),
            None => key,
        };
        req_builder = req_builder.header(WORKSPACE_ID_HEADER, value);
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
    x_database_id: &str,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<(Bytes, Option<i64>, Option<String>), ArrowError> {
    let req = build_result_request(configuration, id, x_database_id, offset, limit).await?;
    crate::http_log::log_request(&req);
    let resp = configuration.client.execute(req).await?;
    let status = resp.status();
    crate::http_log::log_response_status(status);

    if status == reqwest::StatusCode::OK {
        let total_row_count = parse_total_row_count(&resp);
        let next_link = parse_next_link(&resp);
        let bytes = resp.bytes().await?;
        return Ok((bytes, total_row_count, next_link));
    }

    Err(map_error_response(id, resp).await)
}

/// Build the `GET /v1/results/{id}?format=arrow` request.
///
/// Shared by every entry point in this module so the buffered and streaming
/// paths are byte-identical on the wire: same URL, query parameters, API-key
/// header, user-agent, `Accept`, and the bearer credential resolved per request.
async fn build_result_request(
    configuration: &Configuration,
    id: &str,
    x_database_id: &str,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<reqwest::Request, ArrowError> {
    let uri_str = format!(
        "{}/v1/results/{id}",
        configuration.base_path,
        id = crate::apis::urlencode(id)
    );
    let mut req_builder = configuration.client.request(reqwest::Method::GET, &uri_str);

    // `GET /v1/results/{id}` is scoped to a database: the `X-Database-Id` header
    // is required (mirrors the generated `get_result`).
    req_builder = req_builder.header("X-Database-Id", x_database_id.to_string());

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

    // Bearer credential, resolved per request so a pluggable token provider can
    // hand back a freshly refreshed value — matching the generated ops.
    if let Some(token) = configuration.resolve_bearer_token().await {
        req_builder = req_builder.bearer_auth(token);
    }

    req_builder = req_builder.header(reqwest::header::ACCEPT, ARROW_STREAM_MEDIA_TYPE);

    Ok(req_builder.build()?)
}

/// Map a non-200 `GET /v1/results/{id}` response to an [`ArrowError`].
///
/// Every path drains the body, both to produce the message and so the pooled
/// connection is returned rather than left for the pool to reclaim.
async fn map_error_response(id: &str, resp: reqwest::Response) -> ArrowError {
    let status = resp.status();
    match status {
        reqwest::StatusCode::ACCEPTED => {
            let retry_after = parse_retry_after(&resp);
            let body = match resp.text().await {
                Ok(body) => body,
                Err(e) => return ArrowError::Reqwest(e),
            };
            crate::http_log::log_response_body(&body);
            let (result_status, result_id) = parse_status_and_id(&body, id);
            ArrowError::NotReady {
                status: result_status,
                result_id,
                retry_after,
            }
        }
        reqwest::StatusCode::CONFLICT => {
            let body = match resp.text().await {
                Ok(body) => body,
                Err(e) => return ArrowError::Reqwest(e),
            };
            crate::http_log::log_response_body(&body);
            let error_message = parse_error_message(&body);
            ArrowError::Failed { error_message }
        }
        reqwest::StatusCode::NOT_FOUND => {
            // Drain the body so the connection is returned to the pool.
            let body = resp.text().await.unwrap_or_default();
            crate::http_log::log_response_body(&body);
            ArrowError::NotFound
        }
        reqwest::StatusCode::BAD_REQUEST => {
            let message = match resp.text().await {
                Ok(message) => message,
                Err(e) => return ArrowError::Reqwest(e),
            };
            crate::http_log::log_response_body(&message);
            ArrowError::InvalidParams { message }
        }
        other => {
            let body = resp.text().await.unwrap_or_default();
            crate::http_log::log_response_body(&body);
            ArrowError::Http {
                status: other,
                body,
            }
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

    /// The Arrow path forwards `X-Workspace-Id`, matching the generated
    /// `get_result`.
    #[test]
    fn apikey_headers_forward_workspace() {
        let mut configuration = Configuration::new();
        configuration.api_keys.insert(
            WORKSPACE_ID_HEADER.to_owned(),
            ApiKey {
                prefix: None,
                key: "ws-123".to_owned(),
            },
        );

        let req_builder = configuration.client.request(
            reqwest::Method::GET,
            "https://api.hotdata.dev/v1/results/abc",
        );
        let req = apply_apikey_headers(req_builder, &configuration)
            .build()
            .unwrap();
        let headers = req.headers();

        assert_eq!(headers.get(WORKSPACE_ID_HEADER).unwrap(), "ws-123");
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

    /// Regression (database-scoped results): the Arrow path must send the
    /// required `X-Database-Id` header on `GET /v1/results/{id}`. The mock
    /// matches only when the header is present, so a missing header 404s and the
    /// fetch fails.
    #[tokio::test]
    async fn fetch_arrow_sends_database_id_header() {
        use wiremock::matchers::{header, method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let (ipc, _schema) = make_ipc_stream();
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/results/res_1"))
            .and(query_param("format", "arrow"))
            .and(header("X-Database-Id", "db_x"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", ARROW_STREAM_MEDIA_TYPE)
                    .set_body_bytes(ipc),
            )
            .mount(&server)
            .await;

        let mut configuration = Configuration::new();
        configuration.base_path = server.uri();

        let result = get_result_arrow(&configuration, "res_1", "db_x", None, None)
            .await
            .expect("arrow fetch should forward X-Database-Id and succeed");
        assert_eq!(result.num_rows(), 5);
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

    // --- streaming reader (open_result_arrow / ArrowResultStream) -----------

    /// Open a stream over `chunks` using the production pump.
    async fn open_chunks(chunks: Vec<Bytes>) -> Result<ArrowResultStream, ArrowError> {
        open_from_source(ChunkSource::Chunks(chunks.into_iter()), None, None).await
    }

    /// Drain a stream into (batches, total rows).
    async fn drain(mut stream: ArrowResultStream) -> Vec<RecordBatch> {
        let mut batches = Vec::new();
        while let Some(batch) = stream.next_batch().await.expect("decode should succeed") {
            batches.push(batch);
        }
        batches
    }

    /// The whole point: the streaming reader must produce exactly what the
    /// buffered reader produces. Anything else is a silent difference between
    /// two ways of reading the same result.
    #[tokio::test]
    async fn streaming_reader_matches_the_buffered_reader() {
        let (ipc, schema) = make_ipc_stream();

        let buffered = StreamReader::try_new(Cursor::new(Bytes::from(ipc.clone())), None)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let streamed = drain(open_chunks(vec![Bytes::from(ipc)]).await.unwrap()).await;

        assert_eq!(streamed, buffered);
        assert_eq!(streamed.iter().map(|b| b.num_rows()).sum::<usize>(), 5);
        assert_eq!(streamed[0].schema(), schema);
    }

    /// A chunk boundary can land anywhere — inside the schema message, inside a
    /// batch header, inside array data. One-byte chunks put a boundary at every
    /// possible position at once, which is the adversarial case a socket only
    /// produces occasionally.
    #[tokio::test]
    async fn streaming_reader_survives_one_byte_chunks() {
        let (ipc, _schema) = make_ipc_stream();
        let expected = StreamReader::try_new(Cursor::new(Bytes::from(ipc.clone())), None)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let chunks: Vec<Bytes> = ipc.iter().map(|b| Bytes::copy_from_slice(&[*b])).collect();
        let streamed = drain(open_chunks(chunks).await.unwrap()).await;

        assert_eq!(streamed, expected);
    }

    /// Splitting at every offset in turn, so no single boundary position is
    /// left untested by luck of where the one-byte case happens to succeed.
    #[tokio::test]
    async fn streaming_reader_survives_a_split_at_every_offset() {
        let (ipc, _schema) = make_ipc_stream();
        let expected = StreamReader::try_new(Cursor::new(Bytes::from(ipc.clone())), None)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        for split in 1..ipc.len() {
            let chunks = vec![
                Bytes::copy_from_slice(&ipc[..split]),
                Bytes::copy_from_slice(&ipc[split..]),
            ];
            let streamed = drain(open_chunks(chunks).await.unwrap()).await;
            assert_eq!(streamed, expected, "mismatch when split at byte {split}");
        }
    }

    /// A caller writing a CSV header needs the column names before it has read
    /// any data. `open` resolves the schema, so this holds even for a result
    /// whose rows never arrive.
    #[tokio::test]
    async fn streaming_reader_knows_the_schema_before_any_batch() {
        let (ipc, schema) = make_ipc_stream();
        let stream = open_chunks(vec![Bytes::from(ipc)]).await.unwrap();
        assert_eq!(stream.schema(), schema);
    }

    #[tokio::test]
    async fn streaming_reader_yields_zero_batches_for_an_empty_result() {
        let schema = Arc::new(Schema::new(vec![Field::new("id", DataType::Int64, false)]));
        let mut buf: Vec<u8> = Vec::new();
        {
            let mut writer = StreamWriter::try_new(&mut buf, &schema).unwrap();
            writer.finish().unwrap();
        }

        let stream = open_chunks(vec![Bytes::from(buf)]).await.unwrap();
        assert_eq!(
            stream.schema(),
            schema,
            "header info survives an empty result"
        );
        assert!(drain(stream).await.is_empty());
    }

    /// A download cut short must not read as a complete result. This is the
    /// failure the buffered path got for free (a short body fails to parse) and
    /// that a streaming reader has to assert deliberately.
    #[tokio::test]
    async fn a_body_cut_short_is_an_error_not_a_short_read() {
        let (ipc, _schema) = make_ipc_stream();
        // Drop the trailing end-of-stream marker and part of the last batch.
        let truncated = &ipc[..ipc.len() - 16];

        let mut stream = open_chunks(vec![Bytes::copy_from_slice(truncated)])
            .await
            .expect("the schema is intact, so opening succeeds");

        let mut err = None;
        loop {
            match stream.next_batch().await {
                Ok(Some(_)) => continue,
                Ok(None) => break,
                Err(e) => {
                    err = Some(e);
                    break;
                }
            }
        }
        assert!(
            matches!(err, Some(ArrowError::Ipc(_))),
            "a truncated body must surface as an IPC error, got {err:?}"
        );
    }

    /// A truncated body must keep failing. If the stream marked itself done
    /// before confirming a clean end, a caller that logged the first error and
    /// read on would be told the stream ended normally — a short download
    /// silently becoming a complete result.
    #[tokio::test]
    async fn a_cut_short_body_keeps_erroring_on_every_later_call() {
        let (ipc, _schema) = make_ipc_stream();
        let truncated = &ipc[..ipc.len() - 16];

        let mut stream = open_chunks(vec![Bytes::copy_from_slice(truncated)])
            .await
            .expect("the schema is intact, so opening succeeds");

        // Drain to the failure.
        let mut first_err = None;
        loop {
            match stream.next_batch().await {
                Ok(Some(_)) => continue,
                Ok(None) => panic!("a truncated body must not report a clean end of stream"),
                Err(e) => {
                    first_err = Some(e);
                    break;
                }
            }
        }
        assert!(matches!(first_err, Some(ArrowError::Ipc(_))));

        // Reading on must report the same failure, not `Ok(None)`.
        for attempt in 0..3 {
            match stream.next_batch().await {
                Err(ArrowError::Ipc(_)) => {}
                other => panic!(
                    "call {attempt} after a truncated body must repeat the error, got {other:?}"
                ),
            }
        }
    }

    #[tokio::test]
    async fn a_body_with_no_schema_message_is_an_error() {
        let err = open_chunks(vec![]).await.err();
        assert!(
            matches!(err, Some(ArrowError::Ipc(_))),
            "an empty body must not open as a zero-column result, got {err:?}"
        );
    }

    /// End-to-end over HTTP: the streaming opener forwards the same headers and
    /// query parameters as the buffered fetch, and decodes the same rows.
    #[tokio::test]
    async fn open_result_arrow_forwards_headers_and_decodes() {
        use wiremock::matchers::{header, method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let (ipc, _schema) = make_ipc_stream();
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/results/res_1"))
            .and(query_param("format", "arrow"))
            .and(header("X-Database-Id", "db_x"))
            .and(header("accept", ARROW_STREAM_MEDIA_TYPE))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", ARROW_STREAM_MEDIA_TYPE)
                    .insert_header("X-Total-Row-Count", "5")
                    .set_body_bytes(ipc),
            )
            .mount(&server)
            .await;

        let mut configuration = Configuration::new();
        configuration.base_path = server.uri();

        let stream = open_result_arrow(&configuration, "res_1", "db_x", None, None)
            .await
            .expect("streaming open should succeed");
        assert_eq!(stream.total_row_count(), Some(5));
        assert_eq!(
            drain(stream)
                .await
                .iter()
                .map(|b| b.num_rows())
                .sum::<usize>(),
            5
        );
    }

    /// The non-200 mapping moved into a shared helper during this change; both
    /// entry points must still report a not-ready result as such.
    #[tokio::test]
    async fn open_result_arrow_maps_a_not_ready_result() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/results/res_pending"))
            .respond_with(
                ResponseTemplate::new(202)
                    .insert_header("Retry-After", "2")
                    .set_body_string(r#"{"status":"processing","result_id":"res_pending"}"#),
            )
            .mount(&server)
            .await;

        let mut configuration = Configuration::new();
        configuration.base_path = server.uri();

        let err = open_result_arrow(&configuration, "res_pending", "db_x", None, None)
            .await
            .err();
        match err {
            Some(ArrowError::NotReady {
                status,
                result_id,
                retry_after,
            }) => {
                assert_eq!(status, "processing");
                assert_eq!(result_id, "res_pending");
                assert_eq!(retry_after, Some(2));
            }
            other => panic!("expected NotReady, got {other:?}"),
        }
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
