# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Ergonomic `Client` and `ClientBuilder` (`hotdata::Client`) wrapping the generated `Configuration`: set an API token and workspace id, with thin async pass-throughs for `query`, `get_query_run`, `list_results`, and `get_result`.
- Transparent API-token to JWT exchange via a hand-written `TokenManager` (`hotdata::auth`): `hd_` API tokens are exchanged against `/v1/auth/jwt`, cached, refreshed, and re-minted automatically, with a 30s expiry leeway and single-flight concurrency. JWTs (`eyJ…`) are passed through unchanged. Honors `HOTDATA_DISABLE_JWT_EXCHANGE`.
- Pluggable `BearerTokenProvider` hook on `Configuration` so any async token source can drive bearer auth.
- Optional `arrow` feature: `get_result_arrow` / `stream_result_arrow` decode Apache Arrow IPC result streams into `RecordBatch`es, surface the `X-Total-Row-Count` and `rel="next"` Link headers, and map the `202`/`409`/`404`/`400` result states to typed `ArrowError` variants.
- Flat re-export surface (`hotdata::Client`, `hotdata::Configuration`, `hotdata::prelude::*`) alongside the namespaced `hotdata::apis` / `hotdata::models`.

### Changed

- Regeneration is now safe for the hand-written ergonomic layer: the generator only overwrites generated subtrees (`src/apis`, `src/models`, `docs`) and skips `src/lib.rs`, `src/auth.rs`, `src/arrow.rs`, `src/client.rs`, and `Cargo.toml` via `.openapi-generator-ignore`.

## [0.1.0]

### Changed

- Initial generated client from the Hotdata OpenAPI spec.
