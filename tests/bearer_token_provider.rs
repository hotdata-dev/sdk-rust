//! Pluggable per-request bearer credentials (`Configuration::token_provider`).
//!
//! The regression these guard: with `bearer_access_token` alone, the credential
//! is fixed when the `Client` is built, so a host whose access token lives only
//! a few minutes (the CLI's PKCE browser-login session) starts 401ing partway
//! through a long command. A [`BearerTokenProvider`] is asked for a bearer once
//! *per request*, so it can refresh mid-flight.
//!
//! Everything here runs against a local wiremock server — no backend, no
//! credentials — so it runs in CI without secrets.
//!
//! Coverage:
//! * a provider's value reaches the wire on a generated op;
//! * it is consulted per request, not once per `Client` (the actual regression);
//! * with no provider installed, `bearer_access_token` behaves as it did in
//!   0.12.0;
//! * a provider that errors logs a warning and the request proceeds
//!   unauthenticated, so the server sees no bearer;
//! * `upload_file`'s create-session and finalize legs each resolve through the
//!   provider, so a token that rotates between them is picked up;
//! * every *hand-written* bearer call site — `client.query()` via
//!   `query::send_query`, `Client::submit_query`, and the Arrow fetch — resolves
//!   per request too. These sit outside `src/apis/`, which is all the CI regen
//!   guard greps, so nothing else would catch one being reverted to a plain
//!   `bearer_access_token` read.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use hotdata::auth::{async_trait, BearerTokenError, BearerTokenProvider};
use hotdata::models::QueryRequest;
use hotdata::{Client, Configuration, UploadOptions};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

/// A provider that hands out `values[i]` on call `i`, then repeats the last one.
/// Recording the call count lets a test prove the SDK asked once per request
/// rather than caching the first answer.
#[derive(Debug)]
struct SequenceProvider {
    values: Vec<String>,
    calls: AtomicUsize,
}

impl SequenceProvider {
    fn new(values: &[&str]) -> Arc<Self> {
        Arc::new(Self {
            values: values.iter().map(|s| (*s).to_owned()).collect(),
            calls: AtomicUsize::new(0),
        })
    }

    fn call_count(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl BearerTokenProvider for SequenceProvider {
    async fn bearer_value(&self) -> Result<String, BearerTokenError> {
        let i = self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.values[i.min(self.values.len() - 1)].clone())
    }
}

/// A provider that always fails, standing in for an expired refresh token or an
/// unreachable credential store.
#[derive(Debug)]
struct FailingProvider;

#[async_trait]
impl BearerTokenProvider for FailingProvider {
    async fn bearer_value(&self) -> Result<String, BearerTokenError> {
        Err(BearerTokenError::Malformed(
            "refresh token expired".to_owned(),
        ))
    }
}

fn config_for(base_url: &str) -> Configuration {
    Configuration {
        base_path: base_url.to_owned(),
        user_agent: Some("hotdata-rust-test".to_owned()),
        ..Configuration::default()
    }
}

/// Every `Authorization` header value the server saw, in request order.
fn recorded_bearers(requests: &[Request]) -> Vec<Option<String>> {
    requests
        .iter()
        .map(|r| {
            r.headers
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_owned())
        })
        .collect()
}

/// Mount an empty-but-valid `GET /v1/workspaces` so a generated op succeeds
/// regardless of which bearer it carries; the assertions read the recorded
/// requests instead of gating the mock on a header.
async fn mount_workspaces(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/v1/workspaces"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ok": true,
            "workspaces": [],
        })))
        .mount(server)
        .await;
}

#[tokio::test]
async fn provider_value_reaches_the_wire() {
    let server = MockServer::start().await;
    mount_workspaces(&server).await;

    let provider = SequenceProvider::new(&["provided-token"]);
    let mut config = config_for(&server.uri());
    config.token_provider = Some(provider.clone());
    let client = Client::from_configuration(config);

    client
        .workspaces()
        .list(None)
        .await
        .expect("list_workspaces should succeed");

    let requests = server.received_requests().await.expect("requests recorded");
    assert_eq!(
        recorded_bearers(&requests),
        vec![Some("Bearer provided-token".to_owned())],
        "the provider's value must be sent as the Authorization bearer"
    );
    assert_eq!(provider.call_count(), 1);
}

/// The regression under test: the provider must be consulted on EVERY request,
/// not once at `Client` construction. Two calls through one `Client`, a provider
/// returning a different value each time, and both values must reach the wire —
/// a cached-once implementation would send the first token twice.
#[tokio::test]
async fn provider_is_consulted_per_request() {
    let server = MockServer::start().await;
    mount_workspaces(&server).await;

    let provider = SequenceProvider::new(&["token-first", "token-second"]);
    let mut config = config_for(&server.uri());
    config.token_provider = Some(provider.clone());
    let client = Client::from_configuration(config);

    client.workspaces().list(None).await.expect("first call");
    client.workspaces().list(None).await.expect("second call");

    let requests = server.received_requests().await.expect("requests recorded");
    assert_eq!(
        recorded_bearers(&requests),
        vec![
            Some("Bearer token-first".to_owned()),
            Some("Bearer token-second".to_owned()),
        ],
        "each request must carry the value the provider returned for it"
    );
    assert_eq!(
        provider.call_count(),
        2,
        "the provider must be asked once per request"
    );
}

/// A provider overrides `bearer_access_token` when both are set: a host that
/// installs one owns the credential, so a stale static token must not win.
#[tokio::test]
async fn provider_takes_precedence_over_static_token() {
    let server = MockServer::start().await;
    mount_workspaces(&server).await;

    let mut config = config_for(&server.uri());
    config.bearer_access_token = Some("static-token".to_owned());
    config.token_provider = Some(SequenceProvider::new(&["provided-token"]));
    let client = Client::from_configuration(config);

    client.workspaces().list(None).await.expect("call succeeds");

    let requests = server.received_requests().await.expect("requests recorded");
    assert_eq!(
        recorded_bearers(&requests),
        vec![Some("Bearer provided-token".to_owned())]
    );
}

/// The 0.12.0 behavior must be untouched: no provider installed means the static
/// `bearer_access_token` is sent, exactly as before.
#[tokio::test]
async fn static_bearer_still_works_without_a_provider() {
    let server = MockServer::start().await;
    mount_workspaces(&server).await;

    let mut config = config_for(&server.uri());
    config.bearer_access_token = Some("static-token".to_owned());
    assert!(config.token_provider.is_none());
    let client = Client::from_configuration(config);

    client.workspaces().list(None).await.expect("call succeeds");

    let requests = server.received_requests().await.expect("requests recorded");
    assert_eq!(
        recorded_bearers(&requests),
        vec![Some("Bearer static-token".to_owned())]
    );
}

/// `ClientBuilder::api_token` keeps installing the token as the static bearer,
/// and no provider is installed by default — the SDK does not resurrect an
/// implicit token-exchange provider.
#[tokio::test]
async fn builder_installs_no_provider() {
    let server = MockServer::start().await;
    mount_workspaces(&server).await;

    let client = Client::builder()
        .api_token("hd_opaque")
        .workspace_id("ws_x")
        .base_url(server.uri())
        .build()
        .expect("build should succeed");

    assert!(
        client.configuration().token_provider.is_none(),
        "the builder must not install a token provider"
    );
    assert_eq!(
        client.configuration().bearer_access_token.as_deref(),
        Some("hd_opaque")
    );

    client.workspaces().list(None).await.expect("call succeeds");

    let requests = server.received_requests().await.expect("requests recorded");
    assert_eq!(
        recorded_bearers(&requests),
        vec![Some("Bearer hd_opaque".to_owned())],
        "exactly one request, carrying the API token — no exchange round trip"
    );
}

// ---------------------------------------------------------------------------
// Provider failure: log the cause, send the request unauthenticated.
// ---------------------------------------------------------------------------

/// Captures `log` records so the failure test can assert the warning was
/// emitted. `log::set_logger` is once-per-process, and each integration test
/// file is its own binary, so this file owns the process-global logger.
static LOG_BUF: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

fn log_buf() -> &'static Mutex<Vec<String>> {
    LOG_BUF.get_or_init(|| Mutex::new(Vec::new()))
}

struct CaptureLogger;

impl log::Log for CaptureLogger {
    fn enabled(&self, _meta: &log::Metadata) -> bool {
        true
    }
    fn log(&self, record: &log::Record) {
        log_buf()
            .lock()
            .unwrap()
            .push(format!("{} {}", record.level(), record.args()));
    }
    fn flush(&self) {}
}

static LOGGER: CaptureLogger = CaptureLogger;

/// A failing provider must not silently send the stale static token, and must
/// not panic: it logs the cause and proceeds unauthenticated, so the server's
/// 401 is diagnosable from the log rather than a mystery.
#[tokio::test]
async fn failing_provider_logs_and_sends_no_bearer() {
    // `set_logger` (vs `set_boxed_logger`) needs no `std` feature on `log`.
    log::set_logger(&LOGGER).expect("logger installs once");
    log::set_max_level(log::LevelFilter::Warn);

    let server = MockServer::start().await;
    mount_workspaces(&server).await;

    let mut config = config_for(&server.uri());
    // Deliberately ALSO set a static token: a provider failure must not fall
    // back to it. Falling back would resurrect the stale-credential 401 this
    // whole feature exists to avoid, and would mask the real error.
    config.bearer_access_token = Some("static-token".to_owned());
    config.token_provider = Some(Arc::new(FailingProvider));
    let client = Client::from_configuration(config);

    client
        .workspaces()
        .list(None)
        .await
        .expect("the request is still sent, just unauthenticated");

    let requests = server.received_requests().await.expect("requests recorded");
    assert_eq!(
        recorded_bearers(&requests),
        vec![None],
        "a failed resolution must send no Authorization header at all"
    );

    let logged = log_buf().lock().unwrap().join("\n");
    assert!(
        logged.contains("bearer token resolution failed"),
        "the failure must be logged; captured records were:\n{logged}"
    );
    assert!(
        logged.contains("refresh token expired"),
        "the underlying cause must reach the log; captured records were:\n{logged}"
    );
}

// ---------------------------------------------------------------------------
// upload_file: the path the CLI most needs refreshed.
// ---------------------------------------------------------------------------

/// A multi-gigabyte upload is exactly the case a five-minute token cannot span:
/// finalize lands long after create-session. Both legs must resolve through the
/// provider, so a token that rotated in between is picked up rather than
/// 401ing the finalize. The provider returns a different value per call, so the
/// two legs carrying different bearers proves each resolved independently.
#[tokio::test]
async fn upload_file_resolves_create_and_finalize_through_the_provider() {
    let server = MockServer::start().await;
    let storage_url = format!("{}/storage/single", server.uri());
    let contents = b"hello per-request bearer";

    Mock::given(method("POST"))
        .and(path("/v1/uploads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "finalize_token": "ftok_single",
            "headers": {},
            "mode": "single",
            "upload_id": "upl_single",
            "url": storage_url,
        })))
        .mount(&server)
        .await;

    Mock::given(method("PUT"))
        .and(path("/storage/single"))
        .respond_with(ResponseTemplate::new(200).insert_header("ETag", "\"single-etag\""))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/v1/uploads/upl_single/finalize"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "created_at": "2026-06-25T00:00:00Z",
            "size_bytes": contents.len(),
            "status": "ready",
            "upload_id": "upl_single",
        })))
        .mount(&server)
        .await;

    let provider = SequenceProvider::new(&["token-at-create", "token-at-finalize"]);
    let mut config = config_for(&server.uri());
    config.token_provider = Some(provider.clone());
    let client = Client::from_configuration(config);

    let file = std::env::temp_dir().join(format!(
        "hotdata-bearer-provider-{}",
        uuid::Uuid::new_v4().simple()
    ));
    std::fs::write(&file, contents).expect("writing the temp upload file should succeed");
    let result = client.upload_file(&file, UploadOptions::default()).await;
    let _ = std::fs::remove_file(&file);
    result.expect("single upload should succeed");

    let requests = server.received_requests().await.expect("requests recorded");

    let bearer_of = |p: &str| -> Option<String> {
        requests
            .iter()
            .find(|r| r.url.path() == p)
            .unwrap_or_else(|| panic!("a request to {p} should have been made"))
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned())
    };

    assert_eq!(
        bearer_of("/v1/uploads"),
        Some("Bearer token-at-create".to_owned()),
        "create-session must resolve through the provider"
    );
    assert_eq!(
        bearer_of("/v1/uploads/upl_single/finalize"),
        Some("Bearer token-at-finalize".to_owned()),
        "finalize must resolve through the provider independently of create-session"
    );
    assert_eq!(
        provider.call_count(),
        2,
        "exactly the two API legs resolve a bearer"
    );

    // The presigned storage PUT self-authorizes; leaking a bearer onto it makes
    // S3-style storage 403. A provider must not change that.
    assert_eq!(
        bearer_of("/storage/single"),
        None,
        "the storage PUT must carry no Authorization header"
    );
}

// ---------------------------------------------------------------------------
// Hand-written bearer call sites.
//
// `src/query.rs` (send_query), `src/client.rs` (submit_query) and `src/arrow.rs`
// (fetch_arrow_bytes) each build their own request rather than going through a
// generated op, so each has its own `resolve_bearer_token` call. The CI regen
// guard only greps `src/apis/`, so if one of these were reverted to reading
// `bearer_access_token` directly, nothing above would fail. These pin them.
//
// Each asserts the *sequence* of bearers across two calls: one value per call
// proves the site resolved independently rather than reusing a cached token.
// The calls are allowed to fail — the bearer is applied while the request is
// built, long before the response is parsed — so the mocks return bodies these
// entry points won't deserialize and the results are deliberately discarded.
// ---------------------------------------------------------------------------

fn sql(text: &str) -> QueryRequest {
    QueryRequest {
        sql: text.to_owned(),
        ..Default::default()
    }
}

/// `client.query()` reaches the wire through `query::send_query`, the
/// hand-written `POST /v1/query` builder — and it backs the most-used entry
/// point in the SDK.
#[tokio::test]
async fn send_query_resolves_per_request() {
    let server = MockServer::start().await;
    // 200 with a body `QueryResponse` can't deserialize: enough to exercise the
    // request path without tripping the 429 retry loop (only 429s and
    // pre-response connection errors retry, so the provider is asked once per
    // call here).
    Mock::given(method("POST"))
        .and(path("/v1/query"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"unparseable":true}"#))
        .mount(&server)
        .await;

    let provider = SequenceProvider::new(&["query-first", "query-second"]);
    let mut config = config_for(&server.uri());
    config.token_provider = Some(provider.clone());
    let client = Client::from_configuration(config);

    let _ = client.query(sql("SELECT 1")).await;
    let _ = client.query(sql("SELECT 2")).await;

    let requests = server.received_requests().await.expect("requests recorded");
    assert_eq!(
        recorded_bearers(&requests),
        vec![
            Some("Bearer query-first".to_owned()),
            Some("Bearer query-second".to_owned()),
        ],
        "send_query must resolve a fresh bearer for each POST /v1/query"
    );
    assert_eq!(provider.call_count(), 2);
}

/// `Client::submit_query` builds its own `POST /v1/query` (it needs the raw 202
/// body the generated op discards), so it carries a second, separate bearer
/// site from `send_query`.
#[tokio::test]
async fn submit_query_resolves_per_request() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/query"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"unparseable":true}"#))
        .mount(&server)
        .await;

    let provider = SequenceProvider::new(&["submit-first", "submit-second"]);
    let mut config = config_for(&server.uri());
    config.token_provider = Some(provider.clone());
    let client = Client::from_configuration(config);

    let _ = client.submit_query(sql("SELECT 1"), None).await;
    let _ = client.submit_query(sql("SELECT 2"), None).await;

    let requests = server.received_requests().await.expect("requests recorded");
    assert_eq!(
        recorded_bearers(&requests),
        vec![
            Some("Bearer submit-first".to_owned()),
            Some("Bearer submit-second".to_owned()),
        ],
        "submit_query must resolve a fresh bearer for each call"
    );
    assert_eq!(provider.call_count(), 2);
}

/// The Arrow fetch (`arrow::fetch_arrow_bytes`) negotiates `?format=arrow` with
/// its own request builder, so it is a third independent bearer site. Relevant
/// to the CLI, which pulls results this way after a long query.
#[cfg(feature = "arrow")]
#[tokio::test]
async fn arrow_fetch_resolves_per_request() {
    let server = MockServer::start().await;
    // Not valid Arrow IPC — the decode fails, which is fine: the bearer is
    // applied when the request is built.
    Mock::given(method("GET"))
        .and(path("/v1/results/res_1"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not-arrow-ipc"))
        .mount(&server)
        .await;

    let provider = SequenceProvider::new(&["arrow-first", "arrow-second"]);
    let mut config = config_for(&server.uri());
    config.token_provider = Some(provider.clone());
    let client = Client::from_configuration(config);

    let _ = client.get_result_arrow("res_1", "dbid_1", None, None).await;
    let _ = client.get_result_arrow("res_1", "dbid_1", None, None).await;

    let requests = server.received_requests().await.expect("requests recorded");
    assert_eq!(
        recorded_bearers(&requests),
        vec![
            Some("Bearer arrow-first".to_owned()),
            Some("Bearer arrow-second".to_owned()),
        ],
        "the Arrow fetch must resolve a fresh bearer for each call"
    );
    assert_eq!(provider.call_count(), 2);
}
