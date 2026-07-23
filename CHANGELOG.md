# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.10.0] - 2026-07-23

### Changed

- **Breaking:** `list_databases` and `Databases::list` now take `limit` and
  `cursor` pagination parameters; existing callers must pass `None, None`.
- Pagination metadata on `ListDatabasesResponse` (`count`, `limit`, `has_more`)
  is now nullable so the client tolerates responses from a server that predates
  these fields (rolling deploy / version skew).

### Removed

- **Breaking:** the `POST /v1/files` and `GET /v1/files` endpoints are gone in
  favor of the presigned uploads flow. This drops the generated `upload_file` /
  `list_uploads` ops, the `UploadResponse` / `UploadInfo` / `ListUploadsResponse`
  models, and the ergonomic `Client::upload_stream`, `uploads().upload`, and
  `uploads().list` wrappers. Use `Client::upload_file` (presigned
  direct-to-storage) instead.

### Added

- `LoadManagedTableRequest` gains an optional `key` field naming the key columns
  for `delete` / `update` / `upsert` loads.

## [0.9.1] - 2026-07-15

### Added

- Ergonomic `Client::databases().fork()` wrapper for the `fork_database` endpoint.

## [0.9.0] - 2026-07-15

### Changed

- feat(databases): add fork endpoint
- chore(api): exclude datasets from public OpenAPI spec + docs cleanup

## [0.8.1] - 2026-07-09

### Changed

- feat(tables): support loading from query results
- feat(databases): expose created_at on list and detail endpoints

## [0.8.0] - 2026-07-07

### Changed

- feat: support async table loads and append mode
- **Breaking:** results and query runs are now scoped to a database via the
  required `X-Database-Id` header. The ergonomic wrappers gain a `database_id`
  argument to match: `Client::get_result`, `Client::list_results`,
  `Client::list_query_runs`, `Client::await_result`, `Client::get_result_arrow`,
  `Client::stream_result_arrow`, `Client::query_to_arrow`, and the
  `results()` / `query_runs()` resource handles. `Client::query`'s truncation
  auto-follow now forwards the query's database scope (the `X-Database-Id`
  header, or the request-body `database_id` when no header is set) to the
  follow-up result and query-run fetches.

## [0.7.0] - 2026-06-30

### Added

- Multipart uploads now survive transient per-part failures. A single part
  exhausting its inner transport retries no longer aborts the whole transfer:
  an outer round loop re-sweeps just the failed parts (at decaying concurrency,
  with backoff) while completed parts keep their ETags, so a flaky or slow link
  recovers instead of discarding the work already done. Each part PUT also gets
  a part-size-scaled total timeout (bounded by an operational ceiling), so a
  silently stalled connection fails fast rather than hanging the upload.

### Changed

- Streaming multipart uploads now mint one presigned part URL per part, on
  demand immediately before each PUT, instead of pre-minting in batches. This
  keeps each URL's age minimal so it can't expire mid-transfer on a slow link,
  at the cost of more `POST /v1/uploads/{id}/parts` requests for large uploads.

### Removed

- `uploads::MAX_MINT_BATCH` (`pub const`), obsoleted by per-part minting. No
  longer part of the public API.

## [0.6.0] - 2026-06-29

### Added

- Streaming uploads with just-in-time part minting for large files.
  `Client::upload_file` now opens a streaming multipart session for files past
  the multipart threshold and mints presigned part URLs on demand (via
  `POST /v1/uploads/{id}/parts`) immediately before each chunk uploads, instead
  of pre-minting every URL when the session opens. A part URL that expires
  mid-transfer (storage `403`) is transparently re-minted and the chunk retried,
  so large or slow uploads that outlive a presigned URL's ~30-minute TTL still
  complete within the session's 24-hour window rather than failing partway.
  Small files keep the single-`PUT` fast path. (Fixes #76.)
- Low-level `POST /v1/uploads/{id}/parts` part-minting endpoint with its
  `MintUploadPartsRequest` / `MintUploadPartsResponse` / `MintedUploadPartResponse`
  models, generated from the OpenAPI spec.

### Changed

- Token exchange (`POST /v1/auth/jwt`) now retries transient failures before
  giving up: a momentary `5xx` or a transport error (connection/read failure)
  is retried with bounded exponential backoff + jitter (3 attempts total), so a
  brief server-side blip no longer fails the caller outright. A `4xx`
  (bad/expired credential) is never retried, and the last status/body is
  preserved once the budget is exhausted. Applies to both the initial mint and
  the refresh path.

### Fixed

- The default `User-Agent` is now computed from the crate version at build time
  (`CARGO_PKG_VERSION`) instead of a hardcoded string, so it always reflects the
  published version.

## [0.5.0] - 2026-06-26

### Added

- Ergonomic presigned (direct-to-storage) file uploads: `Client::upload_file`
  (and `client.uploads().upload_file`) open an upload session, `PUT` the bytes
  straight to object storage — a single `PUT` for small files, bounded-
  concurrency multipart `PUT`s sliced by the server's `part_size` for large
  ones — then finalize, returning the `FinalizeUploadResponse`. Configurable via
  `UploadOptions` (content type/encoding, filename, part-size hint, and an
  `UploadProgress` callback). Never falls back to the legacy `POST /v1/files`
  proxy; storage `PUT`s carry no SDK auth/scope headers. Multipart concurrency
  is tunable via `UploadOptions::max_concurrency` (default 10), bounded by a
  256 MiB peak-memory budget derived from the server's actual part size; when no
  `part_size` is given, the SDK auto-scales the hint (8 MiB for normal files,
  larger only past ~72 GiB to keep the part count under S3's 10,000-part limit).
  Finalize is exactly-once (sent with retries disabled so an ambiguous failure
  can't be retried into a spurious "already finalized" error); part `PUT`s stay
  retryable. Storage `PUT`s use a dedicated header-bare reqwest client, so a host
  app's default headers on the SDK's main client never leak to object storage.
  The multipart session shape is validated (`part_urls` count must match the
  file's part count) and pathological sizes (`> i64::MAX`) are rejected rather
  than silently wrapped.
- Low-level file upload endpoints generated from the OpenAPI spec, including the
  presigned upload-session and finalize operations that `upload_file` builds on.

## [0.4.0] - 2026-06-19

### Removed

- **Breaking:** the datasets API and all related types, following their removal
  from the Hotdata OpenAPI spec. This drops the `datasets_api` module and the
  `client.datasets()` resource handle; every dataset model
  (`CreateDatasetRequest`/`CreateDatasetResponse`, `GetDatasetResponse`,
  `UpdateDatasetRequest`/`UpdateDatasetResponse`, `ListDatasetsResponse`,
  `ListDatasetVersionsResponse`, `RefreshDatasetResponse`, `DatasetSummary`,
  `DatasetVersionSummary`, `DatasetSource` and its `*OneOf*` variants,
  `InlineDatasetSource`, `SavedQueryDatasetSource`, `SqlQueryDatasetSource`,
  `UploadDatasetSource`, `UrlDatasetSource`, `InlineData`, `ColumnTypeSpec`);
  and the dataset-related `JobType` variants (`DatasetRefresh`,
  `CreateDatasetIndex`).

### Fixed

- Pre-response connection errors are now retried transparently on **any** method,
  including `POST`. A pooled keep-alive socket that an intermediary closed on its
  idle timeout surfaces, on the next reuse, as a connection reset before the
  request reaches the server; since the server did no work, the retry can't
  double-execute. This covers every generated op (via `execute_retrying`) and the
  hand-written `Client::query` / `Client::submit_query` paths, governed by the
  same `RetryPolicy` budget as 429. Response-phase transport errors are left
  un-retried so a non-idempotent `POST` can't double-execute (#63).

## [0.3.2] - 2026-06-18

### Changed

- feat(indexes): add source_column field to index responses

## [0.3.1] - 2026-06-16

### Fixed

- `Client::submit_query` (the hand-written 202/async query path the CLI drives
  directly) now retries HTTP 429 (`OVERLOADED`) admission shedding per
  `Configuration::retry`, like every generated op and `Client::query`. It was
  the one query path the 0.3.0 migration missed, so under admission shedding a
  submitted query surfaced the 429 as an error with no retry (#688).

## [0.3.0] - 2026-06-16

### Added

- Every generated `apis::*` operation now transparently retries HTTP 429
  (`OVERLOADED`) admission shedding, honoring `Retry-After` with backoff before
  the op returns (#58). The policy is the new `Configuration::retry` field
  (`crate::query::RetryPolicy`, defaulting to `RetryPolicy::default`); set
  `max_retries` to 0 to disable. The enhanced query path (`crate::query`) keeps
  using its own per-call `QueryConfig::retry` instead.

## [0.2.0] - 2026-06-15

### Added

- Enhanced `Client::query`: transparently retries HTTP 429 (`OVERLOADED`)
  admission shedding honoring `Retry-After` under a deadline budget, and
  auto-follows truncated results to materialize the full row set, guarded by
  configurable `max_auto_rows` (default 1M) and `max_auto_bytes` (default
  64 MiB) ceilings (#688). The raw generated op remains reachable via
  `client.queries().execute()` / `hotdata::apis::query_api::query`.
- `hotdata::query` module with `QueryConfig`, `RetryPolicy`, `PollPolicy`, the
  `QueryError` enum, and the `ResultError` family (`Failed` / `Timeout` /
  `TooLarge` / `Incomplete` / `Unavailable`). `ClientBuilder::query_config` sets
  the instance default; `Client::query_with` takes a per-call override.
- `Client::query_in` (scope a query to a database) and `Client::query_preview`
  (return the bounded preview without auto-following), plus `QueryConfig::with_*`
  setters for fluent per-call overrides
  (`client.query_config().clone().with_auto_follow(false)`).

### Changed

- `Client::query` now returns `Result<QueryResponse, hotdata::QueryError>`
  instead of `Result<QueryResponse, Error<QueryError>>` to carry the overload
  and result-lifecycle errors the bounded-memory query contract introduces.
  Migration: where you matched the old error, e.g.
  `Err(Error::ResponseError(rc))`, now match `Err(QueryError::Submit(
  Error::ResponseError(rc)))`; overload and auto-follow failures arrive as the
  new `QueryError::Overloaded` / `QueryError::Result(..)` variants.
- `Client::query` rejects an explicit `async = true` request up front with
  `QueryError::AsyncRequested` (it is the synchronous-results path); use
  `Client::submit_query` for asynchronous submissions.

## [0.1.4] - 2026-06-15

### Changed

- feat(queries): add preview and total row count fields

## [0.1.3] - 2026-06-09

### Added

- `list_indexes_collection` (`GET /v1/indexes`) in `hotdata::apis::indexes_api` — lists indexes across every table in a database, scoped by the `X-Database-Id` header — along with the `IndexEntryResponse` and `ListIndexesPageResponse` models.

### Removed

- Sandbox endpoints and their models (`sandboxes_api`, `Sandbox`, `SandboxResponse`, `CreateSandboxRequest`, `UpdateSandboxRequest`, `ListSandboxesResponse`, `DeleteSandboxResponse`), following their removal from the Hotdata OpenAPI spec.

### Changed

- Regenerated the client from the updated Hotdata OpenAPI spec.

## [0.1.2] - 2026-06-06

### Added

- Database and managed-catalog schema/table management endpoints: `add_database_schema`, `add_database_table`, `add_managed_schema`, and `add_managed_table`, with their request/response models (`AddManagedSchemaRequest`, `AddManagedTableRequest`, `AddManagedTableDecl`, `ManagedSchemaResponse`, `ManagedTableResponse`).

### Changed

- Regenerated the client from the updated Hotdata OpenAPI spec, catching up on spec changes since 0.1.1. Generated with openapi-generator 7.22.0 (`useChrono=false`, so date-time fields remain `String` as before).

## [0.1.1] - 2026-06-05

### Changed

- Populate crate metadata (`repository`, `homepage`, `documentation`, `readme`, `keywords`, `categories`) for the crates.io listing, and link the [Hotdata CLI](https://github.com/hotdata-dev/hotdata-cli) from the README.
- Publishing to crates.io now uses Trusted Publishing (OIDC) instead of a stored API token.

## [0.1.0] - 2026-06-05

### Added

- Ergonomic `Client` and `ClientBuilder` (`hotdata::Client`) wrapping the generated `Configuration`: set an API token and workspace id, with thin async pass-throughs for `query`, `list_query_runs`, `list_results`, `get_result`, and `list_workspaces`.
- Grouped resource handles on `Client` (`client.datasets()`, `client.secrets()`, `client.query_runs()`, … one per API) so callers no longer import `hotdata::apis::*_api` free functions or thread a `&Configuration` through every call.
- Query convenience helpers: `Client::await_result` polls a persisted result to `ready` (configurable via `PollConfig`) instead of a hand-rolled loop, and `Client::query_to_arrow` submits, awaits, and decodes a result as Arrow in one call (`arrow` feature).
- `hotdata::field` helpers (`set` / `clear` / `unchanged`) for the `Option<Option<T>>` nullable-and-optional update fields, so `Some(Some(v))` / `Some(None)` intents read clearly.
- Typed `ResultStatus` / `QueryRunStatus` enums (`hotdata::status`) with `result_status()` / `run_status()` accessor traits on the response types. Each carries an `Other(String)` catch-all so unrecognized server statuses round-trip instead of failing — matching runtimedb's lenient parsing, where a generated closed enum would not.
- Transparent API-token to JWT exchange via a hand-written `TokenManager` (`hotdata::auth`): `hd_` API tokens are exchanged against `/v1/auth/jwt`, cached, refreshed, and re-minted automatically, with a 30s expiry leeway and single-flight concurrency. JWTs (`eyJ…`) are passed through unchanged. Honors `HOTDATA_DISABLE_JWT_EXCHANGE`.
- Pluggable `BearerTokenProvider` hook on `Configuration` so any async token source can drive bearer auth.
- Optional `arrow` feature: `get_result_arrow` / `stream_result_arrow` decode Apache Arrow IPC result streams into `RecordBatch`es, surface the `X-Total-Row-Count` and `rel="next"` Link headers, and map the `202`/`409`/`404`/`400` result states to typed `ArrowError` variants.
- Flat re-export surface (`hotdata::Client`, `hotdata::Configuration`, `hotdata::prelude::*`) alongside the namespaced `hotdata::apis` / `hotdata::models`. The prelude also re-exports the resource handles, `PollConfig`, the `field` helpers, and (with the `arrow` feature) `ArrowError`.
- The SDK's own error enums (`ClientError`, `TokenExchangeError`, `ArrowError`, `AwaitResultError`, `QueryToArrowError`) are `#[non_exhaustive]`, so new variants can be added without a breaking change — match them with a wildcard arm.
- Request/response debug logging (`hotdata::http_log`) covering every HTTP call — generated ops plus the hand-written `submit_query` / `upload_stream` / Arrow fetch / JWT mint. Each emits `log::debug!` records on the `hotdata::http` target (`>>> METHOD url`, headers, body; `<<< status`, body) so a host (e.g. the CLI's `--debug`) can render them with any `log` backend. `Authorization` bearer tokens and sensitive JSON/form fields (`api_token`, `secret`, `password`, …) are masked before logging; the SDK installs no logger and stays silent without a backend. The `api.mustache` template emits the hooks so they survive regeneration, and the regen-safety CI guard verifies they remain.

### Changed

- Regeneration is now safe for the hand-written ergonomic layer: the generator only overwrites generated subtrees (`src/apis`, `src/models`, `docs`) and skips `src/lib.rs`, `src/auth.rs`, `src/http_log.rs`, `src/arrow.rs`, `src/client.rs`, `src/resources.rs`, `src/field.rs`, and `Cargo.toml` via `.openapi-generator-ignore`. The regen-safety CI guard verifies all of these survive and stay wired into `lib.rs`.
- Initial generated client from the Hotdata OpenAPI spec.
