//! Presigned direct-to-storage upload tests.
//!
//! These stand up a single wiremock server that plays BOTH roles: the hotdata
//! API (`POST /v1/uploads`, `POST /v1/uploads/{id}/finalize`) and the "object
//! storage" endpoint the SDK `PUT`s bytes to (`/storage/...`). They are fully
//! local and deterministic — no real backend, no credentials — so they run in
//! CI without secrets.
//!
//! Coverage:
//! * single-`PUT` happy path (bytes, header isolation, finalize token + empty
//!   parts, returned upload_id);
//! * multipart happy path (slicing by `part_size`, per-part ETag collection,
//!   ascending finalize parts);
//! * progress callback monotonicity reaching exactly the file size;
//! * storage-PUT header isolation (no SDK bearer/workspace/session headers).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use hotdata::apis::configuration::{ApiKey, Configuration};
use hotdata::{Client, UploadOptions};
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
