//! Shared support for the SDK integration tests.
//!
//! Rust has no conftest, so this module centralizes the env-driven gating that
//! sdk-python expresses with pytest fixtures. Tests run against production; see
//! www.hotdata.dev/api/README.md for the contract (env vars, naming
//! conventions, blast-radius rules).
//!
//! Every test reads the same env vars and SKIPS cleanly (returns early with a
//! notice on stderr) when they are unset, so `cargo test` passes in CI without
//! secrets configured.

#![allow(dead_code)]

use std::time::Duration;

use hotdata::Client;

/// Connect-phase ceiling for the shared test client.
///
/// `reqwest::Client::default()` (what the SDK uses when no client is supplied)
/// has no connect timeout, so an unreachable API host blocks each call on the
/// OS-level TCP timeout (~60s observed in CI). With ~20 scenario binaries run
/// sequentially by `cargo test`, a transient connectivity blip turns into a
/// ~20-minute red run. Bounding the connect phase fails fast — and lets hyper
/// fall through to the next resolved address — so an outage is cheap to retry.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Overall per-request ceiling. Generous enough for the tiny fixture upload and
/// each poll request; purely a backstop against a hung socket.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

/// Build the reqwest client every scenario shares: identical to the SDK default
/// except for the bounded [`CONNECT_TIMEOUT`]/[`REQUEST_TIMEOUT`]. Pass it via
/// `ClientBuilder::reqwest_client` (or assign to `Configuration::client`) so a
/// down API host can't stall the suite.
pub fn test_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        .build()
        .expect("building the test reqwest client should not fail")
}

/// Default API host. The auth-token -> JWT exchange and every endpoint live on
/// the API host, so the ergonomic `Client` always points here unless overridden
/// by `HOTDATA_SDK_TEST_API_URL`.
pub const DEFAULT_API_URL: &str = "https://api.hotdata.dev";

/// Resolved test environment. Mirrors sdk-python's `TestEnv` dataclass.
///
/// GitHub Actions sets `env:` keys even when the underlying secret/var is unset,
/// producing empty strings rather than absent keys. We treat empty strings as
/// absent (see [`load_env`]).
#[derive(Clone, Debug)]
pub struct TestEnv {
    pub api_key: Option<String>,
    pub workspace_id: Option<String>,
    pub api_url: String,
    pub connection_id: Option<String>,
}

impl TestEnv {
    /// True when both the required credentials (api key + workspace id) are
    /// present. Scenarios that need to actually call the API gate on this.
    pub fn has_creds(&self) -> bool {
        self.api_key.is_some() && self.workspace_id.is_some()
    }
}

fn non_empty(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|s| !s.is_empty())
}

/// Read the test environment. Empty strings are treated as absent. `api_url`
/// falls back to [`DEFAULT_API_URL`].
pub fn load_env() -> TestEnv {
    TestEnv {
        api_key: non_empty("HOTDATA_SDK_TEST_API_KEY"),
        workspace_id: non_empty("HOTDATA_SDK_TEST_WORKSPACE_ID"),
        api_url: non_empty("HOTDATA_SDK_TEST_API_URL")
            .unwrap_or_else(|| DEFAULT_API_URL.to_string()),
        connection_id: non_empty("HOTDATA_SDK_TEST_CONNECTION_ID"),
    }
}

/// Build an authenticated [`Client`] from the environment, or return `None` if
/// the required credentials are missing. Callers should pair this with
/// [`skip_if_no_creds!`] (or check the `Option`) so the test skips cleanly.
pub fn client_or_skip() -> Option<Client> {
    let env = load_env();
    if !env.has_creds() {
        return None;
    }
    let client = Client::builder()
        .api_token(env.api_key.expect("checked above"))
        .workspace_id(env.workspace_id.expect("checked above"))
        .base_url(env.api_url)
        .reqwest_client(test_http_client())
        .build()
        .expect("Client::build with valid credentials should not fail");
    Some(client)
}

/// `sdkci-<scenario>-<uuid8>` so any orphaned resources are identifiable and
/// can be swept. See api/README.md — every test-created resource must use this
/// prefix.
pub fn sdkci_name(scenario: &str) -> String {
    let id = uuid::Uuid::new_v4().simple().to_string();
    format!("sdkci-{}-{}", scenario, &id[..8])
}

/// Early-return out of a `#[tokio::test]` when credentials are unavailable,
/// printing a notice to stderr. Mirrors `pytest.skip`. Evaluates to the bound
/// [`Client`] when creds are present.
///
/// ```ignore
/// let client = skip_if_no_creds!();
/// ```
#[macro_export]
macro_rules! skip_if_no_creds {
    () => {{
        match $crate::common::client_or_skip() {
            Some(client) => client,
            None => {
                eprintln!(
                    "SKIP {}: set HOTDATA_SDK_TEST_API_KEY and \
                     HOTDATA_SDK_TEST_WORKSPACE_ID to run this scenario",
                    module_path!()
                );
                return;
            }
        }
    }};
}

/// Like [`skip_if_no_creds!`] but also requires `HOTDATA_SDK_TEST_CONNECTION_ID`.
/// Returns `(Client, connection_id)`.
#[macro_export]
macro_rules! skip_if_no_connection {
    () => {{
        let env = $crate::common::load_env();
        match ($crate::common::client_or_skip(), env.connection_id.clone()) {
            (Some(client), Some(connection_id)) => (client, connection_id),
            (None, _) => {
                eprintln!(
                    "SKIP {}: set HOTDATA_SDK_TEST_API_KEY and \
                     HOTDATA_SDK_TEST_WORKSPACE_ID to run this scenario",
                    module_path!()
                );
                return;
            }
            (Some(_), None) => {
                eprintln!(
                    "SKIP {}: set HOTDATA_SDK_TEST_CONNECTION_ID to run this \
                     scenario",
                    module_path!()
                );
                return;
            }
        }
    }};
}

/// Extract the HTTP status code from a generated [`hotdata::Error`]. Returns
/// `None` for transport/serde/io errors (which carry no HTTP status).
pub fn status_of<T>(err: &hotdata::Error<T>) -> Option<u16> {
    match err {
        hotdata::Error::ResponseError(content) => Some(content.status.as_u16()),
        _ => None,
    }
}

/// Name of the shared database that query-scoped scenarios target.
pub const SHARED_DATABASE_NAME: &str = "sdkci-shared";

/// Find-or-create the shared `sdkci-shared` database and return its id.
///
/// Queries require a database scope (the `X-Database-Id` header or the
/// `database_id` body field); a bare query returns 400 "a database is
/// required". Databases persist (no auto-expiry), so — mirroring sdk-python's
/// conftest — we reuse one stable database keyed by name across runs rather
/// than creating and deleting one per test (which would leak on failure).
pub async fn shared_database_id(client: &Client) -> String {
    use hotdata::apis::databases_api;
    let config = client.configuration();

    let listing = databases_api::list_databases(config, None, None)
        .await
        .expect("list_databases should succeed");
    if let Some(db) = listing
        .databases
        .iter()
        .find(|d| d.name.as_ref().and_then(|n| n.as_deref()) == Some(SHARED_DATABASE_NAME))
    {
        return db.id.clone();
    }

    let mut request = hotdata::models::CreateDatabaseRequest::new();
    request.name = Some(Some(SHARED_DATABASE_NAME.to_string()));
    let created = databases_api::create_database(config, request)
        .await
        .expect("create_database should succeed");
    created.id
}

/// Create a fresh, uniquely-named `sdkci-*` database and return its id.
///
/// Unlike [`shared_database_id`], this is for scenarios that exercise the
/// database lifecycle itself (contexts, catalog attach/detach) and tear the
/// database back down at the end. The `sdkci-` prefix keeps any leak
/// identifiable to the nightly sweep if the test panics before cleanup.
pub async fn create_scratch_database(client: &Client, scenario: &str) -> String {
    use hotdata::apis::databases_api;
    let mut request = hotdata::models::CreateDatabaseRequest::new();
    request.name = Some(Some(sdkci_name(scenario)));
    let created = databases_api::create_database(client.configuration(), request)
        .await
        .expect("create_database should succeed");
    created.id
}
