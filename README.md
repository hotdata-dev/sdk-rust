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

API tokens (prefixed `hd_`) are exchanged transparently for short-lived JWTs the first time a request is made, and the JWT is cached and refreshed automatically. You only ever supply the API token — the `Client` does the exchange against `/v1/auth/jwt` for you, mirroring the Hotdata CLI.

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

    // Submit a query (async; runs server-side).
    let run = client
        .query(QueryRequest::new("SELECT 1 AS n".to_string()))
        .await?;

    // Poll the run to a terminal state, then fetch results.
    let result = client.get_result(&result_id).await?;

    Ok(())
}
```

The `Client` exposes thin async pass-throughs for the common operations — `query`, `get_query_run`, `list_results`, `get_result`. For any other endpoint, reach the full generated surface with `client.configuration()`:

```rust
use hotdata::apis::workspaces_api;

let workspaces = workspaces_api::list_workspaces(client.configuration()).await?;
```

Every resource lives under `hotdata::apis::<resource>_api`, and request/response types under `hotdata::models`. The flat `prelude` re-exports `Client`, `ClientBuilder`, `Configuration`, and all models for convenience.

Errors from generated operations are returned as `hotdata::Error<T>`; builder and configuration failures are `hotdata::ClientError`.

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

## API reference

Generated documentation builds on [docs.rs](https://docs.rs/hotdata) (with `all-features` enabled, so the `arrow` surface is included).

Generated Markdown for every operation and model also lives in [`docs/`](https://github.com/hotdata-dev/sdk-rust/tree/main/docs):

- Resource APIs: `docs/*Api.md` (for example [`QueryApi.md`](https://github.com/hotdata-dev/sdk-rust/blob/main/docs/QueryApi.md))
- Request and response models: `docs/<ModelName>.md`

## Support

Questions and issues: [github.com/hotdata-dev/sdk-rust](https://github.com/hotdata-dev/sdk-rust).
