//! Ergonomic, hand-written direct-to-storage (presigned) file uploads.
//!
//! This module is regeneration-immune: it is protected by `.openapi-generator-ignore`
//! and is never emitted by the OpenAPI generator. It orchestrates the
//! presigned-upload flow that the generated [`apis::uploads_api`](crate::apis::uploads_api)
//! ops expose as raw building blocks:
//!
//! 1. `POST /v1/uploads` ([`create_upload_session_handler`]) opens a session.
//!    A small file declares its size and gets a single `url` (`mode == "single"`)
//!    or, for a known-size multipart upload, a full set of `part_urls` plus a
//!    `part_size` (`mode == "multipart"`). A large file omits its declared size
//!    to open a **streaming** session: `mode == "multipart"` with a `part_size`
//!    but NO `part_urls` — the client mints each part URL on demand from
//!    `POST /v1/uploads/{id}/parts` ([`mint_upload_parts_handler`]) just before
//!    uploading that part, so a URL can't expire mid-transfer on a slow upload.
//!    Every session also carries a one-time `finalize_token`.
//! 2. The client `PUT`s the bytes **directly to object storage** — never back
//!    through the API. Single uploads stream the whole file to `url`; multipart
//!    uploads slice the file into `part_size`-byte chunks and `PUT` each chunk to
//!    its part URL (pre-issued for known-size, minted on demand for streaming),
//!    collecting the storage `ETag` per part.
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
use std::time::Duration;

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

/// File-size boundary between the two upload strategies. A file at or below this
/// size takes the known-size path — a single quick `PUT` (or a short eager
/// multipart) that completes well within a presigned URL's TTL, so there is no
/// expiry risk. A larger file uses the streaming just-in-time path, minting each
/// part URL only moments before it is uploaded. Set to [`DEFAULT_PART_SIZE`],
/// the server's default single-vs-multipart boundary, so small uploads keep the
/// single-`PUT` fast path unchanged.
pub const STREAMING_THRESHOLD: u64 = DEFAULT_PART_SIZE;

/// Target peak-memory budget for in-flight part buffers (256 MiB). Each
/// in-flight part buffers up to `part_size` bytes, so [`effective_in_flight`]
/// derives the in-flight count as `budget / part_size`.
///
/// This is a TARGET, not a hard ceiling: it holds while `part_size` is small
/// relative to the budget (the normal case — 8 MiB parts stay well under it). It
/// cannot bound memory below one in-flight part, so when the server returns a
/// very large `part_size` (e.g. a 5 GiB part on a huge file), a single in-flight
/// part already exceeds this budget and peak memory is `1 * part_size`. In other
/// words the budget caps *concurrency*, not the size of one part.
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
/// `UPLOAD_MEMORY_BUDGET / part_size`, then at `max_concurrency`. Normal 8 MiB
/// parts give `256/8 = 32`, capped to `max_concurrency`; a 64 MiB part gives `4`.
///
/// `max_concurrency` is honored as an explicit floor: a caller asking for `1`
/// (or `0`) gets serial uploads (`1`), so the budget never *raises* concurrency
/// above what was requested. The budget-derived count itself has a floor of 1
/// (you must keep at least one part in flight to make progress), so the overall
/// result is always `>= 1`.
///
/// Pure and total: a zero `part_size` is treated as 1 to avoid division by zero.
pub fn effective_in_flight(max_concurrency: usize, part_size: u64) -> usize {
    // Honor an explicit low request down to serial (1); never below 1.
    let cap = max_concurrency.max(1);
    let by_budget = (UPLOAD_MEMORY_BUDGET / part_size.max(1)).max(1) as usize;
    by_budget.min(cap)
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
    /// A size (the file's declared size, or the part-size hint) did not fit the
    /// wire's signed 64-bit field. Only reachable for pathological sizes beyond
    /// `i64::MAX` bytes (~8 EiB).
    SizeOverflow {
        /// What overflowed (e.g. `"declared_size_bytes"`).
        what: &'static str,
        /// The offending value.
        value: u64,
    },
    /// Finalizing the upload (`POST /v1/uploads/{id}/finalize`) failed.
    Finalize(Error<apis::uploads_api::FinalizeUploadHandlerError>),
    /// Minting part URLs on demand (`POST /v1/uploads/{id}/parts`) failed during
    /// a streaming upload — either the initial batch or an on-403 re-mint.
    MintParts(Error<apis::uploads_api::MintUploadPartsHandlerError>),
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
            UploadError::SizeOverflow { what, value } => {
                write!(
                    f,
                    "{what} ({value} bytes) exceeds the maximum supported size"
                )
            }
            UploadError::MalformedSession(msg) => {
                write!(f, "malformed upload session response: {msg}")
            }
            UploadError::Finalize(e) => write!(f, "finalizing the upload failed: {e}"),
            UploadError::MintParts(e) => write!(f, "minting upload part URLs failed: {e}"),
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
            UploadError::MintParts(e) => Some(e),
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

    // The wire models the part-size hint as a signed i64; reject (rather than
    // silently wrap) a pathological hint beyond i64::MAX.
    let part_size_hint_i64 =
        i64::try_from(part_size_hint).map_err(|_| UploadError::SizeOverflow {
            what: "part_size",
            value: part_size_hint,
        })?;

    // Default a large file to a JUST-IN-TIME (streaming) session: omit
    // `declared_size_bytes` so the server mints NO part URLs up front. The client
    // then mints each part URL moments before it uploads that part (see
    // `upload_multipart_streaming`), so a URL cannot expire mid-transfer no
    // matter how long a slow upload runs — the failure mode of the eager
    // known-size path, whose URLs share a ~30-minute TTL. A small file is a
    // single quick `PUT` with no expiry risk, so it keeps the known-size path
    // (and the server's single-`PUT` fast path) by declaring its size.
    //
    // `declared_size_bytes` is sent (and so range-checked against the wire's
    // i64) ONLY on the known-size path; a streaming upload omits it entirely, so
    // a size beyond i64::MAX is never an obstacle to a streamed file.
    let declared_size_bytes = if total > STREAMING_THRESHOLD {
        None
    } else {
        let size = i64::try_from(total).map_err(|_| UploadError::SizeOverflow {
            what: "declared_size_bytes",
            value: total,
        })?;
        Some(Some(size))
    };
    let create = models::CreateUploadRequest {
        content_type: opts.content_type.clone().map(Some),
        content_encoding: opts.content_encoding.clone().map(Some),
        filename: filename.map(Some),
        part_size: Some(Some(part_size_hint_i64)),
        declared_size_bytes,
        ..models::CreateUploadRequest::new()
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
            upload_single(&session, path, total, opts.progress.as_ref()).await?;
            None
        }
        "multipart" => {
            let max_concurrency = opts.max_concurrency.unwrap_or(DEFAULT_MAX_CONCURRENCY);
            // A streaming (unknown-size) session returns NO part URLs up front
            // (the `part_urls` key is absent or null) — mint them on demand. A
            // known-size session returns the full `part_urls` list to PUT to
            // directly. An explicitly present (even empty) list is a known-size
            // response; `upload_multipart` validates it and rejects an empty one.
            let parts = if matches!(session.part_urls, Some(Some(_))) {
                upload_multipart(
                    configuration,
                    &session,
                    path,
                    total,
                    max_concurrency,
                    opts.progress.as_ref(),
                )
                .await?
            } else {
                upload_multipart_streaming(
                    configuration,
                    &session,
                    path,
                    total,
                    max_concurrency,
                    opts.progress.as_ref(),
                )
                .await?
            };
            Some(parts)
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

    // Finalize is exactly-once on the server: a second finalize of the same
    // upload is rejected. The generated op routes through `execute_retrying`,
    // which would retry an ambiguous failure (a lost response, or a 429 the
    // server actually processed) — turning a finalize that SUCCEEDED into a
    // spurious "already finalized" error on the retry. So we call it with retries
    // disabled (a single attempt). Part PUTs stay retryable (idempotent: storage
    // overwrites a part by number); only finalize is single-shot.
    let mut finalize_config = configuration.clone();
    finalize_config.retry.max_retries = 0;

    apis::uploads_api::finalize_upload_handler(
        &finalize_config,
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

    put_stream_to_storage(&url, &session.headers, body, total).await?;

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

/// A single part's upload work, independent of how the bytes actually get to
/// storage. The known-size path builds one of these per part and hands them to
/// [`upload_parts_resilient`]; tests substitute a fake uploader.
#[derive(Clone, Debug, PartialEq, Eq)]
struct PartPlan {
    /// 0-based position — indexes both the results vector and the URL list.
    index: usize,
    /// 1-based S3 part number.
    part_number: i32,
    /// Byte offset of this part within the source file.
    offset: u64,
    /// Byte length of this part (the last part is the remainder).
    len: u64,
}

/// Outer-loop retry policy for whole-upload resilience, layered ON TOP of the
/// per-part transport retries in [`crate::http::execute_retrying`].
///
/// The inner per-part retries all burn within a few seconds, so they only ride
/// out a momentary blip *during one part*. A longer network interruption (or a
/// flaky uplink that resets connections under load) needs a genuinely later
/// attempt. Each extra round re-sweeps ONLY the parts still failing, after a
/// backoff, and at reduced concurrency — fewer in-flight connections reset less
/// on a saturated link. Crucially, parts that already succeeded keep their
/// ETags across rounds, so a single bad part never discards the whole transfer.
#[derive(Clone, Copy, Debug)]
struct RetryRounds {
    /// Re-sweeps after the initial pass. `0` reproduces the legacy behavior: a
    /// single part exhausting its inner retries fails the entire upload.
    max_extra_rounds: u32,
    /// Backoff before the first re-sweep; doubles each subsequent round.
    base_delay: Duration,
}

impl Default for RetryRounds {
    fn default() -> Self {
        Self {
            max_extra_rounds: 3,
            base_delay: Duration::from_secs(2),
        }
    }
}

impl RetryRounds {
    /// Backoff before `round` (1-based for re-sweeps): capped exponential.
    fn delay_for(self, round: u32) -> Duration {
        let shift = round.saturating_sub(1).min(16);
        self.base_delay.saturating_mul(1u32 << shift)
    }
}

/// In-flight cap for a given 0-based round: halve per round, never below 1. A
/// saturated, jittery uplink (the failure mode this whole loop targets) resets
/// fewer connections when fewer are in flight, so later rounds back off.
fn round_in_flight(base_in_flight: usize, round: u32) -> usize {
    let shift = round.min(usize::BITS - 1);
    (base_in_flight >> shift).max(1)
}

/// Whether an upload error is *terminal* — guaranteed to reproduce on a
/// re-sweep because it reflects a server-contract or sizing violation rather
/// than a transient network condition. Terminal errors fail the upload
/// immediately; everything else (transport resets, timeouts, storage 4xx/5xx,
/// mint failures) stays retryable, so a flaky link is never mistaken for a
/// permanent fault. Kept deliberately narrow: only errors that are deterministic
/// in the part's own inputs belong here.
fn is_terminal(err: &UploadError) -> bool {
    matches!(
        err,
        UploadError::MalformedSession(_) | UploadError::SizeOverflow { .. }
    )
}

/// Upload every part, surviving transient per-part failures without discarding
/// the parts that already succeeded.
///
/// A round runs the still-pending parts through a `JoinSet` bounded by the
/// round's in-flight cap, recording each success and **collecting** (not
/// propagating) each failure. If any parts remain, it waits per [`RetryRounds`]
/// and re-sweeps just those, at reduced concurrency, until they all land or the
/// rounds are exhausted. Completed parts' ETags persist across rounds, so the
/// work already done is never thrown away — the bug this replaces aborted the
/// whole upload the moment one part exhausted its inner retries.
///
/// `upload_part` performs one part's transfer (including its own inner transport
/// retries) and MUST be idempotent: re-running a part overwrites it in storage
/// (S3 `UploadPart` by number), so a re-swept part is safe.
async fn upload_parts_resilient<F, Fut>(
    plans: Vec<PartPlan>,
    base_in_flight: usize,
    rounds: RetryRounds,
    upload_part: F,
) -> Result<Vec<models::FinalizeUploadPart>, UploadError>
where
    F: Fn(PartPlan) -> Fut + Clone + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<models::FinalizeUploadPart, UploadError>>
        + Send
        + 'static,
{
    let total_parts = plans.len();
    // `results` is indexed by `plan.index`, so every plan's index must fall in
    // `0..total_parts` (both callers build consecutive 0-based plans). Enforce
    // it so a future caller passing a sparse/offset set fails loudly in tests
    // rather than panicking or writing the wrong slot.
    debug_assert!(
        plans.iter().all(|p| p.index < total_parts),
        "PartPlan.index must be within 0..plans.len()"
    );
    let mut results: Vec<Option<models::FinalizeUploadPart>> = vec![None; total_parts];
    let mut remaining = plans;
    let mut last_err: Option<UploadError> = None;

    for round in 0..=rounds.max_extra_rounds {
        if remaining.is_empty() {
            break;
        }
        if round > 0 {
            // A genuinely later attempt on a fresh window — the point of the
            // outer loop, distinct from the inner retries that already ran.
            tokio::time::sleep(rounds.delay_for(round)).await;
        }

        let in_flight = round_in_flight(base_in_flight, round);
        let mut pending = std::mem::take(&mut remaining).into_iter();
        let mut failed: Vec<PartPlan> = Vec::new();
        let mut join_set: tokio::task::JoinSet<
            Result<(usize, models::FinalizeUploadPart), (PartPlan, UploadError)>,
        > = tokio::task::JoinSet::new();

        loop {
            while join_set.len() < in_flight {
                let Some(plan) = pending.next() else { break };
                let upload_part = upload_part.clone();
                join_set.spawn(async move {
                    let index = plan.index;
                    match upload_part(plan.clone()).await {
                        Ok(part) => Ok((index, part)),
                        Err(e) => Err((plan, e)),
                    }
                });
            }
            match join_set.join_next().await {
                Some(Ok(Ok((index, part)))) => results[index] = Some(part),
                Some(Ok(Err((plan, e)))) => {
                    // A clearly-terminal error (server-contract / sizing
                    // violation) reproduces identically on every re-sweep, so
                    // fail fast rather than burning the whole round budget on it.
                    // Anything network-ish stays retryable — we never regress
                    // resilience by mistaking a flaky link for a permanent fault.
                    if is_terminal(&e) {
                        join_set.abort_all();
                        return Err(e);
                    }
                    // Record the failure and keep draining the rest — do NOT
                    // abort the other in-flight parts. This part is re-swept in
                    // the next round.
                    failed.push(plan);
                    last_err = Some(e);
                }
                Some(Err(join_err)) => {
                    join_set.abort_all();
                    return Err(UploadError::Io(std::io::Error::other(format!(
                        "part upload task failed: {join_err}"
                    ))));
                }
                None => break,
            }
        }
        remaining = failed;
    }

    if !remaining.is_empty() {
        // Rounds exhausted with parts still failing — surface the last
        // underlying error so the caller's normal error mapping applies.
        return Err(last_err
            .unwrap_or_else(|| UploadError::Io(std::io::Error::other("multipart upload failed"))));
    }

    Ok(results.into_iter().flatten().collect())
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

    if part_urls.is_empty() {
        return Err(UploadError::MalformedSession(
            "multipart upload has empty `part_urls`".to_owned(),
        ));
    }

    // The URL count must match the number of `part_size`-byte chunks the file
    // splits into (last is the remainder). Too many URLs and we'd PUT a
    // zero-length trailing part; too few and we'd finalize an incomplete list.
    // Both mean a session inconsistent with our declared size, so fail loudly.
    let expected_parts = total.div_ceil(part_size).max(1);
    if part_urls.len() as u64 != expected_parts {
        return Err(UploadError::MalformedSession(format!(
            "multipart upload returned {} part URLs but the file ({total} bytes) \
             splits into {expected_parts} parts of {part_size} bytes",
            part_urls.len()
        )));
    }

    // Peak buffered memory is in_flight * part_size; bound in-flight by both the
    // caller's max_concurrency and the memory budget, using the SERVER's actual
    // part size (the same value we slice by below).
    let in_flight_cap = effective_in_flight(max_concurrency, part_size);

    // Aggregate progress across parts via a shared counter; each part adds its
    // own byte count once it lands (on success only — a re-swept part that
    // failed an earlier round did not count, so bytes are never double-counted).
    let done = Arc::new(AtomicU64::new(0));

    // One plan per part. The last part carries the remainder; earlier parts are
    // exactly `part_size`. A part starting at/after EOF (only possible for a
    // zero-length file) is skipped rather than PUT as a zero-length object.
    let mut plans: Vec<PartPlan> = Vec::with_capacity(part_urls.len());
    for index in 0..part_urls.len() {
        let offset = index as u64 * part_size;
        if offset >= total && total > 0 {
            continue;
        }
        let len = part_size.min(total.saturating_sub(offset));
        plans.push(PartPlan {
            index,
            part_number: (index + 1) as i32,
            offset,
            len,
        });
    }

    // Per-part uploader: a positioned read of exactly this part's byte range (so
    // a re-read on retry never shares a cursor) then a header-isolated `PUT`.
    // Captures only `Arc`s and `Copy` values, so the closure is
    // `Clone + Send + Sync + 'static` and `upload_parts_resilient` can re-run it
    // across rounds and concurrent tasks.
    let part_urls = Arc::new(part_urls);
    let headers = Arc::new(session.headers.clone());
    let path = Arc::new(path.to_path_buf());
    let retry = configuration.retry; // RetryPolicy is Copy.
    let progress = progress.cloned();

    let uploader = move |plan: PartPlan| {
        let part_urls = Arc::clone(&part_urls);
        let headers = Arc::clone(&headers);
        let path = Arc::clone(&path);
        let done = Arc::clone(&done);
        let progress = progress.clone();
        async move {
            let url = part_urls[plan.index].clone();
            let chunk = read_range(&path, plan.offset, plan.len).await?;
            let resp = put_to_storage(
                &retry,
                &url,
                &headers,
                chunk,
                plan.len,
                Some(plan.part_number),
            )
            .await?;
            let e_tag = parse_etag(resp.headers(), plan.part_number)?;
            if let Some(progress) = progress.as_ref() {
                let now = done.fetch_add(plan.len, Ordering::SeqCst) + plan.len;
                progress(now, total);
            }
            Ok(models::FinalizeUploadPart {
                e_tag,
                part_number: plan.part_number,
            })
        }
    };

    // Resilient outer loop: a single part's transient failure no longer aborts
    // the whole upload — it is re-swept on a later round while completed parts
    // keep their ETags. `upload_parts_resilient` returns the parts ascending by
    // part number with no duplicates.
    upload_parts_resilient(plans, in_flight_cap, RetryRounds::default(), uploader).await
}

/// Streaming (just-in-time) multipart path: the session was opened WITHOUT a
/// declared size, so the server minted no part URLs up front. We still know the
/// local file's size, so the part count is fixed by the server's echoed
/// `part_size`.
///
/// Each part mints a FRESH presigned URL immediately before its `PUT` (via
/// `POST /v1/uploads/{id}/parts`), so a URL can never expire mid-transfer on a
/// slow upload — and a part re-swept by [`upload_parts_resilient`] simply
/// re-mints. This replaces the earlier batched pre-mint pipeline and its
/// one-shot on-`403` re-mint: per-part minting is simpler and fully resilient,
/// and with bounded concurrency the extra mint round-trip overlaps other parts'
/// in-flight `PUT`s rather than serializing.
///
/// Returns the parts sorted ascending by part number, ready for finalize.
async fn upload_multipart_streaming(
    configuration: &Configuration,
    session: &models::UploadSessionResponse,
    path: &Path,
    total: u64,
    max_concurrency: usize,
    progress: Option<&UploadProgress>,
) -> Result<Vec<models::FinalizeUploadPart>, UploadError> {
    let part_size = session.part_size.flatten().ok_or_else(|| {
        UploadError::MalformedSession("streaming upload missing `part_size`".to_owned())
    })?;
    if part_size <= 0 {
        return Err(UploadError::MalformedSession(format!(
            "streaming upload has non-positive `part_size` {part_size}"
        )));
    }
    let part_size = part_size as u64;

    // Slice by the SERVER's echoed part size (never our hint); the last part is
    // the remainder. We know the file size, so the part count is fixed up front
    // even though the server does not.
    let expected_parts = total.div_ceil(part_size).max(1) as usize;

    // Peak buffered memory is in_flight * part_size; bound in-flight by both the
    // caller's max_concurrency and the memory budget (same as the eager path).
    let in_flight_cap = effective_in_flight(max_concurrency, part_size);

    // One plan per part (same shape as the known-size path). A part starting
    // at/after EOF (only possible for a zero-length file) is skipped rather than
    // PUT as a zero-length object.
    let mut plans: Vec<PartPlan> = Vec::with_capacity(expected_parts);
    for index in 0..expected_parts {
        let offset = index as u64 * part_size;
        if offset >= total && total > 0 {
            continue;
        }
        let len = part_size.min(total.saturating_sub(offset));
        plans.push(PartPlan {
            index,
            part_number: (index + 1) as i32,
            offset,
            len,
        });
    }

    // Per-part uploader: mint a fresh URL for THIS part immediately before
    // uploading it, then PUT. Captures only `Arc`s and `Copy` values, so the
    // closure is `Clone + Send + Sync + 'static` and `upload_parts_resilient`
    // can re-run it across rounds and concurrent tasks; a re-swept part re-mints
    // a fresh URL, so expiry is impossible.
    let config = Arc::new(configuration.clone());
    let upload_id = Arc::new(session.upload_id.clone());
    let finalize_token = Arc::new(session.finalize_token.clone());
    let headers = Arc::new(session.headers.clone());
    let path = Arc::new(path.to_path_buf());
    let retry = configuration.retry;
    let done = Arc::new(AtomicU64::new(0));
    let progress = progress.cloned();

    let uploader = move |plan: PartPlan| {
        let config = Arc::clone(&config);
        let upload_id = Arc::clone(&upload_id);
        let finalize_token = Arc::clone(&finalize_token);
        let headers = Arc::clone(&headers);
        let path = Arc::clone(&path);
        let done = Arc::clone(&done);
        let progress = progress.clone();
        async move {
            let minted = apis::uploads_api::mint_upload_parts_handler(
                &config,
                &upload_id,
                &finalize_token,
                models::MintUploadPartsRequest::new(vec![plan.part_number]),
            )
            .await
            .map_err(UploadError::MintParts)?;
            let url = minted
                .parts
                .into_iter()
                .find(|p| p.part_number == plan.part_number)
                .map(|p| p.url)
                .ok_or_else(|| {
                    UploadError::MalformedSession(format!(
                        "mint returned no URL for part {}",
                        plan.part_number
                    ))
                })?;

            let chunk = read_range(&path, plan.offset, plan.len).await?;
            let resp = put_to_storage(
                &retry,
                &url,
                &headers,
                chunk,
                plan.len,
                Some(plan.part_number),
            )
            .await?;
            let e_tag = parse_etag(resp.headers(), plan.part_number)?;
            if let Some(progress) = progress.as_ref() {
                let now = done.fetch_add(plan.len, Ordering::SeqCst) + plan.len;
                progress(now, total);
            }
            Ok(models::FinalizeUploadPart {
                e_tag,
                part_number: plan.part_number,
            })
        }
    };

    // Same resilient outer loop as the known-size path: a transient part failure
    // is re-swept on a later round (re-minting a fresh URL) instead of aborting
    // the whole upload; completed parts keep their ETags.
    upload_parts_resilient(plans, in_flight_cap, RetryRounds::default(), uploader).await
}

/// Extract and validate the storage `ETag` from a part `PUT` response. Rejects a
/// missing OR empty/whitespace-only header: finalize needs a real ETag per part,
/// and an empty value would be carried into the completion request only to fail
/// (or silently corrupt) it later. Treated as [`UploadError::MissingETag`], so a
/// re-sweep can re-`PUT` the part and pick up a real ETag.
fn parse_etag(
    headers: &reqwest::header::HeaderMap,
    part_number: i32,
) -> Result<String, UploadError> {
    let etag = headers
        .get(reqwest::header::ETAG)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_owned())
        .ok_or(UploadError::MissingETag { part_number })?;
    if etag.trim().is_empty() {
        return Err(UploadError::MissingETag { part_number });
    }
    Ok(etag)
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

/// Connect-phase timeout for storage `PUT`s. Bounds only TCP+TLS establishment
/// (not the transfer), so it is safe for both the bounded multipart parts and
/// the unbounded single-`PUT` whole-file path. Generous: a healthy connect is
/// sub-second, so 30 s only trips a genuinely dead/black-holed endpoint.
const STORAGE_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// Fixed slack added to every per-part timeout for connect/TLS, request
/// queueing, and the response round-trip, independent of part size.
const PART_TIMEOUT_BASE: Duration = Duration::from_secs(60);

/// Throughput floor used to size the per-part timeout. A part is only aborted if
/// it cannot sustain even this rate — 64 KiB/s (≈512 kbit/s), well below any
/// link on which an upload is worth attempting — so a legitimately slow but
/// progressing transfer is never killed; only a true stall is.
const PART_TIMEOUT_MIN_BYTES_PER_SEC: u64 = 64 * 1024;

/// Operational ceiling on the per-part timeout. Without it a huge part (e.g. a
/// 5 GiB part on a multi-TB upload) would compute a ~22-hour timeout, so a
/// stalled giant part would hang for the better part of a day before the outer
/// loop could re-sweep it. 30 minutes still comfortably covers a legitimately
/// slow large part while keeping stall recovery bounded.
const PART_TIMEOUT_MAX: Duration = Duration::from_secs(30 * 60);

/// Generous per-part total `PUT` timeout, scaled to the part size: a fixed base
/// plus the time the part would take at the throughput floor, capped at
/// [`PART_TIMEOUT_MAX`]. Examples: an 8 MiB part → ~188 s; a 64 MiB part →
/// ~18 min; anything above ~111 MiB → the 30 min cap. The goal is to catch a
/// stalled connection (which would otherwise hang the upload forever) without
/// aborting a healthy slow link — the outer [`upload_parts_resilient`] loop then
/// re-sweeps the timed-out part.
fn part_put_timeout(content_length: u64) -> Duration {
    (PART_TIMEOUT_BASE + Duration::from_secs(content_length / PART_TIMEOUT_MIN_BYTES_PER_SEC))
        .min(PART_TIMEOUT_MAX)
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
/// Sent on the dedicated, header-bare [`storage_client`] with a generous,
/// part-size-scaled request timeout (see [`part_put_timeout`]) so a stalled
/// connection fails — into the outer retry loop — instead of hanging forever,
/// while a legitimately slow but progressing part is never aborted. The body
/// buffers in memory so it clones cleanly across retries via
/// [`crate::http::execute_retrying`]. Part `PUT`s are retryable: storage
/// overwrites a part by number, so a retried part is idempotent. `retry` is the
/// SDK's retry policy (carried on `Configuration`), used only for the retry
/// timing here.
async fn put_to_storage(
    retry: &crate::query::RetryPolicy,
    url: &str,
    headers: &HashMap<String, String>,
    body: bytes::Bytes,
    content_length: u64,
    part_number: Option<i32>,
) -> Result<reqwest::Response, UploadError> {
    let client = storage_client();

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

    // Per-PART total timeout, scaled to the part size. Bounds a single part so a
    // silently black-holed connection (no RST, write just stalls — which a
    // read/idle timeout would not catch) fails instead of hanging the whole
    // upload forever. NOT applied to the single-`PUT` whole-file path, which is
    // legitimately unbounded. `try_clone` in `execute_retrying` preserves this
    // per-request timeout, so every inner attempt gets a fresh full budget.
    req_builder = req_builder.timeout(part_put_timeout(content_length));

    let req = req_builder.build().map_err(UploadError::Storage)?;
    crate::http_log::log_request(&req);
    let resp = crate::http::execute_retrying(&client, req, retry)
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
    url: &str,
    headers: &HashMap<String, String>,
    body: S,
    content_length: u64,
) -> Result<reqwest::Response, UploadError>
where
    S: futures_core::Stream<Item = std::io::Result<bytes::Bytes>> + Send + 'static,
{
    let client = storage_client();

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

/// The dedicated, process-wide reqwest client used for storage `PUT`s.
///
/// Deliberately NOT `configuration.client`: a host app may have installed
/// default headers (auth / workspace / `User-Agent` / `Content-Type`) on the
/// SDK's main client, which reqwest would then apply to the storage `PUT` —
/// making S3-compatible storage return `403 SignatureDoesNotMatch`. This client
/// is built bare: no default headers, and no request timeout (a large upload
/// legitimately takes minutes). It is built once and reused.
///
/// Trade-off: TLS / proxy / connection-pool settings on the SDK's main client
/// do NOT apply to storage `PUT`s — they go through this independent client.
/// That is intentional; storage transfers must be header-isolated, and a
/// host-configured proxy for the API host is not assumed to front object
/// storage.
fn storage_client() -> reqwest::Client {
    static STORAGE_CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    STORAGE_CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                // No `default_headers` and no client-wide request `timeout` (the
                // single-`PUT` whole-file path is legitimately unbounded; the
                // multipart path bounds each part per-request — see
                // `part_put_timeout`). A connect timeout is safe for both: it
                // bounds only connection establishment, not the transfer, so a
                // dead endpoint fails fast into the retry/outer loop.
                .connect_timeout(STORAGE_CONNECT_TIMEOUT)
                .build()
                // Falls back to a plain default client if the builder somehow
                // fails (e.g. no TLS backend); still header-bare.
                .unwrap_or_default()
        })
        .clone()
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
    fn effective_in_flight_honors_explicit_low_concurrency() {
        // An explicit max_concurrency of 1 means serial uploads — NOT raised to a
        // floor of 2. (Regression guard for the Codex finding.)
        assert_eq!(effective_in_flight(1, 8 * MIB), 1);
        // 0 is normalized to 1 (you can't run zero in flight), not to 2.
        assert_eq!(effective_in_flight(0, 8 * MIB), 1);
        // 2 stays 2.
        assert_eq!(effective_in_flight(2, 8 * MIB), 2);
    }

    #[test]
    fn effective_in_flight_floors_at_1_for_huge_parts_and_handles_zero() {
        // A part larger than the whole budget still keeps at least 1 in flight
        // (the budget can't bound below a single part).
        assert_eq!(effective_in_flight(12, UPLOAD_MEMORY_BUDGET * 4), 1);
        // Zero part size doesn't divide-by-zero (treated as 1 byte): the budget
        // then allows a huge count, so max_concurrency wins.
        assert_eq!(effective_in_flight(12, 0), 12);
    }
}

#[cfg(test)]
mod resilient_retry_tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Mutex;

    fn plan(n: i32) -> PartPlan {
        PartPlan {
            index: (n - 1) as usize,
            part_number: n,
            offset: (n as u64 - 1) * 16,
            len: 16,
        }
    }
    fn plans(count: i32) -> Vec<PartPlan> {
        (1..=count).map(plan).collect()
    }
    fn no_delay(max_extra_rounds: u32) -> RetryRounds {
        RetryRounds {
            max_extra_rounds,
            base_delay: Duration::ZERO,
        }
    }

    /// A transport-free stand-in for the real per-part uploader. It records
    /// attempts per part and can be told to fail the first K attempts of
    /// specific parts (modelling a part whose inner transport retries were
    /// exhausted by a network blip) before succeeding. Also tracks peak
    /// in-flight concurrency to verify the cap is honored.
    #[derive(Clone)]
    struct FakeUploader {
        fail: Arc<Mutex<HashMap<i32, usize>>>, // part_number -> remaining forced failures
        attempts: Arc<Mutex<HashMap<i32, usize>>>,
        in_flight: Arc<AtomicUsize>,
        peak_in_flight: Arc<AtomicUsize>,
    }

    impl FakeUploader {
        fn new(fail: HashMap<i32, usize>) -> Self {
            Self {
                fail: Arc::new(Mutex::new(fail)),
                attempts: Arc::new(Mutex::new(HashMap::new())),
                in_flight: Arc::new(AtomicUsize::new(0)),
                peak_in_flight: Arc::new(AtomicUsize::new(0)),
            }
        }
        fn attempts_for(&self, n: i32) -> usize {
            *self.attempts.lock().unwrap().get(&n).unwrap_or(&0)
        }
        fn peak(&self) -> usize {
            self.peak_in_flight.load(Ordering::SeqCst)
        }

        fn call(
            &self,
            plan: PartPlan,
        ) -> impl std::future::Future<Output = Result<models::FinalizeUploadPart, UploadError>>
               + Send
               + 'static {
            let fail = Arc::clone(&self.fail);
            let attempts = Arc::clone(&self.attempts);
            let in_flight = Arc::clone(&self.in_flight);
            let peak = Arc::clone(&self.peak_in_flight);
            async move {
                let cur = in_flight.fetch_add(1, Ordering::SeqCst) + 1;
                peak.fetch_max(cur, Ordering::SeqCst);
                // Force overlap so peak-in-flight reflects real concurrency.
                tokio::task::yield_now().await;
                *attempts
                    .lock()
                    .unwrap()
                    .entry(plan.part_number)
                    .or_insert(0) += 1;
                let should_fail = {
                    let mut f = fail.lock().unwrap();
                    match f.get_mut(&plan.part_number) {
                        Some(remaining) if *remaining > 0 => {
                            *remaining -= 1;
                            true
                        }
                        _ => false,
                    }
                };
                in_flight.fetch_sub(1, Ordering::SeqCst);
                if should_fail {
                    Err(UploadError::Io(std::io::Error::other(
                        "simulated connection reset",
                    )))
                } else {
                    Ok(models::FinalizeUploadPart {
                        e_tag: format!("etag-{}", plan.part_number),
                        part_number: plan.part_number,
                    })
                }
            }
        }
    }

    // ---- pure policy ----

    #[test]
    fn round_in_flight_halves_each_round_min_one() {
        assert_eq!(round_in_flight(8, 0), 8);
        assert_eq!(round_in_flight(8, 1), 4);
        assert_eq!(round_in_flight(8, 2), 2);
        assert_eq!(round_in_flight(8, 3), 1);
        assert_eq!(round_in_flight(8, 99), 1);
        assert_eq!(round_in_flight(1, 3), 1);
    }

    #[test]
    fn delay_for_grows_exponentially() {
        let r = RetryRounds {
            max_extra_rounds: 3,
            base_delay: Duration::from_secs(2),
        };
        assert_eq!(r.delay_for(1), Duration::from_secs(2));
        assert_eq!(r.delay_for(2), Duration::from_secs(4));
        assert_eq!(r.delay_for(3), Duration::from_secs(8));
    }

    #[test]
    fn part_put_timeout_is_generous_and_scales_with_part_size() {
        // 8 MiB part: 60s base + 8MiB / 64KiB/s = 60 + 128 = 188s. Comfortably
        // above the ~3s an 8 MiB part takes on a healthy link, so a legit slow
        // transfer is never aborted; only a true stall trips it.
        assert_eq!(part_put_timeout(8 * 1024 * 1024), Duration::from_secs(188));
        // 64 MiB part stays generous (~18 min).
        assert_eq!(
            part_put_timeout(64 * 1024 * 1024),
            Duration::from_secs(60 + 1024)
        );
        // A tiny/empty part still gets the full fixed base.
        assert_eq!(part_put_timeout(0), Duration::from_secs(60));
        // Monotonic in part size (below the cap).
        assert!(part_put_timeout(32 * 1024 * 1024) > part_put_timeout(8 * 1024 * 1024));
        // A huge part is capped at the 30 min operational ceiling rather than the
        // ~22.8 h the raw formula would yield, so stall recovery stays bounded.
        assert_eq!(part_put_timeout(5 * 1024 * 1024 * 1024), PART_TIMEOUT_MAX);
        assert_eq!(part_put_timeout(u64::MAX), PART_TIMEOUT_MAX);
    }

    #[test]
    fn terminal_errors_are_only_contract_violations() {
        assert!(is_terminal(&UploadError::MalformedSession("bad".into())));
        assert!(is_terminal(&UploadError::SizeOverflow {
            what: "x",
            value: 1,
        }));
        // Network-ish failures must stay retryable so the outer loop re-sweeps.
        assert!(!is_terminal(&UploadError::Io(std::io::Error::other(
            "reset"
        ))));
        assert!(!is_terminal(&UploadError::MissingETag { part_number: 1 }));
        assert!(!is_terminal(&UploadError::StorageStatus {
            status: reqwest::StatusCode::INTERNAL_SERVER_ERROR,
            part_number: Some(1),
            body: String::new(),
        }));
    }

    #[test]
    fn parse_etag_rejects_missing_and_blank() {
        use reqwest::header::{HeaderMap, HeaderValue, ETAG};
        let mut ok = HeaderMap::new();
        ok.insert(ETAG, HeaderValue::from_static("\"etag-7\""));
        assert_eq!(parse_etag(&ok, 7).unwrap(), "\"etag-7\"");

        // Missing header.
        assert!(matches!(
            parse_etag(&HeaderMap::new(), 7),
            Err(UploadError::MissingETag { part_number: 7 })
        ));
        // Present but empty / whitespace-only — must be rejected, not finalized.
        for blank in ["", "   "] {
            let mut h = HeaderMap::new();
            h.insert(ETAG, HeaderValue::from_str(blank).unwrap());
            assert!(
                matches!(parse_etag(&h, 7), Err(UploadError::MissingETag { .. })),
                "blank ETag {blank:?} must be rejected"
            );
        }
    }

    // ---- REPRODUCE: legacy behavior = one pass, no outer rounds ----

    #[tokio::test]
    async fn repro_single_part_blip_sinks_whole_upload_without_rounds() {
        // Part 3 fails once. With NO extra rounds (the legacy abort-on-first-
        // exhaustion behavior) that single transient failure fails the entire
        // upload, discarding the work done on parts 1, 2, 4, 5.
        let fake = FakeUploader::new(HashMap::from([(3, 1)]));
        let f = fake.clone();
        let res = upload_parts_resilient(plans(5), 4, no_delay(0), move |p| f.call(p)).await;
        assert!(
            res.is_err(),
            "a single transient part failure should sink the upload under legacy (0-round) semantics"
        );
    }

    // ---- FIX: outer rounds re-sweep only the failed parts ----

    #[tokio::test]
    async fn fix_single_part_blip_recovers_on_a_later_round() {
        let fake = FakeUploader::new(HashMap::from([(3, 1)]));
        let f = fake.clone();
        let res = upload_parts_resilient(plans(5), 4, no_delay(3), move |p| f.call(p))
            .await
            .expect("the flaky part should recover on a later round");
        // All five parts present, ascending, with the right ETags.
        let nums: Vec<i32> = res.iter().map(|p| p.part_number).collect();
        assert_eq!(nums, vec![1, 2, 3, 4, 5]);
        assert_eq!(res[2].e_tag, "etag-3");
        // The flaky part was attempted twice (round 0 fail, round 1 success);
        // every healthy part exactly once — completed work is never redone.
        assert_eq!(fake.attempts_for(3), 2);
        for n in [1, 2, 4, 5] {
            assert_eq!(fake.attempts_for(n), 1, "part {n} must not be re-uploaded");
        }
    }

    #[tokio::test]
    async fn fix_multiple_flaky_parts_all_recover() {
        let fake = FakeUploader::new(HashMap::from([(2, 2), (5, 1), (7, 3)]));
        let f = fake.clone();
        let res = upload_parts_resilient(plans(8), 4, no_delay(3), move |p| f.call(p))
            .await
            .expect("all parts should recover within the round budget");
        assert_eq!(res.len(), 8);
        assert_eq!(fake.attempts_for(2), 3); // 2 fails + success
        assert_eq!(fake.attempts_for(7), 4); // 3 fails + success
        assert_eq!(fake.attempts_for(5), 2);
    }

    #[tokio::test]
    async fn permanent_failure_surfaced_after_exhausting_rounds() {
        // Part 4 always fails (more failures than rounds). After the initial
        // pass plus `max_extra_rounds` re-sweeps the upload gives up — but only
        // after exactly 1 + max_extra_rounds attempts of that part, and without
        // ever re-uploading the healthy parts.
        let fake = FakeUploader::new(HashMap::from([(4, 99)]));
        let f = fake.clone();
        let res = upload_parts_resilient(plans(5), 4, no_delay(2), move |p| f.call(p)).await;
        assert!(res.is_err());
        assert_eq!(fake.attempts_for(4), 3, "1 initial pass + 2 re-sweeps");
        for n in [1, 2, 3, 5] {
            assert_eq!(fake.attempts_for(n), 1);
        }
    }

    #[tokio::test]
    async fn happy_path_uploads_each_part_exactly_once() {
        let fake = FakeUploader::new(HashMap::new());
        let f = fake.clone();
        let res = upload_parts_resilient(plans(6), 4, no_delay(3), move |p| f.call(p))
            .await
            .unwrap();
        assert_eq!(res.len(), 6);
        for n in 1..=6 {
            assert_eq!(fake.attempts_for(n), 1);
        }
    }

    #[tokio::test]
    async fn concurrency_never_exceeds_base_cap() {
        let fake = FakeUploader::new(HashMap::new());
        let f = fake.clone();
        upload_parts_resilient(plans(20), 3, no_delay(3), move |p| f.call(p))
            .await
            .unwrap();
        assert!(
            fake.peak() <= 3,
            "peak in-flight {} exceeded the cap of 3",
            fake.peak()
        );
    }

    #[tokio::test]
    async fn terminal_error_fails_fast_without_resweeping() {
        // A terminal error (server-contract violation) reproduces on every
        // re-sweep, so it must fail the upload immediately — NOT be retried for
        // all rounds the way a transient failure is.
        let p2_attempts = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&p2_attempts);
        let res = upload_parts_resilient(plans(4), 4, no_delay(3), move |plan: PartPlan| {
            let counter = Arc::clone(&counter);
            async move {
                if plan.part_number == 2 {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err(UploadError::MalformedSession("contract violation".into()))
                } else {
                    Ok(models::FinalizeUploadPart {
                        e_tag: format!("etag-{}", plan.part_number),
                        part_number: plan.part_number,
                    })
                }
            }
        })
        .await;
        assert!(matches!(res, Err(UploadError::MalformedSession(_))));
        assert_eq!(
            p2_attempts.load(Ordering::SeqCst),
            1,
            "a terminal error must be attempted once, never re-swept across rounds"
        );
    }
}
