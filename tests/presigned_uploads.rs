//! Presigned direct-to-storage upload tests.
//!
//! Most tests stand up a single wiremock server that plays BOTH roles: the
//! hotdata API (`POST /v1/uploads`, `POST /v1/uploads/{id}/finalize`) and the
//! "object storage" endpoint the SDK `PUT`s bytes to (`/storage/...`). The
//! concurrency tests instead point `part_urls` at a bare raw-TCP storage server
//! ([`concurrency_storage_server`]) that genuinely holds in-flight PUTs to
//! measure real overlap. All are fully local and deterministic — no real
//! backend, no credentials — so they run in CI without secrets.
//!
//! Coverage:
//! * single-`PUT` happy path (bytes, header isolation, finalize token + empty
//!   parts, returned upload_id);
//! * multipart happy path (slicing by `part_size`, per-part ETag collection,
//!   ascending finalize parts);
//! * progress callback monotonicity reaching exactly the file size;
//! * storage-PUT header isolation (no SDK bearer/workspace/session headers, and
//!   no default headers leaking off the SDK's main client);
//! * finalize exactly-once (no retry) and per-part retry;
//! * error surfacing (missing ETag, storage 4xx/5xx, finalize failure,
//!   501 PRESIGN_UNSUPPORTED, malformed sessions);
//! * bounded in-flight concurrency and server-provided Content-Type replay.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use hotdata::apis::configuration::{ApiKey, Configuration};
use hotdata::{Client, RetryPolicy, UploadError, UploadOptions};
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const WORKSPACE_HEADER: &str = "X-Workspace-Id";
const SESSION_HEADER: &str = "X-Session-Id";

/// Build a client pointed at the mock server with a static bearer token and the
/// workspace + session scope headers installed (no JWT-exchange round-trip), so
/// the upload requests carry exactly the headers a real client would.
fn test_client(base_url: &str) -> Client {
    let mut configuration = Configuration {
        base_path: base_url.to_owned(),
        user_agent: Some("hotdata-rust-test".to_owned()),
        bearer_access_token: Some("test-bearer".to_owned()),
        ..Configuration::default()
    };
    configuration.api_keys.insert(
        WORKSPACE_HEADER.to_owned(),
        ApiKey {
            prefix: None,
            key: "ws_test".to_owned(),
        },
    );
    configuration.api_keys.insert(
        SESSION_HEADER.to_owned(),
        ApiKey {
            prefix: None,
            key: "sess_test".to_owned(),
        },
    );
    Client::from_configuration(configuration)
}

/// A fast, deterministic retry policy (tiny backoff, no jitter, several
/// retries) so per-part-retry tests run without real delay.
fn fast_retry(max_retries: u32) -> RetryPolicy {
    RetryPolicy {
        max_retries,
        base_backoff: Duration::from_millis(1),
        max_backoff: Duration::from_millis(5),
        deadline: Duration::from_secs(30),
        jitter: 0.0,
    }
}

/// Like [`test_client`] but with an explicit retry policy installed (storage
/// part PUTs route through it; finalize disables it internally).
fn test_client_with_retry(base_url: &str, retry: RetryPolicy) -> Client {
    let mut client = test_client(base_url);
    client.configuration_mut().retry = retry;
    client
}

/// Write `contents` to a uniquely-named temp file and return its path.
fn temp_file(contents: &[u8]) -> std::path::PathBuf {
    let name = format!("hotdata-presigned-{}", uuid::Uuid::new_v4().simple());
    let path = std::env::temp_dir().join(name);
    std::fs::write(&path, contents).expect("writing the temp upload file should succeed");
    path
}

/// Assert a storage `PUT` request carries NONE of the SDK's auth/scope headers.
/// A presigned URL self-authorizes; an extra signed-ish header makes S3-style
/// storage return 403.
fn assert_no_sdk_headers(req: &Request) {
    for forbidden in [
        "authorization",
        "x-workspace-id",
        "x-session-id",
        "x-upload-finalize-token",
    ] {
        assert!(
            req.headers.get(forbidden).is_none(),
            "storage PUT must not carry the `{forbidden}` header, found one"
        );
    }
}

#[tokio::test]
async fn single_put_happy_path() {
    let server = MockServer::start().await;
    let storage_url = format!("{}/storage/single", server.uri());
    let contents = b"hello presigned world";

    // POST /v1/uploads -> mode=single with a storage url + finalize token.
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

    // The storage PUT target. Accept any bytes; we assert on them afterwards.
    Mock::given(method("PUT"))
        .and(path("/storage/single"))
        .respond_with(ResponseTemplate::new(200).insert_header("ETag", "\"single-etag\""))
        .mount(&server)
        .await;

    // POST /v1/uploads/{id}/finalize -> the finalized upload.
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

    let path = temp_file(contents);
    let client = test_client(&server.uri());
    let result = client.upload_file(&path, UploadOptions::default()).await;
    let _ = std::fs::remove_file(&path);
    let result = result.expect("single upload should succeed");

    assert_eq!(result.upload_id, "upl_single");
    assert_eq!(result.size_bytes as usize, contents.len());
    assert_eq!(result.status, "ready");

    // Inspect the recorded requests.
    let requests = server.received_requests().await.expect("requests recorded");
    let put = requests
        .iter()
        .find(|r| r.url.path() == "/storage/single")
        .expect("a storage PUT should have been made");
    // Exact bytes arrived.
    assert_eq!(
        put.body, contents,
        "storage PUT body must be the file bytes"
    );
    // Explicit Content-Length, framed (not chunked).
    assert_eq!(
        put.headers
            .get("content-length")
            .and_then(|v| v.to_str().ok()),
        Some(contents.len().to_string().as_str()),
        "storage PUT must set an explicit Content-Length"
    );
    // Header isolation.
    assert_no_sdk_headers(put);

    // Finalize carried the token in the header and an empty/absent parts body.
    let finalize = requests
        .iter()
        .find(|r| r.url.path() == "/v1/uploads/upl_single/finalize")
        .expect("a finalize request should have been made");
    assert_eq!(
        finalize
            .headers
            .get("x-upload-finalize-token")
            .and_then(|v| v.to_str().ok()),
        Some("ftok_single"),
        "finalize must carry the token in X-Upload-Finalize-Token"
    );
    // The single-PUT finalize body MUST be a JSON object (`{}`), NOT `null`:
    // prod rejects a `null` finalize body ("invalid type: null, expected struct
    // FinalizeUploadRequest"). Parse the raw bytes strictly so a literal `null`
    // is caught (it would parse to Value::Null and fail this assert).
    let body: serde_json::Value =
        serde_json::from_slice(&finalize.body).expect("finalize body must be valid JSON");
    assert!(
        body.is_object(),
        "single-PUT finalize body must be a JSON object, not {body}"
    );
    assert!(
        !body.is_null(),
        "single-PUT finalize body must not be JSON null"
    );
    // And it must not enumerate parts.
    assert!(
        body.get("parts").is_none(),
        "single-PUT finalize must omit the parts key, got {body}"
    );
}

#[tokio::test]
async fn single_put_progress_is_byte_granular() {
    // A single-PUT body larger than one read chunk must produce MULTIPLE
    // intermediate progress ticks (not just 0 and total), so the CLI renders a
    // smooth bar instead of a 0% -> 100% jump. FramedRead's BytesCodec yields
    // chunks of at most a few KiB, so a 256 KiB body spans many chunks.
    let server = MockServer::start().await;
    let storage_url = format!("{}/storage/big", server.uri());
    let contents: Vec<u8> = (0..256 * 1024).map(|i| (i % 251) as u8).collect();
    let total = contents.len() as u64;

    Mock::given(method("POST"))
        .and(path("/v1/uploads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "finalize_token": "ftok_big",
            "headers": {},
            "mode": "single",
            "upload_id": "upl_big",
            "url": storage_url,
        })))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/storage/big"))
        .respond_with(ResponseTemplate::new(200).insert_header("ETag", "\"big\""))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/uploads/upl_big/finalize"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "created_at": "2026-06-25T00:00:00Z",
            "size_bytes": contents.len(),
            "status": "ready",
            "upload_id": "upl_big",
        })))
        .mount(&server)
        .await;

    // Record every tick. The callback runs on the body-stream task; collect into
    // a shared Vec.
    let ticks: Arc<Mutex<Vec<(u64, u64)>>> = Arc::new(Mutex::new(Vec::new()));
    let ticks_cb = Arc::clone(&ticks);
    let progress: hotdata::UploadProgress = Arc::new(move |done, total| {
        ticks_cb.lock().unwrap().push((done, total));
    });
    let opts = UploadOptions {
        progress: Some(progress),
        ..UploadOptions::default()
    };

    let path = temp_file(&contents);
    let client = test_client(&server.uri());
    let result = client.upload_file(&path, opts).await;
    let _ = std::fs::remove_file(&path);
    result.expect("single upload should succeed");

    let ticks = ticks.lock().unwrap();
    // Many intermediate updates, not just the terminal one.
    let intermediate = ticks.iter().filter(|(d, _)| *d > 0 && *d < total).count();
    assert!(
        intermediate >= 2,
        "single-PUT progress must fire multiple intermediate ticks for a \
         multi-chunk body; saw ticks: {ticks:?}"
    );
    // Total is always the file size; the sequence is monotonic non-decreasing.
    let mut prev = 0u64;
    for (d, t) in ticks.iter() {
        assert_eq!(*t, total, "total must be the file size");
        assert!(
            *d >= prev,
            "progress must be non-decreasing: {d} after {prev}"
        );
        assert!(*d <= total, "progress must never exceed total");
        prev = *d;
    }
    // The final observed value is exactly the file size.
    assert_eq!(
        ticks.last().map(|(d, _)| *d),
        Some(total),
        "single-PUT progress must reach exactly the file size"
    );
}

#[tokio::test]
async fn multipart_happy_path() {
    let server = MockServer::start().await;
    let part_size = 5usize;
    // 13 bytes over part_size=5 -> parts of 5, 5, 3 (last is the remainder).
    let contents: Vec<u8> = (0u8..13).collect();
    let part_urls: Vec<String> = (1..=3)
        .map(|i| format!("{}/storage/part/{i}", server.uri()))
        .collect();

    Mock::given(method("POST"))
        .and(path("/v1/uploads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "finalize_token": "ftok_multi",
            "headers": {},
            "mode": "multipart",
            "part_size": part_size,
            "part_urls": part_urls,
            "upload_id": "upl_multi",
        })))
        .mount(&server)
        .await;

    // Each part endpoint returns a distinct ETag so we can assert per-part
    // collection. The mock echoes its part number into the ETag value.
    for i in 1..=3 {
        Mock::given(method("PUT"))
            .and(path(format!("/storage/part/{i}")))
            .respond_with(
                ResponseTemplate::new(200).insert_header("ETag", format!("\"etag-part-{i}\"")),
            )
            .mount(&server)
            .await;
    }

    Mock::given(method("POST"))
        .and(path("/v1/uploads/upl_multi/finalize"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "created_at": "2026-06-25T00:00:00Z",
            "size_bytes": contents.len(),
            "status": "ready",
            "upload_id": "upl_multi",
        })))
        .mount(&server)
        .await;

    let path = temp_file(&contents);
    let client = test_client(&server.uri());
    let result = client.upload_file(&path, UploadOptions::default()).await;
    let _ = std::fs::remove_file(&path);
    let result = result.expect("multipart upload should succeed");
    assert_eq!(result.upload_id, "upl_multi");

    let requests = server.received_requests().await.expect("requests recorded");

    // Each part received exactly its slice: part i (1-based) gets bytes
    // [(i-1)*part_size, i*part_size), last is the remainder.
    let expected_slices = [&contents[0..5], &contents[5..10], &contents[10..13]];
    for (i, expected) in expected_slices.iter().enumerate() {
        let part_path = format!("/storage/part/{}", i + 1);
        let put = requests
            .iter()
            .find(|r| r.url.path() == part_path)
            .unwrap_or_else(|| panic!("a PUT to {part_path} should have been made"));
        assert_eq!(
            &put.body[..],
            *expected,
            "part {} body must be the {}-byte slice",
            i + 1,
            expected.len()
        );
        assert_eq!(
            put.headers
                .get("content-length")
                .and_then(|v| v.to_str().ok()),
            Some(expected.len().to_string().as_str()),
            "part {} must set Content-Length to its slice length",
            i + 1
        );
        assert_no_sdk_headers(put);
    }

    // Finalize carried the ascending {part_number, e_tag} list, ETags
    // byte-for-byte (quotes preserved).
    let finalize = requests
        .iter()
        .find(|r| r.url.path() == "/v1/uploads/upl_multi/finalize")
        .expect("a finalize request should have been made");
    assert_eq!(
        finalize
            .headers
            .get("x-upload-finalize-token")
            .and_then(|v| v.to_str().ok()),
        Some("ftok_multi"),
    );
    let body: serde_json::Value = serde_json::from_slice(&finalize.body).expect("finalize JSON");
    // The body must be a JSON object carrying `parts` — never `null`.
    assert!(
        body.is_object(),
        "multipart finalize body must be a JSON object, not {body}"
    );
    let parts = body
        .get("parts")
        .and_then(|p| p.as_array())
        .expect("multipart finalize must send a parts array");
    assert_eq!(parts.len(), 3, "all three parts must be finalized");
    for (i, part) in parts.iter().enumerate() {
        assert_eq!(
            part.get("part_number").and_then(|v| v.as_i64()),
            Some((i + 1) as i64),
            "parts must be ascending and 1-based"
        );
        assert_eq!(
            part.get("e_tag").and_then(|v| v.as_str()),
            Some(format!("\"etag-part-{}\"", i + 1).as_str()),
            "ETag must be forwarded byte-for-byte with surrounding quotes"
        );
    }
}

#[tokio::test]
async fn progress_callback_reaches_total() {
    let server = MockServer::start().await;
    let part_size = 4usize;
    let contents: Vec<u8> = (0u8..10).collect(); // 4, 4, 2
    let part_urls: Vec<String> = (1..=3)
        .map(|i| format!("{}/storage/p/{i}", server.uri()))
        .collect();

    Mock::given(method("POST"))
        .and(path("/v1/uploads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "finalize_token": "ftok",
            "headers": {},
            "mode": "multipart",
            "part_size": part_size,
            "part_urls": part_urls,
            "upload_id": "upl_prog",
        })))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path_regex(r"^/storage/p/\d+$"))
        .respond_with(ResponseTemplate::new(200).insert_header("ETag", "\"e\""))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/uploads/upl_prog/finalize"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "created_at": "2026-06-25T00:00:00Z",
            "size_bytes": contents.len(),
            "status": "ready",
            "upload_id": "upl_prog",
        })))
        .mount(&server)
        .await;

    // Record every (done, total) tick; assert monotonic and terminal == size.
    let ticks: Arc<Mutex<Vec<(u64, u64)>>> = Arc::new(Mutex::new(Vec::new()));
    let max_done = Arc::new(AtomicU64::new(0));
    let ticks_cb = Arc::clone(&ticks);
    let max_cb = Arc::clone(&max_done);
    let progress: hotdata::UploadProgress = Arc::new(move |done, total| {
        // Monotonic non-decreasing (tasks complete concurrently, but the shared
        // AtomicU64 counter only grows).
        let prev = max_cb.fetch_max(done, Ordering::SeqCst);
        assert!(
            done >= prev,
            "progress must be non-decreasing: saw {done} after {prev}"
        );
        ticks_cb.lock().unwrap().push((done, total));
    });

    let opts = UploadOptions {
        progress: Some(progress),
        ..UploadOptions::default()
    };

    let path = temp_file(&contents);
    let client = test_client(&server.uri());
    let result = client.upload_file(&path, opts).await;
    let _ = std::fs::remove_file(&path);
    result.expect("upload should succeed");

    let ticks = ticks.lock().unwrap();
    assert!(!ticks.is_empty(), "progress callback must be invoked");
    let total = contents.len() as u64;
    for (_, t) in ticks.iter() {
        assert_eq!(*t, total, "total passed to progress must be the file size");
    }
    let final_done = ticks.iter().map(|(d, _)| *d).max().unwrap();
    assert_eq!(
        final_done, total,
        "progress must reach exactly the file size"
    );
}

#[tokio::test]
async fn storage_put_header_isolation_negative_check() {
    // A focused negative check on the single-PUT path: the storage PUT must not
    // carry the SDK bearer or workspace/session scope headers even though the
    // client is fully configured with all of them.
    let server = MockServer::start().await;
    let storage_url = format!("{}/storage/iso", server.uri());
    let contents = b"isolation bytes";

    Mock::given(method("POST"))
        .and(path("/v1/uploads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "finalize_token": "ftok_iso",
            "headers": {},
            "mode": "single",
            "upload_id": "upl_iso",
            "url": storage_url,
        })))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/storage/iso"))
        .respond_with(ResponseTemplate::new(200).insert_header("ETag", "\"iso\""))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/uploads/upl_iso/finalize"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "created_at": "2026-06-25T00:00:00Z",
            "size_bytes": contents.len(),
            "status": "ready",
            "upload_id": "upl_iso",
        })))
        .mount(&server)
        .await;

    let path = temp_file(contents);
    let client = test_client(&server.uri());
    let result = client.upload_file(&path, UploadOptions::default()).await;
    let _ = std::fs::remove_file(&path);
    result.expect("upload should succeed");

    let requests = server.received_requests().await.expect("requests recorded");
    let put = requests
        .iter()
        .find(|r| r.url.path() == "/storage/iso")
        .expect("a storage PUT should have been made");
    assert_no_sdk_headers(put);

    // Sanity: the API requests (create/finalize) DO carry the SDK headers, so
    // the isolation is specific to the storage PUT, not a client-wide accident.
    let create = requests
        .iter()
        .find(|r| r.url.path() == "/v1/uploads")
        .expect("a create-session request should have been made");
    assert_eq!(
        create
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok()),
        Some("Bearer test-bearer"),
        "the API create request should still carry the bearer token"
    );
    assert_eq!(
        create
            .headers
            .get("x-workspace-id")
            .and_then(|v| v.to_str().ok()),
        Some("ws_test"),
    );
}

/// Drive a single-PUT upload against a fresh mock server and return the JSON
/// body the SDK sent to `POST /v1/uploads` (so tests can assert the part-size
/// hint). The file written has `file_len` bytes; `opts` is passed through.
async fn capture_create_body(file_len: usize, opts: UploadOptions) -> serde_json::Value {
    let server = MockServer::start().await;
    let storage_url = format!("{}/storage/hint", server.uri());

    Mock::given(method("POST"))
        .and(path("/v1/uploads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "finalize_token": "ftok_hint",
            "headers": {},
            "mode": "single",
            "upload_id": "upl_hint",
            "url": storage_url,
        })))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/storage/hint"))
        .respond_with(ResponseTemplate::new(200).insert_header("ETag", "\"h\""))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/uploads/upl_hint/finalize"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "created_at": "2026-06-25T00:00:00Z",
            "size_bytes": file_len,
            "status": "ready",
            "upload_id": "upl_hint",
        })))
        .mount(&server)
        .await;

    let contents = vec![0u8; file_len];
    let path = temp_file(&contents);
    let client = test_client(&server.uri());
    let result = client.upload_file(&path, opts).await;
    let _ = std::fs::remove_file(&path);
    result.expect("upload should succeed");

    let requests = server.received_requests().await.expect("requests recorded");
    let create = requests
        .iter()
        .find(|r| r.url.path() == "/v1/uploads")
        .expect("a create-session request should have been made");
    serde_json::from_slice(&create.body).expect("create body must be valid JSON")
}

#[tokio::test]
async fn create_sends_8mib_part_size_hint_for_normal_file() {
    // A normal-sized file sends the default 8 MiB hint (auto-scaling only kicks
    // in for very large files).
    let body = capture_create_body(4096, UploadOptions::default()).await;
    assert_eq!(
        body.get("part_size").and_then(|v| v.as_u64()),
        Some(hotdata::DEFAULT_PART_SIZE),
        "normal file must send the 8 MiB default hint, body: {body}"
    );
}

#[tokio::test]
async fn create_part_size_hint_matches_auto_scaler() {
    // Whatever the SDK sends must equal the public pure scaler for the file's
    // size, so the CLI can reason about it. (The scaler's large-file behavior is
    // unit-tested directly in src/uploads.rs without writing a giant file.)
    let file_len = 64 * 1024usize; // 64 KiB
    let body = capture_create_body(file_len, UploadOptions::default()).await;
    assert_eq!(
        body.get("part_size").and_then(|v| v.as_u64()),
        Some(hotdata::auto_part_size_hint(file_len as u64)),
        "auto hint on the wire must match auto_part_size_hint(); body: {body}"
    );
    // Sanity: the auto scaler keeps the part count well under the S3 hard limit.
    let hint = hotdata::auto_part_size_hint(file_len as u64);
    assert!(hint.is_multiple_of(1024 * 1024), "hint must be a whole MiB");
}

#[tokio::test]
async fn create_explicit_part_size_overrides_auto_hint() {
    // An explicit opts.part_size must be forwarded verbatim, overriding the
    // auto-scaler.
    let explicit = 16 * 1024 * 1024u64; // 16 MiB
    let opts = UploadOptions {
        part_size: Some(explicit),
        ..UploadOptions::default()
    };
    let body = capture_create_body(4096, opts).await;
    assert_eq!(
        body.get("part_size").and_then(|v| v.as_u64()),
        Some(explicit),
        "explicit part_size must override the auto hint, body: {body}"
    );
}

// ---------------------------------------------------------------------------
// Codex review follow-ups
// ---------------------------------------------------------------------------

/// Mock the standard create-session response for a SINGLE upload at `storage_url`.
fn mock_single_session(upload_id: &str, token: &str, storage_url: &str) -> serde_json::Value {
    serde_json::json!({
        "finalize_token": token,
        "headers": {},
        "mode": "single",
        "upload_id": upload_id,
        "url": storage_url,
    })
}

/// Mock a finalize success body.
fn mock_finalize_ok(upload_id: &str, size: usize) -> serde_json::Value {
    serde_json::json!({
        "created_at": "2026-06-25T00:00:00Z",
        "size_bytes": size,
        "status": "ready",
        "upload_id": upload_id,
    })
}

/// #2: default headers set on the SDK's MAIN client must NOT reach storage PUTs.
/// We install an Authorization + X-Workspace-Id default header on the reqwest
/// client the SDK uses, then assert the storage PUT carries neither (it goes out
/// on the dedicated bare storage client).
#[tokio::test]
async fn default_headers_on_main_client_do_not_reach_storage_put() {
    let server = MockServer::start().await;
    let storage_url = format!("{}/storage/iso2", server.uri());
    let contents = b"bytes with poisoned client";

    Mock::given(method("POST"))
        .and(path("/v1/uploads"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(mock_single_session(
                "upl_iso2",
                "ftok_iso2",
                &storage_url,
            )),
        )
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/storage/iso2"))
        .respond_with(ResponseTemplate::new(200).insert_header("ETag", "\"iso2\""))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/uploads/upl_iso2/finalize"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(mock_finalize_ok("upl_iso2", contents.len())),
        )
        .mount(&server)
        .await;

    // A reqwest client carrying default headers a host might set.
    let mut default_headers = reqwest::header::HeaderMap::new();
    default_headers.insert(
        reqwest::header::AUTHORIZATION,
        "Bearer host-default".parse().unwrap(),
    );
    default_headers.insert("X-Workspace-Id", "ws-default".parse().unwrap());
    default_headers.insert(reqwest::header::USER_AGENT, "host-agent/9".parse().unwrap());
    let poisoned = reqwest::Client::builder()
        .default_headers(default_headers)
        .build()
        .unwrap();

    let mut config = Configuration {
        base_path: server.uri(),
        bearer_access_token: Some("test-bearer".to_owned()),
        client: poisoned,
        ..Configuration::default()
    };
    config.api_keys.insert(
        WORKSPACE_HEADER.to_owned(),
        ApiKey {
            prefix: None,
            key: "ws_test".to_owned(),
        },
    );
    let client = Client::from_configuration(config);

    let path = temp_file(contents);
    let result = client.upload_file(&path, UploadOptions::default()).await;
    let _ = std::fs::remove_file(&path);
    result.expect("upload should succeed");

    let requests = server.received_requests().await.expect("requests recorded");
    let put = requests
        .iter()
        .find(|r| r.url.path() == "/storage/iso2")
        .expect("a storage PUT should have been made");
    // The host's default headers must not be on the storage PUT.
    assert!(
        put.headers.get("authorization").is_none(),
        "storage PUT must not carry the host client's default Authorization"
    );
    assert!(
        put.headers.get("x-workspace-id").is_none(),
        "storage PUT must not carry the host client's default X-Workspace-Id"
    );
    // The bare client sends reqwest's own default UA, not the host's; assert the
    // host UA didn't leak.
    assert_ne!(
        put.headers.get("user-agent").and_then(|v| v.to_str().ok()),
        Some("host-agent/9"),
        "storage PUT must not carry the host client's default User-Agent"
    );
}

/// #1: finalize must be exactly-once — it must NOT retry even when the client's
/// retry policy allows retries. We make finalize return 429 (the status
/// `execute_retrying` DOES retry, with Retry-After: 0) under a policy of 5
/// retries: if finalize were routed through the retry wrapper it would hit the
/// server 6 times; with retries disabled for finalize it must hit exactly once.
/// This is the discriminating regression guard for the no-retry change (a 500
/// wouldn't exercise the wrapper at all).
#[tokio::test]
async fn finalize_is_not_retried() {
    let server = MockServer::start().await;
    let storage_url = format!("{}/storage/fin", server.uri());
    let contents = b"finalize once";

    Mock::given(method("POST"))
        .and(path("/v1/uploads"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(mock_single_session(
                "upl_fin",
                "ftok_fin",
                &storage_url,
            )),
        )
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/storage/fin"))
        .respond_with(ResponseTemplate::new(200).insert_header("ETag", "\"fin\""))
        .mount(&server)
        .await;
    // Finalize returns 429 (Retry-After: 0) — the wrapper WOULD retry this if it
    // were applied. With retries disabled for finalize, only one request lands.
    Mock::given(method("POST"))
        .and(path("/v1/uploads/upl_fin/finalize"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "0"))
        .mount(&server)
        .await;

    let path = temp_file(contents);
    // 5 retries allowed at the policy level — but finalize must ignore them.
    let client = test_client_with_retry(&server.uri(), fast_retry(5));
    let result = client.upload_file(&path, UploadOptions::default()).await;
    let _ = std::fs::remove_file(&path);

    assert!(
        matches!(result, Err(UploadError::Finalize(_))),
        "a finalize 429 must surface as UploadError::Finalize (not be retried away)"
    );
    let finalize_hits = server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.url.path() == "/v1/uploads/upl_fin/finalize")
        .count();
    assert_eq!(
        finalize_hits, 1,
        "finalize must be attempted exactly once despite a retry policy of 5"
    );
}

/// #1 (partner): per-part PUTs ARE retryable. A part returns 500 once (a 429
/// would also retry, but we use the SDK's pre-response handling for transport;
/// here we assert the 429 path which execute_retrying retries) then 200, and the
/// upload still completes. We use 429 because that is what execute_retrying
/// retries on a status.
#[tokio::test]
async fn part_put_retries_429_then_succeeds() {
    let server = MockServer::start().await;
    let part_size = 5usize;
    let contents: Vec<u8> = (0u8..10).collect(); // 2 parts: 5, 5
    let part_urls: Vec<String> = (1..=2)
        .map(|i| format!("{}/storage/rpart/{i}", server.uri()))
        .collect();

    Mock::given(method("POST"))
        .and(path("/v1/uploads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "finalize_token": "ftok_rp",
            "headers": {},
            "mode": "multipart",
            "part_size": part_size,
            "part_urls": part_urls,
            "upload_id": "upl_rp",
        })))
        .mount(&server)
        .await;

    // Part 1: one 429 (Retry-After: 0) then 200.
    Mock::given(method("PUT"))
        .and(path("/storage/rpart/1"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "0"))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/storage/rpart/1"))
        .respond_with(ResponseTemplate::new(200).insert_header("ETag", "\"rp1\""))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/storage/rpart/2"))
        .respond_with(ResponseTemplate::new(200).insert_header("ETag", "\"rp2\""))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/uploads/upl_rp/finalize"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(mock_finalize_ok("upl_rp", contents.len())),
        )
        .mount(&server)
        .await;

    let path = temp_file(&contents);
    let client = test_client_with_retry(&server.uri(), fast_retry(5));
    let result = client.upload_file(&path, UploadOptions::default()).await;
    let _ = std::fs::remove_file(&path);
    result.expect("multipart upload should complete after a part retry");

    // Part 1 was hit twice (429 then 200).
    let p1_hits = server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.url.path() == "/storage/rpart/1")
        .count();
    assert_eq!(p1_hits, 2, "part 1 must be retried after the 429");
}

/// Missing ETag on a part PUT response surfaces as UploadError::MissingETag.
#[tokio::test]
async fn missing_etag_is_an_error() {
    let server = MockServer::start().await;
    let part_size = 5usize;
    let contents: Vec<u8> = (0u8..5).collect(); // 1 part
    let part_urls = vec![format!("{}/storage/noetag/1", server.uri())];

    Mock::given(method("POST"))
        .and(path("/v1/uploads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "finalize_token": "ftok_ne",
            "headers": {},
            "mode": "multipart",
            "part_size": part_size,
            "part_urls": part_urls,
            "upload_id": "upl_ne",
        })))
        .mount(&server)
        .await;
    // 200 but NO ETag header.
    Mock::given(method("PUT"))
        .and(path("/storage/noetag/1"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let path = temp_file(&contents);
    let client = test_client(&server.uri());
    let result = client.upload_file(&path, UploadOptions::default()).await;
    let _ = std::fs::remove_file(&path);
    assert!(
        matches!(result, Err(UploadError::MissingETag { part_number: 1 })),
        "missing ETag must surface as MissingETag, got {result:?}"
    );
}

/// A storage 4xx/5xx surfaces as UploadError::StorageStatus.
#[tokio::test]
async fn storage_error_status_is_surfaced() {
    let server = MockServer::start().await;
    let storage_url = format!("{}/storage/403", server.uri());
    let contents = b"denied";

    Mock::given(method("POST"))
        .and(path("/v1/uploads"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(mock_single_session(
                "upl_403",
                "ftok_403",
                &storage_url,
            )),
        )
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/storage/403"))
        .respond_with(ResponseTemplate::new(403).set_body_string("<Error>AccessDenied</Error>"))
        .mount(&server)
        .await;

    let path = temp_file(contents);
    let client = test_client(&server.uri());
    let result = client.upload_file(&path, UploadOptions::default()).await;
    let _ = std::fs::remove_file(&path);
    match result {
        Err(UploadError::StorageStatus { status, body, .. }) => {
            assert_eq!(status, reqwest::StatusCode::FORBIDDEN);
            assert!(
                body.contains("AccessDenied"),
                "body should carry storage error: {body}"
            );
        }
        other => panic!("expected StorageStatus(403), got {other:?}"),
    }
}

/// 501 PRESIGN_UNSUPPORTED on create-session is a hard error — NO /v1/files
/// fallback is attempted.
#[tokio::test]
async fn presign_unsupported_is_hard_error_no_fallback() {
    let server = MockServer::start().await;
    let contents = b"no presign here";

    Mock::given(method("POST"))
        .and(path("/v1/uploads"))
        .respond_with(ResponseTemplate::new(501).set_body_json(serde_json::json!({
            "error": { "code": "PRESIGN_UNSUPPORTED", "message": "no presign" }
        })))
        .mount(&server)
        .await;

    let path = temp_file(contents);
    let client = test_client(&server.uri());
    let result = client.upload_file(&path, UploadOptions::default()).await;
    let _ = std::fs::remove_file(&path);
    assert!(
        matches!(result, Err(UploadError::CreateSession(_))),
        "501 must surface as CreateSession error, got {result:?}"
    );
    // No request to the legacy proxy.
    let hit_files = server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .any(|r| r.url.path() == "/v1/files");
    assert!(!hit_files, "must NOT fall back to POST /v1/files");
}

/// Malformed multipart sessions are rejected as MalformedSession.
#[tokio::test]
async fn malformed_multipart_sessions_are_rejected() {
    // Helper: run an upload whose create-session returns the given multipart JSON
    // overrides, and return the result.
    async fn run(session: serde_json::Value, file_len: usize) -> Result<(), UploadError> {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/uploads"))
            .respond_with(ResponseTemplate::new(200).set_body_json(session))
            .mount(&server)
            .await;
        // Accept any PUT so a (wrongly) issued part doesn't fail for another
        // reason; we expect to reject BEFORE PUTting.
        Mock::given(method("PUT"))
            .and(path_regex(r"^/storage/.*$"))
            .respond_with(ResponseTemplate::new(200).insert_header("ETag", "\"x\""))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path_regex(r"^/v1/uploads/.*/finalize$"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_finalize_ok("u", file_len)))
            .mount(&server)
            .await;

        let contents = vec![0u8; file_len];
        let path = temp_file(&contents);
        let client = test_client(&server.uri());
        let result = client.upload_file(&path, UploadOptions::default()).await;
        let _ = std::fs::remove_file(&path);
        result.map(|_| ())
    }

    let su = "http://example.invalid/storage/p1";

    // Zero part_size.
    let r = run(
        serde_json::json!({
            "finalize_token": "t", "headers": {}, "mode": "multipart",
            "part_size": 0, "part_urls": [su], "upload_id": "u",
        }),
        10,
    )
    .await;
    assert!(
        matches!(r, Err(UploadError::MalformedSession(_))),
        "zero part_size: {r:?}"
    );

    // Negative part_size.
    let r = run(
        serde_json::json!({
            "finalize_token": "t", "headers": {}, "mode": "multipart",
            "part_size": -5, "part_urls": [su], "upload_id": "u",
        }),
        10,
    )
    .await;
    assert!(
        matches!(r, Err(UploadError::MalformedSession(_))),
        "negative part_size: {r:?}"
    );

    // Empty part_urls.
    let r = run(
        serde_json::json!({
            "finalize_token": "t", "headers": {}, "mode": "multipart",
            "part_size": 5, "part_urls": [], "upload_id": "u",
        }),
        10,
    )
    .await;
    assert!(
        matches!(r, Err(UploadError::MalformedSession(_))),
        "empty part_urls: {r:?}"
    );

    // Too FEW URLs: 10 bytes / 5 = 2 parts, but only 1 URL.
    let r = run(
        serde_json::json!({
            "finalize_token": "t", "headers": {}, "mode": "multipart",
            "part_size": 5, "part_urls": [su], "upload_id": "u",
        }),
        10,
    )
    .await;
    assert!(
        matches!(r, Err(UploadError::MalformedSession(_))),
        "too few URLs: {r:?}"
    );

    // Too MANY URLs: 10 bytes / 5 = 2 parts, but 3 URLs.
    let r = run(
        serde_json::json!({
            "finalize_token": "t", "headers": {}, "mode": "multipart",
            "part_size": 5, "part_urls": [su, su, su], "upload_id": "u",
        }),
        10,
    )
    .await;
    assert!(
        matches!(r, Err(UploadError::MalformedSession(_))),
        "too many URLs: {r:?}"
    );
}

/// Server-provided Content-Type in the session `headers` map is replayed
/// verbatim on the storage PUT.
#[tokio::test]
async fn server_content_type_is_replayed_on_storage_put() {
    let server = MockServer::start().await;
    let storage_url = format!("{}/storage/ct", server.uri());
    let contents = b"typed bytes";

    Mock::given(method("POST"))
        .and(path("/v1/uploads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "finalize_token": "ftok_ct",
            "headers": { "Content-Type": "application/parquet" },
            "mode": "single",
            "upload_id": "upl_ct",
            "url": storage_url,
        })))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/storage/ct"))
        .respond_with(ResponseTemplate::new(200).insert_header("ETag", "\"ct\""))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/uploads/upl_ct/finalize"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(mock_finalize_ok("upl_ct", contents.len())),
        )
        .mount(&server)
        .await;

    let path = temp_file(contents);
    let client = test_client(&server.uri());
    let result = client.upload_file(&path, UploadOptions::default()).await;
    let _ = std::fs::remove_file(&path);
    result.expect("upload should succeed");

    let requests = server.received_requests().await.expect("requests recorded");
    let put = requests
        .iter()
        .find(|r| r.url.path() == "/storage/ct")
        .expect("storage PUT");
    assert_eq!(
        put.headers
            .get("content-type")
            .and_then(|v| v.to_str().ok()),
        Some("application/parquet"),
        "server-provided Content-Type must be replayed verbatim (no charset appended)"
    );
}

/// A bare blocking TCP server that plays "object storage" for part PUTs and
/// GENUINELY measures concurrency: each connection is handled on its own thread,
/// which reads the full request (headers + Content-Length body), bumps an active
/// counter, HOLDS it for `hold` (so overlapping PUTs actually coexist), records
/// the peak, then decrements and replies `200 OK` with an `ETag`. Because the
/// in-flight count is held across the sleep — unlike wiremock's synchronous
/// `Respond`, which can't span its own response delay — `peak` reflects true
/// overlap and can be asserted to reach (not just stay under) the cap.
///
/// Returns `(base_url, peak, served)`. Mirrors the raw-TCP pattern in
/// `src/test_support.rs`.
fn concurrency_storage_server(hold: Duration) -> (String, Arc<AtomicUsize>, Arc<AtomicUsize>) {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    let addr = listener.local_addr().expect("local addr");
    let active = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));
    let served = Arc::new(AtomicUsize::new(0));
    let (a, p, s) = (Arc::clone(&active), Arc::clone(&peak), Arc::clone(&served));

    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut sock) = stream else { continue };
            let (a, p, s, hold) = (Arc::clone(&a), Arc::clone(&p), Arc::clone(&s), hold);
            std::thread::spawn(move || {
                // Read until we have the full headers, then drain the body by its
                // declared Content-Length so the client finishes writing before
                // we respond.
                let mut buf = Vec::new();
                let mut tmp = [0u8; 4096];
                let header_end = loop {
                    match sock.read(&mut tmp) {
                        Ok(0) => break None,
                        Ok(n) => {
                            buf.extend_from_slice(&tmp[..n]);
                            if let Some(pos) = find_subslice(&buf, b"\r\n\r\n") {
                                break Some(pos + 4);
                            }
                        }
                        Err(_) => break None,
                    }
                };
                if let Some(body_start) = header_end {
                    let headers = String::from_utf8_lossy(&buf[..body_start]).to_lowercase();
                    let content_len = headers
                        .lines()
                        .find_map(|l| l.strip_prefix("content-length:"))
                        .and_then(|v| v.trim().parse::<usize>().ok())
                        .unwrap_or(0);
                    let mut have = buf.len() - body_start;
                    while have < content_len {
                        match sock.read(&mut tmp) {
                            Ok(0) => break,
                            Ok(n) => have += n,
                            Err(_) => break,
                        }
                    }
                }

                // Now genuinely occupy an in-flight slot for the hold duration.
                let now = a.fetch_add(1, Ordering::SeqCst) + 1;
                p.fetch_max(now, Ordering::SeqCst);
                std::thread::sleep(hold);
                a.fetch_sub(1, Ordering::SeqCst);
                s.fetch_add(1, Ordering::SeqCst);

                let resp = "HTTP/1.1 200 OK\r\nETag: \"c\"\r\ncontent-length: 0\r\n\
                            connection: close\r\n\r\n";
                let _ = sock.write_all(resp.as_bytes());
                let _ = sock.flush();
            });
        }
    });

    (format!("http://{addr}"), peak, served)
}

/// Find the first index of `needle` in `haystack`.
fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// Bounded in-flight concurrency, measured faithfully. With 6 parts and
/// `max_concurrency = 2` (and 8 MiB-equivalent small parts so the memory budget
/// doesn't reduce the cap), exactly 2 PUTs overlap at the peak — never more, and
/// it actually reaches 2. The storage server holds each PUT for 100ms so the
/// JoinSet bound is genuinely exercised.
#[tokio::test]
async fn in_flight_concurrency_is_bounded_and_reached() {
    let (storage_base, peak, served) = concurrency_storage_server(Duration::from_millis(100));

    let server = MockServer::start().await;
    let part_size = 5usize;
    // 6 parts of 5 bytes: 30 bytes total.
    let contents: Vec<u8> = (0u8..30).collect();
    let part_urls: Vec<String> = (1..=6)
        .map(|i| format!("{storage_base}/cpart/{i}"))
        .collect();

    Mock::given(method("POST"))
        .and(path("/v1/uploads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "finalize_token": "ftok_c",
            "headers": {},
            "mode": "multipart",
            "part_size": part_size,
            "part_urls": part_urls,
            "upload_id": "upl_c",
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/uploads/upl_c/finalize"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(mock_finalize_ok("upl_c", contents.len())),
        )
        .mount(&server)
        .await;

    let opts = UploadOptions {
        max_concurrency: Some(2),
        ..UploadOptions::default()
    };
    let path = temp_file(&contents);
    let client = test_client(&server.uri());
    let result = client.upload_file(&path, opts).await;
    let _ = std::fs::remove_file(&path);
    result.expect("upload should complete");

    // All 6 parts were served by the storage server.
    assert_eq!(served.load(Ordering::SeqCst), 6, "all 6 parts must be PUT");
    // Genuine overlap: peak reaches the cap and never exceeds it.
    let observed = peak.load(Ordering::SeqCst);
    assert_eq!(
        observed, 2,
        "in-flight concurrency must reach exactly max_concurrency=2, observed {observed}"
    );
}

/// Serial when `max_concurrency = 1`: no two PUTs ever overlap.
#[tokio::test]
async fn serial_when_max_concurrency_is_one() {
    let (storage_base, peak, served) = concurrency_storage_server(Duration::from_millis(40));

    let server = MockServer::start().await;
    let part_size = 5usize;
    let contents: Vec<u8> = (0u8..20).collect(); // 4 parts
    let part_urls: Vec<String> = (1..=4)
        .map(|i| format!("{storage_base}/spart/{i}"))
        .collect();

    Mock::given(method("POST"))
        .and(path("/v1/uploads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "finalize_token": "ftok_s",
            "headers": {},
            "mode": "multipart",
            "part_size": part_size,
            "part_urls": part_urls,
            "upload_id": "upl_s",
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/uploads/upl_s/finalize"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(mock_finalize_ok("upl_s", contents.len())),
        )
        .mount(&server)
        .await;

    let opts = UploadOptions {
        max_concurrency: Some(1),
        ..UploadOptions::default()
    };
    let path = temp_file(&contents);
    let client = test_client(&server.uri());
    let result = client.upload_file(&path, opts).await;
    let _ = std::fs::remove_file(&path);
    result.expect("upload should complete");

    assert_eq!(served.load(Ordering::SeqCst), 4, "all 4 parts must be PUT");
    assert_eq!(
        peak.load(Ordering::SeqCst),
        1,
        "max_concurrency=1 must keep PUTs strictly serial (no overlap)"
    );
}

// ---------------------------------------------------------------------------
// Streaming (just-in-time part minting) — issue sdk-rust#76
// ---------------------------------------------------------------------------

/// One mebibyte.
const MIB: usize = 1024 * 1024;

/// A wiremock responder for `POST /v1/uploads/{id}/parts` that reads the
/// requested `part_numbers` from the request body and returns one minted URL per
/// part, each pointing at `{storage_base}/storage/spart/{n}`. This mirrors the
/// real server, which mints exactly the parts asked for, on demand.
fn mint_parts_responder(
    storage_base: String,
) -> impl Fn(&Request) -> ResponseTemplate + Send + Sync + 'static {
    move |req: &Request| {
        let body: serde_json::Value =
            serde_json::from_slice(&req.body).expect("mint request body must be JSON");
        let nums = body
            .get("part_numbers")
            .and_then(|v| v.as_array())
            .expect("mint request must carry a part_numbers array");
        let parts: Vec<serde_json::Value> = nums
            .iter()
            .map(|n| {
                let n = n.as_i64().expect("part_number must be an integer");
                serde_json::json!({
                    "part_number": n,
                    "url": format!("{storage_base}/storage/spart/{n}"),
                })
            })
            .collect();
        ResponseTemplate::new(200).set_body_json(serde_json::json!({ "parts": parts }))
    }
}

/// A large file (> the 8 MiB single-PUT threshold) must use the STREAMING path:
/// open the session WITHOUT `declared_size_bytes`, mint part URLs on demand via
/// `POST /v1/uploads/{id}/parts` (carrying the finalize token in the header), and
/// finalize the ascending `{part_number, e_tag}` list. This is the structural fix
/// for URL expiry — no part URL is minted until just before its chunk is sent.
#[tokio::test]
async fn large_file_uses_streaming_jit_minting() {
    let server = MockServer::start().await;
    let storage_base = server.uri();
    // 9 MiB over a 5 MiB part size -> 2 parts: 5 MiB + 4 MiB.
    let part_size = 5 * MIB;
    let contents: Vec<u8> = (0..9 * MIB).map(|i| (i % 251) as u8).collect();

    // Create: streaming session — NO part_urls up front, part_size echoed.
    Mock::given(method("POST"))
        .and(path("/v1/uploads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "finalize_token": "ftok_stream",
            "headers": {},
            "mode": "multipart",
            "part_size": part_size,
            "upload_id": "upl_stream",
        })))
        .mount(&server)
        .await;

    // On-demand mint endpoint.
    Mock::given(method("POST"))
        .and(path("/v1/uploads/upl_stream/parts"))
        .respond_with(mint_parts_responder(storage_base.clone()))
        .mount(&server)
        .await;

    // Storage PUT targets, one per part, each returning a distinct ETag.
    for n in 1..=2 {
        Mock::given(method("PUT"))
            .and(path(format!("/storage/spart/{n}")))
            .respond_with(ResponseTemplate::new(200).insert_header("ETag", format!("\"etag-{n}\"")))
            .mount(&server)
            .await;
    }

    Mock::given(method("POST"))
        .and(path("/v1/uploads/upl_stream/finalize"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(mock_finalize_ok("upl_stream", contents.len())),
        )
        .mount(&server)
        .await;

    let path = temp_file(&contents);
    let client = test_client(&server.uri());
    let result = client.upload_file(&path, UploadOptions::default()).await;
    let _ = std::fs::remove_file(&path);
    let result = result.expect("streaming upload should succeed");
    assert_eq!(result.upload_id, "upl_stream");

    let requests = server.received_requests().await.expect("requests recorded");

    // 1. The create request omitted declared_size_bytes (the streaming signal)
    //    while still sending the part_size hint.
    let create = requests
        .iter()
        .find(|r| r.url.path() == "/v1/uploads")
        .expect("a create-session request should have been made");
    let create_body: serde_json::Value =
        serde_json::from_slice(&create.body).expect("create body JSON");
    assert!(
        create_body.get("declared_size_bytes").is_none(),
        "streaming create must omit declared_size_bytes, body: {create_body}"
    );
    assert!(
        create_body.get("part_size").is_some(),
        "streaming create must still send the part_size hint, body: {create_body}"
    );

    // 2. The SDK minted part URLs on demand, carrying the finalize token in the
    //    header, and asked for the 1-based part numbers it was about to upload.
    let mints: Vec<&Request> = requests
        .iter()
        .filter(|r| r.url.path() == "/v1/uploads/upl_stream/parts")
        .collect();
    assert!(
        !mints.is_empty(),
        "the SDK must mint part URLs via POST /v1/uploads/{{id}}/parts"
    );
    let mut minted_numbers: Vec<i64> = Vec::new();
    for mint in &mints {
        assert_eq!(
            mint.headers
                .get("x-upload-finalize-token")
                .and_then(|v| v.to_str().ok()),
            Some("ftok_stream"),
            "each mint must carry the finalize token in X-Upload-Finalize-Token"
        );
        let body: serde_json::Value = serde_json::from_slice(&mint.body).expect("mint body JSON");
        let nums = body
            .get("part_numbers")
            .and_then(|v| v.as_array())
            .expect("mint must send part_numbers");
        assert!(!nums.is_empty(), "mint part_numbers must be non-empty");
        for n in nums {
            minted_numbers.push(n.as_i64().expect("part number int"));
        }
    }
    minted_numbers.sort_unstable();
    assert_eq!(
        minted_numbers,
        vec![1, 2],
        "exactly parts 1 and 2 must be minted (each once)"
    );

    // 3. Each part received its slice ([0..5MiB), [5MiB..9MiB)).
    let expected = [&contents[0..5 * MIB], &contents[5 * MIB..9 * MIB]];
    for (i, slice) in expected.iter().enumerate() {
        let part_path = format!("/storage/spart/{}", i + 1);
        let put = requests
            .iter()
            .find(|r| r.url.path() == part_path)
            .unwrap_or_else(|| panic!("a PUT to {part_path} should have been made"));
        assert_eq!(&put.body[..], *slice, "part {} body slice", i + 1);
        assert_no_sdk_headers(put);
    }

    // 4. Finalize carried the ascending {part_number, e_tag} list.
    let finalize = requests
        .iter()
        .find(|r| r.url.path() == "/v1/uploads/upl_stream/finalize")
        .expect("a finalize request should have been made");
    let body: serde_json::Value = serde_json::from_slice(&finalize.body).expect("finalize JSON");
    let parts = body
        .get("parts")
        .and_then(|p| p.as_array())
        .expect("finalize must send a parts array");
    assert_eq!(parts.len(), 2, "both parts must be finalized");
    for (i, part) in parts.iter().enumerate() {
        assert_eq!(
            part.get("part_number").and_then(|v| v.as_i64()),
            Some((i + 1) as i64),
            "parts ascending and 1-based"
        );
        assert_eq!(
            part.get("e_tag").and_then(|v| v.as_str()),
            Some(format!("\"etag-{}\"", i + 1).as_str()),
            "ETag forwarded byte-for-byte"
        );
    }
}

/// An expired part URL (storage 403 / SignatureDoesNotMatch) on the streaming
/// path must NOT abort the upload: the SDK re-mints that single part via
/// `POST /v1/uploads/{id}/parts` and retries the `PUT`, so a slow upload whose
/// pre-minted URL lapsed still completes. (Today a storage 403 aborts the whole
/// upload — see `storage_error_status_is_surfaced`.)
#[tokio::test]
async fn expired_part_url_is_reminted_and_retried() {
    let server = MockServer::start().await;
    let storage_base = server.uri();
    let part_size = 5 * MIB;
    let contents: Vec<u8> = (0..9 * MIB).map(|i| (i % 251) as u8).collect();

    Mock::given(method("POST"))
        .and(path("/v1/uploads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "finalize_token": "ftok_exp",
            "headers": {},
            "mode": "multipart",
            "part_size": part_size,
            "upload_id": "upl_exp",
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/v1/uploads/upl_exp/parts"))
        .respond_with(mint_parts_responder(storage_base.clone()))
        .mount(&server)
        .await;

    // Part 1's URL is "expired": first PUT gets 403 (S3-style), then 200 after a
    // re-mint. Part 2 succeeds outright.
    Mock::given(method("PUT"))
        .and(path("/storage/spart/1"))
        .respond_with(
            ResponseTemplate::new(403)
                .set_body_string("<Error><Code>SignatureDoesNotMatch</Code></Error>"),
        )
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/storage/spart/1"))
        .respond_with(ResponseTemplate::new(200).insert_header("ETag", "\"etag-1\""))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/storage/spart/2"))
        .respond_with(ResponseTemplate::new(200).insert_header("ETag", "\"etag-2\""))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/v1/uploads/upl_exp/finalize"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(mock_finalize_ok("upl_exp", contents.len())),
        )
        .mount(&server)
        .await;

    let path = temp_file(&contents);
    // Serial uploads so the 403/re-mint sequence on part 1 is deterministic.
    let opts = UploadOptions {
        max_concurrency: Some(1),
        ..UploadOptions::default()
    };
    let client = test_client(&server.uri());
    let result = client.upload_file(&path, opts).await;
    let _ = std::fs::remove_file(&path);
    result.expect("upload must complete after re-minting the expired part");

    let requests = server.received_requests().await.expect("requests recorded");

    // Part 1 was PUT twice: the expired 403 then the retry after re-mint.
    let p1_puts = requests
        .iter()
        .filter(|r| r.url.path() == "/storage/spart/1")
        .count();
    assert_eq!(p1_puts, 2, "part 1 must be retried after the 403");

    // Part 1 (number 1) was minted at least twice: the initial batch and the
    // single-part re-mint.
    let p1_mints = requests
        .iter()
        .filter(|r| r.url.path() == "/v1/uploads/upl_exp/parts")
        .filter(|r| {
            serde_json::from_slice::<serde_json::Value>(&r.body)
                .ok()
                .and_then(|b| {
                    b.get("part_numbers")
                        .and_then(|v| v.as_array())
                        .map(|a| a.iter().any(|n| n.as_i64() == Some(1)))
                })
                .unwrap_or(false)
        })
        .count();
    assert!(
        p1_mints >= 2,
        "part 1 must be re-minted after its URL expired, saw {p1_mints} mints for it"
    );
}

/// The SDK must pair each minted URL with the part number the SERVER labelled it
/// — not with the order the URLs arrive in — so a response listing parts out of
/// order still uploads every byte range to the correct part. Here the mint
/// responder deliberately reverses the parts; each storage part must still
/// receive its own slice.
#[tokio::test]
async fn mint_response_order_does_not_corrupt_part_slices() {
    let server = MockServer::start().await;
    let storage_base = server.uri();
    // 9 MiB over a 4 MiB part size -> 3 parts: 4 MiB, 4 MiB, 1 MiB.
    let part_size = 4 * MIB;
    let contents: Vec<u8> = (0..9 * MIB).map(|i| (i % 251) as u8).collect();

    Mock::given(method("POST"))
        .and(path("/v1/uploads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "finalize_token": "ftok_ord",
            "headers": {},
            "mode": "multipart",
            "part_size": part_size,
            "upload_id": "upl_ord",
        })))
        .mount(&server)
        .await;

    // Mint responder that returns the requested parts in REVERSE order.
    let base = storage_base.clone();
    Mock::given(method("POST"))
        .and(path("/v1/uploads/upl_ord/parts"))
        .respond_with(move |req: &Request| {
            let body: serde_json::Value = serde_json::from_slice(&req.body).unwrap();
            let mut parts: Vec<serde_json::Value> = body["part_numbers"]
                .as_array()
                .unwrap()
                .iter()
                .map(|n| {
                    let n = n.as_i64().unwrap();
                    serde_json::json!({ "part_number": n, "url": format!("{base}/storage/spart/{n}") })
                })
                .collect();
            parts.reverse();
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "parts": parts }))
        })
        .mount(&server)
        .await;

    for n in 1..=3 {
        Mock::given(method("PUT"))
            .and(path(format!("/storage/spart/{n}")))
            .respond_with(ResponseTemplate::new(200).insert_header("ETag", format!("\"etag-{n}\"")))
            .mount(&server)
            .await;
    }
    Mock::given(method("POST"))
        .and(path("/v1/uploads/upl_ord/finalize"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(mock_finalize_ok("upl_ord", contents.len())),
        )
        .mount(&server)
        .await;

    let path = temp_file(&contents);
    let client = test_client(&server.uri());
    let result = client.upload_file(&path, UploadOptions::default()).await;
    let _ = std::fs::remove_file(&path);
    result.expect("out-of-order mint response must not break the upload");

    let requests = server.received_requests().await.expect("requests recorded");
    let expected = [
        &contents[0..4 * MIB],
        &contents[4 * MIB..8 * MIB],
        &contents[8 * MIB..9 * MIB],
    ];
    for (i, slice) in expected.iter().enumerate() {
        let part_path = format!("/storage/spart/{}", i + 1);
        let put = requests
            .iter()
            .find(|r| r.url.path() == part_path)
            .unwrap_or_else(|| panic!("a PUT to {part_path} should have been made"));
        assert_eq!(
            &put.body[..],
            *slice,
            "part {} must receive its own {}-byte slice despite the reversed mint response",
            i + 1,
            slice.len()
        );
    }
}

/// A transient storage 500 on ONE part must NOT sink the whole upload: it is a
/// non-retryable status for the inner per-request retry wrapper (which only
/// retries connection resets / 429), so it propagates out of the per-part task
/// and is re-swept by the OUTER `upload_parts_resilient` round loop. Part 1's
/// PUT returns 500 once then 200; the upload must finalize with the full
/// ascending part list. Before the resilience change a single failed part
/// aborted the entire upload. The outer-round backoff (RetryRounds::default,
/// base_delay 2s) means this test takes a couple seconds — expected.
#[tokio::test]
async fn transient_part_500_is_recovered_by_outer_round_loop() {
    let server = MockServer::start().await;
    let storage_base = server.uri();
    // 9 MiB over a 5 MiB part size -> 2 parts: 5 MiB + 4 MiB. Streaming session
    // (no declared size), so part URLs are minted on demand.
    let part_size = 5 * MIB;
    let contents: Vec<u8> = (0..9 * MIB).map(|i| (i % 251) as u8).collect();

    Mock::given(method("POST"))
        .and(path("/v1/uploads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "finalize_token": "ftok_500",
            "headers": {},
            "mode": "multipart",
            "part_size": part_size,
            "upload_id": "upl_500",
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/v1/uploads/upl_500/parts"))
        .respond_with(mint_parts_responder(storage_base.clone()))
        .mount(&server)
        .await;

    // Part 1: ONE 500 (a transient server error, NOT a 429 — so the inner retry
    // wrapper does not swallow it), then 200 on the re-sweep.
    Mock::given(method("PUT"))
        .and(path("/storage/spart/1"))
        .respond_with(ResponseTemplate::new(500).set_body_string("<Error>InternalError</Error>"))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/storage/spart/1"))
        .respond_with(ResponseTemplate::new(200).insert_header("ETag", "\"etag-1\""))
        .mount(&server)
        .await;
    // Part 2 succeeds outright.
    Mock::given(method("PUT"))
        .and(path("/storage/spart/2"))
        .respond_with(ResponseTemplate::new(200).insert_header("ETag", "\"etag-2\""))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/v1/uploads/upl_500/finalize"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(mock_finalize_ok("upl_500", contents.len())),
        )
        .mount(&server)
        .await;

    let path = temp_file(&contents);
    let client = test_client(&server.uri());
    let result = client.upload_file(&path, UploadOptions::default()).await;
    let _ = std::fs::remove_file(&path);
    let result =
        result.expect("upload must survive a transient 500 on one part via the round loop");
    assert_eq!(result.upload_id, "upl_500");

    let requests = server.received_requests().await.expect("requests recorded");

    // Part 1 was PUT twice (the 500, then the recovering 200); part 2 once.
    let p1_puts = requests
        .iter()
        .filter(|r| r.url.path() == "/storage/spart/1")
        .count();
    assert_eq!(
        p1_puts, 2,
        "part 1 must be re-swept after the transient 500"
    );
    let p2_puts = requests
        .iter()
        .filter(|r| r.url.path() == "/storage/spart/2")
        .count();
    assert_eq!(p2_puts, 1, "part 2 succeeded on the first attempt");

    // Finalize carried the FULL ascending part list with both ETags — completed
    // parts kept their ETags across the round.
    let finalize = requests
        .iter()
        .find(|r| r.url.path() == "/v1/uploads/upl_500/finalize")
        .expect("a finalize request should have been made");
    let body: serde_json::Value = serde_json::from_slice(&finalize.body).expect("finalize JSON");
    let parts = body
        .get("parts")
        .and_then(|p| p.as_array())
        .expect("finalize must send a parts array");
    assert_eq!(parts.len(), 2, "both parts must be finalized");
    for (i, part) in parts.iter().enumerate() {
        assert_eq!(
            part.get("part_number").and_then(|v| v.as_i64()),
            Some((i + 1) as i64),
            "parts must be ascending and 1-based after recovery"
        );
        assert_eq!(
            part.get("e_tag").and_then(|v| v.as_str()),
            Some(format!("\"etag-{}\"", i + 1).as_str()),
            "each ETag must be present in the finalized list"
        );
    }
}

/// If a mint response omits a requested part number, the SDK must fail loudly
/// (rather than shift later parts' byte offsets), since the byte range for each
/// part is keyed to its number.
#[tokio::test]
async fn mint_missing_requested_part_is_rejected() {
    let server = MockServer::start().await;
    let storage_base = server.uri();
    let part_size = 5 * MIB;
    let contents: Vec<u8> = (0..9 * MIB).map(|i| (i % 251) as u8).collect(); // 2 parts

    Mock::given(method("POST"))
        .and(path("/v1/uploads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "finalize_token": "ftok_miss",
            "headers": {},
            "mode": "multipart",
            "part_size": part_size,
            "upload_id": "upl_miss",
        })))
        .mount(&server)
        .await;

    // Responder returns ONLY part 1 even when more were requested.
    let base = storage_base.clone();
    Mock::given(method("POST"))
        .and(path("/v1/uploads/upl_miss/parts"))
        .respond_with(move |_req: &Request| {
            ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "parts": [{ "part_number": 1, "url": format!("{base}/storage/spart/1") }]
            }))
        })
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path_regex(r"^/storage/spart/\d+$"))
        .respond_with(ResponseTemplate::new(200).insert_header("ETag", "\"e\""))
        .mount(&server)
        .await;

    let path = temp_file(&contents);
    let client = test_client(&server.uri());
    let result = client.upload_file(&path, UploadOptions::default()).await;
    let _ = std::fs::remove_file(&path);
    assert!(
        matches!(result, Err(UploadError::MalformedSession(_))),
        "a mint response missing a requested part must be rejected, got {result:?}"
    );
}
