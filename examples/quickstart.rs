//! HotData Rust SDK quickstart.
//!
//! End-to-end tour of the ergonomic surface, using *only* the public API
//! (`hotdata::prelude::*` plus a couple of re-exported error types). It:
//!
//!   1. builds a [`Client`] from an API token + workspace id,
//!   2. lists the workspaces the token can see,
//!   3. submits a SQL query and polls the result until it is `ready`,
//!   4. fetches the same result as Arrow record batches (behind the `arrow`
//!      feature).
//!
//! Transparent JWT exchange is automatic: you pass the opaque `hd_...` API
//! token and the SDK mints/refreshes a short-lived JWT behind the scenes on the
//! first authenticated request. There is nothing to call and nothing to cache.
//!
//! ## Running
//!
//! ```sh
//! export HOTDATA_API_KEY="hd_live_..."
//! export HOTDATA_WORKSPACE_ID="ws_..."
//! # optional: export HOTDATA_API_URL="https://api.hotdata.dev"
//!
//! cargo run --example quickstart --all-features
//! ```
//!
//! With no credentials set the example prints a short notice and exits 0, so it
//! always compiles and always runs cleanly in CI.

use std::time::Duration;

use hotdata::prelude::*;
use hotdata::ClientError;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        // A single human-readable error chain. Every error type in the SDK
        // implements `std::error::Error`, so `{err}` is always meaningful.
        eprintln!("quickstart failed: {err}");
        let mut source = err.source();
        while let Some(cause) = source {
            eprintln!("  caused by: {cause}");
            source = cause.source();
        }
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // --- 1. Build a client -----------------------------------------------
    //
    // `api_token` / `workspace_id` are optional on the builder: when omitted
    // they fall back to HOTDATA_API_KEY / HOTDATA_WORKSPACE_ID. We let that
    // fallback drive the example, and turn the "missing creds" error into a
    // graceful skip rather than a failure.
    let client = match Client::builder().build() {
        Ok(client) => client,
        // ClientError is small and exhaustive: the only failure modes are
        // missing credentials. Treat both as a graceful skip.
        Err(ClientError::MissingApiToken) | Err(ClientError::MissingWorkspaceId) => {
            eprintln!(
                "No credentials found. Set HOTDATA_API_KEY and HOTDATA_WORKSPACE_ID \
                 to run this example against the live API. Skipping."
            );
            return Ok(());
        }
    };

    println!("Built client targeting {}", client.configuration().base_path);

    // --- 2. List workspaces ----------------------------------------------
    //
    // The first authenticated call transparently exchanges the API token for a
    // JWT; subsequent calls reuse the cached token until it nears expiry.
    let workspaces = client.list_workspaces(None).await?;
    println!("Visible workspaces ({}):", workspaces.workspaces.len());
    for ws in &workspaces.workspaces {
        println!("  - {} ({})", ws.name, ws.public_id);
    }

    // --- 3. Submit a query and poll the result --------------------------
    //
    // POST /query returns rows inline *and* a result_id; persistence to the
    // result store then completes asynchronously, so we poll get_result until
    // its status is "ready".
    let response = client
        .query(QueryRequest::new(
            "select 1 as id, 'hello' as greeting".to_string(),
        ))
        .await?;

    println!(
        "Query ran in {} ms, returned {} row(s) inline. Columns: {:?}",
        response.execution_time_ms, response.row_count, response.columns
    );

    // result_id is Option<Option<String>>: outer None = field absent, inner
    // None = explicit null (persistence could not be initiated, see `warning`).
    let result_id = match response.result_id.flatten() {
        Some(id) => id,
        None => {
            if let Some(Some(warning)) = response.warning {
                println!("Result was not persisted: {warning}");
            } else {
                println!("Query returned no persisted result_id; nothing to fetch.");
            }
            return Ok(());
        }
    };

    println!("Polling result {result_id} until ready...");
    let ready = poll_until_ready(&client, &result_id).await?;
    println!("Result status: {}", ready.status);

    // --- 4. Fetch the result as Arrow (feature-gated) -------------------
    fetch_arrow(&client, &result_id).await?;

    Ok(())
}

/// Poll `get_result` until the persisted result reaches a terminal state.
///
/// Returns the response once `status == "ready"`; errors out on `"failed"`.
async fn poll_until_ready(
    client: &Client,
    result_id: &str,
) -> Result<hotdata::models::GetResultResponse, Box<dyn std::error::Error>> {
    const MAX_ATTEMPTS: u32 = 30;
    for attempt in 1..=MAX_ATTEMPTS {
        let result = client.get_result(result_id).await?;
        match result.status.as_str() {
            "ready" => return Ok(result),
            "failed" => {
                let msg = result
                    .error_message
                    .flatten()
                    .unwrap_or_else(|| "unknown error".to_string());
                return Err(format!("result {result_id} failed: {msg}").into());
            }
            other => {
                println!("  attempt {attempt}/{MAX_ATTEMPTS}: status={other}, waiting...");
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
    Err(format!("result {result_id} not ready after {MAX_ATTEMPTS} attempts").into())
}

/// Fetch the result as Arrow record batches when the `arrow` feature is on.
#[cfg(feature = "arrow")]
async fn fetch_arrow(
    client: &Client,
    result_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use hotdata::ArrowError;

    println!("Fetching result {result_id} as Arrow...");
    match client.get_result_arrow(result_id, None, None).await {
        Ok(arrow) => {
            println!(
                "Arrow result: {} batch(es), {} row(s){}",
                arrow.batches.len(),
                arrow.num_rows(),
                arrow
                    .total_row_count
                    .map(|t| format!(" ({t} total)"))
                    .unwrap_or_default(),
            );
            println!("Schema:");
            for field in arrow.schema.fields() {
                println!("  - {}: {:?}", field.name(), field.data_type());
            }
        }
        // The Arrow error enum maps the result endpoint's status codes to named
        // variants, so callers can react without string-matching on HTTP codes.
        Err(ArrowError::NotReady { status, .. }) => {
            println!("Result not ready yet (status={status}); try polling longer.");
        }
        Err(other) => return Err(other.into()),
    }
    Ok(())
}

/// Stub used when the `arrow` feature is disabled, so the call site in `run`
/// type-checks either way.
#[cfg(not(feature = "arrow"))]
async fn fetch_arrow(
    _client: &Client,
    _result_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("(build with --features arrow to fetch results as Arrow record batches)");
    Ok(())
}
