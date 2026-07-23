//! HotData Rust SDK quickstart.
//!
//! End-to-end tour of the ergonomic surface, using *only* the public API
//! (`hotdata::prelude::*`). It:
//!
//!   1. builds a [`Client`] from an API token + workspace id,
//!   2. lists workspaces and connections via grouped resource handles,
//!   3. submits a SQL query and awaits the persisted result with one call,
//!   4. fetches that result as Arrow record batches (behind the `arrow` feature),
//!   5. shows the one-call `query_to_arrow` shortcut.
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

use hotdata::prelude::*;

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
        // The only construction failures today are missing credentials; treat
        // any build error as a graceful skip. ClientError is #[non_exhaustive],
        // so the wildcard keeps this robust if new variants are added.
        Err(err) => {
            eprintln!(
                "Could not build client ({err}). Set HOTDATA_API_KEY and \
                 HOTDATA_WORKSPACE_ID to run this example against the live API. Skipping."
            );
            return Ok(());
        }
    };

    println!(
        "Built client targeting {}",
        client.configuration().base_path
    );

    // --- 2. Browse resources via grouped handles -------------------------
    //
    // `client.<resource>()` returns a handle that hides the `&Configuration`
    // plumbing, so you never reach for `hotdata::apis::*_api` free functions.
    // The first authenticated call transparently exchanges the API token for a
    // JWT; subsequent calls reuse the cached token until it nears expiry.
    let workspaces = client.workspaces().list(None).await?;
    println!("Visible workspaces ({}):", workspaces.workspaces.len());
    for ws in &workspaces.workspaces {
        println!("  - {} ({})", ws.name, ws.public_id);
    }

    let connections = client.connections().list().await?;
    println!(
        "Connections in this workspace ({}):",
        connections.connections.len()
    );
    for conn in &connections.connections {
        println!("  - {} ({})", conn.name, conn.id);
    }

    // --- 3. Submit a query -----------------------------------------------
    //
    // Queries, results, and query runs are all scoped to a database via the
    // `X-Database-Id` header, so resolve one first. Prefer HOTDATA_DATABASE, else
    // fall back to the first database in the workspace; if there are none, skip
    // the query portion of the tour.
    let database_id = match resolve_database_id(&client).await? {
        Some(id) => id,
        None => {
            println!(
                "No database available (set HOTDATA_DATABASE or create one). \
                 Skipping the query portion of the tour."
            );
            return Ok(());
        }
    };
    println!("Using database {database_id}");

    // POST /query returns rows inline *and* a result_id; persistence to the
    // result store then completes asynchronously.
    //
    // `query_in` is the database-scoped form of the enhanced default `query`: it
    // retries HTTP 429 (`OVERLOADED`) transparently and, if the server truncates
    // a large result, auto-follows it — paging the full row set into
    // `response.rows` — bounded by the instance `QueryConfig` (default ceilings:
    // 1M rows / 64 MiB). Exceed a ceiling and you get
    // `QueryError::Result(ResultError::TooLarge { .. })` instead of an OOM.
    let response = client
        .query_in(
            QueryRequest::new("select 1 as id, 'hello' as greeting".to_string()),
            &database_id,
        )
        .await?;

    println!(
        "Query ran in {} ms, returned {} row(s) inline. Columns: {:?}",
        response.execution_time_ms, response.row_count, response.columns
    );

    // Tuning the auto-follow behavior per call: clone the instance config and
    // override just what you need. Here we opt out of auto-follow entirely, while
    // still scoping the query to the database via `query_with`.
    let preview_config = client.query_config().clone().with_auto_follow(false);
    let preview = client
        .query_with(
            QueryRequest::new("select 1 as id".to_string()),
            Some(&database_id),
            &preview_config,
        )
        .await?;
    println!(
        "Preview: {} row(s){}",
        preview.row_count,
        if preview.truncated {
            " (truncated — full result available via result_id)"
        } else {
            ""
        }
    );
    // For finer control, e.g. a tighter row guard scoped to a database:
    let _tight = QueryConfig::default().with_max_auto_rows(Some(10_000));

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

    // --- 4. Await the persisted result -----------------------------------
    //
    // `await_result` polls until the result is `ready` (or fails / times out),
    // so you don't hand-roll a poll loop. `PollConfig::default()` is a 120s
    // timeout polled every second.
    println!("Awaiting result {result_id}...");
    let ready = client
        .await_result(&result_id, &database_id, PollConfig::default())
        .await?;
    println!("Result status: {}", ready.status);

    // --- 5. Fetch the result as Arrow (feature-gated) --------------------
    fetch_arrow(&client, &database_id, &result_id).await?;

    // --- 6. One-call query -> Arrow (feature-gated) ----------------------
    one_shot_arrow(&client, &database_id).await?;

    Ok(())
}

/// Resolve a database to scope the query tour to: `HOTDATA_DATABASE` if set,
/// otherwise the first database visible in the workspace (`None` if there are
/// none).
async fn resolve_database_id(client: &Client) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if let Ok(id) = std::env::var("HOTDATA_DATABASE") {
        if !id.is_empty() {
            return Ok(Some(id));
        }
    }
    let databases = client.databases().list(None, None).await?;
    Ok(databases.databases.into_iter().next().map(|db| db.id))
}

/// Fetch an already-ready result as Arrow record batches.
#[cfg(feature = "arrow")]
async fn fetch_arrow(
    client: &Client,
    database_id: &str,
    result_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Fetching result {result_id} as Arrow...");
    match client
        .get_result_arrow(result_id, database_id, None, None)
        .await
    {
        Ok(arrow) => print_arrow(&arrow),
        // The Arrow error enum maps the result endpoint's status codes to named
        // variants, so callers react without string-matching on HTTP codes.
        Err(ArrowError::NotReady { status, .. }) => {
            println!("Result not ready yet (status={status}); try polling longer.");
        }
        Err(other) => return Err(other.into()),
    }
    Ok(())
}

/// Submit a fresh query and get its result as Arrow in a single call —
/// `query_to_arrow` runs the query, awaits `ready`, and decodes the stream.
#[cfg(feature = "arrow")]
async fn one_shot_arrow(
    client: &Client,
    database_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Duration;

    println!("One-call query_to_arrow...");
    let poll = PollConfig {
        timeout: Duration::from_secs(30),
        interval: Duration::from_millis(500),
    };
    let arrow = client
        .query_to_arrow(
            QueryRequest::new("select 42 as answer".to_string()),
            database_id,
            poll,
            None,
            None,
        )
        .await?;
    print_arrow(&arrow);
    Ok(())
}

#[cfg(feature = "arrow")]
fn print_arrow(arrow: &ArrowResult) {
    println!(
        "Arrow result: {} batch(es), {} row(s){}",
        arrow.batches.len(),
        arrow.num_rows(),
        arrow
            .total_row_count
            .map(|t| format!(" ({t} total)"))
            .unwrap_or_default(),
    );
    for field in arrow.schema.fields() {
        println!("  - {}: {:?}", field.name(), field.data_type());
    }
}

/// Stubs used when the `arrow` feature is disabled, so the call sites in `run`
/// type-check either way.
#[cfg(not(feature = "arrow"))]
async fn fetch_arrow(_client: &Client, _result_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("(build with --features arrow to fetch results as Arrow record batches)");
    Ok(())
}

#[cfg(not(feature = "arrow"))]
async fn one_shot_arrow(_client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
