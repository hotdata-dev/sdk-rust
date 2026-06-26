//! Ergonomic, hand-written direct-to-storage (presigned) file uploads.
//!
//! This module is regeneration-immune: it is protected by `.openapi-generator-ignore`
//! and is never emitted by the OpenAPI generator. It orchestrates the
//! presigned-upload flow that the generated [`apis::uploads_api`](crate::apis::uploads_api)
//! ops expose as raw building blocks:
//!
//! 1. `POST /v1/uploads` ([`create_upload_session_handler`]) opens a session and
//!    returns either a single `url` (`mode == "single"`) or a set of `part_urls`
//!    plus a `part_size` (`mode == "multipart"`), along with a one-time
//!    `finalize_token`.
//! 2. The client `PUT`s the bytes **directly to object storage** — never back
//!    through the API. Single uploads stream the whole file to `url`; multipart
//!    uploads slice the file into `part_size`-byte chunks and `PUT` each chunk to
//!    its `part_urls[i - 1]`, collecting the storage `ETag` per part.
//! 3. `POST /v1/uploads/{upload_id}/finalize` ([`finalize_upload_handler`])
//!    confirms the upload with the finalize token in the `X-Upload-Finalize-Token`
//!    header (empty body for single; the ascending `{part_number, e_tag}` list
//!    for multipart) and returns a [`models::FinalizeUploadResponse`].
//!
//! # Storage PUT header isolation
//!
//! A presigned storage URL already carries its authorization in the query string
//! (or in the server-provided `headers` map). Object stores (S3 and compatible)
//! reject a `PUT` with `403 SignatureDoesNotMatch` if it carries extra
//! signed-ish headers, so [`put_to_storage`] sends a *bare* request: NONE of the
//! SDK's bearer / workspace / session headers, only an explicit `Content-Length`
//! and whatever the server placed in `headers` (currently always empty). It also
//! refuses to let reqwest auto-append a charset to a `Content-Type` — a type is
//! sent only when the server's `headers` map asks for one.
//!
//! No S3/AWS SDK is involved: storage `PUT`s are plain `reqwest`.

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::apis::configuration::Configuration;
use crate::apis::{self, Error};
use crate::models;

/// One mebibyte, the unit the storage part-size range is expressed in.
const MIB: u64 = 1024 * 1024;

/// Default cap on concurrent part `PUT`s when the caller doesn't set
/// [`UploadOptions::max_concurrency`]. Matches the boto3 / AWS CLI default of 10.
/// The effective in-flight count is the MIN of this and a memory budget (see
/// [`effective_in_flight`]).
pub const DEFAULT_MAX_CONCURRENCY: usize = 10;

/// Default part-size hint, in bytes (8 MiB), sent when the caller doesn't set
/// [`UploadOptions::part_size`]. The server clamps the hint to its own range and
/// returns the actual size. See [`auto_part_size_hint`].
pub const DEFAULT_PART_SIZE: u64 = 8 * MIB;

/// Target ceiling on part count when auto-scaling the part-size hint for very
/// large files, with headroom under S3's hard 10,000-part limit. See
/// [`auto_part_size_hint`].
pub const TARGET_MAX_PARTS: u64 = 9000;

/// Minimum part size storage accepts (5 MiB). The hint is clamped to at least
/// this; the server enforces it too.
pub const MIN_PART_SIZE: u64 = 5 * MIB;

/// Maximum part size storage accepts (5 GiB). The hint is clamped to at most
/// this.
pub const MAX_PART_SIZE: u64 = 5 * 1024 * MIB;

/// Peak-memory budget for in-flight part buffers (256 MiB). Each in-flight part
/// buffers up to `part_size` bytes, so the effective in-flight count is bounded
/// by `budget / part_size`. See [`effective_in_flight`].
pub const UPLOAD_MEMORY_BUDGET: u64 = 256 * MIB;

/// Compute the part-size HINT to send to the server in
/// `CreateUploadRequest.part_size` when the caller did not specify one.
///
/// Starts from [`DEFAULT_PART_SIZE`] (8 MiB) and grows only for files large
/// enough that 8 MiB parts would exceed [`TARGET_MAX_PARTS`] — so the common
/// case is unchanged and only very large files (beyond ~72 GiB) get a larger
/// hint to keep the part count bounded. The result is rounded UP to a whole MiB
/// and clamped to `[MIN_PART_SIZE, MAX_PART_SIZE]`. The server still has the
/// final say and clamps to its own range.
///
/// Pure and total: `declared_size == 0` yields [`DEFAULT_PART_SIZE`].
pub fn auto_part_size_hint(declared_size: u64) -> u64 {
    // Smallest part size that keeps the count at or under the target.
    let by_count = declared_size.div_ceil(TARGET_MAX_PARTS);
    let raw = DEFAULT_PART_SIZE.max(by_count);
    // Round up to a whole MiB so the hint is a clean multiple.
    let rounded = raw.div_ceil(MIB) * MIB;
    rounded.clamp(MIN_PART_SIZE, MAX_PART_SIZE)
}

/// Compute how many part `PUT`s to keep in flight, given the caller's
/// `max_concurrency` (already defaulted to [`DEFAULT_MAX_CONCURRENCY`]) and the
/// SERVER's actual returned `part_size`.
///
/// Peak buffered memory is `in_flight * part_size`, so we cap in-flight at
/// `UPLOAD_MEMORY_BUDGET / part_size` and then at `max_concurrency`, with a floor
/// of 2 so progress is always made even for very large parts. Normal 8 MiB parts
/// give `256/8 = 32`, capped to `max_concurrency`; a 64 MiB part gives `4`.
///
/// Pure and total: a zero `part_size` is treated as 1 to avoid division by zero
/// (it still floors to 2).
pub fn effective_in_flight(max_concurrency: usize, part_size: u64) -> usize {
    let by_budget = (UPLOAD_MEMORY_BUDGET / part_size.max(1)) as usize;
    by_budget.min(max_concurrency).max(2)
}

/// Progress callback: invoked as bytes flow with `(bytes_done_total, total)`,
/// where `total` is the full declared file size. `bytes_done_total` is
/// monotonically non-decreasing and reaches exactly `total` when the transfer
/// completes. Shared (`Arc`) so it can be cloned across concurrent part tasks;
/// it must therefore be `Send + Sync`.
pub type UploadProgress = Arc<dyn Fn(u64, u64) + Send + Sync>;

/// Options for [`Client::upload_file`](crate::Client::upload_file).
///
/// All fields are optional. `content_type` / `content_encoding` / `filename`
/// are recorded with the upload (advisory metadata; they do not change where the
/// bytes are stored). `part_size` is a hint the server clamps to its allowed
/// range and ignores for single-`PUT` uploads. `progress`, when set, is invoked
/// as bytes flow.
#[derive(Default, Clone)]
pub struct UploadOptions {
    /// Content type to record for the uploaded file (e.g. a Parquet/CSV/JSON
    /// MIME type). Advisory.
    pub content_type: Option<String>,
    /// Content encoding to record for the uploaded file (e.g. `gzip`). Advisory.
    pub content_encoding: Option<String>,
    /// Original file name, recorded for bookkeeping. Advisory. Defaults to the
    /// source path's file name when not set.
    pub filename: Option<String>,
    /// Preferred part size, in bytes, for a large (multipart) upload. A hint;
    /// the server clamps it and ignores it for single-`PUT` uploads. When unset,
    /// the SDK auto-scales a hint via [`auto_part_size_hint`] (8 MiB for normal
    /// files, larger only for very large ones to bound the part count).
    pub part_size: Option<u64>,
    /// Maximum number of part `PUT`s to keep in flight for a multipart upload.
    /// `None` uses [`DEFAULT_MAX_CONCURRENCY`]. The effective in-flight count is
    /// the MIN of this and a peak-memory budget derived from the server's actual
    /// part size (see [`effective_in_flight`]), so memory stays bounded.
    pub max_concurrency: Option<usize>,
    /// Optional progress callback invoked with `(bytes_done_total, total)`.
    pub progress: Option<UploadProgress>,
}

impl std::fmt::Debug for UploadOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UploadOptions")
            .field("content_type", &self.content_type)
            .field("content_encoding", &self.content_encoding)
            .field("filename", &self.filename)
            .field("part_size", &self.part_size)
            .field("max_concurrency", &self.max_concurrency)
            .field("progress", &self.progress.as_ref().map(|_| "<callback>"))
            .finish()
    }
}

/// Error returned by [`Client::upload_file`](crate::Client::upload_file).
///
/// Marked `#[non_exhaustive]`: new variants may be added without a breaking
/// change, so downstream `match`es should carry a wildcard arm.
#[derive(Debug)]
#[non_exhaustive]
pub enum UploadError {
    /// Opening or reading the local source file failed.
    Io(std::io::Error),
    /// Opening the upload session (`POST /v1/uploads`) failed. A `501`
    /// `PRESIGN_UNSUPPORTED` lands here too — the presigned path is a hard
    /// requirement and never falls back to the legacy `POST /v1/files` proxy.
    CreateSession(Error<apis::uploads_api::CreateUploadSessionHandlerError>),
    /// A storage `PUT` (or the request building / transport around it) failed.
    Storage(reqwest::Error),
    /// A storage `PUT` returned a non-2xx status. Carries the status and the
    /// response body for diagnosis.
    StorageStatus {
        /// The HTTP status the storage endpoint returned.
        status: reqwest::StatusCode,
        /// The 1-based part number for a multipart `PUT`, or `None` for the
        /// single-`PUT` path.
        part_number: Option<i32>,
        /// The storage response body (often XML for S3-style errors).
        body: String,
    },
    /// Storage accepted a part `PUT` but returned no `ETag` header, so the part
    /// cannot be finalized.
    MissingETag {
        /// The 1-based part number whose `PUT` response lacked an `ETag`.
        part_number: i32,
    },
    /// The create-session response was internally inconsistent for its declared
    /// `mode` (e.g. `single` without a `url`, or `multipart` without
    /// `part_urls` / `part_size`).
    MalformedSession(String),
    /// Finalizing the upload (`POST /v1/uploads/{id}/finalize`) failed.
    Finalize(Error<apis::uploads_api::FinalizeUploadHandlerError>),
}

impl std::fmt::Display for UploadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UploadError::Io(e) => write!(f, "reading the source file failed: {e}"),
            UploadError::CreateSession(e) => write!(f, "opening the upload session failed: {e}"),
            UploadError::Storage(e) => write!(f, "uploading to storage failed: {e}"),
            UploadError::StorageStatus {
                status,
                part_number,
                body,
            } => match part_number {
                Some(n) => write!(f, "storage rejected part {n} with status {status}: {body}"),
                None => write!(
                    f,
                    "storage rejected the upload with status {status}: {body}"
                ),
            },
            UploadError::MissingETag { part_number } => write!(
                f,
                "storage returned no ETag for part {part_number}; cannot finalize"
            ),
            UploadError::MalformedSession(msg) => {
                write!(f, "malformed upload session response: {msg}")
            }
            UploadError::Finalize(e) => write!(f, "finalizing the upload failed: {e}"),
        }
    }
}

impl std::error::Error for UploadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            UploadError::Io(e) => Some(e),
            UploadError::CreateSession(e) => Some(e),
            UploadError::Storage(e) => Some(e),
            UploadError::Finalize(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for UploadError {
    fn from(e: std::io::Error) -> Self {
        UploadError::Io(e)
    }
}

/// Upload a local file directly to object storage and finalize it.
///
/// This is the orchestration behind [`Client::upload_file`](crate::Client::upload_file);
/// see that method for the public contract. It stats `path` for the declared
/// size, opens a session, drives the single-`PUT` or multipart path, and
/// finalizes — never touching the legacy `POST /v1/files` proxy.
pub(crate) async fn upload_file(
    configuration: &Configuration,
    path: &Path,
    opts: UploadOptions,
) -> Result<models::FinalizeUploadResponse, UploadError> {
    let metadata = tokio::fs::metadata(path).await?;
    let total = metadata.len();

    let filename = opts
        .filename
        .clone()
        .or_else(|| path.file_name().map(|n| n.to_string_lossy().into_owned()));

    // Part-size hint: honor an explicit caller value, else auto-scale from the
    // declared size so the common case stays at 8 MiB and only very large files
    // grow the hint (bounding the part count). The server clamps it regardless.
    let part_size_hint = opts.part_size.unwrap_or_else(|| auto_part_size_hint(total));

    // Open the session. `declared_size_bytes` is the exact byte count finalize
    // validates against, so it must match the bytes we actually upload.
    let create = models::CreateUploadRequest {
        declared_size_bytes: total as i64,
        content_type: opts.content_type.clone().map(Some),
        content_encoding: opts.content_encoding.clone().map(Some),
        filename: filename.map(Some),
        part_size: Some(Some(part_size_hint as i64)),
        ..models::CreateUploadRequest::new(total as i64)
    };
    let session = apis::uploads_api::create_upload_session_handler(configuration, create)
        .await
        .map_err(UploadError::CreateSession)?;

    // Report initial progress so a 0-byte file (or an instant single PUT) still
    // emits a terminal (0/0 or total/total) tick.
    if let Some(ref progress) = opts.progress {
        progress(0, total);
    }

    let parts = match session.mode.as_str() {
        "single" => {
            upload_single(configuration, &session, path, total, opts.progress.as_ref()).await?;
            None
        }
        "multipart" => {
            let max_concurrency = opts.max_concurrency.unwrap_or(DEFAULT_MAX_CONCURRENCY);
            Some(
                upload_multipart(
                    configuration,
                    &session,
                    path,
                    total,
                    max_concurrency,
                    opts.progress.as_ref(),
                )
                .await?,
            )
        }
        other => {
            return Err(UploadError::MalformedSession(format!(
                "unknown upload mode `{other}`"
            )))
        }
    };

    // Finalize: single sends an empty object `{}`; multipart sends
    // `{"parts": [...]}` with the ascending, non-duplicate parts list. The token
    // rides the X-Upload-Finalize-Token header (handled by the generated op).
    //
    // The body MUST be a JSON object, never `null`: the server rejects a `null`
    // finalize body ("invalid type: null, expected struct FinalizeUploadRequest")
    // even though the field is logically optional for single uploads. So we wrap
    // in `Some(..)` for both modes — the generated op then serializes a struct,
    // and `parts` (skip_serializing_if = Option::is_none) drops out for single,
    // yielding `{}`.
    let finalize_body = Some(
        parts
            .map(|parts| models::FinalizeUploadRequest {
                parts: Some(Some(parts)),
            })
            .unwrap_or_default(),
    );

    apis::uploads_api::finalize_upload_handler(
        configuration,
        &session.upload_id,
        &session.finalize_token,
        finalize_body,
    )
    .await
    .map_err(UploadError::Finalize)
}

/// Single-`PUT` path: stream the whole file to `session.url`, invoking the
/// progress callback incrementally as chunks are sent to storage.
///
/// The body is a [`progress_stream`] wrapping the file reader, so progress is
/// byte-granular (a multi-GB upload reports smooth `done/total` ticks rather
/// than jumping 0% -> 100%). A streaming body is not clonable, so this single
/// `PUT` is sent once with no 429/reset retry — an intentional trade for smooth
/// progress on the large, common single-`PUT` path; a presigned storage `PUT`
/// is not expected to be admission-shed.
async fn upload_single(
    configuration: &Configuration,
    session: &models::UploadSessionResponse,
    path: &Path,
    total: u64,
    progress: Option<&UploadProgress>,
) -> Result<(), UploadError> {
    let url =
        session.url.clone().flatten().ok_or_else(|| {
            UploadError::MalformedSession("single upload missing `url`".to_owned())
        })?;

    let file = tokio::fs::File::open(path).await?;
    let body = progress_stream(file, total, progress.cloned());

    put_stream_to_storage(configuration, &url, &session.headers, body, total).await?;

    // Guarantee a terminal tick at exactly `total`, even if the stream's last
    // chunk boundary or an empty file left the counter short. Monotonic: the
    // streamed ticks never exceed `total`.
    if let Some(progress) = progress {
        progress(total, total);
    }
    Ok(())
}

/// Wrap a file reader in a byte-counting stream of `Bytes` chunks. Each chunk
/// advances a running total and invokes `progress(done, total)` as it is yielded
/// to the request body, so progress reflects bytes actually handed to the
/// transport. Monotonic non-decreasing; the running total never exceeds `total`.
fn progress_stream(
    file: tokio::fs::File,
    total: u64,
    progress: Option<UploadProgress>,
) -> ProgressStream {
    use tokio_util::codec::{BytesCodec, FramedRead};

    ProgressStream {
        inner: FramedRead::new(file, BytesCodec::new()),
        done: 0,
        total,
        progress,
    }
}

/// A [`Stream`](futures_core::Stream) of `Bytes` chunks read from a file that
/// reports cumulative byte progress as each chunk is yielded. Hand-rolled over
/// `futures_core` (the crate's only direct futures dep) rather than pulling in
/// `futures_util`, mirroring how [`Client::upload_stream`](crate::Client::upload_stream)
/// stays on `futures_core::Stream`.
struct ProgressStream {
    inner: tokio_util::codec::FramedRead<tokio::fs::File, tokio_util::codec::BytesCodec>,
    done: u64,
    total: u64,
    progress: Option<UploadProgress>,
}

impl futures_core::Stream for ProgressStream {
    type Item = std::io::Result<bytes::Bytes>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use std::task::Poll;
        // `inner` (FramedRead) is Unpin, and our other fields are too, so a
        // mutable projection through `get_mut` is sound without pin-project.
        let this = self.get_mut();
        match std::pin::Pin::new(&mut this.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                let chunk = chunk.freeze();
                this.done = (this.done + chunk.len() as u64).min(this.total);
                if let Some(ref progress) = this.progress {
                    progress(this.done, this.total);
                }
                Poll::Ready(Some(Ok(chunk)))
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Multipart path: slice the file into `part_size`-byte chunks (the last is the
/// remainder), `PUT` each chunk to its `part_urls[i - 1]` with bounded
/// concurrency, and collect `(part_number, e_tag)` per part.
///
/// `max_concurrency` is the caller's ceiling on in-flight parts; the effective
/// count also honors a peak-memory budget derived from the server's actual
/// `part_size` (see [`effective_in_flight`]).
///
/// Returns the parts sorted ascending by part number, ready for finalize.
async fn upload_multipart(
    configuration: &Configuration,
    session: &models::UploadSessionResponse,
    path: &Path,
    total: u64,
    max_concurrency: usize,
    progress: Option<&UploadProgress>,
) -> Result<Vec<models::FinalizeUploadPart>, UploadError> {
    let part_urls = session.part_urls.clone().flatten().ok_or_else(|| {
        UploadError::MalformedSession("multipart upload missing `part_urls`".to_owned())
    })?;
    let part_size = session.part_size.flatten().ok_or_else(|| {
        UploadError::MalformedSession("multipart upload missing `part_size`".to_owned())
    })?;
    if part_size <= 0 {
        return Err(UploadError::MalformedSession(format!(
            "multipart upload has non-positive `part_size` {part_size}"
        )));
    }
    let part_size = part_size as u64;

    // Peak buffered memory is in_flight * part_size; bound in-flight by both the
    // caller's max_concurrency and the memory budget, using the SERVER's actual
    // part size (the same value we slice by below).
    let in_flight_cap = effective_in_flight(max_concurrency, part_size);

    // Aggregate progress across parts via a shared counter; each part adds its
    // own byte count as it completes.
    let done = Arc::new(AtomicU64::new(0));

    // Drive part PUTs with a bounded number in flight via a JoinSet. Each task
    // opens its own file handle and does a positioned read of exactly its byte
    // range so a retry inside `put_to_storage` re-reads cleanly and tasks never
    // share a cursor. Each task carries its 0-based `index` so the completion
    // order (which JoinSet does not preserve) is undone when placing results.
    let mut results: Vec<Option<models::FinalizeUploadPart>> = vec![None; part_urls.len()];
    let mut next = 0usize;
    let mut join_set: tokio::task::JoinSet<
        Result<(usize, models::FinalizeUploadPart), UploadError>,
    > = tokio::task::JoinSet::new();

    loop {
        while join_set.len() < in_flight_cap && next < part_urls.len() {
            let index = next;
            next += 1;
            let part_number = (index + 1) as i32;
            let url = part_urls[index].clone();
            let offset = index as u64 * part_size;
            // The last part carries the remainder; earlier parts are exactly
            // `part_size`. Guard against `offset > total` (more URLs than the
            // file needs) by clamping the length to zero.
            let len = part_size.min(total.saturating_sub(offset));
            let headers = session.headers.clone();
            let done = Arc::clone(&done);
            let progress = progress.cloned();
            let configuration = configuration.clone();
            let path = path.to_path_buf();

            join_set.spawn(async move {
                let chunk = read_range(&path, offset, len).await?;
                let resp = put_to_storage(
                    &configuration,
                    &url,
                    &headers,
                    chunk,
                    len,
                    Some(part_number),
                )
                .await?;
                let e_tag = resp
                    .headers()
                    .get(reqwest::header::ETAG)
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_owned())
                    .ok_or(UploadError::MissingETag { part_number })?;

                // Aggregate progress: add this part's bytes once it lands.
                if let Some(progress) = progress.as_ref() {
                    let now = done.fetch_add(len, Ordering::SeqCst) + len;
                    progress(now, total);
                }

                Ok::<_, UploadError>((index, models::FinalizeUploadPart { e_tag, part_number }))
            });
        }

        match join_set.join_next().await {
            // A part finished. `join_next` yields the task's `Result`; the outer
            // `Result` is the JoinError (panic/cancel), the inner is our
            // `UploadError`.
            Some(Ok(Ok((index, part)))) => results[index] = Some(part),
            Some(Ok(Err(e))) => {
                join_set.abort_all();
                return Err(e);
            }
            Some(Err(join_err)) => {
                join_set.abort_all();
                // A part task panicked or was cancelled — surface it as an I/O
                // error so the upload fails loudly rather than silently dropping
                // a part.
                return Err(UploadError::Io(std::io::Error::other(format!(
                    "part upload task failed: {join_err}"
                ))));
            }
            None => break,
        }
    }

    // `results` is indexed by 0-based part position, so collecting it in order
    // yields parts ascending by part_number with no duplicates.
    Ok(results.into_iter().flatten().collect())
}

/// Read exactly `len` bytes starting at `offset` from `path`. A positioned read
/// (seek + read_exact) so multipart part tasks never share a cursor and a retry
/// re-reads the same range cleanly.
async fn read_range(path: &Path, offset: u64, len: u64) -> Result<bytes::Bytes, UploadError> {
    use tokio::io::{AsyncReadExt, AsyncSeekExt};

    let mut file = tokio::fs::File::open(path).await?;
    file.seek(std::io::SeekFrom::Start(offset)).await?;
    let mut buf = vec![0u8; len as usize];
    file.read_exact(&mut buf).await?;
    Ok(bytes::Bytes::from(buf))
}

/// `PUT` a body to a presigned storage URL with strict header isolation.
///
/// Attaches NONE of the SDK's auth/workspace/session/user-agent headers — a
/// presigned URL already carries its authorization, and an extra signed-ish
/// header makes S3-compatible storage return `403`. Only an explicit
/// `Content-Length` and the server-provided `headers` map (replayed verbatim;
/// currently always empty) are sent. A `Content-Type` is set ONLY when the
/// `headers` map includes one, so reqwest never auto-appends a charset.
///
/// Uses a dedicated reqwest client with **no request timeout** (a large upload
/// legitimately takes minutes), mirroring the CLI's rationale; the body buffers
/// in memory so it clones cleanly across retries via [`crate::http::execute_retrying`].
async fn put_to_storage(
    configuration: &Configuration,
    url: &str,
    headers: &HashMap<String, String>,
    body: bytes::Bytes,
    content_length: u64,
    part_number: Option<i32>,
) -> Result<reqwest::Response, UploadError> {
    let client = storage_client(configuration);

    let mut req_builder = client
        .request(reqwest::Method::PUT, url)
        .header(reqwest::header::CONTENT_LENGTH, content_length);

    // Replay the server-provided headers verbatim. Currently always empty; this
    // is the only place a Content-Type may be set, so reqwest can't auto-append
    // a charset.
    for (name, value) in headers {
        req_builder = req_builder.header(name.as_str(), value.as_str());
    }

    // A buffered Bytes body clones cleanly, so 429 / pre-response-reset retries
    // in `execute_retrying` can re-send it.
    req_builder = req_builder.body(reqwest::Body::from(body));

    let req = req_builder.build().map_err(UploadError::Storage)?;
    crate::http_log::log_request(&req);
    let resp = crate::http::execute_retrying(&client, req, &configuration.retry)
        .await
        .map_err(UploadError::Storage)?;

    let status = resp.status();
    crate::http_log::log_response_status(status);
    if status.is_client_error() || status.is_server_error() {
        let body = resp.text().await.unwrap_or_default();
        crate::http_log::log_response_body(&body);
        return Err(UploadError::StorageStatus {
            status,
            part_number,
            body,
        });
    }
    Ok(resp)
}

/// `PUT` a streaming body to a presigned storage URL with the same strict
/// header isolation as [`put_to_storage`] (no SDK auth/scope headers; explicit
/// `Content-Length`; `Content-Type` only from the server `headers` map).
///
/// Used by the single-`PUT` path so progress is byte-granular. A streamed body
/// is not clonable, so this is a SINGLE attempt with no 429/reset retry — unlike
/// the buffered, retryable [`put_to_storage`] used per multipart part.
async fn put_stream_to_storage<S>(
    configuration: &Configuration,
    url: &str,
    headers: &HashMap<String, String>,
    body: S,
    content_length: u64,
) -> Result<reqwest::Response, UploadError>
where
    S: futures_core::Stream<Item = std::io::Result<bytes::Bytes>> + Send + 'static,
{
    let client = storage_client(configuration);

    let mut req_builder = client
        .request(reqwest::Method::PUT, url)
        // Explicit Content-Length so the body is sized (not chunked) — storage
        // can reject an oversized upload up front, and reqwest honors it as the
        // framing for a wrapped stream.
        .header(reqwest::header::CONTENT_LENGTH, content_length);

    for (name, value) in headers {
        req_builder = req_builder.header(name.as_str(), value.as_str());
    }

    req_builder = req_builder.body(reqwest::Body::wrap_stream(body));

    let req = req_builder.build().map_err(UploadError::Storage)?;
    crate::http_log::log_request(&req);
    // A streamed body can't be cloned, so send once (no retry helper).
    let resp = client.execute(req).await.map_err(UploadError::Storage)?;

    let status = resp.status();
    crate::http_log::log_response_status(status);
    if status.is_client_error() || status.is_server_error() {
        let body = resp.text().await.unwrap_or_default();
        crate::http_log::log_response_body(&body);
        return Err(UploadError::StorageStatus {
            status,
            part_number: None,
            body,
        });
    }
    Ok(resp)
}

/// The reqwest client used for storage `PUT`s.
///
/// Reuses the configured client (so a caller-supplied transport — custom TLS,
/// proxy, connection pool — applies) but the caller is expected to supply one
/// with no request timeout for large uploads. We do not mutate the configured
/// client's timeout here; the SDK's `Configuration::client` is shared by
/// reference and reqwest clients are cheap to clone (an `Arc` internally).
fn storage_client(configuration: &Configuration) -> reqwest::Client {
    configuration.client.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The part count a given hint would produce for a file of `size`.
    fn part_count(size: u64, part: u64) -> u64 {
        size.div_ceil(part)
    }

    #[test]
    fn auto_part_size_keeps_8mib_for_normal_files() {
        // Empty and small files default to 8 MiB.
        assert_eq!(auto_part_size_hint(0), DEFAULT_PART_SIZE);
        assert_eq!(auto_part_size_hint(1), DEFAULT_PART_SIZE);
        assert_eq!(auto_part_size_hint(100 * MIB), DEFAULT_PART_SIZE);
        assert_eq!(auto_part_size_hint(1024 * MIB), DEFAULT_PART_SIZE); // 1 GiB
                                                                        // Right at the boundary: 8 MiB * 9000 parts = 72 GiB still fits 8 MiB.
        let boundary = DEFAULT_PART_SIZE * TARGET_MAX_PARTS;
        assert_eq!(auto_part_size_hint(boundary), DEFAULT_PART_SIZE);
    }

    #[test]
    fn auto_part_size_scales_up_for_very_large_files_and_caps_parts() {
        // Beyond ~72 GiB the hint must grow above 8 MiB.
        let big = 200 * 1024 * MIB; // 200 GiB
        let hint = auto_part_size_hint(big);
        assert!(
            hint > DEFAULT_PART_SIZE,
            "hint should scale above 8 MiB for a 200 GiB file, got {hint}"
        );
        // Hint is a whole number of MiB.
        assert_eq!(hint % MIB, 0, "hint must be a whole MiB, got {hint}");
        // Part count stays at or under the target ceiling.
        assert!(
            part_count(big, hint) <= TARGET_MAX_PARTS,
            "part count {} must be <= {TARGET_MAX_PARTS}",
            part_count(big, hint)
        );
        // And always within storage's accepted range.
        assert!((MIN_PART_SIZE..=MAX_PART_SIZE).contains(&hint));
    }

    #[test]
    fn auto_part_size_clamps_to_max_for_enormous_files() {
        // A file so large the count-driven size would exceed 5 GiB clamps to the
        // 5 GiB ceiling (the part count then necessarily exceeds the soft target,
        // which is fine — it's a hint and the server has the final say).
        let enormous = 100 * 1024 * 1024 * MIB; // 100 PiB
        assert_eq!(auto_part_size_hint(enormous), MAX_PART_SIZE);
    }

    #[test]
    fn effective_in_flight_capped_by_max_concurrency_for_small_parts() {
        // 8 MiB parts: budget allows 256/8 = 32, so max_concurrency wins.
        assert_eq!(effective_in_flight(12, 8 * MIB), 12);
        assert_eq!(effective_in_flight(10, 8 * MIB), 10);
        // A tiny part size still can't exceed max_concurrency.
        assert_eq!(effective_in_flight(12, MIB), 12);
    }

    #[test]
    fn effective_in_flight_reduced_by_memory_budget_for_large_parts() {
        // 64 MiB parts: budget allows 256/64 = 4, below max_concurrency.
        assert_eq!(effective_in_flight(12, 64 * MIB), 4);
        // 128 MiB parts: 256/128 = 2.
        assert_eq!(effective_in_flight(12, 128 * MIB), 2);
    }

    #[test]
    fn effective_in_flight_floors_at_2_and_handles_zero() {
        // A part larger than the whole budget still keeps at least 2 in flight.
        assert_eq!(effective_in_flight(12, UPLOAD_MEMORY_BUDGET * 4), 2);
        // max_concurrency below the floor is raised to 2.
        assert_eq!(effective_in_flight(1, 8 * MIB), 2);
        // Zero part size doesn't divide-by-zero (treated as 1 byte): the budget
        // then allows a huge count, so max_concurrency wins.
        assert_eq!(effective_in_flight(12, 0), 12);
    }
}
