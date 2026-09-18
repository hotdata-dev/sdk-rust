//! Scenario: response_compression.
//!
//! The SDK must negotiate compressed responses. `reqwest` only emits an
//! `Accept-Encoding` request header — and only transparently decodes the
//! reply — when one of its compression cargo features is compiled in; with
//! `default-features = false` and no `gzip` in the feature list, every SDK
//! request went out asking for uncompressed bytes. The API is fronted by a
//! compression-aware origin (it answers `Vary: Accept-Encoding`), so the
//! result was full-size JSON on exactly the responses that are largest: query
//! result cells and paginated listings.
//!
//! These tests run against a local wiremock server — no backend, no
//! credentials — so they run in CI without secrets.
//!
//! Coverage:
//! * the generated `apis::*` ops negotiate gzip;
//! * so does every *hand-written* request builder — `query::send_query`,
//!   `Client::submit_query`, and the Arrow fetch. These sit outside
//!   `src/apis/`, so a regen guard would never catch one of them regressing;
//! * both client construction sites — `ClientBuilder::build` and
//!   `Configuration::default` — produce a negotiating client, since the two
//!   build their `reqwest::Client` independently;
//! * a gzip-encoded reply is actually decoded end to end, not merely asked
//!   for (a header assertion alone would still pass if decoding were off);
//! * the presigned storage `PUT` stays header-bare, preserving the isolation
//!   `uploads::storage_client` documents.

use std::io::Write;

use hotdata::models::QueryRequest;
use hotdata::{Client, Configuration, UploadOptions};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

/// Build a `Configuration` pointed at the mock server, exercising the
/// `Configuration::default()` client construction site.
fn config_for(base_url: &str) -> Configuration {
    Configuration {
        base_path: base_url.to_owned(),
        user_agent: Some("hotdata-rust-test".to_owned()),
        bearer_access_token: Some("test-token".to_owned()),
        ..Configuration::default()
    }
}

/// Build a `Client` via the ergonomic builder, exercising the
/// `ClientBuilder::build` client construction site.
fn client_for(base_url: &str) -> Client {
    Client::builder()
        .api_token("test-token")
        .workspace_id("ws_test")
        .base_url(base_url)
        .build()
        .expect("building the test client should succeed")
}

/// The `Accept-Encoding` value the server saw for the first request whose path
/// matches, or `None` when the header was absent.
fn accept_encoding_for(requests: &[Request], p: &str) -> Option<String> {
    requests
        .iter()
        .find(|r| r.url.path() == p)
        .expect("a request to that path should have been recorded")
        .headers
        .get("accept-encoding")
        .map(|v| v.to_str().expect("header should be ASCII").to_owned())
}

/// Assert the header is present and offers gzip.
fn assert_negotiates_gzip(encoding: Option<String>, site: &str) {
    let encoding = encoding
        .unwrap_or_else(|| panic!("{site} must send an Accept-Encoding header; none was present"));
    assert!(
        encoding.contains("gzip"),
        "{site} must offer gzip; got Accept-Encoding: {encoding}"
    );
}

fn sql(text: &str) -> QueryRequest {
    QueryRequest::new(text.to_owned())
}

// ---------------------------------------------------------------------------
// Generated ops.
// ---------------------------------------------------------------------------

/// The generated `apis::*` surface builds its requests from
/// `configuration.client`, so one op standing in for the rest is enough to
/// prove the shared client negotiates.
#[tokio::test]
async fn generated_op_negotiates_gzip() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/workspaces"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "workspaces": [],
        })))
        .mount(&server)
        .await;

    let config = config_for(&server.uri());
    let _ = hotdata::apis::workspaces_api::list_workspaces(&config, None).await;

    let requests = server.received_requests().await.expect("requests recorded");
    assert_negotiates_gzip(
        accept_encoding_for(&requests, "/v1/workspaces"),
        "a generated op via Configuration::default",
    );
}

// ---------------------------------------------------------------------------
// Hand-written request builders (outside src/apis/).
// ---------------------------------------------------------------------------

/// `client.query()` reaches the wire through `query::send_query`, which builds
/// its own request rather than going through a generated op.
#[tokio::test]
async fn send_query_negotiates_gzip() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "columns": [],
            "rows": [],
        })))
        .mount(&server)
        .await;

    let client = client_for(&server.uri());
    let _ = client.query(sql("SELECT 1")).await;

    let requests = server.received_requests().await.expect("requests recorded");
    assert_negotiates_gzip(
        accept_encoding_for(&requests, "/v1/query"),
        "query::send_query",
    );
}

/// `Client::submit_query` is a second hand-written builder, distinct from
/// `send_query`.
#[tokio::test]
async fn submit_query_negotiates_gzip() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "columns": [],
            "rows": [],
        })))
        .mount(&server)
        .await;

    let client = client_for(&server.uri());
    let _ = client.submit_query(sql("SELECT 1"), None).await;

    let requests = server.received_requests().await.expect("requests recorded");
    assert_negotiates_gzip(
        accept_encoding_for(&requests, "/v1/query"),
        "Client::submit_query",
    );
}

/// The Arrow fetch sets its own `Accept`; that must not displace the
/// negotiated `Accept-Encoding`, which is an orthogonal header. The body here
/// is not valid Arrow IPC — the decode fails, which is fine, since the headers
/// are settled when the request is built.
#[cfg(feature = "arrow")]
#[tokio::test]
async fn arrow_fetch_negotiates_gzip() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/results/res_1"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not-arrow-ipc"))
        .mount(&server)
        .await;

    let client = client_for(&server.uri());
    let _ = client.get_result_arrow("res_1", "dbid_1", None, None).await;

    let requests = server.received_requests().await.expect("requests recorded");
    assert_negotiates_gzip(
        accept_encoding_for(&requests, "/v1/results/res_1"),
        "the Arrow fetch",
    );
}

/// The Arrow fetch's own `Accept` must survive intact. Default headers are
/// merged only into vacant slots, but `RequestBuilder::header` *appends* — so
/// a default `Accept` set carelessly would ride along as a second value and
/// change content negotiation.
#[cfg(feature = "arrow")]
#[tokio::test]
async fn arrow_fetch_keeps_its_own_accept_header() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/results/res_1"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not-arrow-ipc"))
        .mount(&server)
        .await;

    let client = client_for(&server.uri());
    let _ = client.get_result_arrow("res_1", "dbid_1", None, None).await;

    let requests = server.received_requests().await.expect("requests recorded");
    let accept = requests
        .iter()
        .find(|r| r.url.path() == "/v1/results/res_1")
        .expect("the Arrow request should have been recorded")
        .headers
        .get("accept")
        .map(|v| v.to_str().expect("header should be ASCII").to_owned())
        .expect("the Arrow fetch must send an Accept header");

    assert_eq!(
        accept, "application/vnd.apache.arrow.stream",
        "the Arrow Accept header must be exactly the Arrow media type, not a \
         list that a default header joined"
    );
}

// ---------------------------------------------------------------------------
// End-to-end decode.
// ---------------------------------------------------------------------------

/// Asking for gzip is only half the job: the reply must decode. Serving a real
/// gzip-encoded body and parsing it through the normal op path proves the
/// decoder is compiled in and wired up. Without it the JSON parse fails on
/// compressed bytes.
#[tokio::test]
async fn gzip_encoded_response_is_transparently_decoded() {
    let server = MockServer::start().await;

    let body = serde_json::json!({ "ok": true, "workspaces": [] }).to_string();
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder
        .write_all(body.as_bytes())
        .expect("gzip encoding should succeed");
    let compressed = encoder.finish().expect("gzip encoding should finish");

    assert_ne!(
        compressed.as_slice(),
        body.as_bytes(),
        "the fixture must actually be compressed, or this test proves nothing"
    );

    Mock::given(method("GET"))
        .and(path("/v1/workspaces"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(compressed)
                .insert_header("content-type", "application/json")
                .insert_header("content-encoding", "gzip"),
        )
        .mount(&server)
        .await;

    let config = config_for(&server.uri());
    hotdata::apis::workspaces_api::list_workspaces(&config, None)
        .await
        .expect("a gzip-encoded response must decode and parse");
}

// ---------------------------------------------------------------------------
// Presigned storage PUT: header isolation.
// ---------------------------------------------------------------------------

/// `uploads::storage_client` is a deliberately separate, header-bare client:
/// a presigned URL self-authorizes and carries its own signed header set, so
/// the SDK must not decorate those requests. Enabling compression crate-wide
/// would otherwise leak `Accept-Encoding` onto the storage `PUT`. The response
/// to a `PUT` is an empty body or a short ack, so there is nothing to gain and
/// a signature to risk.
#[tokio::test]
async fn presigned_storage_put_stays_header_bare() {
    let server = MockServer::start().await;
    let storage_url = format!("{}/storage/single", server.uri());
    let contents = b"hello compression";

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

    let client = client_for(&server.uri());
    let file = std::env::temp_dir().join(format!(
        "hotdata-compression-{}",
        uuid::Uuid::new_v4().simple()
    ));
    std::fs::write(&file, contents).expect("writing the temp upload file should succeed");
    let result = client.upload_file(&file, UploadOptions::default()).await;
    let _ = std::fs::remove_file(&file);
    result.expect("single upload should succeed");

    let requests = server.received_requests().await.expect("requests recorded");
    assert_eq!(
        accept_encoding_for(&requests, "/storage/single"),
        None,
        "the presigned storage PUT must stay header-bare; an SDK-added \
         Accept-Encoding can fall outside the signed header set"
    );

    // The API legs of the same upload still negotiate, proving the isolation is
    // scoped to the storage client rather than disabling compression globally.
    assert_negotiates_gzip(
        accept_encoding_for(&requests, "/v1/uploads"),
        "the upload create-session leg",
    );
}
