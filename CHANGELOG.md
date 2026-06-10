# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
