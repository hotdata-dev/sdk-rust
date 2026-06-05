# hotdata

Official Rust client for the [Hotdata](https://www.hotdata.dev) HTTP API: workspaces, connections, datasets, SQL queries, results, secrets, uploads, indexes, jobs, embedding providers, and workspace context.

The crate pairs a fully generated, typed API surface (`hotdata::apis`, `hotdata::models`) with a hand-written ergonomic layer: a flat [`Client`](#quickstart) that handles transparent API-token to JWT exchange, plus an optional Apache Arrow result decoder.

## Requirements

Rust 1.74+ and a [Tokio](https://tokio.rs/) runtime (the client is async).

## Install

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
hotdata = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

For an unreleased revision:

```toml
[dependencies]
hotdata = { git = "https://github.com/hotdata-dev/sdk-rust.git" }
```

By default the crate builds against `native-tls`. To use `rustls` instead:

```toml
[dependencies]
hotdata = { version = "0.1", default-features = false, features = ["rustls"] }
```

## Authentication

The API authenticates with an **API token** sent as `Authorization: Bearer <token>`, plus an **`X-Workspace-Id`** header on requests scoped to a workspace.

API tokens (prefixed `hd_`) are exchanged transparently for short-lived JWTs the first time a request is made, and the JWT is cached and refreshed automatically. You only ever supply the API token — the `Client` does the exchange against `/v1/auth/jwt` for you, mirroring the [Hotdata CLI](https://github.com/hotdata-dev/hotdata-cli).

If you already hold a JWT (a value beginning with `eyJ`), it is passed through unchanged with no exchange. To disable the exchange entirely, set `HOTDATA_DISABLE_JWT_EXCHANGE` to `1`, `true`, `yes`, or `on`.

```rust
use hotdata::prelude::*;

let client = Client::builder()
    .api_token("hd_your_api_token")
    .workspace_id("your_workspace_id")
    .build()?;
```

`base_url` defaults to `https://api.hotdata.dev`. Override it if you target another environment.

## Quickstart

```rust
use hotdata::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .api_token("hd_your_api_token")
        .workspace_id("your_workspace_id")
        // .base_url("https://api.hotdata.dev")  // optional
        .build()?;

    // Submit a query. Rows come back inline, plus a result_id that is persisted
    // asynchronously for later retrieval.
    let response = client
        .query(QueryRequest::new("SELECT 1 AS n".to_string()))
        .await?;

    if let Some(result_id) = response.result_id.flatten() {
        // Poll the persisted result to `ready` without hand-rolling a loop.
        let result = client.await_result(&result_id, PollConfig::default()).await?;
        println!("result {} is {}", result.result_id, result.status);
    }

    Ok(())
}
```

### Resource handles

The OpenAPI generator emits free functions; the `Client` groups them into
ergonomic, workspace-scoped handles so you never pass a `Configuration` around:

```rust
// Grouped handles: client.<resource>().<operation>(..)
let datasets = client.datasets().list(Some(20), None).await?;
let dataset  = client.datasets().get(&datasets.datasets[0].id).await?;
let secrets  = client.secrets().list().await?;
let runs     = client.query_runs().list(Some(50), None, None, None).await?;
```

Handles exist for every resource — `datasets`, `connections`, `connection_types`,
`databases`, `database_context`, `embedding_providers`, `indexes`,
`information_schema`, `jobs`, `queries`, `query_runs`, `results`, `refresh`,
`sandboxes`, `saved_queries`, `secrets`, `uploads`, `workspaces`. The hottest
operations also have flat shortcuts directly on `Client` (`query`, `get_result`,
`list_results`, `list_query_runs`, `list_workspaces`).

For anything not yet wrapped, the full generated surface is one call away via
`client.configuration()`:

```rust
use hotdata::apis::workspaces_api;
let workspaces = workspaces_api::list_workspaces(client.configuration(), None).await?;
```

### Typed status

Result and query-run `status` fields are plain strings on the wire. Interpret
them with the typed [`ResultStatus`] / [`QueryRunStatus`] enums via the
`result_status()` / `run_status()` accessors:

```rust
use hotdata::prelude::*;

let result = client.await_result(&result_id, PollConfig::default()).await?;
if result.result_status().is_ready() {
    // ...
}

let run = client.query_runs().get(&query_run_id).await?;
if run.run_status().is_terminal() { /* ... */ }
```

Both enums carry an `Other(String)` variant, so a status the server adds later
round-trips instead of breaking deserialization.

### Updating nullable fields

Several update requests model a field that is both optional (omit to leave
unchanged) and nullable (send `null` to clear) as `Option<Option<T>>`. The
[`field`](https://docs.rs/hotdata/latest/hotdata/field/) helpers name the three
intents so call sites read clearly:

```rust
use hotdata::field;

let mut req = UpdateDatasetRequest::new();
req.label = field::set("renamed");    // set
req.pinned_version = field::clear();  // send null (unpin)
// req.table_name left as None -> omitted -> unchanged
client.datasets().update(&dataset.id, req).await?;
```

Every resource lives under `hotdata::apis::<resource>_api`, and request/response
types under `hotdata::models`. The flat `prelude` re-exports `Client`,
`ClientBuilder`, `PollConfig`, `Configuration`, the resource handles, and all
models for convenience.

Errors from generated operations are returned as `hotdata::Error<T>`; builder
and configuration failures are `hotdata::ClientError`. Result-polling and
one-call helpers return `hotdata::AwaitResultError` / `hotdata::QueryToArrowError`.
The SDK's own error enums are `#[non_exhaustive]`, so match them with a wildcard
arm.

## Arrow results

Query results can be fetched as an [Apache Arrow](https://arrow.apache.org/) IPC stream instead of JSON, which is faster and far more memory-efficient for large result sets. The decoder is behind an optional `arrow` feature (off by default):

```toml
[dependencies]
hotdata = { version = "0.1", features = ["arrow"] }
```

```rust
use hotdata::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .api_token("hd_your_api_token")
        .workspace_id("your_workspace_id")
        .build()?;

    // Buffered: decodes every batch into a Vec<RecordBatch>.
    let result = client.get_result_arrow(&result_id, None, None).await?;
    println!("schema: {:?}", result.schema);
    println!("total rows: {:?}", result.total_row_count);
    for batch in &result.batches {
        // work with each arrow_array::RecordBatch
    }

    // Streaming: yields batches lazily without holding them all at once.
    let mut stream = client.stream_result_arrow(&result_id, None, None).await?;
    for batch in stream.by_ref() {
        let batch = batch?;
        // ...
    }

    Ok(())
}
```

Both methods accept `offset` and `limit` for pagination, and both honor the transparent JWT exchange. They return `ArrowError::NotReady` if the result is still pending or processing — poll `client.get_result(result_id)` until its status is `ready` first. `ArrowResult` also surfaces the `X-Total-Row-Count` header (`total_row_count`) and the `rel="next"` pagination `Link` (`next_link`).

To run a query and get its result as Arrow in a single call — submit, await
`ready`, and decode — use `query_to_arrow`:

```rust
let arrow = client
    .query_to_arrow(
        QueryRequest::new("SELECT * FROM big_table".to_string()),
        PollConfig::default(),
        None, // offset
        None, // limit
    )
    .await?;
```

## Debug logging

Every HTTP call the SDK makes — generated operations and the hand-written `submit_query`, `upload_stream`, Arrow fetch, and JWT mint — emits `log::debug!` records on the `hotdata::http` target: the request (`>>> METHOD url`, headers, body) and the response (`<<< status`, body). `Authorization` bearer tokens and sensitive body fields (`api_token`, `secret`, `password`, …) are masked before logging.

The SDK installs no logger and prints nothing on its own. To see the records, wire any [`log`](https://docs.rs/log) backend and enable the `hotdata::http` target at debug level. For example with [`env_logger`](https://docs.rs/env_logger):

```rust
// RUST_LOG=hotdata::http=debug cargo run
env_logger::init();
```

```toml
[dependencies]
env_logger = "0.11"
```

```text
>>> POST https://api.hotdata.dev/v1/query
  authorization: Bearer hd_a...cdef
  content-type: application/json
{"sql":"SELECT 1"}
<<< 200 OK
{"result_id":"…","columns":[…]}
```

## API reference

Generated documentation builds on [docs.rs](https://docs.rs/hotdata) (with `all-features` enabled, so the `arrow` surface is included).

Generated Markdown for every operation and model also lives in [`docs/`](https://github.com/hotdata-dev/sdk-rust/tree/main/docs):

- Resource APIs: `docs/*Api.md` (for example [`QueryApi.md`](https://github.com/hotdata-dev/sdk-rust/blob/main/docs/QueryApi.md))
- Request and response models: `docs/<ModelName>.md`

## Support

Questions and issues: [github.com/hotdata-dev/sdk-rust](https://github.com/hotdata-dev/sdk-rust).
