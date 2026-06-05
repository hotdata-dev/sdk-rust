//! End-to-end check that the SDK emits redacted request/response debug logs on
//! the `hotdata::http` target for real HTTP traffic (issue #135).
//!
//! This is the SDK side of restoring the CLI's `--debug` logging: the SDK emits
//! plain `log` records and a host installs a backend to render them. Here we
//! install a capturing `log` backend, drive a generated op against a wiremock
//! server, and assert that the request line, headers, status, and bodies were
//! logged — with the bearer token and sensitive body fields masked, never the
//! raw secret.
//!
//! It lives in its own test binary so it is the only thing installing a process
//! global logger; `log::set_boxed_logger` can only be called once per process.

use std::sync::{Mutex, OnceLock};

use hotdata::apis::{query_api, workspaces_api};
use hotdata::{models, Configuration};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Captures `hotdata::http` log records so the test can assert on them.
static LOG_BUF: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

fn log_buf() -> &'static Mutex<Vec<String>> {
    LOG_BUF.get_or_init(|| Mutex::new(Vec::new()))
}

struct CaptureLogger;

impl log::Log for CaptureLogger {
    fn enabled(&self, meta: &log::Metadata) -> bool {
        meta.target() == hotdata::http_log::TARGET
    }
    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            log_buf().lock().unwrap().push(record.args().to_string());
        }
    }
    fn flush(&self) {}
}

static LOGGER: CaptureLogger = CaptureLogger;

fn install_logger() {
    // `set_logger` (vs `set_boxed_logger`) needs no `std` feature on `log`.
    log::set_logger(&LOGGER).expect("logger installs once");
    log::set_max_level(log::LevelFilter::Debug);
}

fn captured() -> String {
    log_buf().lock().unwrap().join("\n")
}

#[tokio::test]
async fn debug_logs_redact_request_and_response() {
    install_logger();
    let server = MockServer::start().await;

    // --- GET op: exercises request line, header redaction, status, and a
    //     response body whose sensitive field must be masked. ---
    Mock::given(method("GET"))
        .and(path("/v1/workspaces"))
        .respond_with(
            // Body carries a `token` field so we can prove response-body
            // redaction; the op won't deserialize this into the real type, but
            // logging fires before deserialization either way.
            ResponseTemplate::new(200)
                .set_body_string(r#"{"token":"supersecrettoken123","ok":true}"#),
        )
        .mount(&server)
        .await;

    let mut config = Configuration::new();
    config.base_path = server.uri();
    config.bearer_access_token = Some("bearersecretvalue999".to_string());

    let _ = workspaces_api::list_workspaces(&config, None).await;

    // --- POST op: exercises a JSON request body being logged. ---
    Mock::given(method("POST"))
        .and(path("/v1/query"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ignored":true}"#))
        .mount(&server)
        .await;

    let request = models::QueryRequest {
        sql: "SELECT 1".to_string(),
        ..Default::default()
    };
    let _ = query_api::query(&config, request, None).await;

    let logs = captured();

    // Request line + status line, CLI-style markers.
    assert!(logs.contains(">>> GET"), "missing request line:\n{logs}");
    assert!(logs.contains(">>> POST"), "missing POST request line:\n{logs}");
    assert!(logs.contains("<<< 200"), "missing response status:\n{logs}");

    // The Authorization header is logged with the scheme preserved but the
    // token masked — never the raw bearer value.
    assert!(
        logs.contains("Bearer bear...e999"),
        "authorization not masked as expected:\n{logs}"
    );
    assert!(
        !logs.contains("bearersecretvalue999"),
        "raw bearer token leaked into logs:\n{logs}"
    );

    // The response body's sensitive field is masked, raw value never logged.
    assert!(
        logs.contains("supe...n123"),
        "response token not masked:\n{logs}"
    );
    assert!(
        !logs.contains("supersecrettoken123"),
        "raw response token leaked into logs:\n{logs}"
    );

    // The JSON request body is logged (non-sensitive field shown verbatim).
    assert!(
        logs.contains("SELECT 1"),
        "request body not logged:\n{logs}"
    );
}
